# LayDraw

LayDraw is a lightweight, extensible drawing and image-editing framework and application built with Rust using egui/eframe. It follows a Paint-style workflow and a plugin-oriented architecture for tools, brushes, shapes, panels, hooks, and app panels. This project is not intended to replace more professional image-editing software; rather it aims to provide a simple, lightweight, extensible image tool for macOS and Linux (similar to Windows Paint — Windows is supported too).

简体中文（翻译）：[README.zh-cn.md](README.zh-cn.md)

The framework focuses on making the core modular so new functionality can be added by implementing traits and registering components through `AppHost`.

## Features

- Paint-style canvas with zoom, pan, rulers, grid, and resizable canvas bounds.
- Tools: Select, Pencil, Brush, Eraser, Fill, Picker, Text, Magnifier, Shapes, plus plugin tools.
- Shape drawing with outline/fill modes, active edit box, resize/move, copy/cut/paste, and context menu support.
- Text tool with system font scanning, CJK font support, inline text editing, text box move/resize, style controls, and shared text rendering path for screen/export.
- Layers with visibility, opacity, blend mode, rename, thumbnails, virtualized scrolling, and drag-to-reorder.
- Undo/redo operation stack with plugin-accessible commands.
- Clipboard support for internal selections, bitmap image data, and copied image files on Windows.
- Image import/export through `image`.
- Plugin manager window for enabling/disabling plugins and registered components.
- Extensible panels for top ribbon, app tabs, side bars, bottom bar, and floating windows.

## Build And Run

Install a recent Rust toolchain, then run:

```powershell
cargo run
```

Check without running:

```powershell
cargo check
```

Format:

```powershell
cargo fmt
```

Build release:

```powershell
cargo build --release
```

`cargo run` from the workspace root runs the `laydraw_app` package. The root `laydraw` package is the framework/library crate; the app crate is responsible for selecting which plugin packages are compiled into the executable.

## Project Structure

- `laydraw_app/src/main.rs`: application entry point and static plugin loader.
- `laydraw_app/build.rs`: compile-time plugin compatibility checker.
- `src/lib.rs`: public library surface used by plugin subpackages.
- `src/ui/mod.rs`: main `PaintApp`, document state, canvas UI, command dispatch, plugin/component management.
- `src/ui/panel`: generic panel traits and built-in app panels.
- `src/tools`: tool traits and built-in tools.
- `src/tools/brush`: brush traits and built-in brush implementations.
- `src/tools/shape`: shape traits and built-in shape implementations.
- `src/plugins.rs`: plugin, hook, host, event, and command interfaces.
- `plugin_packages/example_package_plugin`: package-style plugin crate with its own `Cargo.toml`, dependencies, compatibility manifest, and plugin factory.
- `src/canvas`: canvas storage and pixel operations.
- `src/image_io.rs`: import/export helpers.
- `src/i18n`: language text mappings.


## Plugin Model

Plugins implement `Plugin` from `src/plugins.rs` and register components through `AppHost`.

```rust
impl Plugin for MyPlugin {
    fn plugin_name(&self) -> &'static str {
        "my.plugin"
    }

    fn plugin_title(&self, language: &Language) -> String {
        match language.plugin_locale_key() {
            "zh-CN" => language.plugin_text("我的插件"),
            _ => language.plugin_text("My Plugin"),
        }
    }

    fn plugin_author(&self) -> &'static str {
        "Your Name"
    }

    fn plugin_version(&self) -> &'static str {
        "1.0.0"
    }

    fn supported_laydraw_versions(&self) -> &'static str {
        ">=0.1.0, <0.2.0"
    }

    fn plugin_url(&self) -> &'static str {
        "https://example.com/my-plugin"
    }

    fn plugin_email(&self) -> &'static str {
        "plugins@example.com"
    }

    fn active(&mut self, app_host: &mut dyn AppHost) {
        app_host.load_tool(Box::new(MyTool::new()));
        app_host.load_panel(Box::new(MyPanel));
        app_host.load_hook(Box::new(MyHook));
    }

    fn inactive(&mut self, app_host: &mut dyn AppHost) {
        app_host.mark_canvas_dirty();
    }
}
```

Available registration points:

- `load_tool`: regular tool button.
- `load_cursor_tool`: tool plus custom cursor drawing.
- `load_brush`: brush dropdown entry with preview.
- `load_shape`: shape palette entry.
- `load_panel`: generic plugin panel for ribbon tabs, side bars, bottom bar, or window.
- `load_app_panel`: app-tab panel such as Home/View/Layers extension.
- `load_hook`: event hook for app lifecycle and document events.
- `load_plugin`: nested plugin registration.

Plugin metadata shown in the Plugins window:

- `plugin_name`: stable plugin id used for component ownership and persisted enable/disable state.
- `plugin_title`: localized display name.
- `plugin_author`: author or organization.
- `plugin_version`: plugin version.
- `supported_laydraw_versions`: runtime/display version support string.
- `plugin_url`: project/homepage URL.
- `plugin_email`: support/contact email.

## Plugin Subpackages

LayDraw exposes its framework modules through `src/lib.rs`, so a plugin can live as its own Cargo package and depend on the main framework crate:

```toml
[package]
name = "my-laydraw-plugin"
version = "0.1.0"
edition = "2024"

[dependencies]
laydraw = { path = "../.." }
egui = "0.28"
once_cell = "1"
```

Each plugin package also declares host compatibility in `laydraw-plugin.toml`:

```toml
id = "my.plugin"
title = "My Plugin"

# Exact host framework version:
support = "0.1.0"

# Or maximum host framework version:
support = "<=0.1.0"

# Or a range:
support = ">=0.1.0, <0.2.0"
```

Equivalent accepted keys are `support`, `supported_laydraw`, `laydraw_version`, and `max_laydraw_version`. `max_laydraw_version = "0.1.0"` is interpreted as `<=0.1.0`.

The sample subpackage is in:

```text
plugin_packages/example_package_plugin
```

It demonstrates:

- importing framework traits from `laydraw::plugins` and `laydraw::ui::panel`;
- declaring its own dependency, `once_cell`;
- declaring `laydraw-plugin.toml` compatibility metadata;
- exporting `pub fn plugin() -> Box<dyn Plugin>`;
- registering a panel through `AppHost`.

To compile a plugin into the app, add it to `laydraw_app/Cargo.toml` and load its factory in `laydraw_app/src/main.rs`:

```toml
[dependencies]
my-laydraw-plugin = { path = "../plugin_packages/my_laydraw_plugin" }
```

```rust
let mut app = ui::PaintApp::new(cc);
app.load_plugin(my_laydraw_plugin::plugin());
```

Check the whole workspace, including plugin subpackages:

```powershell
cargo check --workspace
```

Compile-time version check: `laydraw_app/build.rs` scans `plugin_packages/*/laydraw-plugin.toml` and compares each declared support expression with the root `laydraw` framework version. If any plugin package is incompatible, compilation fails before the app binary is built.

Static loading note: Rust plugins implemented with trait objects are type-safe and compiled into a host binary. A subpackage can depend on `laydraw` and compile independently. To ship it in a concrete application build, the application crate depends on the plugin package and calls its exported `plugin()` factory with `app_host.load_plugin(...)`. Dynamic runtime loading is intentionally not implemented yet because Rust trait object ABI is not stable across separately compiled dynamic libraries.

## Plugin I18n

Plugins receive the current language through `PanelContext`, `CanvasToolContext`, `ToolUiContext`, and `EventContext`.

For built-in languages, plugins can inspect the concrete language:

```rust
if let Some(zh_cn) = context.language.as_zh_cn_simple() {
    let label = zh_cn.get_text(LanguageText::Extra("插件文本".to_owned()));
}

if let Some(en_us) = context.language.as_en_us() {
    let label = en_us.get_text(LanguageText::Extra("Plugin text".to_owned()));
}
```

For simpler plugin code, use `plugin_locale_key()` and provide the text yourself:

```rust
let text = match context.language.plugin_locale_key() {
    "zh-CN" => "印章设置",
    "en-US" => "Stamp settings",
    other if other == "my-language-pack" => "Custom text",
    _ => "Stamp settings",
};

let label = context.language.plugin_text(text);
```

`LanguageText::Extra(String)` is passed through by the built-in language packs, so plugins can provide their own localized text while still using the same `get_text` path as the rest of the app. For third-party language packs represented by `Language::Extra(name)`, `plugin_locale_key()` returns that `name`, allowing plugins to select their own matching translation table.

Use the same pattern for every plugin-owned label, button, panel title, status message, context menu item, and floating window title. The example package plugin in `plugin_packages/example_package_plugin` demonstrates localized plugin metadata and localized plugin UI content.

## Tool Extension Points

`Tool` supports canvas events, overlays, context menus, and floating tool windows.

Useful methods:

- `tool_id`: stable component id.
- `get_tool_kind`: built-in or `ToolKind::Extra`.
- `tool_button`: custom button rendering.
- `tool_button_context_menu`: right-click menu on the tool button.
- `wants_canvas_events`: opt into canvas events.
- `on_canvas_event`: handle click, drag, hover, and stop events.
- `paint_canvas_overlay`: draw temporary UI over the canvas.
- `has_canvas_context_menu` and `canvas_context_menu`: right-click menu on the canvas while the tool is active.
- `has_tool_window` and `tool_window`: floating settings/tool window.

Use `CanvasToolContext` commands instead of mutating app state directly:

```rust
context.command(AppCommand::PushHistorySnapshot);
context.command(AppCommand::MarkCanvasDirty);
context.command(AppCommand::SetStatus("Done".to_owned()));
```

## Panel Extension Points

Generic panels implement `Panel`:

- `PanelArea::TopBar`
- `PanelArea::RibbonTab("tab id")`
- `PanelArea::LeftBar`
- `PanelArea::RightBar`
- `PanelArea::BottomBar`
- `PanelArea::Window`

App panels implement `AppPanel` and can target built-in app areas like Home, View, and Layers. This is the preferred route when a plugin wants to extend the main ribbon layout instead of opening a separate window.

## Hook Events

Hooks implement `AppHook` and receive `AppEvent` with an `EventContext`.

Examples of event categories:

- App lifecycle: `Startup`, `BeforeUi`, `AfterUi`.
- Canvas: `BeforeCanvasPaint`, `AfterCanvasPaint`, `CanvasDirty`, `CanvasResized`.
- Tools: `ActiveToolChanged`, `BrushSizeChanged`, `ColorChanged`.
- Layers: add/delete/move/merge/clear.
- Text/selection/image import.
- History: undo, redo, snapshot pushed, history cleared.
- Plugin lifecycle: load/unload success/failure.

Hooks should use `context.command(...)` for app changes so the host remains the single command dispatcher.

Minimal hook:

```rust
struct MyHook;

impl AppHook for MyHook {
    fn hook_id(&self) -> &'static str {
        "my.hook"
    }

    fn on_event(&mut self, event: &AppEvent, context: &mut EventContext<'_>) {
        match event {
            AppEvent::Startup => {
                context.command(AppCommand::SetStatus("App started".to_owned()));
            }
            _ => {}
        }
    }
}
```

Every `AppEvent` has a matching hook point:

```rust
fn on_event(&mut self, event: &AppEvent, context: &mut EventContext<'_>) {
    match event {
        AppEvent::Startup => {
            context.command(AppCommand::SetStatus("Startup".to_owned()));
        }
        AppEvent::BeforeUi => {
            // Read state before panels/canvas are drawn.
            let _tool = context.active_tool;
        }
        AppEvent::AfterUi => {
            // Queue UI-follow-up commands after the frame.
        }
        AppEvent::BeforeCanvasPaint => {
            // Inspect canvas state before canvas rendering.
        }
        AppEvent::AfterCanvasPaint => {
            // Good place for overlays driven by plugin state.
        }
        AppEvent::ActiveToolChanged { tool } => {
            context.command(AppCommand::SetStatus(format!("Tool: {tool:?}")));
        }
        AppEvent::ViewChanged { zoom, pan } => {
            context.command(AppCommand::SetStatus(format!("Zoom {zoom:.2}, pan {pan:?}")));
        }
        AppEvent::CanvasResized { width, height } => {
            context.command(AppCommand::SetStatus(format!("Canvas {width}x{height}")));
        }
        AppEvent::CanvasDirty => {
            *context.dirty_texture = true;
        }
        AppEvent::ColorChanged { primary, secondary } => {
            let _ = (*primary, *secondary);
        }
        AppEvent::BrushSizeChanged { size } => {
            context.command(AppCommand::SetStatus(format!("Brush size {size}")));
        }
        AppEvent::ActiveLayerChanged { layer } => {
            context.command(AppCommand::SetStatus(format!("Layer {layer}")));
        }
        AppEvent::LayerAdded { layer } => {
            context.command(AppCommand::SetStatus(format!("Layer added {layer}")));
        }
        AppEvent::LayerDeleted { layer } => {
            context.command(AppCommand::SetStatus(format!("Layer deleted {layer}")));
        }
        AppEvent::LayerMoved { layer } => {
            context.command(AppCommand::SetStatus(format!("Layer moved {layer}")));
        }
        AppEvent::LayerMerged => {
            context.command(AppCommand::SetStatus("Layer merged".to_owned()));
        }
        AppEvent::LayerCleared { layer } => {
            context.command(AppCommand::SetStatus(format!("Layer cleared {layer}")));
        }
        AppEvent::SelectionChanged => {
            let _selection = context.selected_rect;
        }
        AppEvent::TextCommitted => {
            context.command(AppCommand::SetStatus("Text committed".to_owned()));
        }
        AppEvent::TextCanceled => {
            context.command(AppCommand::SetStatus("Text canceled".to_owned()));
        }
        AppEvent::ImageImported { width, height } => {
            context.command(AppCommand::SetStatus(format!("Imported {width}x{height}")));
        }
        AppEvent::LanguageChanged => {
            context.command(AppCommand::SetStatus(
                context.language.plugin_text("Language changed"),
            ));
        }
        AppEvent::BrushStrokeCommitted => {
            context.command(AppCommand::SetStatus("Brush stroke committed".to_owned()));
        }
        AppEvent::ShapeCommitted => {
            context.command(AppCommand::SetStatus("Shape committed".to_owned()));
        }
        AppEvent::Undo => {
            context.command(AppCommand::SetStatus("Undo".to_owned()));
        }
        AppEvent::Redo => {
            context.command(AppCommand::SetStatus("Redo".to_owned()));
        }
        AppEvent::HistorySnapshotPushed => {
            // Snapshot exists; avoid pushing another snapshot here.
        }
        AppEvent::HistoryCleared => {
            context.command(AppCommand::SetStatus("History cleared".to_owned()));
        }
        AppEvent::PluginActivated => {
            context.command(AppCommand::SetStatus("Plugin activated".to_owned()));
        }
        AppEvent::PluginDeactivated => {
            context.command(AppCommand::SetStatus("Plugin deactivated".to_owned()));
        }
        AppEvent::PluginAfterLoad { plugin } => {
            context.command(AppCommand::SetStatus(format!("{plugin} loaded")));
        }
        AppEvent::PluginBeforeUnload { plugin } => {
            context.command(AppCommand::SetStatus(format!("{plugin} unloading")));
        }
        AppEvent::PluginLoadFailed { plugin, error } => {
            context.command(AppCommand::SetStatus(format!("{plugin}: {error}")));
        }
        AppEvent::PluginUnloadFailed { plugin, error } => {
            context.command(AppCommand::SetStatus(format!("{plugin}: {error}")));
        }
    }
}
```

## Internal Architecture

This section documents non-plugin internals so built-in features and plugin code share the same mental model.

### Command System

`AppCommand` is the write path into `PaintApp`. Tools, panels, and hooks should queue commands instead of mutating host fields directly:

```rust
context.command(AppCommand::PushHistorySnapshot);
context.command(AppCommand::SetPrimaryColor(egui::Color32::RED));
context.command(AppCommand::MarkCanvasDirty);
```

Commands are applied by `PaintApp::apply_commands`, which centralizes history, dirty state, layer changes, selection clearing, and view updates.

### Event System

`PaintApp::emit_event` builds an `EventContext` and sends each enabled hook the same event. Disabled plugins and disabled hooks are skipped. Hooks can read canvas state and queue commands; commands are applied after hook iteration.

### Canvas And Texture

`Canvas` stores `width`, `height`, and `pixels: Vec<Color32>`. The UI does not upload the canvas texture every frame. Pixel-changing operations call `mark_canvas_dirty`, which sets `dirty_texture = true`; `upload_texture` then rebuilds the egui texture once.

### Layers

The background canvas is layer `0`. Extra pixel layers are stored in `pixel_layers`. Each layer has:

- `name`
- `canvas`
- `visible`
- `opacity`
- `blend_mode`

Layer UI uses virtual rows so many layers do not instantiate all controls each frame. Layer compositing has a fast path for fully opaque normal pixels.

### History

Undo/redo snapshots use `DocumentSnapshot`, which captures background canvas, pixel layers, active layer, layer panel state, and text items. A tool should push one snapshot at the start of an operation, not every pointer movement.

### Selection

Selections use `SelectionContent`, which contains a `CanvasRegion` plus selected `TextItem`s. Moving/resizing/pasting selections preserves text metadata where possible instead of always rasterizing text.

### Text Rendering

`TextRenderer` scans system font directories, registers valid TTF/OTF fonts with egui, and is shared by screen drawing and export drawing. CJK-compatible fonts are preferred when available.

### Tools

Tools implement `Tool`. Canvas tools receive `CanvasToolEvent` through `on_canvas_event` when `wants_canvas_events()` returns true. UI-only tools can expose buttons, context menus, overlays, and floating windows without handling drag events.

### Brushes

Brushes implement `Brush` and are selected inside the Brush tool. Brush previews are drawn through `paint_preview`. Large brush sizes should sample sparsely or use optimized line primitives instead of stamping every line pixel.

### Shapes

Shapes implement `Shape` and are registered through `ShapeGroup` or `AppHost::load_shape`. Built-in shapes include Line, Curve, Rectangles, Ellipse, Polygons, Callouts, and Lightning. Shape drawing receives outline/fill colors, thickness, and `ShapeMode`.

### Panels

Generic panels implement `Panel` and receive `PanelContext`. App ribbon panels implement `AppPanel` and receive `&mut PaintApp`, so they are intended for trusted built-in or tightly integrated plugin UI.

### Plugin Manager

The plugin manager tracks source ownership for every loaded component. `plugin_name()` is the stable source id used by persisted enable/disable state. `plugin_title(&Language)` is display-only and can change with language.

## Performance Notes

Current performance-sensitive paths:

- Thick lines are drawn with a capsule-fill path instead of stamping a disc on every line pixel.
- Texture brushes sample stamps based on brush size to avoid redundant work at very large sizes.
- Layer list uses virtualized rows so many layers do not generate all controls every frame.
- Layer compositing has a fast path for normal, fully opaque pixels.
- Canvas texture upload is gated by `dirty_texture`.

When adding plugins:

- Avoid allocating large temporary vectors during pointer drag.
- Push undo snapshots only once at stroke/operation start.
- Prefer `CanvasToolContext` commands over direct side effects.
- Mark the canvas dirty only when pixels or document-visible state actually changed.
- Keep panel UI stable in height; use horizontal scroll or windows for wide content.

## Configuration

Runtime component/settings state is saved to:

```text
laydraw_components.cfg
```

The file is ignored by git because it is local user state.

## Development Checklist

Before handing off changes:

```powershell
cargo fmt
cargo check
```

For UI-heavy changes, manually verify:

- Narrow window ribbon does not grow vertically.
- Canvas pan/zoom/resize handles stay aligned.
- Select/text/shape active boxes commit correctly when focus changes.
- Plugin component toggles remove and restore UI safely.
- Clipboard paste works for internal selection, copied bitmap, and copied image file.

## About Font

The recommended font is `Noto Sans CJK`. The project will automatically scan the system font directory and attempt to load available font files. Therefore, you can place the font file in the system directory or under `./assert/font/`. Regarding font preference, it is determined by the `preferred_cjk_font` and `preferred_ui_font` functions in `./src/ui/mod. rs`.

```rust
fn preferred_cjk_font(family: &str) -> bool {
    let name = family.to_lowercase();
    [
        "microsoft yahei",
        "yahei",
        "simsun",
        "simhei",
        "simkai",
        "simfang",
        "dengxian",
        "noto sans sc",
        "noto serif sc",
        "pingfang",
        "hiragino",
        "noto sans cjk",
        "noto serif cjk",
        "source han",
        "wenquanyi",
        "sarasa",
    ]
    .iter()
    .any(|candidate| name.contains(candidate))
}
```


```rust
fn preferred_ui_font(family: &str) -> bool {
    let name = family.to_lowercase();
    [
        "segoe ui",
        "san francisco",
        "apple system",
        "dejavu sans",
        "arial",
        "liberation sans",
    ]
    .iter()
    .any(|candidate| name.contains(candidate))
}
```

In `TextRenderer::scan()`, first use `preferred_cjk_font` to search for the preferred CJK font. If it cannot be found, then use `preferred_ui_font` to search for the UI font, and finally use the first scanned font.

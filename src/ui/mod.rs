pub mod cursor;
pub mod panel;
pub mod ribbon_tab;

use ab_glyph::{Font, FontArc, ScaleFont, point};
use egui::{
    Color32, Context, CursorIcon, FontData, FontDefinitions, FontFamily, FontId, Pos2, Rect, Sense,
    Stroke, TextureHandle, TextureOptions, Vec2,
};
use std::{
    collections::HashSet,
    env, fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
};

use crate::{
    algorithm::LineAlgorithm,
    canvas::{Canvas, CanvasRegion, CanvasSnapshot},
    constants::{DEFAULT_HEIGHT, DEFAULT_WIDTH, MAX_UNDO},
    i18n::{Language, LanguageText, en_us::EnUs, zh_cn_simple::ZhCnSimple},
    image_io,
    plugins::{AppCommand, AppEvent, AppHook, AppHost, EventContext, Plugin},
    tools::{
        CanvasToolContext, CanvasToolEvent, CursorTool, Tool, ToolKind, ToolUiContext,
        brush::{Brush, BrushGroup},
        eraser::Eraser,
        fill::FillTool,
        magnifier::MagnifierTool,
        pencil::PencilTool,
        picker::PickerTool,
        select::SelectTool,
        shape::{Shape, ShapeGroup, ShapeKind, ShapeMode},
        text::TextTool,
    },
    ui::{
        cursor::MyCursorIcon,
        panel::{
            Panel, PanelArea, PanelContext,
            app::{
                AppPanel, AppPanelArea,
                builtin::{
                    BrushesPanel, ColorsPanel, HandlePanel, LayersPanel, OutlinePanel, ShapesPanel,
                    SizePanel, ToolsPanel, ViewPanel,
                },
            },
        },
        ribbon_tab::RibbonTab,
    },
};

const RULER_SIZE: f32 = 24.0;
const RIBBON_HEIGHT: f32 = 124.0;
const CANVAS_RESIZE_HIT: f32 = 18.0;
const CANVAS_RESIZE_MARGIN: f32 = 26.0;
const LAYER_ROW_HEIGHT: f32 = 92.0;
const APP_STATE_FILE: &str = "laydraw_components.cfg";
const BUILTIN_COMPONENT_SOURCE: &str = "Built-in";
const EXTERNAL_COMPONENT_SOURCE: &str = "External";
const BASIC_COLORS: [Color32; 48] = [
    Color32::from_rgb(0, 0, 0),
    Color32::from_rgb(127, 127, 127),
    Color32::from_rgb(136, 0, 21),
    Color32::from_rgb(237, 28, 36),
    Color32::from_rgb(255, 127, 39),
    Color32::from_rgb(255, 242, 0),
    Color32::from_rgb(34, 177, 76),
    Color32::from_rgb(0, 162, 232),
    Color32::from_rgb(63, 72, 204),
    Color32::from_rgb(163, 73, 164),
    Color32::from_rgb(255, 255, 255),
    Color32::from_rgb(195, 195, 195),
    Color32::from_rgb(185, 122, 87),
    Color32::from_rgb(255, 174, 201),
    Color32::from_rgb(255, 201, 14),
    Color32::from_rgb(239, 228, 176),
    Color32::from_rgb(181, 230, 29),
    Color32::from_rgb(153, 217, 234),
    Color32::from_rgb(112, 146, 190),
    Color32::from_rgb(200, 191, 231),
    Color32::from_rgb(242, 122, 122),
    Color32::from_rgb(238, 53, 40),
    Color32::from_rgb(120, 72, 68),
    Color32::from_rgb(128, 64, 54),
    Color32::from_rgb(75, 23, 14),
    Color32::from_rgb(141, 225, 229),
    Color32::from_rgb(76, 211, 224),
    Color32::from_rgb(52, 131, 222),
    Color32::from_rgb(10, 44, 225),
    Color32::from_rgb(12, 31, 142),
    Color32::from_rgb(255, 250, 125),
    Color32::from_rgb(255, 242, 64),
    Color32::from_rgb(242, 157, 78),
    Color32::from_rgb(241, 132, 74),
    Color32::from_rgb(112, 70, 31),
    Color32::from_rgb(116, 124, 209),
    Color32::from_rgb(126, 102, 232),
    Color32::from_rgb(55, 132, 181),
    Color32::from_rgb(0, 8, 85),
    Color32::from_rgb(84, 28, 97),
    Color32::from_rgb(141, 242, 120),
    Color32::from_rgb(124, 234, 65),
    Color32::from_rgb(88, 224, 75),
    Color32::from_rgb(87, 222, 96),
    Color32::from_rgb(107, 232, 139),
    Color32::from_rgb(96, 104, 54),
    Color32::from_rgb(226, 124, 173),
    Color32::from_rgb(221, 70, 213),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ResizeHandle {
    Right,
    Bottom,
    Corner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorEditMode {
    Rgb,
    Hsv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayerBlendMode {
    Normal,
    Multiply,
    Screen,
}

#[derive(Clone)]
struct PixelLayer {
    name: String,
    canvas: Canvas,
    visible: bool,
    opacity: f32,
    blend_mode: LayerBlendMode,
}

#[derive(Clone)]
struct MovingSelection {
    content: SelectionContent,
    origin: (i32, i32),
    position: (i32, i32),
    drag_anchor: (i32, i32),
}

#[derive(Clone)]
struct ResizingSelection {
    fixed: (i32, i32),
    original_content: SelectionContent,
}

#[derive(Clone)]
struct SelectionContent {
    region: CanvasRegion,
    text_items: Vec<TextItem>,
}

#[derive(Clone)]
struct DocumentSnapshot {
    canvas: CanvasSnapshot,
    pixel_layers: Vec<PixelLayer>,
    active_layer: usize,
    show_layers_panel: bool,
    text_items: Vec<TextItem>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
struct TextStyle {
    font_family: String,
    font_size: f32,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    align: TextAlign,
    background_fill: bool,
}

#[derive(Clone)]
struct TextItem {
    layer: usize,
    position: (i32, i32),
    size: (i32, i32),
    text: String,
    color: Color32,
    background: Color32,
    style: TextStyle,
}

#[derive(Clone)]
struct SystemFont {
    family: String,
    path: PathBuf,
    egui_key: String,
}

#[derive(Clone)]
struct TextRenderer {
    fonts: Vec<SystemFont>,
    default_family: String,
}

impl TextRenderer {
    fn scan() -> Self {
        let mut fonts = Vec::new();
        let mut seen = HashSet::new();
        for dir in system_font_dirs() {
            collect_font_files(&dir, &mut fonts, &mut seen, 0);
        }
        fonts.sort_by(|a, b| a.family.to_lowercase().cmp(&b.family.to_lowercase()));

        let default_family = fonts
            .iter()
            .find(|font| preferred_cjk_font(&font.family))
            .or_else(|| fonts.iter().find(|font| preferred_ui_font(&font.family)))
            .or_else(|| fonts.first())
            .map(|font| font.family.clone())
            .unwrap_or_else(|| "Proportional".to_owned());

        Self {
            fonts,
            default_family,
        }
    }

    fn install_egui_fonts(&self, ctx: &Context) {
        let mut definitions = FontDefinitions::default();
        let mut default_keys = Vec::new();

        for font in &self.fonts {
            let Ok(bytes) = fs::read(&font.path) else {
                continue;
            };
            definitions
                .font_data
                .insert(font.egui_key.clone(), FontData::from_owned(bytes));
            definitions
                .families
                .entry(FontFamily::Name(font.family.clone().into()))
                .or_default()
                .push(font.egui_key.clone());
            if preferred_cjk_font(&font.family) || font.family == self.default_family {
                default_keys.push(font.egui_key.clone());
            }
        }

        for key in default_keys.iter().rev() {
            definitions
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, key.clone());
            definitions
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, key.clone());
        }

        ctx.set_fonts(definitions);
    }

    fn default_family(&self) -> String {
        self.default_family.clone()
    }

    fn font_families(&self) -> Vec<String> {
        if self.fonts.is_empty() {
            return vec!["Proportional".to_owned(), "Monospace".to_owned()];
        }
        self.fonts.iter().map(|font| font.family.clone()).collect()
    }

    fn font_id(&self, style: &TextStyle, zoom: f32) -> FontId {
        let size = (style.font_size * zoom).clamp(1.0, 512.0);
        let family = match style.font_family.as_str() {
            "Monospace" | "Consolas" => FontFamily::Monospace,
            "Proportional" => FontFamily::Proportional,
            family => FontFamily::Name(family.to_owned().into()),
        };
        FontId::new(size, family)
    }

    fn layout_galley(
        &self,
        ui: &egui::Ui,
        text: &str,
        style: &TextStyle,
        color: Color32,
        zoom: f32,
        wrap_width: f32,
    ) -> std::sync::Arc<egui::Galley> {
        let mut job = self.layout_job(text, style, color, zoom, wrap_width);
        job.halign = match style.align {
            TextAlign::Left => egui::Align::Min,
            TextAlign::Center => egui::Align::Center,
            TextAlign::Right => egui::Align::Max,
        };
        ui.fonts(|fonts| fonts.layout_job(job))
    }

    fn layout_edit_galley(
        &self,
        ui: &egui::Ui,
        text: &str,
        style: &TextStyle,
        color: Color32,
        zoom: f32,
        wrap_width: f32,
    ) -> std::sync::Arc<egui::Galley> {
        let mut job = self.layout_job(text, style, color, zoom, wrap_width);
        job.halign = egui::Align::Min;
        ui.fonts(|fonts| fonts.layout_job(job))
    }

    fn layout_job(
        &self,
        text: &str,
        style: &TextStyle,
        color: Color32,
        zoom: f32,
        wrap_width: f32,
    ) -> egui::text::LayoutJob {
        let stroke_width = (zoom * 1.0).clamp(1.0, 2.5);
        let mut format = egui::text::TextFormat::simple(self.font_id(style, zoom), color);
        format.italics = style.italic;
        format.underline = if style.underline {
            Stroke::new(stroke_width, color)
        } else {
            Stroke::NONE
        };
        format.strikethrough = if style.strikethrough {
            Stroke::new(stroke_width, color)
        } else {
            Stroke::NONE
        };

        let mut job = egui::text::LayoutJob::single_section(text.to_owned(), format);
        job.wrap.max_width = wrap_width.max(1.0);
        job
    }

    fn render_text_item_to_pixels(
        &self,
        pixels: &mut [Color32],
        width: usize,
        height: usize,
        item: &TextItem,
    ) {
        let Some(font) = self.load_export_font(&item.style.font_family) else {
            return;
        };
        render_text_item_to_pixels(pixels, width, height, item, &font);
    }

    fn load_export_font(&self, family: &str) -> Option<FontArc> {
        self.fonts
            .iter()
            .find(|font| font.family == family)
            .and_then(|font| load_font_arc(&font.path))
            .or_else(|| {
                self.fonts
                    .iter()
                    .find(|font| font.family == self.default_family)
                    .and_then(|font| load_font_arc(&font.path))
            })
            .or_else(|| {
                self.fonts
                    .iter()
                    .find(|font| !font.path_has_collection_extension())
                    .and_then(|font| load_font_arc(&font.path))
            })
    }
}

impl SystemFont {
    fn path_has_collection_extension(&self) -> bool {
        self.path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| {
                let extension = extension.to_ascii_lowercase();
                extension == "ttc" || extension == "otc"
            })
            .unwrap_or(false)
    }
}

#[derive(Clone)]
struct ActiveTextBox {
    item: TextItem,
}

#[derive(Clone)]
struct CurveDraft {
    start: (i32, i32),
    end: (i32, i32),
    control1: Option<(i32, i32)>,
    control2: Option<(i32, i32)>,
    snapshot: CanvasSnapshot,
}

pub struct PaintApp {
    canvas: Canvas,
    pixel_layers: Vec<PixelLayer>,
    active_layer: usize,
    show_layers_panel: bool,
    texture: Option<TextureHandle>,
    dirty_texture: bool,
    language: Language,
    ribbon_tab: RibbonTab,
    tools: Vec<Box<dyn Tool>>,
    cursor_tools: Vec<Box<dyn CursorTool>>,
    active_tool: ToolKind,
    active_tool_slot: Option<usize>,
    active_cursor_tool_slot: Option<usize>,
    brush_group: BrushGroup,
    shape_group: ShapeGroup,
    app_panels: Vec<Box<dyn AppPanel>>,
    panels: Vec<Box<dyn Panel>>,
    primary: Color32,
    secondary: Color32,
    brush_size: i32,
    shape_mode: ShapeMode,
    zoom: f32,
    canvas_pan: Vec2,
    show_grid: bool,
    show_rulers: bool,
    status: String,
    drag_start: Option<(i32, i32)>,
    last_point: Option<(i32, i32)>,
    preview_point: Option<(i32, i32)>,
    resize_start_size: Option<(usize, usize)>,
    pan_start: Option<Vec2>,
    pointer_canvas_pos: Option<(i32, i32)>,
    selected_rect: Option<((i32, i32), (i32, i32))>,
    save_path: Option<PathBuf>,
    document_dirty: bool,
    recent_files: Vec<PathBuf>,
    moving_selection: Option<MovingSelection>,
    resizing_selection: Option<ResizingSelection>,
    selection_clipboard: Option<SelectionContent>,
    dragging_layer: Option<usize>,
    transparent_selection: bool,
    active_shape_rect: Option<((i32, i32), (i32, i32))>,
    active_shape_snapshot: Option<CanvasSnapshot>,
    moving_shape: Option<((i32, i32), ((i32, i32), (i32, i32)))>,
    moving_text_box: Option<((i32, i32), (i32, i32))>,
    curve_draft: Option<CurveDraft>,
    show_color_editor: bool,
    show_plugins_window: bool,
    editing_primary_color: bool,
    editor_color: Color32,
    color_edit_mode: ColorEditMode,
    editor_hue: f32,
    editor_saturation: f32,
    editor_value: f32,
    custom_colors: Vec<Option<Color32>>,
    recent_colors: Vec<Option<Color32>>,
    text_items: Vec<TextItem>,
    active_text_box: Option<ActiveTextBox>,
    text_renderer: TextRenderer,
    undo_stack: Vec<DocumentSnapshot>,
    redo_stack: Vec<DocumentSnapshot>,
    hooks: Vec<Box<dyn AppHook>>,
    plugins: Vec<Box<dyn Plugin>>,
    plugin_sources: Vec<&'static str>,
    app_panel_sources: Vec<&'static str>,
    tool_sources: Vec<&'static str>,
    cursor_tool_sources: Vec<&'static str>,
    brush_sources: Vec<&'static str>,
    shape_sources: Vec<&'static str>,
    panel_sources: Vec<&'static str>,
    hook_sources: Vec<&'static str>,
    loading_component_source: Option<&'static str>,
    disabled_tools: HashSet<String>,
    disabled_cursor_tools: HashSet<String>,
    disabled_brushes: HashSet<String>,
    disabled_shapes: HashSet<String>,
    disabled_panels: HashSet<String>,
    disabled_hooks: HashSet<String>,
    disabled_plugins: HashSet<String>,
    disabled_app_panels: HashSet<String>,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let language = Language::default();
        let text_renderer = TextRenderer::scan();
        let mut app = Self {
            canvas: Canvas::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, Color32::WHITE),
            pixel_layers: Vec::new(),
            active_layer: 0,
            show_layers_panel: true,
            texture: None,
            dirty_texture: true,
            language,
            ribbon_tab: RibbonTab::Home,
            tools: Vec::new(),
            cursor_tools: Vec::new(),
            active_tool: ToolKind::Brush,
            active_tool_slot: None,
            active_cursor_tool_slot: None,
            brush_group: BrushGroup::new(),
            shape_group: ShapeGroup::new(),
            app_panels: Vec::new(),
            panels: Vec::new(),
            primary: Color32::BLACK,
            secondary: Color32::WHITE,
            brush_size: 8,
            shape_mode: ShapeMode::Outline,
            zoom: 1.0,
            canvas_pan: Vec2::new(40.0, 40.0),
            show_grid: false,
            show_rulers: true,
            status: String::new(),
            drag_start: None,
            last_point: None,
            preview_point: None,
            resize_start_size: None,
            pan_start: None,
            pointer_canvas_pos: None,
            selected_rect: None,
            save_path: None,
            document_dirty: false,
            recent_files: Vec::new(),
            moving_selection: None,
            resizing_selection: None,
            selection_clipboard: None,
            dragging_layer: None,
            transparent_selection: false,
            active_shape_rect: None,
            active_shape_snapshot: None,
            moving_shape: None,
            moving_text_box: None,
            curve_draft: None,
            show_color_editor: false,
            show_plugins_window: false,
            editing_primary_color: true,
            editor_color: Color32::BLACK,
            color_edit_mode: ColorEditMode::Rgb,
            editor_hue: 0.0,
            editor_saturation: 0.0,
            editor_value: 0.0,
            custom_colors: vec![None; 24],
            recent_colors: vec![None; 12],
            text_items: Vec::new(),
            active_text_box: None,
            text_renderer,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            hooks: Vec::new(),
            plugins: Vec::new(),
            plugin_sources: Vec::new(),
            app_panel_sources: Vec::new(),
            tool_sources: Vec::new(),
            cursor_tool_sources: Vec::new(),
            brush_sources: Vec::new(),
            shape_sources: Vec::new(),
            panel_sources: Vec::new(),
            hook_sources: Vec::new(),
            loading_component_source: None,
            disabled_tools: HashSet::new(),
            disabled_cursor_tools: HashSet::new(),
            disabled_brushes: HashSet::new(),
            disabled_shapes: HashSet::new(),
            disabled_panels: HashSet::new(),
            disabled_hooks: HashSet::new(),
            disabled_plugins: HashSet::new(),
            disabled_app_panels: HashSet::new(),
        };
        app.text_renderer.install_egui_fonts(&cc.egui_ctx);
        app.status = app.tr(LanguageText::Ready);
        app.load_app_state();
        app.load_builtin_brushes_and_shapes();
        app.load_builtin_tools();
        app.load_builtin_panels();
        app.ensure_active_tool_enabled();
        app.ensure_active_brush_enabled();
        app.ensure_active_shape_enabled();
        app.upload_texture(&cc.egui_ctx);
        app.emit_event(AppEvent::Startup);
        app
    }

    fn load_builtin_tools(&mut self) {
        self.tools.push(Box::new(SelectTool::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools.push(Box::new(PencilTool::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools.push(Box::new(BrushGroup::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools
            .push(Box::new(Eraser::new(self.brush_size, self.zoom)));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools.push(Box::new(FillTool::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools.push(Box::new(PickerTool::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools.push(Box::new(TextTool::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools.push(Box::new(MagnifierTool::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.tools.push(Box::new(ShapeGroup::new()));
        self.tool_sources.push(BUILTIN_COMPONENT_SOURCE);
        self.active_tool_slot = self
            .tools
            .iter()
            .position(|tool| tool.get_tool_kind() == self.active_tool);
    }

    fn load_builtin_brushes_and_shapes(&mut self) {
        self.brush_sources = vec![BUILTIN_COMPONENT_SOURCE; self.brush_group.brushes().len()];
        self.shape_sources = vec![BUILTIN_COMPONENT_SOURCE; self.shape_group.shapes().len()];
    }

    fn load_builtin_panels(&mut self) {
        let previous_source = self
            .loading_component_source
            .replace(BUILTIN_COMPONENT_SOURCE);
        self.load_app_panel(Box::new(ToolsPanel));
        self.load_app_panel(Box::new(HandlePanel));
        self.load_app_panel(Box::new(ShapesPanel));
        self.load_app_panel(Box::new(OutlinePanel));
        self.load_app_panel(Box::new(BrushesPanel));
        self.load_app_panel(Box::new(SizePanel));
        self.load_app_panel(Box::new(ColorsPanel));
        self.load_app_panel(Box::new(ViewPanel));
        self.load_app_panel(Box::new(LayersPanel));
        self.loading_component_source = previous_source;
    }

    pub(crate) fn load_app_panel(&mut self, panel: Box<dyn AppPanel>) {
        self.app_panels.push(panel);
        self.app_panel_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    #[allow(dead_code)]
    pub fn load_panel(&mut self, panel: Box<dyn Panel>) {
        self.panels.push(panel);
        self.panel_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    fn emit_event(&mut self, event: AppEvent) {
        let active_tool = self.active_tool;
        let primary = self.primary;
        let secondary = self.secondary;
        let brush_size = self.brush_size;
        let active_layer = self.active_layer;
        let layer_count = self.pixel_layers.len() + 1;
        let selected_rect = self.selected_rect;
        let pointer_canvas_pos = self.pointer_canvas_pos;
        let zoom = self.zoom;
        let pan = self.canvas_pan;
        let mut context = EventContext {
            canvas: &mut self.canvas,
            dirty_texture: &mut self.dirty_texture,
            language: &self.language,
            status: &mut self.status,
            active_tool,
            primary,
            secondary,
            brush_size,
            active_layer,
            layer_count,
            selected_rect,
            pointer_canvas_pos,
            zoom,
            pan,
            commands: Vec::new(),
        };
        for (index, hook) in self.hooks.iter_mut().enumerate() {
            let source = self
                .hook_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE);
            if self.disabled_plugins.contains(&Self::plugin_key(source)) {
                continue;
            }
            let key = Self::component_key(source, hook.hook_id());
            if self.disabled_hooks.contains(&key) {
                continue;
            }
            hook.on_event(&event, &mut context);
        }
        let commands = context.commands;
        self.apply_commands(commands);
    }

    fn apply_commands(&mut self, commands: Vec<AppCommand>) {
        for command in commands {
            self.apply_command(command);
        }
    }

    fn apply_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::MarkCanvasDirty => self.mark_canvas_dirty(),
            AppCommand::PushHistorySnapshot => self.push_undo_snapshot(),
            AppCommand::Undo => {
                self.undo_action();
            }
            AppCommand::Redo => {
                self.redo_action();
            }
            AppCommand::ClearHistory => self.clear_history_stack(),
            AppCommand::SetStatus(status) => {
                self.status = status;
            }
            AppCommand::SetActiveTool(tool) => self.set_active_tool(tool),
            AppCommand::SetView { zoom, pan } => self.set_view(zoom, pan),
            AppCommand::ResizeCanvas { width, height } => self.resize_canvas_live(width, height),
            AppCommand::SetPrimaryColor(color) => {
                if self.primary != color {
                    self.primary = color;
                    self.remember_recent_color(color);
                    self.emit_event(AppEvent::ColorChanged {
                        primary: self.primary,
                        secondary: self.secondary,
                    });
                }
            }
            AppCommand::SetSecondaryColor(color) => {
                if self.secondary != color {
                    self.secondary = color;
                    self.remember_recent_color(color);
                    self.emit_event(AppEvent::ColorChanged {
                        primary: self.primary,
                        secondary: self.secondary,
                    });
                }
            }
            AppCommand::SwapColors => {
                std::mem::swap(&mut self.primary, &mut self.secondary);
                self.remember_recent_color(self.primary);
                self.remember_recent_color(self.secondary);
                self.emit_event(AppEvent::ColorChanged {
                    primary: self.primary,
                    secondary: self.secondary,
                });
            }
            AppCommand::SetBrushSize(size) => {
                let size = size.clamp(1, 80);
                if self.brush_size != size {
                    self.brush_size = size;
                    self.emit_event(AppEvent::BrushSizeChanged { size });
                }
            }
            AppCommand::SetActiveLayer(layer) => self.set_active_layer(layer),
            AppCommand::AddLayer => self.add_pixel_layer(),
            AppCommand::DeleteActiveLayer => self.delete_active_layer(),
            AppCommand::MoveActiveLayerUp => self.move_active_layer_up(),
            AppCommand::MoveActiveLayerDown => self.move_active_layer_down(),
            AppCommand::MergeActiveLayerDown => self.merge_active_layer_down(),
            AppCommand::MergeVisibleLayers => self.merge_visible_layers(),
            AppCommand::ClearCurrentLayer => self.clear_current_layer(),
            AppCommand::ClearSelection => {
                self.selected_rect = None;
                self.moving_selection = None;
                self.resizing_selection = None;
                self.emit_event(AppEvent::SelectionChanged);
            }
        }
    }

    pub(crate) fn set_active_tool(&mut self, tool: ToolKind) {
        let slot = self
            .tools
            .iter()
            .enumerate()
            .find(|(index, candidate)| {
                !self.disabled_tools.contains(&self.tool_key(*index))
                    && candidate.get_tool_kind() == tool
            })
            .map(|(index, _)| index);
        self.set_active_tool_with_slot(tool, slot, None);
    }

    fn set_active_tool_with_slot(
        &mut self,
        tool: ToolKind,
        slot: Option<usize>,
        cursor_slot: Option<usize>,
    ) {
        let had_active_tool = self.active_tool_is_enabled();
        if self.active_tool == ToolKind::Select && tool != ToolKind::Select {
            self.commit_active_selection();
        }
        if tool == ToolKind::Shape && self.shape_group.active_index().is_none() {
            self.ensure_active_shape_enabled();
        }
        if tool != ToolKind::Shape {
            self.active_shape_rect = None;
            self.active_shape_snapshot = None;
        }
        if self.active_tool != tool
            || self.active_tool_slot != slot
            || self.active_cursor_tool_slot != cursor_slot
        {
            self.active_tool = tool;
            self.active_tool_slot = slot;
            self.active_cursor_tool_slot = cursor_slot;
            if !had_active_tool && (slot.is_some() || cursor_slot.is_some()) {
                self.status = self.tr(LanguageText::Ready);
            }
            self.emit_event(AppEvent::ActiveToolChanged { tool });
        }
    }

    fn active_tool_is_enabled(&self) -> bool {
        if let Some(slot) = self.active_tool_slot {
            if slot >= self.tools.len() {
                return false;
            }
            return !self.disabled_tools.contains(&self.tool_key(slot));
        }
        if let Some(slot) = self.active_cursor_tool_slot {
            if slot >= self.cursor_tools.len() {
                return false;
            }
            return !self
                .disabled_cursor_tools
                .contains(&self.cursor_tool_key(slot));
        }
        false
    }

    fn ensure_active_tool_enabled(&mut self) {
        if self.active_tool_is_enabled() {
            return;
        }
        if let Some((index, tool)) = self
            .tools
            .iter()
            .enumerate()
            .find(|(index, _)| !self.disabled_tools.contains(&self.tool_key(*index)))
        {
            self.set_active_tool_with_slot(tool.get_tool_kind(), Some(index), None);
            return;
        }
        if let Some((index, tool)) = self.cursor_tools.iter().enumerate().find(|(index, _)| {
            !self
                .disabled_cursor_tools
                .contains(&self.cursor_tool_key(*index))
        }) {
            self.set_active_tool_with_slot(tool.get_tool_kind(), None, Some(index));
            return;
        }
        self.set_no_active_tool();
    }

    fn set_no_active_tool(&mut self) {
        if self.active_tool == ToolKind::Select {
            self.commit_active_selection();
        }
        self.active_tool_slot = None;
        self.active_cursor_tool_slot = None;
        self.active_shape_rect = None;
        self.active_shape_snapshot = None;
        self.moving_shape = None;
        self.status = self.tr(LanguageText::NoToolActive);
    }

    fn ensure_active_brush_enabled(&mut self) {
        let active = self.brush_group.active_index();
        if active < self.brush_group.brushes().len()
            && !self.disabled_brushes.contains(&self.brush_key(active))
        {
            return;
        }
        if let Some((index, _)) = self
            .brush_group
            .brushes()
            .iter()
            .enumerate()
            .find(|(index, _)| !self.disabled_brushes.contains(&self.brush_key(*index)))
        {
            self.brush_group.select(index);
        }
    }

    fn ensure_active_shape_enabled(&mut self) {
        if let Some(active) = self.shape_group.active_index() {
            if active < self.shape_group.shapes().len()
                && !self.disabled_shapes.contains(&self.shape_key(active))
            {
                return;
            }
        }
        if let Some((index, _)) = self
            .shape_group
            .shapes()
            .iter()
            .enumerate()
            .find(|(index, _)| !self.disabled_shapes.contains(&self.shape_key(*index)))
        {
            self.shape_group.select(index);
        }
    }

    fn set_view(&mut self, zoom: f32, pan: Vec2) {
        let zoom = zoom.clamp(0.05, 8.0);
        if (self.zoom - zoom).abs() > f32::EPSILON || self.canvas_pan != pan {
            self.zoom = zoom;
            self.canvas_pan = pan;
            self.emit_event(AppEvent::ViewChanged {
                zoom: self.zoom,
                pan: self.canvas_pan,
            });
        }
    }

    fn remember_recent_color(&mut self, color: Color32) {
        let mut colors: Vec<Color32> = self.recent_colors.iter().flatten().copied().collect();
        colors.retain(|candidate| *candidate != color);
        colors.insert(0, color);
        colors.truncate(self.recent_colors.len());
        self.recent_colors.fill(None);
        for (slot, color) in self.recent_colors.iter_mut().zip(colors) {
            *slot = Some(color);
        }
    }

    fn remember_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|candidate| candidate != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(10);
    }

    fn set_active_layer(&mut self, layer: usize) {
        let layer = layer.min(self.pixel_layers.len());
        if self.active_layer != layer {
            self.active_layer = layer;
            self.clear_transient_selection_state();
            self.status = if layer == 0 {
                self.tr(LanguageText::BackgroundLayer)
            } else {
                self.pixel_layers[layer - 1].name.clone()
            };
            self.emit_event(AppEvent::ActiveLayerChanged { layer });
        }
    }

    fn render_app_panels(&mut self, ui: &mut egui::Ui, area: AppPanelArea) {
        let mut panels = std::mem::take(&mut self.app_panels);
        let mut first = true;
        for (index, panel) in panels.iter_mut().enumerate() {
            let panel_key = Self::component_key(
                self.app_panel_sources
                    .get(index)
                    .copied()
                    .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                panel.panel_id(),
            );
            if self.disabled_app_panels.contains(&panel_key) {
                continue;
            }
            if panel.panel_area() != area {
                continue;
            }
            if !first {
                ui.separator();
            }
            panel.ui(self, ui);
            first = false;
        }
        self.app_panels = panels;
    }

    fn app_panel_enabled(&self, index: usize) -> bool {
        !self
            .disabled_app_panels
            .contains(&self.app_panel_key(index))
    }

    fn has_enabled_app_panel_area(&self, area: AppPanelArea) -> bool {
        self.app_panels
            .iter()
            .enumerate()
            .any(|(index, panel)| self.app_panel_enabled(index) && panel.panel_area() == area)
    }

    fn active_ribbon_has_body(&self) -> bool {
        match self.ribbon_tab {
            RibbonTab::File => true,
            RibbonTab::Home => self.has_enabled_app_panel_area(AppPanelArea::Home),
            RibbonTab::View => self.has_enabled_app_panel_area(AppPanelArea::View),
            RibbonTab::Plugin(tab_id) => self.panels.iter().enumerate().any(|(index, panel)| {
                !self.disabled_panels.contains(&self.panel_key(index))
                    && panel.panel_area() == PanelArea::RibbonTab(tab_id)
            }),
            RibbonTab::Plugins | RibbonTab::Extra => false,
        }
    }

    fn top_bar_height(&self) -> f32 {
        if self.active_ribbon_has_body() {
            RIBBON_HEIGHT
        } else {
            38.0
        }
    }

    fn panel_context(&self) -> PanelContext<'_> {
        PanelContext {
            language: &self.language,
            active_tool: self.active_tool,
            primary: self.primary,
            secondary: self.secondary,
            brush_size: self.brush_size,
            active_layer: self.active_layer,
            layer_count: self.pixel_layers.len() + 1,
            selected_rect: self.selected_rect,
            pointer_canvas_pos: self.pointer_canvas_pos,
            zoom: self.zoom,
            pan: self.canvas_pan,
            status: &self.status,
            commands: Vec::new(),
        }
    }

    fn component_key(source: &'static str, id: &'static str) -> String {
        format!("{source}::{id}")
    }

    fn tool_key(&self, index: usize) -> String {
        Self::component_key(
            self.tool_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
            self.tools[index].tool_id(),
        )
    }

    fn app_panel_key(&self, index: usize) -> String {
        Self::component_key(
            self.app_panel_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
            self.app_panels[index].panel_id(),
        )
    }

    fn cursor_tool_key(&self, index: usize) -> String {
        Self::component_key(
            self.cursor_tool_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
            self.cursor_tools[index].tool_id(),
        )
    }

    fn brush_key(&self, index: usize) -> String {
        Self::component_key(
            self.brush_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
            self.brush_group.brushes()[index].brush_id(),
        )
    }

    fn shape_key(&self, index: usize) -> String {
        Self::component_key(
            self.shape_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
            self.shape_group.shapes()[index].shape_id(),
        )
    }

    fn panel_key(&self, index: usize) -> String {
        Self::component_key(
            self.panel_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
            self.panels[index].panel_id(),
        )
    }

    fn hook_key(&self, index: usize) -> String {
        Self::component_key(
            self.hook_sources
                .get(index)
                .copied()
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
            self.hooks[index].hook_id(),
        )
    }

    fn plugin_key(source: &'static str) -> String {
        source.to_owned()
    }

    fn plugin_source_index(&self, source: &'static str) -> Option<usize> {
        self.plugin_sources
            .iter()
            .position(|plugin_source| *plugin_source == source)
    }

    fn plugin_for_source(&self, source: &'static str) -> Option<&dyn Plugin> {
        self.plugin_source_index(source)
            .and_then(|index| self.plugins.get(index).map(Box::as_ref))
    }

    fn plugin_source_title(&self, source: &'static str) -> String {
        self.plugin_for_source(source)
            .map(|plugin| plugin.plugin_title(&self.language))
            .unwrap_or_else(|| source.to_owned())
    }

    fn plugin_source_enabled(&self, source: &'static str) -> bool {
        !self.disabled_plugins.contains(&Self::plugin_key(source))
    }

    fn activate_plugin_components(
        &mut self,
        plugin: &mut dyn Plugin,
        source: &'static str,
    ) -> Result<(), String> {
        self.loading_component_source = Some(source);
        let result = catch_unwind(AssertUnwindSafe(|| {
            plugin.before_load(self)?;
            plugin.active(self);
            plugin.after_load(self)?;
            Ok::<(), String>(())
        }));
        self.loading_component_source = None;

        match result {
            Ok(Ok(())) => {
                self.emit_event(AppEvent::PluginAfterLoad { plugin: source });
                Ok(())
            }
            Ok(Err(error)) => {
                self.rollback_plugin_load(plugin, source, &error);
                Err(error)
            }
            Err(_) => {
                let error = "plugin panicked while loading".to_owned();
                self.rollback_plugin_load(plugin, source, &error);
                Err(error)
            }
        }
    }

    fn rollback_plugin_load(&mut self, plugin: &mut dyn Plugin, source: &'static str, error: &str) {
        self.loading_component_source = None;
        self.remove_components_from_source(source);
        plugin.on_load_error(self, error);
        self.status = format!("{source}: {error}");
        self.emit_event(AppEvent::PluginLoadFailed {
            plugin: source,
            error: error.to_owned(),
        });
    }

    fn deactivate_plugin_components(
        &mut self,
        plugin: &mut dyn Plugin,
        source: &'static str,
    ) -> Result<(), String> {
        self.emit_event(AppEvent::PluginBeforeUnload { plugin: source });
        let result = catch_unwind(AssertUnwindSafe(|| {
            plugin.before_unload(self)?;
            plugin.inactive(self);
            plugin.after_unload(self)?;
            Ok::<(), String>(())
        }));

        match result {
            Ok(Ok(())) => {
                self.remove_components_from_source(source);
                self.emit_event(AppEvent::PluginDeactivated);
                Ok(())
            }
            Ok(Err(error)) => {
                plugin.on_unload_error(self, &error);
                self.status = format!("{source}: {error}");
                self.emit_event(AppEvent::PluginUnloadFailed {
                    plugin: source,
                    error: error.clone(),
                });
                Err(error)
            }
            Err(_) => {
                let error = "plugin panicked while unloading".to_owned();
                plugin.on_unload_error(self, &error);
                self.status = format!("{source}: {error}");
                self.emit_event(AppEvent::PluginUnloadFailed {
                    plugin: source,
                    error: error.clone(),
                });
                Err(error)
            }
        }
    }

    fn remove_components_from_source(&mut self, source: &'static str) {
        for index in (0..self.tools.len()).rev() {
            if self.tool_sources.get(index).copied() != Some(source) {
                continue;
            }
            self.tools.remove(index);
            self.tool_sources.remove(index);
            if self.active_tool_slot == Some(index) {
                self.active_tool_slot = None;
            } else if let Some(slot) = self.active_tool_slot {
                if slot > index {
                    self.active_tool_slot = Some(slot - 1);
                }
            }
        }

        for index in (0..self.cursor_tools.len()).rev() {
            if self.cursor_tool_sources.get(index).copied() != Some(source) {
                continue;
            }
            self.cursor_tools.remove(index);
            self.cursor_tool_sources.remove(index);
            if self.active_cursor_tool_slot == Some(index) {
                self.active_cursor_tool_slot = None;
            } else if let Some(slot) = self.active_cursor_tool_slot {
                if slot > index {
                    self.active_cursor_tool_slot = Some(slot - 1);
                }
            }
        }

        for index in (0..self.brush_sources.len()).rev() {
            if self.brush_sources.get(index).copied() == Some(source) {
                self.brush_group.remove_brush(index);
                self.brush_sources.remove(index);
            }
        }

        for index in (0..self.shape_sources.len()).rev() {
            if self.shape_sources.get(index).copied() == Some(source) {
                self.shape_group.remove_shape(index);
                self.shape_sources.remove(index);
            }
        }

        for index in (0..self.panels.len()).rev() {
            if self.panel_sources.get(index).copied() == Some(source) {
                if let RibbonTab::Plugin(tab_id) = self.ribbon_tab {
                    if self.panels[index].panel_area() == PanelArea::RibbonTab(tab_id) {
                        self.ribbon_tab = RibbonTab::Home;
                    }
                }
                self.panels.remove(index);
                self.panel_sources.remove(index);
            }
        }

        for index in (0..self.app_panels.len()).rev() {
            if self.app_panel_sources.get(index).copied() == Some(source) {
                self.app_panels.remove(index);
                self.app_panel_sources.remove(index);
            }
        }

        for index in (0..self.hooks.len()).rev() {
            if self.hook_sources.get(index).copied() == Some(source) {
                self.hooks.remove(index);
                self.hook_sources.remove(index);
            }
        }

        self.ensure_active_tool_enabled();
        self.ensure_active_brush_enabled();
        self.ensure_active_shape_enabled();
    }

    fn set_plugin_source_enabled(&mut self, source: &'static str, enabled: bool) {
        let Some(index) = self.plugin_source_index(source) else {
            return;
        };
        let key = Self::plugin_key(source);
        if enabled {
            if self.disabled_plugins.remove(&key) {
                let mut plugin = self.plugins.remove(index);
                match self.activate_plugin_components(plugin.as_mut(), source) {
                    Ok(()) => {
                        self.ensure_active_tool_enabled();
                        self.ensure_active_brush_enabled();
                        self.ensure_active_shape_enabled();
                        self.emit_event(AppEvent::PluginActivated);
                    }
                    Err(_) => {
                        self.disabled_plugins.insert(key);
                    }
                }
                self.plugins.insert(index, plugin);
            }
        } else if self.disabled_plugins.insert(key.clone()) {
            let mut plugin = self.plugins.remove(index);
            if self
                .deactivate_plugin_components(plugin.as_mut(), source)
                .is_err()
            {
                self.disabled_plugins.remove(&key);
            }
            self.plugins.insert(index, plugin);
        }
        self.save_app_state();
    }

    fn load_app_state(&mut self) {
        let Ok(contents) = fs::read_to_string(APP_STATE_FILE) else {
            return;
        };
        self.disabled_tools.clear();
        self.disabled_cursor_tools.clear();
        self.disabled_brushes.clear();
        self.disabled_shapes.clear();
        self.disabled_panels.clear();
        self.disabled_hooks.clear();
        self.disabled_plugins.clear();
        self.disabled_app_panels.clear();
        self.custom_colors.fill(None);
        self.recent_colors.fill(None);
        self.recent_files.clear();
        for line in contents.lines() {
            let mut parts = line.splitn(3, '|');
            let Some(kind) = parts.next() else {
                continue;
            };
            let Some(key) = parts.next() else {
                continue;
            };
            match kind {
                "tool" => {
                    self.disabled_tools.insert(key.to_owned());
                }
                "cursor_tool" => {
                    self.disabled_cursor_tools.insert(key.to_owned());
                }
                "brush" => {
                    self.disabled_brushes.insert(key.to_owned());
                }
                "shape" => {
                    self.disabled_shapes.insert(key.to_owned());
                }
                "panel" => {
                    self.disabled_panels.insert(key.to_owned());
                }
                "app_panel" => {
                    self.disabled_app_panels.insert(key.to_owned());
                }
                "hook" => {
                    self.disabled_hooks.insert(key.to_owned());
                }
                "plugin" => {
                    self.disabled_plugins.insert(key.to_owned());
                }
                "setting" => {
                    if let Some(value) = parts.next() {
                        self.apply_persisted_setting(key, value);
                    }
                }
                "custom_color" => {
                    if let Some(value) = parts.next() {
                        if let (Ok(index), Some(color)) =
                            (key.parse::<usize>(), parse_hex_color(value))
                        {
                            if let Some(slot) = self.custom_colors.get_mut(index) {
                                *slot = Some(color);
                            }
                        }
                    }
                }
                "recent_color" => {
                    if let Some(value) = parts.next() {
                        if let (Ok(index), Some(color)) =
                            (key.parse::<usize>(), parse_hex_color(value))
                        {
                            if let Some(slot) = self.recent_colors.get_mut(index) {
                                *slot = Some(color);
                            }
                        }
                    }
                }
                "recent_file" => {
                    if let Some(value) = parts.next() {
                        if key.parse::<usize>().is_ok() {
                            self.recent_files.push(PathBuf::from(value));
                        }
                    }
                }
                _ => {}
            }
        }
        self.ensure_canvas_size_constraints();
    }

    fn save_component_state(&self) {
        self.save_app_state();
    }

    fn save_app_state(&self) {
        let mut lines = Vec::new();
        lines.push(format!(
            "setting|language|{}",
            match self.language {
                Language::EnUs(_) => "en-US",
                Language::ZhCnSimple(_) => "zh-CN",
                Language::Extra(_) => "extra",
            }
        ));
        lines.push(format!("setting|canvas_width|{}", self.canvas.width));
        lines.push(format!("setting|canvas_height|{}", self.canvas.height));
        lines.push(format!("setting|zoom|{:.4}", self.zoom));
        lines.push(format!("setting|pan_x|{:.2}", self.canvas_pan.x));
        lines.push(format!("setting|pan_y|{:.2}", self.canvas_pan.y));
        lines.push(format!("setting|show_grid|{}", self.show_grid));
        lines.push(format!("setting|show_rulers|{}", self.show_rulers));
        lines.push(format!(
            "setting|show_layers_panel|{}",
            self.show_layers_panel
        ));
        lines.push(format!(
            "setting|show_plugins_window|{}",
            self.show_plugins_window
        ));
        lines.push(format!("setting|primary|{}", format_color(self.primary)));
        lines.push(format!(
            "setting|secondary|{}",
            format_color(self.secondary)
        ));
        lines.push(format!("setting|brush_size|{}", self.brush_size));
        lines.push(format!(
            "setting|shape_mode|{}",
            match self.shape_mode {
                ShapeMode::Outline => "outline",
                ShapeMode::Filled => "filled",
                ShapeMode::FilledOutline => "filled_outline",
            }
        ));
        lines.push(format!(
            "setting|color_edit_mode|{}",
            match self.color_edit_mode {
                ColorEditMode::Rgb => "rgb",
                ColorEditMode::Hsv => "hsv",
            }
        ));
        lines.push(format!(
            "setting|ribbon_tab|{}",
            match self.ribbon_tab {
                RibbonTab::Home => "home".to_owned(),
                RibbonTab::File => "file".to_owned(),
                RibbonTab::View => "view".to_owned(),
                RibbonTab::Plugins => "plugins".to_owned(),
                RibbonTab::Plugin(id) => format!("plugin:{id}"),
                RibbonTab::Extra => "extra".to_owned(),
            }
        ));
        for (index, color) in self.custom_colors.iter().enumerate() {
            if let Some(color) = color {
                lines.push(format!("custom_color|{index}|{}", format_color(*color)));
            }
        }
        for (index, color) in self.recent_colors.iter().enumerate() {
            if let Some(color) = color {
                lines.push(format!("recent_color|{index}|{}", format_color(*color)));
            }
        }
        for (index, path) in self.recent_files.iter().take(10).enumerate() {
            lines.push(format!("recent_file|{index}|{}", path.display()));
        }
        for key in &self.disabled_tools {
            lines.push(format!("tool|{key}"));
        }
        for key in &self.disabled_cursor_tools {
            lines.push(format!("cursor_tool|{key}"));
        }
        for key in &self.disabled_brushes {
            lines.push(format!("brush|{key}"));
        }
        for key in &self.disabled_shapes {
            lines.push(format!("shape|{key}"));
        }
        for key in &self.disabled_panels {
            lines.push(format!("panel|{key}"));
        }
        for key in &self.disabled_app_panels {
            lines.push(format!("app_panel|{key}"));
        }
        for key in &self.disabled_hooks {
            lines.push(format!("hook|{key}"));
        }
        for key in &self.disabled_plugins {
            lines.push(format!("plugin|{key}"));
        }
        lines.sort();
        let _ = fs::write(APP_STATE_FILE, lines.join("\n"));
    }

    fn apply_persisted_setting(&mut self, key: &str, value: &str) {
        match key {
            "language" => match value {
                "en-US" => self.language = Language::EnUs(EnUs),
                "zh-CN" => self.language = Language::ZhCnSimple(ZhCnSimple),
                _ => {}
            },
            "canvas_width" => {
                if let Ok(width) = value.parse::<usize>() {
                    self.canvas.resize(
                        width.clamp(
                            crate::constants::MIN_CANVAS_SIDE,
                            crate::constants::MAX_CANVAS_SIDE,
                        ),
                        self.canvas.height,
                        Color32::WHITE,
                    );
                }
            }
            "canvas_height" => {
                if let Ok(height) = value.parse::<usize>() {
                    self.canvas.resize(
                        self.canvas.width,
                        height.clamp(
                            crate::constants::MIN_CANVAS_SIDE,
                            crate::constants::MAX_CANVAS_SIDE,
                        ),
                        Color32::WHITE,
                    );
                }
            }
            "zoom" => {
                if let Ok(zoom) = value.parse::<f32>() {
                    self.zoom = zoom.clamp(0.05, 8.0);
                }
            }
            "pan_x" => {
                if let Ok(pan_x) = value.parse::<f32>() {
                    self.canvas_pan.x = pan_x;
                }
            }
            "pan_y" => {
                if let Ok(pan_y) = value.parse::<f32>() {
                    self.canvas_pan.y = pan_y;
                }
            }
            "show_grid" => self.show_grid = parse_bool(value, self.show_grid),
            "show_rulers" => self.show_rulers = parse_bool(value, self.show_rulers),
            "show_layers_panel" => {
                self.show_layers_panel = parse_bool(value, self.show_layers_panel);
            }
            "show_plugins_window" => {
                self.show_plugins_window = parse_bool(value, self.show_plugins_window);
            }
            "primary" => {
                if let Some(color) = parse_hex_color(value) {
                    self.primary = color;
                }
            }
            "secondary" => {
                if let Some(color) = parse_hex_color(value) {
                    self.secondary = color;
                }
            }
            "brush_size" => {
                if let Ok(size) = value.parse::<i32>() {
                    self.brush_size = size.clamp(1, 128);
                }
            }
            "shape_mode" => match value {
                "outline" => self.shape_mode = ShapeMode::Outline,
                "filled" => self.shape_mode = ShapeMode::Filled,
                "filled_outline" => self.shape_mode = ShapeMode::FilledOutline,
                _ => {}
            },
            "color_edit_mode" => match value {
                "rgb" => self.color_edit_mode = ColorEditMode::Rgb,
                "hsv" => self.color_edit_mode = ColorEditMode::Hsv,
                _ => {}
            },
            "ribbon_tab" => {
                self.ribbon_tab = match value {
                    "home" => RibbonTab::Home,
                    "file" => RibbonTab::File,
                    "view" => RibbonTab::View,
                    "plugins" => RibbonTab::Plugins,
                    "extra" => RibbonTab::Extra,
                    plugin if plugin.starts_with("plugin:") => {
                        let id = plugin.trim_start_matches("plugin:");
                        RibbonTab::Plugin(Box::leak(id.to_owned().into_boxed_str()))
                    }
                    _ => self.ribbon_tab,
                };
            }
            _ => {}
        }
    }

    fn ensure_canvas_size_constraints(&mut self) {
        let width = self.canvas.width.clamp(
            crate::constants::MIN_CANVAS_SIDE,
            crate::constants::MAX_CANVAS_SIDE,
        );
        let height = self.canvas.height.clamp(
            crate::constants::MIN_CANVAS_SIDE,
            crate::constants::MAX_CANVAS_SIDE,
        );
        if width != self.canvas.width || height != self.canvas.height {
            self.canvas.resize(width, height, Color32::WHITE);
        }
    }

    fn tr(&self, text: LanguageText) -> String {
        self.language.get_text(text)
    }

    fn active_canvas(&self) -> &Canvas {
        if self.active_layer == 0 {
            &self.canvas
        } else {
            &self.pixel_layers[self.active_layer - 1].canvas
        }
    }

    fn active_canvas_mut(&mut self) -> &mut Canvas {
        if self.active_layer == 0 {
            &mut self.canvas
        } else {
            &mut self.pixel_layers[self.active_layer - 1].canvas
        }
    }

    fn dispatch_active_tool_event(&mut self, event: CanvasToolEvent) -> bool {
        let tool_slot = self.active_tool_slot;
        let cursor_slot = self.active_cursor_tool_slot;
        if let Some(slot) = tool_slot {
            if self.disabled_tools.contains(&self.tool_key(slot))
                || slot >= self.tools.len()
                || !self.tools[slot].wants_canvas_events()
            {
                return false;
            }
            let mut tools = std::mem::take(&mut self.tools);
            let consumed = self.dispatch_tool_event_to(tools[slot].as_mut(), event);
            self.tools = tools;
            return consumed;
        }

        if let Some(slot) = cursor_slot {
            if self
                .disabled_cursor_tools
                .contains(&self.cursor_tool_key(slot))
                || slot >= self.cursor_tools.len()
                || !self.cursor_tools[slot].wants_canvas_events()
            {
                return false;
            }
            let mut tools = std::mem::take(&mut self.cursor_tools);
            let consumed = self.dispatch_tool_event_to(tools[slot].as_mut(), event);
            self.cursor_tools = tools;
            return consumed;
        }

        false
    }

    fn dispatch_tool_event_to(&mut self, tool: &mut dyn Tool, event: CanvasToolEvent) -> bool {
        let language = self.language.clone();
        let primary = self.primary;
        let secondary = self.secondary;
        let brush_size = self.brush_size;
        let zoom = self.zoom;
        let pan = self.canvas_pan;
        let active_layer = self.active_layer;
        let layer_count = self.pixel_layers.len() + 1;
        let mut context = CanvasToolContext {
            canvas: self.active_canvas_mut(),
            language: &language,
            primary,
            secondary,
            brush_size,
            zoom,
            pan,
            active_layer,
            layer_count,
            commands: Vec::new(),
        };
        let consumed = tool.on_canvas_event(event, &mut context);
        let commands = context.commands;
        self.apply_commands(commands);
        consumed
    }

    fn paint_active_tool_overlay(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let tool_slot = self.active_tool_slot;
        let cursor_slot = self.active_cursor_tool_slot;
        if let Some(slot) = tool_slot {
            if self.disabled_tools.contains(&self.tool_key(slot))
                || slot >= self.tools.len()
                || !self.tools[slot].wants_canvas_events()
            {
                return;
            }
            let mut tools = std::mem::take(&mut self.tools);
            self.paint_tool_overlay_with(tools[slot].as_mut(), ui, canvas_rect);
            self.tools = tools;
            return;
        }

        if let Some(slot) = cursor_slot {
            if self
                .disabled_cursor_tools
                .contains(&self.cursor_tool_key(slot))
                || slot >= self.cursor_tools.len()
                || !self.cursor_tools[slot].wants_canvas_events()
            {
                return;
            }
            let mut tools = std::mem::take(&mut self.cursor_tools);
            self.paint_tool_overlay_with(tools[slot].as_mut(), ui, canvas_rect);
            self.cursor_tools = tools;
        }
    }

    fn active_tool_canvas_context_menu(&mut self, response: &egui::Response) {
        let tool_slot = self.active_tool_slot;
        let cursor_slot = self.active_cursor_tool_slot;
        if let Some(slot) = tool_slot {
            if self.disabled_tools.contains(&self.tool_key(slot))
                || slot >= self.tools.len()
                || !self.tools[slot].has_canvas_context_menu()
            {
                return;
            }
            let mut tools = std::mem::take(&mut self.tools);
            response.context_menu(|ui| {
                self.tool_canvas_context_menu_with(tools[slot].as_mut(), ui);
            });
            self.tools = tools;
            return;
        }

        if let Some(slot) = cursor_slot {
            if self
                .disabled_cursor_tools
                .contains(&self.cursor_tool_key(slot))
                || slot >= self.cursor_tools.len()
                || !self.cursor_tools[slot].has_canvas_context_menu()
            {
                return;
            }
            let mut tools = std::mem::take(&mut self.cursor_tools);
            response.context_menu(|ui| {
                self.tool_canvas_context_menu_with(tools[slot].as_mut(), ui);
            });
            self.cursor_tools = tools;
        }
    }

    fn active_tool_window(&mut self, ctx: &Context) {
        let tool_slot = self.active_tool_slot;
        let cursor_slot = self.active_cursor_tool_slot;
        if let Some(slot) = tool_slot {
            if self.disabled_tools.contains(&self.tool_key(slot))
                || slot >= self.tools.len()
                || !self.tools[slot].has_tool_window()
            {
                return;
            }
            let mut tools = std::mem::take(&mut self.tools);
            self.tool_window_with(tools[slot].as_mut(), ctx);
            self.tools = tools;
            return;
        }

        if let Some(slot) = cursor_slot {
            if self
                .disabled_cursor_tools
                .contains(&self.cursor_tool_key(slot))
                || slot >= self.cursor_tools.len()
                || !self.cursor_tools[slot].has_tool_window()
            {
                return;
            }
            let mut tools = std::mem::take(&mut self.cursor_tools);
            self.tool_window_with(tools[slot].as_mut(), ctx);
            self.cursor_tools = tools;
        }
    }

    fn paint_tool_overlay_with(
        &mut self,
        tool: &mut dyn Tool,
        ui: &mut egui::Ui,
        canvas_rect: Rect,
    ) {
        let language = self.language.clone();
        let primary = self.primary;
        let secondary = self.secondary;
        let brush_size = self.brush_size;
        let zoom = self.zoom;
        let pan = self.canvas_pan;
        let active_layer = self.active_layer;
        let layer_count = self.pixel_layers.len() + 1;
        let mut context = CanvasToolContext {
            canvas: self.active_canvas_mut(),
            language: &language,
            primary,
            secondary,
            brush_size,
            zoom,
            pan,
            active_layer,
            layer_count,
            commands: Vec::new(),
        };
        tool.paint_canvas_overlay(ui, canvas_rect, &mut context);
        let commands = context.commands;
        self.apply_commands(commands);
    }

    fn tool_canvas_context_menu_with(&mut self, tool: &mut dyn Tool, ui: &mut egui::Ui) {
        let language = self.language.clone();
        let primary = self.primary;
        let secondary = self.secondary;
        let brush_size = self.brush_size;
        let zoom = self.zoom;
        let pan = self.canvas_pan;
        let active_layer = self.active_layer;
        let layer_count = self.pixel_layers.len() + 1;
        let mut context = CanvasToolContext {
            canvas: self.active_canvas_mut(),
            language: &language,
            primary,
            secondary,
            brush_size,
            zoom,
            pan,
            active_layer,
            layer_count,
            commands: Vec::new(),
        };
        tool.canvas_context_menu(ui, &mut context);
        let commands = context.commands;
        self.apply_commands(commands);
    }

    fn tool_window_with(&mut self, tool: &mut dyn Tool, ctx: &Context) {
        let language = self.language.clone();
        let primary = self.primary;
        let secondary = self.secondary;
        let brush_size = self.brush_size;
        let zoom = self.zoom;
        let pan = self.canvas_pan;
        let active_layer = self.active_layer;
        let layer_count = self.pixel_layers.len() + 1;
        let mut context = CanvasToolContext {
            canvas: self.active_canvas_mut(),
            language: &language,
            primary,
            secondary,
            brush_size,
            zoom,
            pan,
            active_layer,
            layer_count,
            commands: Vec::new(),
        };
        tool.tool_window(ctx, &mut context);
        let commands = context.commands;
        self.apply_commands(commands);
    }

    fn active_shape_kind(&self) -> Option<ShapeKind> {
        self.shape_group
            .active_shape()
            .map(|shape| shape.get_shape_kind())
    }

    fn clear_transient_selection_state(&mut self) {
        self.selected_rect = None;
        self.moving_selection = None;
        self.resizing_selection = None;
        self.active_shape_rect = None;
        self.active_shape_snapshot = None;
        self.moving_shape = None;
        self.moving_text_box = None;
        self.curve_draft = None;
    }

    fn commit_active_selection(&mut self) -> bool {
        let mut changed = false;
        if let Some(selection) = self.moving_selection.take() {
            let position = selection.position;
            let transparent = self.transparent_selection;
            let transparent_color = self.secondary;
            self.paste_selection_region(
                position,
                &selection.content.region,
                transparent,
                transparent_color,
            );
            self.paste_selection_text_items(position, &selection.content.text_items);
            self.mark_canvas_dirty();
            changed = true;
        }
        if self.selected_rect.take().is_some() {
            changed = true;
        }
        self.resizing_selection = None;
        if changed {
            self.emit_event(AppEvent::SelectionChanged);
        }
        changed
    }

    fn paste_selection_region(
        &mut self,
        position: (i32, i32),
        region: &CanvasRegion,
        transparent: bool,
        transparent_color: Color32,
    ) {
        if !transparent {
            self.active_canvas_mut()
                .paste_region(position.0, position.1, region);
            return;
        }
        let canvas = self.active_canvas_mut();
        for y in 0..region.height {
            for x in 0..region.width {
                let color = region.pixels[y * region.width + x];
                if color == transparent_color {
                    continue;
                }
                canvas.set_pixel(position.0 + x as i32, position.1 + y as i32, color);
            }
        }
    }

    fn paste_selection_text_items(&mut self, position: (i32, i32), text_items: &[TextItem]) {
        let active_layer = self.active_layer;
        self.text_items
            .extend(text_items.iter().cloned().map(|mut item| {
                item.layer = active_layer;
                item.position = (position.0 + item.position.0, position.1 + item.position.1);
                item
            }));
    }

    fn mark_canvas_dirty(&mut self) {
        self.dirty_texture = true;
        self.document_dirty = true;
        self.emit_event(AppEvent::CanvasDirty);
    }

    fn document_snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            canvas: self.canvas.snapshot(),
            pixel_layers: self.pixel_layers.clone(),
            active_layer: self.active_layer,
            show_layers_panel: self.show_layers_panel,
            text_items: self.text_items.clone(),
        }
    }

    fn restore_document_snapshot(&mut self, snapshot: DocumentSnapshot) {
        self.canvas.restore(snapshot.canvas);
        self.pixel_layers = snapshot.pixel_layers;
        self.active_layer = snapshot.active_layer.min(self.pixel_layers.len());
        self.show_layers_panel = snapshot.show_layers_panel;
        self.text_items = snapshot.text_items;
        self.active_text_box = None;
        self.moving_text_box = None;
        self.curve_draft = None;
        self.clear_transient_selection_state();
        self.dirty_texture = true;
        self.document_dirty = true;
    }

    fn push_undo_snapshot(&mut self) {
        self.undo_stack.push(self.document_snapshot());
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.emit_event(AppEvent::HistorySnapshotPushed);
    }

    fn undo_action(&mut self) -> bool {
        let Some(snapshot) = self.undo_stack.pop() else {
            return false;
        };
        let current = self.document_snapshot();
        self.redo_stack.push(current);
        self.restore_document_snapshot(snapshot);
        self.status = self.tr(LanguageText::UndoDone);
        self.emit_event(AppEvent::Undo);
        true
    }

    fn redo_action(&mut self) -> bool {
        let Some(snapshot) = self.redo_stack.pop() else {
            return false;
        };
        let current = self.document_snapshot();
        self.undo_stack.push(current);
        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.restore_document_snapshot(snapshot);
        self.status = self.tr(LanguageText::RedoDone);
        self.emit_event(AppEvent::Redo);
        true
    }

    fn clear_history_stack(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.emit_event(AppEvent::HistoryCleared);
    }

    fn document_title(&self) -> String {
        let name = self
            .save_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_owned)
            .unwrap_or_else(|| self.tr(LanguageText::Untitled));
        if self.document_dirty {
            format!("{name} *")
        } else {
            name
        }
    }

    fn confirm_discard_changes(&self) -> bool {
        if !self.document_dirty {
            return true;
        }
        matches!(
            rfd::MessageDialog::new()
                .set_title(self.tr(LanguageText::DiscardChangesTitle))
                .set_description(self.tr(LanguageText::DiscardChangesMessage))
                .set_buttons(rfd::MessageButtons::YesNo)
                .show(),
            rfd::MessageDialogResult::Yes
        )
    }

    fn composited_pixels(&self) -> Vec<Color32> {
        let mut pixels = self.canvas.pixels.clone();
        for layer in &self.pixel_layers {
            if layer.visible {
                composite_layer(
                    &mut pixels,
                    &layer.canvas.pixels,
                    layer.opacity,
                    layer.blend_mode,
                );
            }
        }
        pixels
    }

    fn export_pixels(&self) -> Vec<Color32> {
        let mut pixels = self.composited_pixels();
        self.render_text_items_to_pixels(&mut pixels);
        pixels
    }

    fn upload_texture(&mut self, ctx: &Context) {
        if !self.dirty_texture {
            return;
        }

        let image = egui::ColorImage {
            size: [self.canvas.width, self.canvas.height],
            pixels: self.composited_pixels(),
        };

        if let Some(texture) = &mut self.texture {
            texture.set(image, TextureOptions::NEAREST);
        } else {
            self.texture = Some(ctx.load_texture("canvas", image, TextureOptions::NEAREST));
        }

        self.dirty_texture = false;
    }

    fn top_bar(&mut self, ctx: &Context) {
        let top_bar_height = self.top_bar_height();
        let has_body = self.active_ribbon_has_body();
        egui::TopBottomPanel::top("ribbon")
            .exact_height(top_bar_height)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                egui::ScrollArea::horizontal()
                    .id_source("ribbon_scroll")
                    .auto_shrink([false, false])
                    .max_height(top_bar_height - 8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(format!("LayDraw - {}", self.document_title()));
                            ui.separator();
                            let home = self.tr(LanguageText::HomeTab);
                            let file = self.tr(LanguageText::FileTab);
                            let view = self.tr(LanguageText::ViewTab);
                            let plugins = self.tr(LanguageText::PluginsTab);
                            ui.selectable_value(&mut self.ribbon_tab, RibbonTab::Home, home);
                            ui.selectable_value(&mut self.ribbon_tab, RibbonTab::File, file);
                            ui.selectable_value(&mut self.ribbon_tab, RibbonTab::View, view);
                            if ui
                                .selectable_label(self.show_plugins_window, plugins)
                                .clicked()
                            {
                                self.show_plugins_window = true;
                            }
                            for (index, panel) in self.panels.iter().enumerate() {
                                if self.disabled_panels.contains(&self.panel_key(index)) {
                                    continue;
                                }
                                if let PanelArea::RibbonTab(id) = panel.panel_area() {
                                    let title = panel.panel_title(&self.language);
                                    ui.selectable_value(
                                        &mut self.ribbon_tab,
                                        RibbonTab::Plugin(id),
                                        title,
                                    );
                                }
                            }
                            ui.separator();
                            egui::ComboBox::from_id_source("language_switch")
                                .width(116.0)
                                .selected_text(self.language.get_name())
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_label(
                                            matches!(self.language, Language::EnUs(_)),
                                            "English(US)",
                                        )
                                        .clicked()
                                    {
                                        self.language = Language::EnUs(EnUs);
                                        if !self.active_tool_is_enabled() {
                                            self.status = self.tr(LanguageText::NoToolActive);
                                        }
                                        self.emit_event(AppEvent::LanguageChanged);
                                    }
                                    if ui
                                        .selectable_label(
                                            matches!(self.language, Language::ZhCnSimple(_)),
                                            "简体中文",
                                        )
                                        .clicked()
                                    {
                                        self.language = Language::ZhCnSimple(ZhCnSimple);
                                        if !self.active_tool_is_enabled() {
                                            self.status = self.tr(LanguageText::NoToolActive);
                                        }
                                        self.emit_event(AppEvent::LanguageChanged);
                                    }
                                });
                        });

                        if has_body {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| match self.ribbon_tab {
                                RibbonTab::File => self.file_ribbon(ui),
                                RibbonTab::Home => self.home_ribbon(ui),
                                RibbonTab::View => self.view_ribbon(ui),
                                RibbonTab::Plugins => {}
                                RibbonTab::Plugin(id) => self.plugin_ribbon(ui, id),
                                RibbonTab::Extra => {}
                            });
                        }
                    });
                ui.add_space(4.0);
            });
    }

    fn file_ribbon(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button(self.tr(LanguageText::New)).clicked() {
                if self.confirm_discard_changes() {
                    self.new_document();
                }
            }
            if ui.button(self.tr(LanguageText::Open)).clicked() {
                if self.confirm_discard_changes() {
                    self.open_image();
                }
            }
            if ui.button(self.tr(LanguageText::ImportImage)).clicked() {
                self.import_image();
            }
            if ui.button(self.tr(LanguageText::Save)).clicked() {
                self.save_image();
            }
            if ui.button(self.tr(LanguageText::SaveAs)).clicked() {
                self.save_image_as();
            }
            if !self.recent_files.is_empty() {
                ui.menu_button(self.tr(LanguageText::RecentFiles), |ui| {
                    for path in self.recent_files.clone() {
                        if ui.button(display_path_label(&path)).clicked() {
                            if self.confirm_discard_changes() {
                                self.open_image_path(path);
                            }
                            ui.close_menu();
                        }
                    }
                });
            }
        });
    }

    fn home_ribbon(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.render_app_panels(ui, AppPanelArea::Home);
            self.plugin_top_panels(ui);
        });
    }

    pub(crate) fn tools_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(160.0);
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Tools));
            let mut shown_tool = false;
            let visible_tool_count = self
                .tools
                .iter()
                .enumerate()
                .filter(|(index, tool)| {
                    let tool_key = Self::component_key(
                        self.tool_sources
                            .get(*index)
                            .copied()
                            .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                        tool.tool_id(),
                    );
                    !self.disabled_tools.contains(&tool_key)
                })
                .count()
                + self
                    .cursor_tools
                    .iter()
                    .enumerate()
                    .filter(|(index, tool)| {
                        let tool_key = Self::component_key(
                            self.cursor_tool_sources
                                .get(*index)
                                .copied()
                                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                            tool.tool_id(),
                        );
                        !self.disabled_cursor_tools.contains(&tool_key)
                    })
                    .count();
            let tool_columns = visible_tool_count.div_ceil(3).max(3);
            let mut tool_commands = Vec::new();
            egui::Grid::new("tools_group")
                .num_columns(tool_columns)
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    let mut selected = None;
                    let mut visible_index = 0;
                    for (index, tool) in self.tools.iter_mut().enumerate() {
                        let tool_key = Self::component_key(
                            self.tool_sources
                                .get(index)
                                .copied()
                                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                            tool.tool_id(),
                        );
                        if self.disabled_tools.contains(&tool_key) {
                            continue;
                        }
                        shown_tool = true;
                        let kind = tool.get_tool_kind();
                        let response = tool.tool_button(
                            ui,
                            &self.language,
                            self.active_tool == kind
                                && self.active_tool_slot == Some(index)
                                && self.active_cursor_tool_slot.is_none(),
                        );
                        response.context_menu(|ui| {
                            let mut context = ToolUiContext {
                                language: &self.language,
                                active_tool: self.active_tool,
                                primary: self.primary,
                                secondary: self.secondary,
                                brush_size: self.brush_size,
                                active_layer: self.active_layer,
                                layer_count: self.pixel_layers.len() + 1,
                                selected_rect: self.selected_rect,
                                pointer_canvas_pos: self.pointer_canvas_pos,
                                zoom: self.zoom,
                                pan: self.canvas_pan,
                                commands: Vec::new(),
                            };
                            tool.tool_button_context_menu(ui, &mut context);
                            tool_commands.append(&mut context.commands);
                        });
                        if response.clicked() {
                            selected = Some(index);
                        }
                        visible_index += 1;
                        if visible_index % tool_columns == 0 {
                            ui.end_row();
                        }
                    }
                    let mut selected_cursor_tool = None;
                    for (index, tool) in self.cursor_tools.iter_mut().enumerate() {
                        let tool_key = Self::component_key(
                            self.cursor_tool_sources
                                .get(index)
                                .copied()
                                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                            tool.tool_id(),
                        );
                        if self.disabled_cursor_tools.contains(&tool_key) {
                            continue;
                        }
                        shown_tool = true;
                        let kind = tool.get_tool_kind();
                        let response = tool.tool_button(
                            ui,
                            &self.language,
                            self.active_tool == kind && self.active_cursor_tool_slot == Some(index),
                        );
                        response.context_menu(|ui| {
                            let mut context = ToolUiContext {
                                language: &self.language,
                                active_tool: self.active_tool,
                                primary: self.primary,
                                secondary: self.secondary,
                                brush_size: self.brush_size,
                                active_layer: self.active_layer,
                                layer_count: self.pixel_layers.len() + 1,
                                selected_rect: self.selected_rect,
                                pointer_canvas_pos: self.pointer_canvas_pos,
                                zoom: self.zoom,
                                pan: self.canvas_pan,
                                commands: Vec::new(),
                            };
                            tool.tool_button_context_menu(ui, &mut context);
                            tool_commands.append(&mut context.commands);
                        });
                        if response.clicked() {
                            selected = None;
                            selected_cursor_tool = Some((index, kind));
                        }
                        visible_index += 1;
                        if visible_index % tool_columns == 0 {
                            ui.end_row();
                        }
                    }
                    if let Some(index) = selected {
                        self.set_active_tool_with_slot(
                            self.tools[index].get_tool_kind(),
                            Some(index),
                            None,
                        );
                    } else if let Some((index, kind)) = selected_cursor_tool {
                        self.set_active_tool_with_slot(kind, None, Some(index));
                    }
                });
            self.apply_commands(tool_commands);
            if !shown_tool {
                ui.label(self.tr(LanguageText::NoToolActive));
            }
        });
    }

    pub(crate) fn handle_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(310.0);
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Handle));
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(
                        !self.undo_stack.is_empty(),
                        egui::Button::new(self.tr(LanguageText::Undo)),
                    )
                    .clicked()
                {
                    self.undo_action();
                }
                if ui
                    .add_enabled(
                        !self.redo_stack.is_empty(),
                        egui::Button::new(self.tr(LanguageText::Redo)),
                    )
                    .clicked()
                {
                    self.redo_action();
                }
                if ui.button(self.tr(LanguageText::FlipHorizontal)).clicked() {
                    self.push_undo_snapshot();
                    self.active_canvas_mut().flip_horizontal();
                    self.mark_canvas_dirty();
                }
                if ui.button(self.tr(LanguageText::FlipVertical)).clicked() {
                    self.push_undo_snapshot();
                    self.active_canvas_mut().flip_vertical();
                    self.mark_canvas_dirty();
                }
            });
            ui.horizontal(|ui| {
                let has_selection = self.selected_rect.is_some();
                let has_clipboard = self.selection_clipboard.is_some();
                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(self.tr(LanguageText::SelectionCopy)),
                    )
                    .clicked()
                {
                    self.copy_selection();
                }
                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(self.tr(LanguageText::SelectionCut)),
                    )
                    .clicked()
                {
                    self.cut_selection();
                }
                if ui
                    .add_enabled(
                        has_clipboard,
                        egui::Button::new(self.tr(LanguageText::SelectionPaste)),
                    )
                    .clicked()
                {
                    self.paste_selection();
                }
            });
            ui.horizontal(|ui| {
                let has_selection = self.selected_rect.is_some();
                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(self.tr(LanguageText::SelectionCrop)),
                    )
                    .clicked()
                {
                    self.crop_selection();
                }
                if ui
                    .add_enabled(
                        has_selection,
                        egui::Button::new(self.tr(LanguageText::SelectionDelete)),
                    )
                    .clicked()
                {
                    self.delete_selection();
                }
                let transparent = self.tr(LanguageText::TransparentSelection);
                ui.checkbox(&mut self.transparent_selection, transparent);
            });
        });
    }

    pub(crate) fn size_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(132.0);
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Size));
            let old_size = self.brush_size;
            ui.add(
                egui::Slider::new(&mut self.brush_size, 1..=128)
                    .show_value(true)
                    .clamp_to_range(true),
            );
            if self.brush_size != old_size {
                self.emit_event(AppEvent::BrushSizeChanged {
                    size: self.brush_size,
                });
            }
        });
    }

    pub(crate) fn outline_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(96.0);
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Outline));
            for mode in ShapeMode::ALL {
                ui.selectable_value(
                    &mut self.shape_mode,
                    mode,
                    self.language.get_text(mode.get_label()),
                );
            }
        });
    }

    pub(crate) fn brushes_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(148.0);
        self.ensure_active_brush_enabled();
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Brushes));
            let selected_text = self
                .brush_group
                .active_brush()
                .map(|brush| {
                    self.language
                        .get_text(brush.get_brush_label(&self.language))
                })
                .unwrap_or_else(|| self.tr(LanguageText::Brushes));

            egui::ComboBox::from_id_source("brush_kind")
                .width(136.0)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    let mut selected = None;
                    for (index, brush) in self.brush_group.brushes().iter().enumerate() {
                        if self.disabled_brushes.contains(&self.brush_key(index)) {
                            continue;
                        }
                        let label = self
                            .language
                            .get_text(brush.get_brush_label(&self.language));
                        if ui
                            .selectable_label(self.brush_group.active_index() == index, label)
                            .clicked()
                        {
                            selected = Some(index);
                        }
                    }
                    if let Some(index) = selected {
                        self.brush_group.select(index);
                    }
                });

            let (preview_rect, _) =
                ui.allocate_exact_size(egui::vec2(136.0, 30.0), egui::Sense::hover());
            if let Some(brush) = self.brush_group.active_brush_mut() {
                crate::tools::brush::paint_live_brush_preview(
                    brush,
                    ui,
                    preview_rect.shrink(4.0),
                    self.primary,
                    self.brush_size,
                );
            }
        });
    }

    pub(crate) fn shapes_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(350.0);
        self.ensure_active_shape_enabled();
        let mut changed = false;
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Shapes));
            egui::Frame::default()
                .fill(egui::Color32::WHITE)
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(218, 222, 228),
                ))
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                .show(ui, |ui| {
                    egui::Grid::new("shape_palette")
                        .num_columns(8)
                        .spacing([2.0, 2.0])
                        .show(ui, |ui| {
                            let mut clicked = None;
                            let active = self.shape_group.active_index();
                            let disabled: Vec<bool> = (0..self.shape_group.shapes().len())
                                .map(|index| self.disabled_shapes.contains(&self.shape_key(index)))
                                .collect();
                            for (index, shape) in
                                self.shape_group.shapes_mut().iter_mut().enumerate()
                            {
                                if disabled[index] {
                                    continue;
                                }
                                if shape
                                    .shape_button(ui, &self.language, active == Some(index))
                                    .clicked()
                                {
                                    clicked = Some(index);
                                }
                                if (index + 1) % 8 == 0 {
                                    ui.end_row();
                                }
                            }
                            if let Some(index) = clicked {
                                self.shape_group.select(index);
                                changed = true;
                            }
                        });
                });
        });
        if changed {
            self.set_active_tool(ToolKind::Shape);
        }
    }

    pub(crate) fn color_panel(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(330.0);
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Colors));
            ui.horizontal(|ui| {
                if color_swatch(ui, self.primary, true).clicked() {
                    self.open_color_editor(true);
                }
                if color_swatch(ui, self.secondary, false).clicked() {
                    self.open_color_editor(false);
                }
                ui.vertical(|ui| {
                    for row in BASIC_COLORS.chunks(10).take(2) {
                        ui.horizontal(|ui| {
                            for color in row {
                                if mini_color_dot(ui, *color).clicked() {
                                    self.primary = *color;
                                    self.remember_recent_color(*color);
                                    self.emit_event(AppEvent::ColorChanged {
                                        primary: self.primary,
                                        secondary: self.secondary,
                                    });
                                }
                            }
                        });
                    }
                });
                let recent_colors: Vec<Color32> =
                    self.recent_colors.iter().flatten().copied().collect();
                if !recent_colors.is_empty() {
                    ui.vertical(|ui| {
                        ui.label(self.tr(LanguageText::RecentColors));
                        ui.horizontal(|ui| {
                            for color in recent_colors {
                                if mini_color_dot(ui, color).clicked() {
                                    self.primary = color;
                                    self.remember_recent_color(color);
                                    self.emit_event(AppEvent::ColorChanged {
                                        primary: self.primary,
                                        secondary: self.secondary,
                                    });
                                }
                            }
                        });
                    });
                }
                if ui.button("+").clicked() {
                    self.open_color_editor(true);
                }
            });
            if ui.button(self.tr(LanguageText::Swap)).clicked() {
                std::mem::swap(&mut self.primary, &mut self.secondary);
                self.remember_recent_color(self.primary);
                self.remember_recent_color(self.secondary);
                self.emit_event(AppEvent::ColorChanged {
                    primary: self.primary,
                    secondary: self.secondary,
                });
            }
        });
    }

    fn open_color_editor(&mut self, primary: bool) {
        self.editing_primary_color = primary;
        self.editor_color = if primary {
            self.primary
        } else {
            self.secondary
        };
        let (h, s, v) = rgb_to_hsv(self.editor_color);
        self.editor_hue = h;
        self.editor_saturation = s;
        self.editor_value = v;
        self.show_color_editor = true;
    }

    fn view_ribbon(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.render_app_panels(ui, AppPanelArea::View);
        });
    }

    pub(crate) fn view_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(self.tr(LanguageText::Zoom));
            let mut zoom = self.zoom;
            if ui
                .add(egui::Slider::new(&mut zoom, 0.05..=8.0).logarithmic(true))
                .changed()
            {
                self.set_view(zoom, self.canvas_pan);
            }
            if ui.button("100%").clicked() {
                self.set_view(1.0, self.canvas_pan);
            }
            if ui.button(self.tr(LanguageText::Fit)).clicked() {
                self.set_view(0.5, Vec2::new(40.0, 40.0));
            }
            ui.label("X");
            let mut pan_x = self.canvas_pan.x;
            let pan_x_changed = ui.add(egui::DragValue::new(&mut pan_x).speed(4)).changed();
            ui.label("Y");
            let mut pan_y = self.canvas_pan.y;
            let pan_y_changed = ui.add(egui::DragValue::new(&mut pan_y).speed(4)).changed();
            if pan_x_changed || pan_y_changed {
                self.set_view(self.zoom, Vec2::new(pan_x, pan_y));
            }
            if ui.button("Reset pan").clicked() {
                self.set_view(self.zoom, Vec2::new(40.0, 40.0));
            }
            ui.separator();
            let grid = self.tr(LanguageText::Grid);
            ui.checkbox(&mut self.show_grid, grid);
            let rulers = self.tr(LanguageText::Rulers);
            ui.checkbox(&mut self.show_rulers, rulers);
            let layers = self.tr(LanguageText::Layers);
            ui.checkbox(&mut self.show_layers_panel, layers);

            ui.separator();
            ui.label(self.tr(LanguageText::Canvas));
            let mut width = self.canvas.width as u32;
            let mut height = self.canvas.height as u32;
            let width_changed = ui
                .add(egui::DragValue::new(&mut width).speed(8).range(
                    crate::constants::MIN_CANVAS_SIDE as u32
                        ..=crate::constants::MAX_CANVAS_SIDE as u32,
                ))
                .changed();
            ui.label("x");
            let height_changed = ui
                .add(egui::DragValue::new(&mut height).speed(8).range(
                    crate::constants::MIN_CANVAS_SIDE as u32
                        ..=crate::constants::MAX_CANVAS_SIDE as u32,
                ))
                .changed();
            if width_changed || height_changed {
                self.resize_canvas(width as usize, height as usize);
            }
        });
    }

    fn plugins_window(&mut self, ctx: &Context) {
        if !self.show_plugins_window {
            return;
        }

        let mut open = self.show_plugins_window;
        egui::Window::new(self.tr(LanguageText::PluginsTab))
            .id(egui::Id::new("plugins_component_window"))
            .open(&mut open)
            .resizable(true)
            .default_size(Vec2::new(900.0, 520.0))
            .min_size(Vec2::new(520.0, 320.0))
            .show(ctx, |ui| {
                egui::ScrollArea::both()
                    .id_source("plugins_component_window_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(760.0);
                        self.plugins_page(ui);
                    });
            });
        self.show_plugins_window = open;
    }

    fn plugins_page(&mut self, ui: &mut egui::Ui) {
        self.plugin_component_manager(ui);
        let mut panels = std::mem::take(&mut self.panels);
        let mut commands = Vec::new();
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            let mut shown = false;
            for (index, panel) in panels.iter_mut().enumerate() {
                let panel_key = Self::component_key(
                    self.panel_sources
                        .get(index)
                        .copied()
                        .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                    panel.panel_id(),
                );
                if self.disabled_panels.contains(&panel_key) {
                    continue;
                }
                if panel.panel_area() == PanelArea::TopBar {
                    shown = true;
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(panel.panel_title(&self.language));
                        let mut context = self.panel_context();
                        panel.ui(ui, &mut context);
                        commands.append(&mut context.commands);
                    });
                }
            }
            if !shown {
                ui.label(self.tr(LanguageText::BuiltInPlugins));
            }
        });
        self.panels = panels;
        self.apply_commands(commands);
    }

    fn plugin_component_manager(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Components));
            for source in self.component_sources() {
                let mut enabled = self.component_source_enabled(source);
                egui::CollapsingHeader::new(self.plugin_source_title(source))
                    .default_open(source != BUILTIN_COMPONENT_SOURCE)
                    .show(ui, |ui| {
                        if self.plugin_source_index(source).is_some() {
                            self.plugin_metadata_ui(ui, source);
                            let mut plugin_enabled = self.plugin_source_enabled(source);
                            if ui
                                .checkbox(&mut plugin_enabled, self.tr(LanguageText::PluginActive))
                                .changed()
                            {
                                self.set_plugin_source_enabled(source, plugin_enabled);
                            }
                        }
                        if ui
                            .checkbox(&mut enabled, self.tr(LanguageText::EnableAllComponents))
                            .changed()
                        {
                            self.set_component_source_enabled(source, enabled);
                        }
                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            self.component_tool_toggles(ui, source);
                            ui.separator();
                            self.component_brush_toggles(ui, source);
                            ui.separator();
                            self.component_shape_toggles(ui, source);
                            ui.separator();
                            self.component_panel_toggles(ui, source);
                            ui.separator();
                            self.component_hook_toggles(ui, source);
                        });
                    });
            }
        });
    }

    fn plugin_metadata_ui(&self, ui: &mut egui::Ui, source: &'static str) {
        let Some(plugin) = self.plugin_for_source(source) else {
            return;
        };

        ui.horizontal_wrapped(|ui| {
            ui.strong(plugin.plugin_title(&self.language));
            let version = plugin.plugin_version();
            if !version.is_empty() {
                ui.label(format!("v{version}"));
            }
            let support = plugin.supported_laydraw_versions();
            if !support.is_empty() && support != "*" {
                ui.separator();
                ui.label(format!("LayDraw {support}"));
            }
            let author = plugin.plugin_author();
            if !author.is_empty() {
                ui.separator();
                ui.label(format!("Author: {author}"));
            }
            let url = plugin.plugin_url();
            if !url.is_empty() {
                ui.separator();
                ui.hyperlink_to(url, url);
            }
            let email = plugin.plugin_email();
            if !email.is_empty() {
                ui.separator();
                ui.hyperlink_to(email, format!("mailto:{email}"));
            }
        });
        ui.add_space(4.0);
    }

    fn component_sources(&self) -> Vec<&'static str> {
        let mut sources = Vec::new();
        for source in self.plugin_sources.iter().chain(
            self.app_panel_sources.iter().chain(
                self.tool_sources
                    .iter()
                    .chain(self.cursor_tool_sources.iter())
                    .chain(self.brush_sources.iter())
                    .chain(self.shape_sources.iter())
                    .chain(self.panel_sources.iter())
                    .chain(self.hook_sources.iter()),
            ),
        ) {
            if !sources.contains(source) {
                sources.push(*source);
            }
        }
        sources
    }

    fn component_source_enabled(&self, source: &'static str) -> bool {
        let plugin_enabled = self
            .plugin_source_index(source)
            .map(|_| self.plugin_source_enabled(source))
            .unwrap_or(true);
        let tools_enabled = self
            .tool_sources
            .iter()
            .enumerate()
            .filter(|(_, item_source)| **item_source == source)
            .all(|(index, _)| !self.disabled_tools.contains(&self.tool_key(index)));
        let cursor_tools_enabled = self
            .cursor_tool_sources
            .iter()
            .enumerate()
            .filter(|(_, item_source)| **item_source == source)
            .all(|(index, _)| {
                !self
                    .disabled_cursor_tools
                    .contains(&self.cursor_tool_key(index))
            });
        let panels_enabled = self
            .panels
            .iter()
            .enumerate()
            .zip(self.panel_sources.iter())
            .filter(|(_, item_source)| **item_source == source)
            .all(|((index, _), _)| !self.disabled_panels.contains(&self.panel_key(index)));
        let app_panels_enabled = self
            .app_panel_sources
            .iter()
            .enumerate()
            .filter(|(_, item_source)| **item_source == source)
            .all(|(index, _)| {
                !self
                    .disabled_app_panels
                    .contains(&self.app_panel_key(index))
            });
        let hooks_enabled = self
            .hook_sources
            .iter()
            .enumerate()
            .filter(|(_, item_source)| **item_source == source)
            .all(|(index, _)| !self.disabled_hooks.contains(&self.hook_key(index)));
        let brushes_enabled = self
            .brush_sources
            .iter()
            .enumerate()
            .filter(|(_, item_source)| **item_source == source)
            .all(|(index, _)| !self.disabled_brushes.contains(&self.brush_key(index)));
        let shapes_enabled = self
            .shape_sources
            .iter()
            .enumerate()
            .filter(|(_, item_source)| **item_source == source)
            .all(|(index, _)| !self.disabled_shapes.contains(&self.shape_key(index)));
        plugin_enabled
            && tools_enabled
            && cursor_tools_enabled
            && brushes_enabled
            && shapes_enabled
            && app_panels_enabled
            && panels_enabled
            && hooks_enabled
    }

    fn set_component_source_enabled(&mut self, source: &'static str, enabled: bool) {
        if enabled
            && self.plugin_source_index(source).is_some()
            && !self.plugin_source_enabled(source)
        {
            self.set_plugin_source_enabled(source, true);
        }
        for (index, item_source) in self.tool_sources.iter().enumerate() {
            if *item_source == source {
                let key = self.tool_key(index);
                if enabled {
                    self.disabled_tools.remove(&key);
                } else {
                    self.disabled_tools.insert(key);
                }
            }
        }
        for (index, item_source) in self.cursor_tool_sources.iter().enumerate() {
            if *item_source == source {
                let key = self.cursor_tool_key(index);
                if enabled {
                    self.disabled_cursor_tools.remove(&key);
                } else {
                    self.disabled_cursor_tools.insert(key);
                }
            }
        }
        for (index, item_source) in self.brush_sources.iter().enumerate() {
            if *item_source == source {
                let key = self.brush_key(index);
                if enabled {
                    self.disabled_brushes.remove(&key);
                } else {
                    self.disabled_brushes.insert(key);
                }
            }
        }
        for (index, item_source) in self.shape_sources.iter().enumerate() {
            if *item_source == source {
                let key = self.shape_key(index);
                if enabled {
                    self.disabled_shapes.remove(&key);
                } else {
                    self.disabled_shapes.insert(key);
                }
            }
        }
        for (panel, item_source) in self.panels.iter().zip(self.panel_sources.iter()) {
            if *item_source == source {
                let key = Self::component_key(*item_source, panel.panel_id());
                if enabled {
                    self.disabled_panels.remove(&key);
                } else {
                    self.disabled_panels.insert(key);
                }
            }
        }
        for (panel, item_source) in self.app_panels.iter().zip(self.app_panel_sources.iter()) {
            if *item_source == source {
                let key = Self::component_key(*item_source, panel.panel_id());
                if enabled {
                    self.disabled_app_panels.remove(&key);
                } else {
                    self.disabled_app_panels.insert(key);
                }
            }
        }
        for (index, item_source) in self.hook_sources.iter().enumerate() {
            if *item_source == source {
                let key = self.hook_key(index);
                if enabled {
                    self.disabled_hooks.remove(&key);
                } else {
                    self.disabled_hooks.insert(key);
                }
            }
        }
        self.ensure_active_tool_enabled();
        self.ensure_active_brush_enabled();
        self.ensure_active_shape_enabled();
        self.save_component_state();
    }

    fn component_tool_toggles(&mut self, ui: &mut egui::Ui, source: &'static str) {
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Tools));
            let tools: Vec<(usize, String)> = self
                .tools
                .iter()
                .enumerate()
                .filter(|(index, _)| self.tool_sources.get(*index).copied() == Some(source))
                .map(|(index, tool)| {
                    (
                        index,
                        self.language.get_text(tool.get_tool_label(&self.language)),
                    )
                })
                .chain(
                    self.cursor_tools
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| {
                            self.cursor_tool_sources.get(*index).copied() == Some(source)
                        })
                        .map(|(index, tool)| {
                            (
                                self.tools.len() + index,
                                self.language.get_text(tool.get_tool_label(&self.language)),
                            )
                        }),
                )
                .collect();
            if tools.is_empty() {
                ui.label("-");
                return;
            }
            egui::Grid::new(("plugin_tool_components", source))
                .num_columns(3)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    for (visible_index, (slot, label)) in tools.iter().enumerate() {
                        let is_cursor = *slot >= self.tools.len();
                        let local_slot = if is_cursor {
                            *slot - self.tools.len()
                        } else {
                            *slot
                        };
                        let key = if is_cursor {
                            self.cursor_tool_key(local_slot)
                        } else {
                            self.tool_key(local_slot)
                        };
                        let disabled = if is_cursor {
                            self.disabled_cursor_tools.contains(&key)
                        } else {
                            self.disabled_tools.contains(&key)
                        };
                        let mut enabled = !disabled;
                        if ui.checkbox(&mut enabled, label).changed() {
                            if is_cursor {
                                if enabled {
                                    self.disabled_cursor_tools.remove(&key);
                                } else {
                                    self.disabled_cursor_tools.insert(key);
                                }
                            } else if enabled {
                                self.disabled_tools.remove(&key);
                            } else {
                                self.disabled_tools.insert(key);
                            }
                            self.ensure_active_tool_enabled();
                            self.save_component_state();
                        }
                        if (visible_index + 1) % 3 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn component_panel_toggles(&mut self, ui: &mut egui::Ui, source: &'static str) {
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Panels));
            let panels: Vec<(String, String, bool)> = self
                .panels
                .iter()
                .enumerate()
                .zip(self.panel_sources.iter())
                .filter(|(_, item_source)| **item_source == source)
                .map(|((index, panel), _)| {
                    (
                        self.panel_key(index),
                        panel.panel_title(&self.language),
                        false,
                    )
                })
                .chain(
                    self.app_panels
                        .iter()
                        .enumerate()
                        .zip(self.app_panel_sources.iter())
                        .filter(|(_, item_source)| **item_source == source)
                        .map(|((index, panel), _)| {
                            (
                                self.app_panel_key(index),
                                panel.panel_id().replace('.', " "),
                                true,
                            )
                        }),
                )
                .collect();
            if panels.is_empty() {
                ui.label("-");
                return;
            }
            egui::Grid::new(("plugin_panel_components", source))
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    for (index, (key, title, is_app_panel)) in panels.iter().enumerate() {
                        let mut enabled = if *is_app_panel {
                            !self.disabled_app_panels.contains(key)
                        } else {
                            !self.disabled_panels.contains(key)
                        };
                        if ui.checkbox(&mut enabled, title).changed() {
                            if *is_app_panel {
                                if enabled {
                                    self.disabled_app_panels.remove(key);
                                } else {
                                    self.disabled_app_panels.insert(key.clone());
                                }
                            } else if enabled {
                                self.disabled_panels.remove(key);
                            } else {
                                self.disabled_panels.insert(key.clone());
                            }
                            self.save_component_state();
                        }
                        if (index + 1) % 2 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn component_brush_toggles(&mut self, ui: &mut egui::Ui, source: &'static str) {
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Brushes));
            let brushes: Vec<(usize, String)> = self
                .brush_group
                .brushes()
                .iter()
                .enumerate()
                .filter(|(index, _)| self.brush_sources.get(*index).copied() == Some(source))
                .map(|(index, brush)| {
                    (
                        index,
                        self.language
                            .get_text(brush.get_brush_label(&self.language)),
                    )
                })
                .collect();
            if brushes.is_empty() {
                ui.label("-");
                return;
            }
            egui::Grid::new(("plugin_brush_components", source))
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    for (visible_index, (index, label)) in brushes.iter().enumerate() {
                        let key = self.brush_key(*index);
                        let mut enabled = !self.disabled_brushes.contains(&key);
                        if ui.checkbox(&mut enabled, label).changed() {
                            if enabled {
                                self.disabled_brushes.remove(&key);
                            } else {
                                self.disabled_brushes.insert(key);
                            }
                            self.ensure_active_brush_enabled();
                            self.save_component_state();
                        }
                        if (visible_index + 1) % 2 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn component_shape_toggles(&mut self, ui: &mut egui::Ui, source: &'static str) {
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Shapes));
            let shapes: Vec<(usize, String)> = self
                .shape_group
                .shapes()
                .iter()
                .enumerate()
                .filter(|(index, _)| self.shape_sources.get(*index).copied() == Some(source))
                .map(|(index, shape)| {
                    (
                        index,
                        self.language
                            .get_text(shape.get_shape_label(&self.language)),
                    )
                })
                .collect();
            if shapes.is_empty() {
                ui.label("-");
                return;
            }
            egui::Grid::new(("plugin_shape_components", source))
                .num_columns(2)
                .spacing([8.0, 2.0])
                .show(ui, |ui| {
                    for (visible_index, (index, label)) in shapes.iter().enumerate() {
                        let key = self.shape_key(*index);
                        let mut enabled = !self.disabled_shapes.contains(&key);
                        if ui.checkbox(&mut enabled, label).changed() {
                            if enabled {
                                self.disabled_shapes.remove(&key);
                            } else {
                                self.disabled_shapes.insert(key);
                            }
                            self.ensure_active_shape_enabled();
                            self.save_component_state();
                        }
                        if (visible_index + 1) % 2 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn component_hook_toggles(&mut self, ui: &mut egui::Ui, source: &'static str) {
        ui.vertical(|ui| {
            ui.label(self.tr(LanguageText::Hooks));
            let hooks: Vec<usize> = self
                .hook_sources
                .iter()
                .enumerate()
                .filter(|(_, item_source)| **item_source == source)
                .map(|(index, _)| index)
                .collect();
            if hooks.is_empty() {
                ui.label("-");
                return;
            }
            for index in hooks {
                let key = self.hook_key(index);
                let title = self.hooks[index].hook_title();
                let mut enabled = !self.disabled_hooks.contains(&key);
                if ui.checkbox(&mut enabled, title).changed() {
                    if enabled {
                        self.disabled_hooks.remove(&key);
                    } else {
                        self.disabled_hooks.insert(key);
                    }
                    self.save_component_state();
                }
            }
        });
    }

    fn plugin_ribbon(&mut self, ui: &mut egui::Ui, tab_id: &'static str) {
        let mut panels = std::mem::take(&mut self.panels);
        let mut commands = Vec::new();
        ui.horizontal_wrapped(|ui| {
            for (index, panel) in panels.iter_mut().enumerate() {
                let panel_key = Self::component_key(
                    self.panel_sources
                        .get(index)
                        .copied()
                        .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                    panel.panel_id(),
                );
                if self.disabled_panels.contains(&panel_key) {
                    continue;
                }
                if panel.panel_area() == PanelArea::RibbonTab(tab_id) {
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label(panel.panel_title(&self.language));
                        let mut context = self.panel_context();
                        panel.ui(ui, &mut context);
                        commands.append(&mut context.commands);
                    });
                }
            }
        });
        self.panels = panels;
        self.apply_commands(commands);
    }

    fn plugin_top_panels(&mut self, ui: &mut egui::Ui) {
        let mut panels = std::mem::take(&mut self.panels);
        let mut commands = Vec::new();
        for (index, panel) in panels.iter_mut().enumerate() {
            let panel_key = Self::component_key(
                self.panel_sources
                    .get(index)
                    .copied()
                    .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                panel.panel_id(),
            );
            if self.disabled_panels.contains(&panel_key) {
                continue;
            }
            if panel.panel_area() == PanelArea::TopBar {
                ui.separator();
                ui.vertical(|ui| {
                    ui.label(panel.panel_title(&self.language));
                    let mut context = self.panel_context();
                    panel.ui(ui, &mut context);
                    commands.append(&mut context.commands);
                });
            }
        }
        self.panels = panels;
        self.apply_commands(commands);
    }

    fn status_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::bottom("status")
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{} x {}", self.canvas.width, self.canvas.height));
                    ui.separator();
                    ui.label(format!("{:.0}%", self.zoom * 100.0));
                    ui.separator();
                    let coords = self
                        .pointer_canvas_pos
                        .map(|(x, y)| format!("X: {x}, Y: {y}"))
                        .unwrap_or_else(|| "X: -, Y: -".to_owned());
                    ui.label(coords);
                    ui.separator();
                    ui.label(if self.document_dirty {
                        self.tr(LanguageText::Unsaved)
                    } else {
                        self.tr(LanguageText::SavedState)
                    });
                    ui.separator();
                    ui.label(&self.status);
                    let mut panels = std::mem::take(&mut self.panels);
                    let mut commands = Vec::new();
                    for (index, panel) in panels.iter_mut().enumerate() {
                        let panel_key = Self::component_key(
                            self.panel_sources
                                .get(index)
                                .copied()
                                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                            panel.panel_id(),
                        );
                        if self.disabled_panels.contains(&panel_key) {
                            continue;
                        }
                        if panel.panel_area() == PanelArea::BottomBar {
                            ui.separator();
                            let mut context = self.panel_context();
                            panel.ui(ui, &mut context);
                            commands.append(&mut context.commands);
                        }
                    }
                    self.panels = panels;
                    self.apply_commands(commands);
                });
            });
    }

    fn handle_selection_shortcuts(&mut self, ctx: &Context) {
        if self.active_text_box.is_some() {
            return;
        }
        if ctx.input_mut(|input| consume_ctrl_or_command(input, egui::Key::V)) {
            self.paste_selection();
        }
        if !matches!(self.active_tool, ToolKind::Select | ToolKind::Shape) {
            return;
        }
        if ctx.input_mut(|input| consume_ctrl_or_command(input, egui::Key::C)) {
            self.copy_active_box();
        }
        if ctx.input_mut(|input| consume_ctrl_or_command(input, egui::Key::X)) {
            self.cut_active_box();
        }
        if ctx.input_mut(|input| consume_ctrl_or_command(input, egui::Key::A)) {
            self.selected_rect = Some((
                (0, 0),
                (self.canvas.width as i32 - 1, self.canvas.height as i32 - 1),
            ));
            self.emit_event(AppEvent::SelectionChanged);
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Delete)) {
            self.delete_active_box();
        }
        if self.active_tool != ToolKind::Select {
            return;
        }
        let step = ctx.input(|input| if input.modifiers.shift { 10 } else { 1 });
        let arrow_modifiers = ctx.input(|input| {
            if input.modifiers.shift {
                egui::Modifiers::SHIFT
            } else {
                egui::Modifiers::NONE
            }
        });
        if ctx.input_mut(|input| input.consume_key(arrow_modifiers, egui::Key::ArrowLeft)) {
            self.nudge_selection(-step, 0);
        }
        if ctx.input_mut(|input| input.consume_key(arrow_modifiers, egui::Key::ArrowRight)) {
            self.nudge_selection(step, 0);
        }
        if ctx.input_mut(|input| input.consume_key(arrow_modifiers, egui::Key::ArrowUp)) {
            self.nudge_selection(0, -step);
        }
        if ctx.input_mut(|input| input.consume_key(arrow_modifiers, egui::Key::ArrowDown)) {
            self.nudge_selection(0, step);
        }
    }

    fn selection_context_menu(&mut self, response: &egui::Response) {
        if !matches!(self.active_tool, ToolKind::Select | ToolKind::Shape)
            || !self.has_active_edit_box()
        {
            return;
        }
        response.context_menu(|ui| {
            let has_selection = self.has_active_edit_box();
            let has_clipboard = self.selection_clipboard.is_some() || system_clipboard_has_image();
            if ui
                .add_enabled(has_selection, egui::Button::new("剪切    Ctrl+X"))
                .clicked()
            {
                self.cut_active_box();
                ui.close_menu();
            }
            if ui
                .add_enabled(has_selection, egui::Button::new("复制    Ctrl+C"))
                .clicked()
            {
                self.copy_active_box();
                ui.close_menu();
            }
            if ui
                .add_enabled(has_clipboard, egui::Button::new("粘贴    Ctrl+V"))
                .clicked()
            {
                self.paste_selection();
                ui.close_menu();
            }
            ui.separator();
            if ui
                .add_enabled(has_selection, egui::Button::new("裁剪    Ctrl+Shift+X"))
                .clicked()
            {
                if self.active_tool == ToolKind::Select {
                    self.crop_selection();
                }
                ui.close_menu();
            }
            if ui
                .add_enabled(has_selection, egui::Button::new("删除    Delete"))
                .clicked()
            {
                self.delete_active_box();
                ui.close_menu();
            }
            if ui.button("全选    Ctrl+A").clicked() {
                self.selected_rect = Some((
                    (0, 0),
                    (self.canvas.width as i32 - 1, self.canvas.height as i32 - 1),
                ));
                self.emit_event(AppEvent::SelectionChanged);
                ui.close_menu();
            }
        });
    }

    fn plugin_side_panels(&mut self, ctx: &Context) {
        let has_left = self.panels.iter().enumerate().any(|(index, panel)| {
            !self.disabled_panels.contains(&self.panel_key(index))
                && panel.panel_area() == PanelArea::LeftBar
        });
        if has_left {
            egui::SidePanel::left("plugin_left_panels")
                .resizable(true)
                .show(ctx, |ui| {
                    self.render_plugin_panel_area(ui, PanelArea::LeftBar)
                });
        }

        let has_right = self.panels.iter().enumerate().any(|(index, panel)| {
            !self.disabled_panels.contains(&self.panel_key(index))
                && panel.panel_area() == PanelArea::RightBar
        });
        if has_right {
            egui::SidePanel::right("plugin_right_panels")
                .resizable(true)
                .show(ctx, |ui| {
                    self.render_plugin_panel_area(ui, PanelArea::RightBar)
                });
        }
    }

    fn plugin_windows(&mut self, ctx: &Context) {
        let mut panels = std::mem::take(&mut self.panels);
        let mut commands = Vec::new();
        for (index, panel) in panels.iter_mut().enumerate() {
            let panel_key = Self::component_key(
                self.panel_sources
                    .get(index)
                    .copied()
                    .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                panel.panel_id(),
            );
            if self.disabled_panels.contains(&panel_key) {
                continue;
            }
            if panel.panel_area() != PanelArea::Window {
                continue;
            }
            let title = panel.panel_title(&self.language);
            egui::Window::new(title)
                .id(egui::Id::new(panel.panel_id()))
                .show(ctx, |ui| {
                    let mut context = self.panel_context();
                    panel.ui(ui, &mut context);
                    commands.append(&mut context.commands);
                });
        }
        self.panels = panels;
        self.apply_commands(commands);
    }

    fn render_plugin_panel_area(&mut self, ui: &mut egui::Ui, area: PanelArea) {
        let mut panels = std::mem::take(&mut self.panels);
        let mut commands = Vec::new();
        let mut first = true;
        for (index, panel) in panels.iter_mut().enumerate() {
            let panel_key = Self::component_key(
                self.panel_sources
                    .get(index)
                    .copied()
                    .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
                panel.panel_id(),
            );
            if self.disabled_panels.contains(&panel_key) {
                continue;
            }
            if panel.panel_area() != area {
                continue;
            }
            if !first {
                ui.separator();
            }
            ui.vertical(|ui| {
                ui.label(panel.panel_title(&self.language));
                let mut context = self.panel_context();
                panel.ui(ui, &mut context);
                commands.append(&mut context.commands);
            });
            first = false;
        }
        self.panels = panels;
        self.apply_commands(commands);
    }

    fn color_editor_window(&mut self, ctx: &Context) {
        if !self.show_color_editor {
            return;
        }

        let mut open = self.show_color_editor;
        let mut confirmed = false;
        let mut canceled = false;
        egui::Window::new(self.tr(LanguageText::EditColors))
            .open(&mut open)
            .resizable(false)
            .default_width(620.0)
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(10.0, 8.0);
                ui.horizontal(|ui| {
                    self.color_field_ui(ui);
                    self.color_value_ui(ui);
                });
                ui.add_space(16.0);
                self.color_palette_ui(ui);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [296.0, 32.0],
                            egui::Button::new(self.tr(LanguageText::Confirm)),
                        )
                        .clicked()
                    {
                        confirmed = true;
                    }
                    if ui
                        .add_sized(
                            [296.0, 32.0],
                            egui::Button::new(self.tr(LanguageText::Cancel)),
                        )
                        .clicked()
                    {
                        canceled = true;
                    }
                });
            });
        if confirmed {
            if self.editing_primary_color {
                self.primary = self.editor_color;
            } else {
                self.secondary = self.editor_color;
            }
            self.remember_recent_color(self.editor_color);
            self.emit_event(AppEvent::ColorChanged {
                primary: self.primary,
                secondary: self.secondary,
            });
            open = false;
        }
        if canceled {
            open = false;
        }
        self.show_color_editor = open;
    }

    fn color_field_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let field_size = Vec2::new(256.0, 256.0);
            let (field_rect, field_response) =
                ui.allocate_exact_size(field_size, Sense::click_and_drag());
            let painter = ui.painter();
            let columns = 32;
            let rows = 32;
            for row in 0..rows {
                for col in 0..columns {
                    let s = col as f32 / (columns - 1) as f32;
                    let v = 1.0 - row as f32 / (rows - 1) as f32;
                    let color = hsv_to_rgb(self.editor_hue, s, v);
                    let cell = Rect::from_min_max(
                        Pos2::new(
                            field_rect.left() + field_rect.width() * col as f32 / columns as f32,
                            field_rect.top() + field_rect.height() * row as f32 / rows as f32,
                        ),
                        Pos2::new(
                            field_rect.left()
                                + field_rect.width() * (col + 1) as f32 / columns as f32,
                            field_rect.top() + field_rect.height() * (row + 1) as f32 / rows as f32,
                        ),
                    );
                    painter.rect_filled(cell, 0.0, color);
                }
            }
            painter.rect_stroke(
                field_rect,
                3.0,
                Stroke::new(1.0, Color32::from_rgb(210, 214, 220)),
            );
            if (field_response.dragged() || field_response.clicked())
                && field_response.interact_pointer_pos().is_some()
            {
                let pointer = field_response.interact_pointer_pos().unwrap();
                self.editor_saturation =
                    ((pointer.x - field_rect.left()) / field_rect.width()).clamp(0.0, 1.0);
                self.editor_value =
                    (1.0 - (pointer.y - field_rect.top()) / field_rect.height()).clamp(0.0, 1.0);
                self.sync_editor_color_from_hsv();
            }
            let marker = Pos2::new(
                field_rect.left() + self.editor_saturation * field_rect.width(),
                field_rect.top() + (1.0 - self.editor_value) * field_rect.height(),
            );
            painter.circle_stroke(marker, 5.0, Stroke::new(1.5, Color32::WHITE));
            painter.circle_stroke(marker, 6.5, Stroke::new(1.0, Color32::BLACK));

            ui.add_space(2.0);
            let (preview_rect, _) = ui.allocate_exact_size(Vec2::new(44.0, 256.0), Sense::hover());
            ui.painter()
                .rect_filled(preview_rect, 4.0, self.editor_color);
            ui.painter().rect_stroke(
                preview_rect,
                4.0,
                Stroke::new(1.0, Color32::from_rgb(20, 20, 20)),
            );

            ui.add_space(6.0);
            let (hue_rect, hue_response) =
                ui.allocate_exact_size(Vec2::new(18.0, 256.0), Sense::click_and_drag());
            for row in 0..64 {
                let h = 1.0 - row as f32 / 63.0;
                let color = hsv_to_rgb(h * 360.0, 1.0, 1.0);
                let cell = Rect::from_min_max(
                    Pos2::new(
                        hue_rect.left(),
                        hue_rect.top() + hue_rect.height() * row as f32 / 64.0,
                    ),
                    Pos2::new(
                        hue_rect.right(),
                        hue_rect.top() + hue_rect.height() * (row + 1) as f32 / 64.0,
                    ),
                );
                ui.painter().rect_filled(cell, 0.0, color);
            }
            if (hue_response.dragged() || hue_response.clicked())
                && hue_response.interact_pointer_pos().is_some()
            {
                let pointer = hue_response.interact_pointer_pos().unwrap();
                self.editor_hue = (1.0 - (pointer.y - hue_rect.top()) / hue_rect.height())
                    .clamp(0.0, 1.0)
                    * 360.0;
                self.sync_editor_color_from_hsv();
            }
            let hue_y = hue_rect.bottom() - (self.editor_hue / 360.0) * hue_rect.height();
            ui.painter()
                .circle_filled(Pos2::new(hue_rect.center().x, hue_y), 6.0, Color32::BLACK);
            ui.painter().circle_stroke(
                Pos2::new(hue_rect.center().x, hue_y),
                7.0,
                Stroke::new(1.0, Color32::WHITE),
            );
        });
    }

    fn color_value_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            let mut hex = format!(
                "#{:02X}{:02X}{:02X}",
                self.editor_color.r(),
                self.editor_color.g(),
                self.editor_color.b()
            );
            if ui
                .add_sized([120.0, 28.0], egui::TextEdit::singleline(&mut hex))
                .lost_focus()
            {
                if let Some(color) = parse_hex_color(&hex) {
                    self.set_editor_color(color);
                }
            }
            egui::ComboBox::from_id_source("color_edit_mode")
                .width(120.0)
                .selected_text(match self.color_edit_mode {
                    ColorEditMode::Rgb => "RGB",
                    ColorEditMode::Hsv => "HSV",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.color_edit_mode, ColorEditMode::Rgb, "RGB");
                    ui.selectable_value(&mut self.color_edit_mode, ColorEditMode::Hsv, "HSV");
                });

            match self.color_edit_mode {
                ColorEditMode::Rgb => {
                    let mut r = self.editor_color.r() as i32;
                    let mut g = self.editor_color.g() as i32;
                    let mut b = self.editor_color.b() as i32;
                    let changed = color_number(ui, &mut r, self.tr(LanguageText::Red), 0..=255)
                        | color_number(ui, &mut g, self.tr(LanguageText::Green), 0..=255)
                        | color_number(ui, &mut b, self.tr(LanguageText::Blue), 0..=255);
                    if changed {
                        self.set_editor_color(Color32::from_rgb(r as u8, g as u8, b as u8));
                    }
                }
                ColorEditMode::Hsv => {
                    let mut h = self.editor_hue.round() as i32;
                    let mut s = (self.editor_saturation * 100.0).round() as i32;
                    let mut v = (self.editor_value * 100.0).round() as i32;
                    let changed = color_number(ui, &mut h, "H".to_owned(), 0..=360)
                        | color_number(ui, &mut s, "S".to_owned(), 0..=100)
                        | color_number(ui, &mut v, "V".to_owned(), 0..=100);
                    if changed {
                        self.editor_hue = h as f32;
                        self.editor_saturation = s as f32 / 100.0;
                        self.editor_value = v as f32 / 100.0;
                        self.sync_editor_color_from_hsv();
                    }
                }
            }
        });
    }

    fn color_palette_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(self.tr(LanguageText::BasicColors));
                for row in BASIC_COLORS.chunks(12) {
                    ui.horizontal(|ui| {
                        for color in row {
                            if palette_dot(ui, *color, false).clicked() {
                                self.set_editor_color(*color);
                            }
                        }
                    });
                }
            });
            ui.add_space(40.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(self.tr(LanguageText::CustomColors));
                    if ui.button("+").clicked() {
                        if let Some(slot) =
                            self.custom_colors.iter_mut().find(|slot| slot.is_none())
                        {
                            *slot = Some(self.editor_color);
                            self.remember_recent_color(self.editor_color);
                        }
                    }
                });
                for row in 0..4 {
                    ui.horizontal(|ui| {
                        for col in 0..6 {
                            let index = row * 6 + col;
                            let color = self.custom_colors[index];
                            let response = palette_dot(
                                ui,
                                color.unwrap_or(Color32::TRANSPARENT),
                                color.is_none(),
                            );
                            if response.clicked() {
                                if let Some(color) = color {
                                    self.set_editor_color(color);
                                    self.remember_recent_color(color);
                                } else {
                                    self.custom_colors[index] = Some(self.editor_color);
                                    self.remember_recent_color(self.editor_color);
                                }
                            }
                        }
                    });
                }
            });
        });
    }

    fn set_editor_color(&mut self, color: Color32) {
        self.editor_color = color;
        let (h, s, v) = rgb_to_hsv(color);
        self.editor_hue = h;
        self.editor_saturation = s;
        self.editor_value = v;
    }

    fn sync_editor_color_from_hsv(&mut self) {
        self.editor_color = hsv_to_rgb(self.editor_hue, self.editor_saturation, self.editor_value);
    }

    fn layers_panel(&mut self, ctx: &Context) {
        if !self.show_layers_panel {
            return;
        }

        egui::SidePanel::right("layers_panel")
            .resizable(true)
            .default_width(190.0)
            .show(ctx, |ui| {
                self.render_app_panels(ui, AppPanelArea::Layers);
            });
    }

    pub(crate) fn layers_panel_contents(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.tr(LanguageText::Layers));
        ui.separator();

        let mut selected_layer = self.active_layer;
        let mut visibility_changes = Vec::new();
        let mut layer_dirty = false;
        let mut move_request = None;
        let mut drop_target = None;
        let background_label = self.tr(LanguageText::BackgroundLayer);
        let opacity_label = self.tr(LanguageText::LayerOpacity);
        let blend_label = self.tr(LanguageText::LayerBlendMode);
        let blend_normal = self.tr(LanguageText::BlendNormal);
        let blend_multiply = self.tr(LanguageText::BlendMultiply);
        let blend_screen = self.tr(LanguageText::BlendScreen);
        let layer_drag_hint = self.tr(LanguageText::LayerDragHint);

        ui.push_id("background_layer_row", |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(&mut selected_layer, 0, "");
                paint_layer_thumbnail(ui, &self.canvas, true);
                ui.label(background_label);
            });
        });

        let list_height = (ui.available_height() - 92.0).max(160.0);
        egui::ScrollArea::vertical()
            .id_source("layers_list_scroll")
            .auto_shrink([false, false])
            .max_height(list_height)
            .show_rows(
                ui,
                LAYER_ROW_HEIGHT,
                self.pixel_layers.len(),
                |ui, row_range| {
                    for display_row in row_range {
                        let index = self.pixel_layers.len() - 1 - display_row;
                        let layer_id = index + 1;
                        ui.push_id(("pixel_layer_row", layer_id), |ui| {
                            let row_inner = ui.allocate_ui_with_layout(
                                Vec2::new(ui.available_width(), LAYER_ROW_HEIGHT - 4.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    if self.dragging_layer == Some(layer_id) {
                                        let rect = ui.available_rect_before_wrap();
                                        ui.painter().rect_filled(
                                            rect,
                                            2.0,
                                            Color32::from_rgba_unmultiplied(0, 120, 215, 18),
                                        );
                                    }
                                    ui.horizontal(|ui| {
                                        let layer = &mut self.pixel_layers[index];
                                        let mut visible = layer.visible;
                                        if ui.checkbox(&mut visible, "").changed() {
                                            visibility_changes.push((index, visible));
                                        }
                                        ui.radio_value(&mut selected_layer, layer_id, "");
                                        paint_layer_thumbnail(ui, &layer.canvas, false);
                                        let name_response = ui.add_sized(
                                            [116.0, 22.0],
                                            egui::TextEdit::singleline(&mut layer.name),
                                        );
                                        if name_response.changed() {
                                            layer_dirty = true;
                                        }
                                    });
                                    ui.indent("options", |ui| {
                                        let layer = &mut self.pixel_layers[index];
                                        ui.horizontal(|ui| {
                                            ui.label(&opacity_label);
                                            let opacity_response = ui.add(
                                                egui::Slider::new(&mut layer.opacity, 0.0..=1.0)
                                                    .show_value(false)
                                                    .clamp_to_range(true),
                                            );
                                            ui.label(format!("{:.0}%", layer.opacity * 100.0));
                                            if opacity_response.changed() {
                                                layer_dirty = true;
                                            }
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(&blend_label);
                                            let mut blend = layer.blend_mode;
                                            egui::ComboBox::from_id_source("blend")
                                                .width(112.0)
                                                .selected_text(layer_blend_label(
                                                    blend,
                                                    &blend_normal,
                                                    &blend_multiply,
                                                    &blend_screen,
                                                ))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut blend,
                                                        LayerBlendMode::Normal,
                                                        &blend_normal,
                                                    );
                                                    ui.selectable_value(
                                                        &mut blend,
                                                        LayerBlendMode::Multiply,
                                                        &blend_multiply,
                                                    );
                                                    ui.selectable_value(
                                                        &mut blend,
                                                        LayerBlendMode::Screen,
                                                        &blend_screen,
                                                    );
                                                });
                                            if blend != layer.blend_mode {
                                                layer.blend_mode = blend;
                                                layer_dirty = true;
                                            }
                                        });
                                        if selected_layer == layer_id {
                                            ui.label(&layer_drag_hint);
                                        }
                                    });
                                },
                            );
                            let row_response = row_inner.response.interact(Sense::click_and_drag());
                            if row_response.clicked() {
                                selected_layer = layer_id;
                            }
                            if row_response.drag_started() {
                                self.dragging_layer = Some(layer_id);
                                selected_layer = layer_id;
                            }
                            if self.dragging_layer.is_some()
                                && row_response.hovered()
                                && self.dragging_layer != Some(layer_id)
                            {
                                drop_target = Some(layer_id);
                                ui.painter().hline(
                                    row_response.rect.x_range(),
                                    row_response.rect.top(),
                                    Stroke::new(2.0, Color32::from_rgb(0, 120, 215)),
                                );
                            }
                        });
                    }
                },
            );
        if ui.input(|input| input.pointer.any_released()) {
            if let Some(from_layer) = self.dragging_layer.take() {
                if let Some(to_layer) = drop_target {
                    if from_layer != to_layer {
                        move_request = Some((from_layer, to_layer));
                        selected_layer = to_layer;
                    }
                }
            }
        }

        if let Some((from, to)) = move_request {
            self.move_layer_to(from, to);
            selected_layer = to;
        }
        if selected_layer != self.active_layer {
            self.set_active_layer(selected_layer);
        }
        if !visibility_changes.is_empty() {
            self.push_undo_snapshot();
            for (index, visible) in visibility_changes {
                if let Some(layer) = self.pixel_layers.get_mut(index) {
                    layer.visible = visible;
                }
            }
            self.mark_canvas_dirty();
        }
        if layer_dirty {
            self.mark_canvas_dirty();
        }

        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if ui.button(self.tr(LanguageText::AddLayer)).clicked() {
                self.add_pixel_layer();
            }
            if ui
                .add_enabled(
                    self.active_layer > 0,
                    egui::Button::new(self.tr(LanguageText::DeleteLayer)),
                )
                .clicked()
            {
                self.delete_active_layer();
            }
            if ui
                .add_enabled(
                    self.active_layer > 0 && self.active_layer < self.pixel_layers.len(),
                    egui::Button::new(self.tr(LanguageText::MoveLayerUp)),
                )
                .clicked()
            {
                self.move_active_layer_up();
            }
            if ui
                .add_enabled(
                    self.active_layer > 1,
                    egui::Button::new(self.tr(LanguageText::MoveLayerDown)),
                )
                .clicked()
            {
                self.move_active_layer_down();
            }
            if ui
                .add_enabled(
                    self.active_layer > 0,
                    egui::Button::new(self.tr(LanguageText::MergeLayerDown)),
                )
                .clicked()
            {
                self.merge_active_layer_down();
            }
            if ui
                .button(self.tr(LanguageText::MergeVisibleLayers))
                .clicked()
            {
                self.merge_visible_layers();
            }
            if ui.button(self.tr(LanguageText::ClearLayer)).clicked() {
                self.clear_current_layer();
            }
        });
    }

    fn canvas_panel(&mut self, ctx: &Context) {
        self.emit_event(AppEvent::BeforeCanvasPaint);
        self.upload_texture(ctx);
        self.layers_panel(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(texture_id) = self.texture.as_ref().map(TextureHandle::id) else {
                return;
            };

            let viewport = ui.available_rect_before_wrap();
            let viewport_response = ui.interact(
                viewport,
                ui.make_persistent_id("canvas_viewport"),
                Sense::drag(),
            );
            ui.painter()
                .rect_filled(viewport, 0.0, Color32::from_rgb(232, 235, 240));

            let canvas_size = Vec2::new(
                self.canvas.width as f32 * self.zoom,
                self.canvas.height as f32 * self.zoom,
            );
            let ruler_offset = if self.show_rulers { RULER_SIZE } else { 0.0 };
            let canvas_min = viewport.min + self.canvas_pan + Vec2::splat(ruler_offset);
            let canvas_rect = Rect::from_min_size(canvas_min, canvas_size);
            let canvas_response = ui.interact(
                canvas_rect,
                ui.make_persistent_id("canvas_surface"),
                Sense::click_and_drag(),
            );

            ui.painter().rect_filled(
                canvas_rect.translate(Vec2::new(0.0, 2.0)).expand(10.0),
                6.0,
                Color32::from_rgba_unmultiplied(0, 0, 0, 14),
            );
            ui.painter().rect_filled(canvas_rect, 0.0, Color32::WHITE);
            ui.painter().image(
                texture_id,
                canvas_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            self.draw_text_items(ui, canvas_rect);
            self.draw_grid(ui, canvas_rect);
            self.draw_selection_overlay(ui, canvas_rect);
            self.draw_shape_preview(ui, canvas_rect);
            self.draw_active_text_preview(ui, canvas_rect);
            self.paint_active_tool_overlay(ui, canvas_rect);

            let mut top_ruler_rect = None;
            let mut left_ruler_rect = None;
            if self.show_rulers {
                let corner = Rect::from_min_size(
                    canvas_rect.min - Vec2::splat(RULER_SIZE),
                    Vec2::splat(RULER_SIZE),
                );
                ui.painter()
                    .rect_filled(corner, 0.0, Color32::from_rgb(238, 240, 244));
                let top_ruler = Rect::from_min_size(
                    Pos2::new(canvas_rect.left(), canvas_rect.top() - RULER_SIZE),
                    Vec2::new(canvas_rect.width(), RULER_SIZE),
                );
                let left_ruler = Rect::from_min_size(
                    Pos2::new(canvas_rect.left() - RULER_SIZE, canvas_rect.top()),
                    Vec2::new(RULER_SIZE, canvas_rect.height()),
                );
                draw_horizontal_ruler(ui.painter(), top_ruler, self.canvas.width, self.zoom);
                draw_vertical_ruler(ui.painter(), left_ruler, self.canvas.height, self.zoom);
                top_ruler_rect = Some(top_ruler);
                left_ruler_rect = Some(left_ruler);
            }

            self.pointer_canvas_pos = canvas_response
                .hover_pos()
                .and_then(|pointer| self.canvas_point_from_screen(canvas_rect, pointer));
            if let Some((x, y)) = self.pointer_canvas_pos {
                if let Some(ruler) = top_ruler_rect {
                    draw_horizontal_ruler_cursor(ui.painter(), ruler, x, self.zoom);
                }
                if let Some(ruler) = left_ruler_rect {
                    draw_vertical_ruler_cursor(ui.painter(), ruler, y, self.zoom);
                }
            }

            self.paint_active_cursor(ui, canvas_rect, &canvas_response);
            let resize_consumed = self.resize_handles(ui, viewport, canvas_rect);
            if !resize_consumed {
                self.handle_viewport_pan(&viewport_response, canvas_rect);
                self.selection_context_menu(&canvas_response);
                self.active_tool_canvas_context_menu(&canvas_response);
                self.handle_canvas_drag(canvas_rect, &canvas_response);
            }
        });
        self.emit_event(AppEvent::AfterCanvasPaint);
    }

    fn canvas_point_from_screen(&self, canvas_rect: Rect, pointer: Pos2) -> Option<(i32, i32)> {
        if !canvas_rect.contains(pointer) {
            return None;
        }
        let x = ((pointer.x - canvas_rect.left()) / self.zoom).floor() as i32;
        let y = ((pointer.y - canvas_rect.top()) / self.zoom).floor() as i32;
        if x < 0 || y < 0 || x >= self.canvas.width as i32 || y >= self.canvas.height as i32 {
            None
        } else {
            Some((x, y))
        }
    }

    fn add_pixel_layer(&mut self) {
        self.push_undo_snapshot();
        let index = self.pixel_layers.len() + 1;
        let label = self.tr(LanguageText::Layer);
        self.pixel_layers.push(PixelLayer {
            name: format!("{label} {index}"),
            canvas: Canvas::new(self.canvas.width, self.canvas.height, Color32::TRANSPARENT),
            visible: true,
            opacity: 1.0,
            blend_mode: LayerBlendMode::Normal,
        });
        self.set_active_layer(index);
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::AddLayer);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerAdded { layer: index });
    }

    fn delete_active_layer(&mut self) {
        if self.active_layer == 0 {
            return;
        }
        self.push_undo_snapshot();
        let removed_layer = self.active_layer;
        self.pixel_layers.remove(removed_layer - 1);
        self.text_items.retain_mut(|item| {
            if item.layer == removed_layer {
                false
            } else {
                if item.layer > removed_layer {
                    item.layer -= 1;
                }
                true
            }
        });
        let next_layer = self
            .active_layer
            .saturating_sub(1)
            .min(self.pixel_layers.len());
        self.set_active_layer(next_layer);
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::LayerDeleted);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerDeleted {
            layer: removed_layer,
        });
    }

    fn move_active_layer_up(&mut self) {
        if self.active_layer == 0 || self.active_layer >= self.pixel_layers.len() {
            return;
        }
        self.push_undo_snapshot();
        let index = self.active_layer - 1;
        self.pixel_layers.swap(index, index + 1);
        swap_text_layers(
            &mut self.text_items,
            self.active_layer,
            self.active_layer + 1,
        );
        self.set_active_layer(self.active_layer + 1);
        self.status = self.tr(LanguageText::LayerMoved);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerMoved {
            layer: self.active_layer,
        });
    }

    fn move_active_layer_down(&mut self) {
        if self.active_layer <= 1 {
            return;
        }
        self.push_undo_snapshot();
        let index = self.active_layer - 1;
        self.pixel_layers.swap(index, index - 1);
        swap_text_layers(
            &mut self.text_items,
            self.active_layer,
            self.active_layer - 1,
        );
        self.set_active_layer(self.active_layer - 1);
        self.status = self.tr(LanguageText::LayerMoved);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerMoved {
            layer: self.active_layer,
        });
    }

    fn move_layer_to(&mut self, from_layer: usize, to_layer: usize) {
        if from_layer == 0
            || to_layer == 0
            || from_layer > self.pixel_layers.len()
            || to_layer > self.pixel_layers.len()
            || from_layer == to_layer
        {
            return;
        }
        self.push_undo_snapshot();
        let from_index = from_layer - 1;
        let to_index = to_layer - 1;
        let layer = self.pixel_layers.remove(from_index);
        self.pixel_layers.insert(to_index, layer);
        for item in &mut self.text_items {
            if item.layer == from_layer {
                item.layer = to_layer;
            } else if from_layer < to_layer && item.layer > from_layer && item.layer <= to_layer {
                item.layer -= 1;
            } else if from_layer > to_layer && item.layer >= to_layer && item.layer < from_layer {
                item.layer += 1;
            }
        }
        self.set_active_layer(to_layer);
        self.status = self.tr(LanguageText::LayerMoved);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerMoved { layer: to_layer });
    }

    fn merge_active_layer_down(&mut self) {
        if self.active_layer == 0 {
            return;
        }
        self.push_undo_snapshot();
        let layer_index = self.active_layer - 1;
        let merged_layer = self.active_layer;
        let target_layer = if layer_index == 0 { 0 } else { layer_index };
        let layer = self.pixel_layers.remove(layer_index);
        if layer_index == 0 {
            composite_layer(
                &mut self.canvas.pixels,
                &layer.canvas.pixels,
                layer.opacity,
                layer.blend_mode,
            );
            self.active_layer = 0;
        } else {
            composite_layer(
                &mut self.pixel_layers[layer_index - 1].canvas.pixels,
                &layer.canvas.pixels,
                layer.opacity,
                layer.blend_mode,
            );
            self.active_layer = layer_index;
        }
        for item in &mut self.text_items {
            if item.layer == merged_layer {
                item.layer = target_layer;
            } else if item.layer > merged_layer {
                item.layer -= 1;
            }
        }
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::LayerMergedDown);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerMerged);
    }

    fn merge_visible_layers(&mut self) {
        self.push_undo_snapshot();
        for layer in &self.pixel_layers {
            if layer.visible {
                composite_layer(
                    &mut self.canvas.pixels,
                    &layer.canvas.pixels,
                    layer.opacity,
                    layer.blend_mode,
                );
            }
        }
        let layer_visibility: Vec<bool> = self
            .pixel_layers
            .iter()
            .map(|layer| layer.visible)
            .collect();
        self.text_items.retain_mut(|item| {
            if item.layer == 0 {
                true
            } else if layer_visibility
                .get(item.layer - 1)
                .copied()
                .unwrap_or(false)
            {
                item.layer = 0;
                true
            } else {
                false
            }
        });
        self.pixel_layers.clear();
        self.active_layer = 0;
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::LayersMerged);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerMerged);
    }

    fn clear_current_layer(&mut self) {
        self.push_undo_snapshot();
        if self.active_layer == 0 {
            let width = self.canvas.width;
            let height = self.canvas.height;
            self.canvas.clear(width, height, Color32::WHITE);
        } else if let Some(layer) = self.pixel_layers.get_mut(self.active_layer - 1) {
            let width = layer.canvas.width;
            let height = layer.canvas.height;
            layer.canvas.clear(width, height, Color32::TRANSPARENT);
        }
        let active_layer = self.active_layer;
        self.text_items.retain(|item| item.layer != active_layer);
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::LayerCleared);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::LayerCleared {
            layer: active_layer,
        });
    }

    fn normalized_selection(&self) -> Option<((i32, i32), (i32, i32))> {
        let (start, end) = self.selected_rect?;
        let left = start.0.min(end.0).max(0);
        let top = start.1.min(end.1).max(0);
        let right = start.0.max(end.0).min(self.canvas.width as i32 - 1);
        let bottom = start.1.max(end.1).min(self.canvas.height as i32 - 1);
        if left > right || top > bottom {
            None
        } else {
            Some(((left, top), (right, bottom)))
        }
    }

    fn point_in_selection(&self, point: (i32, i32)) -> bool {
        self.normalized_selection()
            .map(|((left, top), (right, bottom))| {
                point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom
            })
            .unwrap_or(false)
    }

    fn point_in_rect(&self, point: (i32, i32), rect: ((i32, i32), (i32, i32))) -> bool {
        let (start, end) = rect;
        let left = start.0.min(end.0);
        let top = start.1.min(end.1);
        let right = start.0.max(end.0);
        let bottom = start.1.max(end.1);
        point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom
    }

    fn handle_viewport_pan(&mut self, response: &egui::Response, canvas_rect: Rect) {
        let over_canvas = response
            .hover_pos()
            .map(|pos| canvas_rect.expand(CANVAS_RESIZE_MARGIN).contains(pos))
            .unwrap_or(false);
        if over_canvas && self.pan_start.is_none() {
            return;
        }

        let pan_gesture = response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
            || response.ctx.input(|input| {
                (input.modifiers.command || input.key_down(egui::Key::Space))
                    && response.dragged_by(egui::PointerButton::Primary)
            });
        if pan_gesture {
            let start = *self.pan_start.get_or_insert(self.canvas_pan);
            self.set_view(self.zoom, start + response.drag_delta());
            response.ctx.set_cursor_icon(CursorIcon::Grab);
        }
        if response.drag_stopped() {
            self.pan_start = None;
        }
    }

    fn paint_active_cursor(&self, ui: &egui::Ui, canvas_rect: Rect, response: &egui::Response) {
        let Some(pointer) = response.hover_pos() else {
            return;
        };
        if !canvas_rect.contains(pointer) {
            return;
        }
        if !self.active_tool_is_enabled() {
            ui.ctx().set_cursor_icon(CursorIcon::Default);
            return;
        }

        if let Some(slot) = self.active_cursor_tool_slot {
            let Some(tool) = self.cursor_tools.get(slot) else {
                return;
            };
            if self
                .disabled_cursor_tools
                .contains(&self.cursor_tool_key(slot))
            {
                return;
            }
            match tool.cursor() {
                MyCursorIcon::EguiCursorIcon(icon) => ui.ctx().set_cursor_icon(icon),
                MyCursorIcon::Custom(paint) => {
                    ui.ctx().set_cursor_icon(CursorIcon::None);
                    paint(ui, canvas_rect, pointer);
                }
            }
            return;
        }

        match self.active_tool {
            ToolKind::Eraser => {
                let size = (self.brush_size.max(1) as f32 * self.zoom).max(4.0);
                let cursor_rect =
                    Rect::from_center_size(pointer, Vec2::splat(size)).intersect(canvas_rect);
                ui.ctx().set_cursor_icon(CursorIcon::None);
                ui.painter().rect_stroke(
                    cursor_rect,
                    0.0,
                    Stroke::new(3.0, Color32::from_rgba_unmultiplied(255, 255, 255, 220)),
                );
                ui.painter().rect_stroke(
                    cursor_rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_rgb(35, 35, 35)),
                );
            }
            ToolKind::Fill => ui.ctx().set_cursor_icon(CursorIcon::PointingHand),
            ToolKind::Text => ui.ctx().set_cursor_icon(CursorIcon::Text),
            ToolKind::Magnifier => ui.ctx().set_cursor_icon(CursorIcon::ZoomIn),
            ToolKind::Select => ui.ctx().set_cursor_icon(CursorIcon::Default),
            ToolKind::Brush | ToolKind::Pencil | ToolKind::Shape => {
                ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
            }
            _ => {}
        }
    }

    fn draw_selection_overlay(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        self.draw_moving_selection(ui, canvas_rect);
        if let Some((start, end)) = self.selected_rect.or(self.active_shape_rect) {
            self.draw_canvas_rect_outline(ui, canvas_rect, start, end);
            if self.active_tool == ToolKind::Shape && self.active_shape_rect.is_some() {
                self.handle_active_shape_resize(ui, canvas_rect);
            } else if self.active_tool == ToolKind::Select && self.selected_rect.is_some() {
                self.handle_selection_resize(ui, canvas_rect);
            }
        }
    }

    fn draw_moving_selection(&self, ui: &egui::Ui, canvas_rect: Rect) {
        let Some(selection) = &self.moving_selection else {
            return;
        };
        let max_pixels = 12_000usize;
        let region = &selection.content.region;
        let step = ((region.width * region.height) / max_pixels).max(1);
        for y in (0..region.height).step_by(step) {
            for x in (0..region.width).step_by(step) {
                let color = region.pixels[y * region.width + x];
                if color.a() == 0 || (self.transparent_selection && color == self.secondary) {
                    continue;
                }
                let pixel_rect = Rect::from_min_size(
                    Pos2::new(
                        canvas_rect.left() + (selection.position.0 + x as i32) as f32 * self.zoom,
                        canvas_rect.top() + (selection.position.1 + y as i32) as f32 * self.zoom,
                    ),
                    Vec2::splat((self.zoom * step as f32).max(1.0)),
                );
                ui.painter().rect_filled(pixel_rect, 0.0, color);
            }
        }
        for item in &selection.content.text_items {
            let mut translated = item.clone();
            translated.position = (
                selection.position.0 + item.position.0,
                selection.position.1 + item.position.1,
            );
            translated.layer = self.active_layer;
            self.paint_text_item(ui, canvas_rect, &translated, false);
        }
    }

    fn draw_canvas_rect_outline(
        &self,
        ui: &egui::Ui,
        canvas_rect: Rect,
        start: (i32, i32),
        end: (i32, i32),
    ) {
        let rect = self.canvas_rect_to_screen(canvas_rect, start, end);
        ui.painter()
            .rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(0, 95, 184)));
        for pos in [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ] {
            let handle = Rect::from_center_size(pos, Vec2::splat(7.0));
            ui.painter().rect_filled(handle, 1.0, Color32::WHITE);
            ui.painter()
                .rect_stroke(handle, 1.0, Stroke::new(1.0, Color32::from_rgb(0, 95, 184)));
        }
    }

    fn handle_active_shape_resize(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let Some((start, end)) = self.active_shape_rect else {
            return;
        };
        let left = start.0.min(end.0);
        let right = start.0.max(end.0);
        let top = start.1.min(end.1);
        let bottom = start.1.max(end.1);
        let rect = self.canvas_rect_to_screen(canvas_rect, start, end);
        let handles = [0usize, 1usize, 2usize, 3usize];

        for index in handles {
            let pos = match index {
                0 => rect.left_top(),
                1 => rect.right_top(),
                2 => rect.left_bottom(),
                _ => rect.right_bottom(),
            };
            let handle = Rect::from_center_size(pos, Vec2::splat(9.0));
            let response = ui.interact(
                handle,
                ui.make_persistent_id(("active_shape_resize", index)),
                Sense::drag(),
            );
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
            }
            if response.dragged() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    if let Some(point) = self.canvas_point_from_screen(canvas_rect, pointer) {
                        let new_rect = match index {
                            0 => (point, (right, bottom)),
                            1 => ((left, point.1), (point.0, bottom)),
                            2 => ((point.0, top), (right, point.1)),
                            _ => ((left, top), point),
                        };
                        self.redraw_active_shape(new_rect.0, new_rect.1);
                    }
                }
            }
        }
    }

    fn handle_selection_resize(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let Some((start, end)) = self.selected_rect else {
            return;
        };
        let left = start.0.min(end.0);
        let right = start.0.max(end.0);
        let top = start.1.min(end.1);
        let bottom = start.1.max(end.1);
        let rect = self.canvas_rect_to_screen(canvas_rect, start, end);

        for index in 0..4 {
            let pos = match index {
                0 => rect.left_top(),
                1 => rect.right_top(),
                2 => rect.left_bottom(),
                _ => rect.right_bottom(),
            };
            let handle = Rect::from_center_size(pos, Vec2::splat(9.0));
            let response = ui.interact(
                handle,
                ui.make_persistent_id(("selection_resize", index)),
                Sense::drag(),
            );
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
            }
            if response.drag_started() {
                self.begin_selection_resize(index, (left, top), (right, bottom));
            }
            if response.dragged() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    if let Some(point) = self.canvas_point_from_screen(canvas_rect, pointer) {
                        self.resize_selection_to(point);
                    }
                }
            }
            if response.drag_stopped() {
                self.resizing_selection = None;
            }
        }
    }

    fn begin_selection_resize(&mut self, handle: usize, start: (i32, i32), end: (i32, i32)) {
        if self.resizing_selection.is_some() {
            return;
        }
        let Some(content) = self.selection_content() else {
            return;
        };
        self.push_undo_snapshot();
        if self.moving_selection.is_none() {
            let fill = self.selection_clear_fill();
            self.active_canvas_mut().clear_rect(start, end, fill);
            self.remove_text_items_in_rect(start, end);
            self.mark_canvas_dirty();
        }
        let fixed = match handle {
            0 => end,
            1 => (start.0, end.1),
            2 => (end.0, start.1),
            _ => start,
        };
        self.resizing_selection = Some(ResizingSelection {
            fixed,
            original_content: content.clone(),
        });
        self.moving_selection = Some(MovingSelection {
            content,
            origin: start,
            position: start,
            drag_anchor: start,
        });
    }

    fn resize_selection_to(&mut self, point: (i32, i32)) {
        let Some(resize) = self.resizing_selection.clone() else {
            return;
        };
        let fixed = resize.fixed;
        let left = point.0.min(fixed.0).max(0);
        let top = point.1.min(fixed.1).max(0);
        let right = point.0.max(fixed.0).min(self.canvas.width as i32 - 1);
        let bottom = point.1.max(fixed.1).min(self.canvas.height as i32 - 1);
        let width = (right - left + 1).max(1) as usize;
        let height = (bottom - top + 1).max(1) as usize;
        let content = scale_selection_content(&resize.original_content, width, height);
        let position = (left, top);
        self.moving_selection = Some(MovingSelection {
            content,
            origin: position,
            position,
            drag_anchor: position,
        });
        self.selected_rect = Some((position, (right, bottom)));
        self.status = self.tr(LanguageText::SelectionMove);
    }

    fn canvas_rect_to_screen(&self, canvas_rect: Rect, start: (i32, i32), end: (i32, i32)) -> Rect {
        let left = start.0.min(end.0) as f32;
        let right = start.0.max(end.0) as f32 + 1.0;
        let top = start.1.min(end.1) as f32;
        let bottom = start.1.max(end.1) as f32 + 1.0;
        Rect::from_min_max(
            Pos2::new(
                canvas_rect.left() + left * self.zoom,
                canvas_rect.top() + top * self.zoom,
            ),
            Pos2::new(
                canvas_rect.left() + right * self.zoom,
                canvas_rect.top() + bottom * self.zoom,
            ),
        )
    }

    fn redraw_active_shape(&mut self, start: (i32, i32), end: (i32, i32)) {
        let Some(snapshot) = self.active_shape_snapshot.clone() else {
            return;
        };
        if self.active_layer == 0 {
            self.canvas.restore(snapshot);
        } else if let Some(layer) = self.pixel_layers.get_mut(self.active_layer - 1) {
            layer.canvas.restore(snapshot);
        }
        if let Some(shape) = self.shape_group.active_shape_mut() {
            let active_canvas = if self.active_layer == 0 {
                &mut self.canvas
            } else {
                &mut self.pixel_layers[self.active_layer - 1].canvas
            };
            shape.draw(
                active_canvas,
                start,
                end,
                self.primary,
                self.secondary,
                self.brush_size,
                self.shape_mode,
            );
            self.active_shape_rect = Some((start, end));
            self.dirty_texture = true;
            self.document_dirty = true;
            self.emit_event(AppEvent::CanvasDirty);
        }
    }

    fn resize_canvas(&mut self, width: usize, height: usize) {
        if width == self.canvas.width && height == self.canvas.height {
            return;
        }
        self.push_undo_snapshot();
        self.resize_canvas_live(width, height);
    }

    fn resize_canvas_live(&mut self, width: usize, height: usize) {
        self.canvas.resize(width, height, Color32::WHITE);
        for layer in &mut self.pixel_layers {
            layer.canvas.resize(width, height, Color32::TRANSPARENT);
        }
        self.dirty_texture = true;
        self.document_dirty = true;
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::CanvasResized);
        self.emit_event(AppEvent::CanvasResized {
            width: self.canvas.width,
            height: self.canvas.height,
        });
    }

    fn new_document(&mut self) {
        self.push_undo_snapshot();
        self.canvas
            .clear(DEFAULT_WIDTH, DEFAULT_HEIGHT, Color32::WHITE);
        self.pixel_layers.clear();
        self.text_items.clear();
        self.active_text_box = None;
        self.active_layer = 0;
        self.save_path = None;
        self.document_dirty = true;
        self.dirty_texture = true;
        self.status = self.tr(LanguageText::NewCanvas);
        self.clear_transient_selection_state();
        self.clear_history_stack();
    }

    fn open_image(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(
                self.tr(LanguageText::ImageFilter),
                &["png", "jpg", "jpeg", "bmp", "webp"],
            )
            .pick_file()
        else {
            return;
        };
        self.open_image_path(path);
    }

    fn open_image_path(&mut self, path: PathBuf) {
        match image_io::load_canvas(&path) {
            Ok(image) => {
                self.canvas = image;
                self.pixel_layers.clear();
                self.text_items.clear();
                self.active_text_box = None;
                self.active_layer = 0;
                self.save_path = Some(path.clone());
                self.document_dirty = false;
                self.dirty_texture = true;
                self.clear_transient_selection_state();
                self.clear_history_stack();
                self.remember_recent_file(path);
                self.status = self.tr(LanguageText::Opened);
            }
            Err(err) => {
                self.status = format!("{}: {err}", self.tr(LanguageText::OpenFailed));
            }
        }
    }

    fn import_image(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(
                self.tr(LanguageText::ImageFilter),
                &["png", "jpg", "jpeg", "bmp", "webp"],
            )
            .pick_file()
        else {
            return;
        };

        match image_io::load_canvas(&path) {
            Ok(image) => {
                self.push_undo_snapshot();
                let width = image.width.min(self.canvas.width);
                let height = image.height.min(self.canvas.height);
                let region = CanvasRegion {
                    width: image.width,
                    height: image.height,
                    pixels: image.pixels,
                };
                self.active_canvas_mut().paste_region(0, 0, &region);
                self.set_active_tool(ToolKind::Select);
                self.selected_rect = Some((
                    (0, 0),
                    (
                        width.saturating_sub(1) as i32,
                        height.saturating_sub(1) as i32,
                    ),
                ));
                self.moving_selection = None;
                self.status = self.tr(LanguageText::ImportedImage);
                self.mark_canvas_dirty();
                self.emit_event(AppEvent::ImageImported {
                    width: image.width,
                    height: image.height,
                });
                self.emit_event(AppEvent::SelectionChanged);
            }
            Err(_) => {
                self.status = self.tr(LanguageText::OpenFailed);
            }
        }
    }

    fn save_image(&mut self) {
        if let Some(path) = self.save_path.clone() {
            self.save_image_to_path(path);
        } else {
            self.save_image_as();
        }
    }

    fn save_image_as(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(self.tr(LanguageText::PngFilter), &["png"])
            .add_filter(self.tr(LanguageText::JpegFilter), &["jpg", "jpeg"])
            .add_filter(self.tr(LanguageText::BmpFilter), &["bmp"])
            .set_file_name("untitled.png")
            .save_file()
        else {
            return;
        };
        self.save_image_to_path(ensure_image_extension(path));
    }

    fn save_image_to_path(&mut self, path: PathBuf) {
        self.commit_active_selection();
        self.commit_active_text();
        let canvas =
            Canvas::from_pixels(self.canvas.width, self.canvas.height, self.export_pixels());
        match image_io::save_canvas(&canvas, &path) {
            Ok(()) => {
                self.remember_recent_file(path.clone());
                self.save_path = Some(path);
                self.document_dirty = false;
                self.status = self.tr(LanguageText::Saved);
            }
            Err(err) => {
                self.status = format!("{}: {err}", self.tr(LanguageText::SaveFailed));
            }
        }
    }

    fn crop_selection(&mut self) {
        let Some((start, end)) = self.normalized_selection() else {
            return;
        };
        self.push_undo_snapshot();
        if self.canvas.crop_rect(start, end) {
            for layer in &mut self.pixel_layers {
                layer.canvas.crop_rect(start, end);
            }
            self.clear_transient_selection_state();
            self.status = self.tr(LanguageText::SelectionCrop);
            self.emit_event(AppEvent::CanvasResized {
                width: self.canvas.width,
                height: self.canvas.height,
            });
            self.mark_canvas_dirty();
            self.emit_event(AppEvent::SelectionChanged);
        }
    }

    fn delete_selection(&mut self) {
        if self.moving_selection.is_some() {
            self.moving_selection = None;
            self.clear_transient_selection_state();
            self.status = self.tr(LanguageText::SelectionDelete);
            self.mark_canvas_dirty();
            self.emit_event(AppEvent::SelectionChanged);
            return;
        }
        let Some((start, end)) = self.normalized_selection() else {
            return;
        };
        self.push_undo_snapshot();
        let fill = self.selection_clear_fill();
        self.active_canvas_mut().clear_rect(start, end, fill);
        self.remove_text_items_in_rect(start, end);
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::SelectionDelete);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::SelectionChanged);
    }

    fn selection_clear_fill(&self) -> Color32 {
        if self.active_layer == 0 {
            Color32::WHITE
        } else {
            Color32::TRANSPARENT
        }
    }

    fn has_active_edit_box(&self) -> bool {
        match self.active_tool {
            ToolKind::Select => self.selected_rect.is_some() || self.moving_selection.is_some(),
            ToolKind::Shape => self.active_shape_rect.is_some(),
            _ => false,
        }
    }

    fn copy_active_box(&mut self) {
        match self.active_tool {
            ToolKind::Shape => self.copy_active_shape(),
            _ => self.copy_selection(),
        }
    }

    fn cut_active_box(&mut self) {
        match self.active_tool {
            ToolKind::Shape => self.cut_active_shape(),
            _ => self.cut_selection(),
        }
    }

    fn delete_active_box(&mut self) {
        match self.active_tool {
            ToolKind::Shape => self.delete_active_shape(),
            _ => self.delete_selection(),
        }
    }

    fn active_shape_content(&self) -> Option<SelectionContent> {
        let (start, end) = self.active_shape_rect?;
        Some(SelectionContent {
            region: self.active_canvas().copy_rect(start, end)?,
            text_items: Vec::new(),
        })
    }

    fn copy_active_shape(&mut self) {
        let Some(content) = self.active_shape_content() else {
            return;
        };
        write_selection_to_system_clipboard(&content);
        self.selection_clipboard = Some(content);
        self.status = self.tr(LanguageText::SelectionCopy);
        self.emit_event(AppEvent::SelectionChanged);
    }

    fn cut_active_shape(&mut self) {
        self.copy_active_shape();
        self.delete_active_shape();
        self.status = self.tr(LanguageText::SelectionCut);
    }

    fn delete_active_shape(&mut self) {
        let Some(snapshot) = self.active_shape_snapshot.clone() else {
            return;
        };
        self.push_undo_snapshot();
        if self.active_layer == 0 {
            self.canvas.restore(snapshot);
        } else if let Some(layer) = self.pixel_layers.get_mut(self.active_layer - 1) {
            layer.canvas.restore(snapshot);
        }
        self.active_shape_rect = None;
        self.active_shape_snapshot = None;
        self.status = self.tr(LanguageText::SelectionDelete);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::SelectionChanged);
    }

    fn selection_content(&self) -> Option<SelectionContent> {
        if let Some(selection) = &self.moving_selection {
            return Some(selection.content.clone());
        }
        let (start, end) = self.normalized_selection()?;
        Some(SelectionContent {
            region: self.active_canvas().copy_rect(start, end)?,
            text_items: self.selected_text_items(start, end),
        })
    }

    fn selected_text_items(&self, start: (i32, i32), end: (i32, i32)) -> Vec<TextItem> {
        self.text_items
            .iter()
            .filter(|item| {
                item.layer == self.active_layer
                    && canvas_rects_intersect((start, end), text_item_canvas_rect(item))
            })
            .cloned()
            .map(|mut item| {
                item.position = (item.position.0 - start.0, item.position.1 - start.1);
                item
            })
            .collect()
    }

    fn remove_text_items_in_rect(&mut self, start: (i32, i32), end: (i32, i32)) -> bool {
        let active_layer = self.active_layer;
        let before = self.text_items.len();
        self.text_items.retain(|item| {
            item.layer != active_layer
                || !canvas_rects_intersect((start, end), text_item_canvas_rect(item))
        });
        before != self.text_items.len()
    }

    fn copy_selection(&mut self) {
        let Some(content) = self.selection_content() else {
            return;
        };
        write_selection_to_system_clipboard(&content);
        self.selection_clipboard = Some(content);
        self.status = self.tr(LanguageText::SelectionCopy);
        self.emit_event(AppEvent::SelectionChanged);
    }

    fn cut_selection(&mut self) {
        let Some(content) = self.selection_content() else {
            return;
        };
        self.push_undo_snapshot();
        self.selection_clipboard = Some(content);
        if self.moving_selection.take().is_none() {
            if let Some((start, end)) = self.normalized_selection() {
                let fill = self.selection_clear_fill();
                self.active_canvas_mut().clear_rect(start, end, fill);
                self.remove_text_items_in_rect(start, end);
            }
        }
        self.clear_transient_selection_state();
        self.status = self.tr(LanguageText::SelectionCut);
        self.mark_canvas_dirty();
        self.emit_event(AppEvent::SelectionChanged);
    }

    fn paste_selection(&mut self) {
        let Some(content) = self
            .selection_clipboard
            .clone()
            .or_else(read_system_clipboard_selection)
        else {
            return;
        };
        let target_position = self
            .normalized_selection()
            .map(|(start, _)| start)
            .or(self.pointer_canvas_pos)
            .unwrap_or((0, 0));
        self.commit_active_selection();
        self.push_undo_snapshot();
        self.set_active_tool(ToolKind::Select);
        let position = (
            target_position
                .0
                .clamp(0, self.canvas.width.saturating_sub(1) as i32),
            target_position
                .1
                .clamp(0, self.canvas.height.saturating_sub(1) as i32),
        );
        let end = (
            position.0 + content.region.width as i32 - 1,
            position.1 + content.region.height as i32 - 1,
        );
        self.moving_selection = Some(MovingSelection {
            content,
            origin: position,
            position,
            drag_anchor: position,
        });
        self.selected_rect = Some((position, end));
        self.status = self.tr(LanguageText::SelectionPaste);
        self.emit_event(AppEvent::SelectionChanged);
    }

    fn nudge_selection(&mut self, dx: i32, dy: i32) {
        if self.active_tool != ToolKind::Select || self.selected_rect.is_none() {
            return;
        }
        if self.moving_selection.is_none() {
            let Some((start, end)) = self.normalized_selection() else {
                return;
            };
            let Some(content) = self.selection_content() else {
                return;
            };
            self.push_undo_snapshot();
            let fill = self.selection_clear_fill();
            self.active_canvas_mut().clear_rect(start, end, fill);
            self.remove_text_items_in_rect(start, end);
            self.moving_selection = Some(MovingSelection {
                content,
                origin: start,
                position: start,
                drag_anchor: start,
            });
            self.mark_canvas_dirty();
        }
        if let Some(selection) = &mut self.moving_selection {
            let max_x = self.canvas.width as i32 - 1;
            let max_y = self.canvas.height as i32 - 1;
            let new_x = (selection.position.0 + dx)
                .clamp(-(selection.content.region.width as i32 - 1), max_x);
            let new_y = (selection.position.1 + dy)
                .clamp(-(selection.content.region.height as i32 - 1), max_y);
            selection.position = (new_x, new_y);
            self.selected_rect = Some((
                selection.position,
                (
                    selection.position.0 + selection.content.region.width as i32 - 1,
                    selection.position.1 + selection.content.region.height as i32 - 1,
                ),
            ));
            self.status = self.tr(LanguageText::SelectionMove);
            self.emit_event(AppEvent::SelectionChanged);
        }
    }

    fn fill_at(&mut self, point: (i32, i32)) {
        self.push_undo_snapshot();
        let color = self.primary;
        FillTool::new().fill(self.active_canvas_mut(), point.0, point.1, color);
        self.status = self.tr(LanguageText::FilledStatus);
        self.mark_canvas_dirty();
    }

    fn pick_composited_pixel(&self, point: (i32, i32)) -> Option<Color32> {
        if point.0 < 0
            || point.1 < 0
            || point.0 >= self.canvas.width as i32
            || point.1 >= self.canvas.height as i32
        {
            return None;
        }
        let canvas = Canvas::from_pixels(
            self.canvas.width,
            self.canvas.height,
            self.composited_pixels(),
        );
        canvas.get_pixel(point.0, point.1)
    }

    fn handle_curve_shape_input(&mut self, point: (i32, i32), response: &egui::Response) {
        if self.curve_draft.is_none() {
            if response.drag_started() {
                self.push_undo_snapshot();
                self.drag_start = Some(point);
                self.preview_point = Some(point);
                self.active_shape_snapshot = Some(self.active_canvas().snapshot());
            }
            if response.dragged() {
                self.preview_point = Some(point);
            }
            if response.drag_stopped() {
                if let (Some(start), Some(snapshot)) =
                    (self.drag_start, self.active_shape_snapshot.clone())
                {
                    if start != point {
                        self.curve_draft = Some(CurveDraft {
                            start,
                            end: point,
                            control1: None,
                            control2: None,
                            snapshot,
                        });
                        self.redraw_curve_draft();
                    }
                }
                self.drag_start = None;
                self.preview_point = None;
            }
            return;
        }

        if response.clicked() {
            self.set_curve_control(point, true);
            return;
        }

        if response.dragged() {
            self.set_curve_control(point, false);
        }
        if response.drag_stopped() {
            self.set_curve_control(point, true);
        }
    }

    fn set_curve_control(&mut self, point: (i32, i32), finalize: bool) {
        let Some(draft) = &mut self.curve_draft else {
            return;
        };
        if draft.control1.is_none() {
            draft.control1 = Some(point);
        } else {
            draft.control2 = Some(point);
        }
        self.redraw_curve_draft();
        if finalize {
            let done = self
                .curve_draft
                .as_ref()
                .is_some_and(|draft| draft.control1.is_some() && draft.control2.is_some());
            if done {
                self.curve_draft = None;
                self.active_shape_snapshot = None;
                self.active_shape_rect = None;
                self.emit_event(AppEvent::ShapeCommitted);
            }
        }
    }

    fn redraw_curve_draft(&mut self) {
        let Some(draft) = self.curve_draft.clone() else {
            return;
        };
        if self.active_layer == 0 {
            self.canvas.restore(draft.snapshot.clone());
        } else if let Some(layer) = self.pixel_layers.get_mut(self.active_layer - 1) {
            layer.canvas.restore(draft.snapshot.clone());
        }

        let c1 = draft.control1.unwrap_or(draft.start);
        let c2 = draft.control2.or(draft.control1).unwrap_or(draft.end);
        let primary = self.primary;
        let brush_size = self.brush_size;
        let active_canvas = self.active_canvas_mut();
        if draft.control1.is_none() && draft.control2.is_none() {
            crate::algorithm::draw_polyline(
                active_canvas,
                &[draft.start, draft.end],
                primary,
                brush_size.max(1),
                false,
            );
        } else {
            crate::tools::shape::curve::draw_cubic_curve(
                active_canvas,
                draft.start,
                c1,
                c2,
                draft.end,
                primary,
                brush_size,
            );
        }
        self.active_shape_rect = None;
        self.mark_canvas_dirty();
    }

    fn default_text_style(&self) -> TextStyle {
        TextStyle {
            font_family: self.text_renderer.default_family(),
            font_size: 18.0,
            bold: true,
            italic: false,
            underline: false,
            strikethrough: false,
            align: TextAlign::Left,
            background_fill: false,
        }
    }

    fn begin_text_edit(&mut self, position: (i32, i32)) {
        let item = TextItem {
            layer: self.active_layer,
            position,
            size: (260, 72),
            text: self.tr(LanguageText::TextValue),
            color: self.primary,
            background: self.secondary,
            style: self.default_text_style(),
        };
        self.active_text_box = Some(ActiveTextBox { item });
        self.status = self.tr(LanguageText::TextEditing);
    }

    fn commit_active_text(&mut self) {
        let Some(active) = self.active_text_box.take() else {
            return;
        };
        if active.item.text.trim().is_empty() {
            return;
        }
        self.push_undo_snapshot();
        self.text_items.push(active.item);
        self.document_dirty = true;
        self.status = self.tr(LanguageText::TextPlaced);
        self.emit_event(AppEvent::TextCommitted);
    }

    fn cancel_active_text(&mut self) {
        self.active_text_box = None;
        self.status = self.tr(LanguageText::Ready);
        self.emit_event(AppEvent::TextCanceled);
    }

    fn draw_text_items(&self, ui: &egui::Ui, canvas_rect: Rect) {
        for item in &self.text_items {
            if !self.text_layer_visible(item.layer) {
                continue;
            }
            self.paint_text_item(ui, canvas_rect, item, false);
        }
    }

    fn text_layer_visible(&self, layer: usize) -> bool {
        layer == 0
            || self
                .pixel_layers
                .get(layer - 1)
                .map(|layer| layer.visible)
                .unwrap_or(false)
    }

    fn render_text_items_to_pixels(&self, pixels: &mut [Color32]) {
        for item in &self.text_items {
            if !self.text_layer_visible(item.layer) {
                continue;
            }
            self.text_renderer.render_text_item_to_pixels(
                pixels,
                self.canvas.width,
                self.canvas.height,
                item,
            );
        }
    }

    fn draw_active_text_preview(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let Some(active) = &self.active_text_box else {
            return;
        };
        self.paint_active_text_frame(ui, canvas_rect, &active.item);
        self.handle_active_text_move(ui, canvas_rect);
        self.handle_active_text_resize(ui, canvas_rect);
        self.active_text_editor(ui, canvas_rect);
    }

    fn paint_text_item(&self, ui: &egui::Ui, canvas_rect: Rect, item: &TextItem, active: bool) {
        let text_rect = self.text_item_rect(canvas_rect, item);
        let galley = self.text_renderer.layout_galley(
            ui,
            &item.text,
            &item.style,
            item.color,
            self.zoom,
            (text_rect.width() - 8.0).max(1.0),
        );
        if item.style.background_fill {
            ui.painter().rect_filled(text_rect, 0.0, item.background);
        }
        if active {
            ui.painter().rect_stroke(
                text_rect,
                0.0,
                Stroke::new(1.0, Color32::from_rgb(0, 120, 215)),
            );
        }
        let text_pos = aligned_text_origin(text_rect, item.style.align);
        ui.painter().galley(text_pos, galley.clone(), item.color);
        if item.style.bold {
            ui.painter()
                .galley(text_pos + Vec2::new(0.7, 0.0), galley.clone(), item.color);
        }
    }

    fn paint_active_text_frame(&self, ui: &egui::Ui, canvas_rect: Rect, item: &TextItem) {
        let rect = self.text_item_rect(canvas_rect, item);
        if item.style.background_fill {
            ui.painter().rect_filled(rect, 0.0, item.background);
        }
        ui.painter()
            .rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(0, 120, 215)));
    }

    fn active_text_editor(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let Some(active) = &self.active_text_box else {
            return;
        };
        let rect = self.text_item_rect(canvas_rect, &active.item).shrink(5.0);
        if rect.width() <= 4.0 || rect.height() <= 4.0 {
            return;
        }
        let font_id = self.text_renderer.font_id(&active.item.style, self.zoom);
        let align = text_align_to_egui(active.item.style.align);
        let layout_style = active.item.style.clone();
        let text_renderer = self.text_renderer.clone();
        let color = active.item.color;
        let zoom = self.zoom;
        let width = rect.width();
        let height = rect.height();

        ui.allocate_ui_at_rect(rect, |ui| {
            if let Some(active) = &mut self.active_text_box {
                ui.set_clip_rect(rect);
                ui.set_min_size(rect.size());
                ui.style_mut().override_font_id = Some(font_id.clone());
                ui.visuals_mut().override_text_color = Some(color);
                let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
                    text_renderer.layout_edit_galley(
                        ui,
                        text,
                        &layout_style,
                        color,
                        zoom,
                        wrap_width,
                    )
                };
                let response = ui.add_sized(
                    [width, height],
                    egui::TextEdit::multiline(&mut active.item.text)
                        .desired_width(width)
                        .font(font_id.clone())
                        .horizontal_align(align)
                        .layouter(&mut layouter)
                        .frame(false),
                );
                if response.changed() {
                    ui.ctx().request_repaint();
                }
            }
        });
    }

    fn text_item_rect(&self, canvas_rect: Rect, item: &TextItem) -> Rect {
        Rect::from_min_size(
            Pos2::new(
                canvas_rect.left() + item.position.0 as f32 * self.zoom,
                canvas_rect.top() + item.position.1 as f32 * self.zoom,
            ),
            Vec2::new(
                item.size.0.max(8) as f32 * self.zoom,
                item.size.1.max(8) as f32 * self.zoom,
            ),
        )
    }

    fn handle_active_text_resize(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let Some(active) = &self.active_text_box else {
            return;
        };
        let rect = self.text_item_rect(canvas_rect, &active.item);
        for (index, pos) in [
            rect.left_top(),
            rect.right_top(),
            rect.left_bottom(),
            rect.right_bottom(),
        ]
        .into_iter()
        .enumerate()
        {
            let handle = Rect::from_center_size(pos, Vec2::splat(9.0));
            let response = ui.interact(
                handle,
                ui.make_persistent_id(("active_text_resize", index)),
                Sense::drag(),
            );
            ui.painter().rect_filled(handle, 1.0, Color32::WHITE);
            ui.painter().rect_stroke(
                handle,
                1.0,
                Stroke::new(1.0, Color32::from_rgb(0, 120, 215)),
            );
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::ResizeNwSe);
            }
            if response.dragged() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    self.resize_active_text_box(canvas_rect, index, pointer);
                }
            }
        }
    }

    fn handle_active_text_move(&mut self, ui: &mut egui::Ui, canvas_rect: Rect) {
        let Some(active) = &self.active_text_box else {
            return;
        };
        let rect = self.text_item_rect(canvas_rect, &active.item);
        let active_position = active.item.position;
        let edge = (8.0 * self.zoom).clamp(6.0, 14.0);
        let move_zones = [
            Rect::from_min_max(rect.left_top(), Pos2::new(rect.right(), rect.top() + edge)),
            Rect::from_min_max(
                Pos2::new(rect.left(), rect.bottom() - edge),
                rect.right_bottom(),
            ),
            Rect::from_min_max(
                Pos2::new(rect.left(), rect.top() + edge),
                Pos2::new(rect.left() + edge, rect.bottom() - edge),
            ),
            Rect::from_min_max(
                Pos2::new(rect.right() - edge, rect.top() + edge),
                Pos2::new(rect.right(), rect.bottom() - edge),
            ),
        ];

        for (index, zone) in move_zones.into_iter().enumerate() {
            let response = ui.interact(
                zone,
                ui.make_persistent_id(("active_text_move", index)),
                Sense::drag(),
            );
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            }
            if response.drag_started() {
                if let Some(pointer) = response.interact_pointer_pos() {
                    if let Some(point) = self.canvas_point_from_screen(canvas_rect, pointer) {
                        self.moving_text_box = Some((point, active_position));
                    }
                }
            }
            if response.dragged() {
                if let (Some((anchor, origin)), Some(pointer)) =
                    (self.moving_text_box, response.interact_pointer_pos())
                {
                    if let Some(point) = self.canvas_point_from_screen(canvas_rect, pointer) {
                        let dx = point.0 - anchor.0;
                        let dy = point.1 - anchor.1;
                        self.move_active_text_box((origin.0 + dx, origin.1 + dy));
                    }
                }
            }
            if response.drag_stopped() {
                self.moving_text_box = None;
            }
        }
    }

    fn move_active_text_box(&mut self, position: (i32, i32)) {
        let Some(active) = &mut self.active_text_box else {
            return;
        };
        let max_x = (self.canvas.width as i32 - active.item.size.0).max(0);
        let max_y = (self.canvas.height as i32 - active.item.size.1).max(0);
        active.item.position = (position.0.clamp(0, max_x), position.1.clamp(0, max_y));
    }

    fn resize_active_text_box(&mut self, canvas_rect: Rect, handle_index: usize, pointer: Pos2) {
        let zoom = self.zoom.max(0.001);
        let Some(active) = &mut self.active_text_box else {
            return;
        };
        let pointer_x = ((pointer.x - canvas_rect.left()) / zoom).round() as i32;
        let pointer_y = ((pointer.y - canvas_rect.top()) / zoom).round() as i32;
        let min_w = 24;
        let min_h = 18;
        let max_x = self.canvas.width as i32 - 1;
        let max_y = self.canvas.height as i32 - 1;
        let left = active.item.position.0;
        let top = active.item.position.1;
        let right = left + active.item.size.0;
        let bottom = top + active.item.size.1;
        let (new_left, new_top, new_right, new_bottom) = match handle_index {
            0 => (
                pointer_x.min(right - min_w).clamp(0, max_x),
                pointer_y.min(bottom - min_h).clamp(0, max_y),
                right,
                bottom,
            ),
            1 => (
                left,
                pointer_y.min(bottom - min_h).clamp(0, max_y),
                pointer_x
                    .max(left + min_w)
                    .clamp(0, self.canvas.width as i32),
                bottom,
            ),
            2 => (
                pointer_x.min(right - min_w).clamp(0, max_x),
                top,
                right,
                pointer_y
                    .max(top + min_h)
                    .clamp(0, self.canvas.height as i32),
            ),
            _ => (
                left,
                top,
                pointer_x
                    .max(left + min_w)
                    .clamp(0, self.canvas.width as i32),
                pointer_y
                    .max(top + min_h)
                    .clamp(0, self.canvas.height as i32),
            ),
        };
        active.item.position = (new_left, new_top);
        active.item.size = (
            (new_right - new_left).max(min_w),
            (new_bottom - new_top).max(min_h),
        );
    }

    fn draw_grid(&self, ui: &egui::Ui, rect: Rect) {
        if !self.show_grid {
            return;
        }

        let color = Color32::from_rgba_unmultiplied(0, 0, 0, 35);
        let step = if self.zoom >= 6.0 { 1 } else { 10 };
        for x in (0..=self.canvas.width).step_by(step) {
            let screen_x = rect.left() + x as f32 * self.zoom;
            ui.painter().line_segment(
                [
                    Pos2::new(screen_x, rect.top()),
                    Pos2::new(screen_x, rect.bottom()),
                ],
                Stroke::new(1.0, color),
            );
        }
        for y in (0..=self.canvas.height).step_by(step) {
            let screen_y = rect.top() + y as f32 * self.zoom;
            ui.painter().line_segment(
                [
                    Pos2::new(rect.left(), screen_y),
                    Pos2::new(rect.right(), screen_y),
                ],
                Stroke::new(1.0, color),
            );
        }
    }

    fn draw_shape_preview(&mut self, ui: &egui::Ui, rect: Rect) {
        if self.active_tool != ToolKind::Shape {
            return;
        }
        let (Some(start), Some(end)) = (self.drag_start, self.preview_point) else {
            return;
        };

        if self.active_shape_kind() == Some(ShapeKind::Curve) {
            let p0 = Pos2::new(
                rect.left() + start.0 as f32 * self.zoom,
                rect.top() + start.1 as f32 * self.zoom,
            );
            let p1 = Pos2::new(
                rect.left() + end.0 as f32 * self.zoom,
                rect.top() + end.1 as f32 * self.zoom,
            );
            ui.painter().line_segment(
                [p0, p1],
                Stroke::new(
                    (self.brush_size.max(1) as f32 * self.zoom).max(1.0),
                    self.primary,
                ),
            );
            return;
        }

        let margin = self.brush_size.max(1) + 8;
        let left = (start.0.min(end.0) - margin).max(0);
        let top = (start.1.min(end.1) - margin).max(0);
        let right = (start.0.max(end.0) + margin).min(self.canvas.width as i32 - 1);
        let bottom = (start.1.max(end.1) + margin).min(self.canvas.height as i32 - 1);
        if left > right || top > bottom {
            return;
        }

        let preview_width = (right - left + 1) as usize;
        let preview_height = (bottom - top + 1) as usize;
        let mut preview = Canvas::scratch(preview_width, preview_height, Color32::TRANSPARENT);
        let translated_start = (start.0 - left, start.1 - top);
        let translated_end = (end.0 - left, end.1 - top);
        if let Some(shape) = self.shape_group.active_shape_mut() {
            let preview_mode = if self.shape_mode == ShapeMode::Outline {
                ShapeMode::Outline
            } else {
                ShapeMode::Outline
            };
            shape.draw(
                &mut preview,
                translated_start,
                translated_end,
                self.primary,
                self.secondary,
                self.brush_size,
                preview_mode,
            );
        }

        for y in 0..preview_height.min(preview.height) {
            for x in 0..preview_width.min(preview.width) {
                let color = preview.pixels[y * preview.width + x];
                if color.a() == 0 {
                    continue;
                }
                let pixel_rect = Rect::from_min_size(
                    Pos2::new(
                        rect.left() + (left + x as i32) as f32 * self.zoom,
                        rect.top() + (top + y as i32) as f32 * self.zoom,
                    ),
                    Vec2::splat(self.zoom.max(1.0)),
                );
                ui.painter().rect_filled(pixel_rect, 0.0, color);
            }
        }
    }

    fn resize_handles(&mut self, ui: &mut egui::Ui, viewport: Rect, canvas_rect: Rect) -> bool {
        let size = CANVAS_RESIZE_HIT;
        let half = size / 2.0;
        let visible = canvas_rect.intersect(viewport);
        if !visible.is_positive() {
            return false;
        }
        let right_x = canvas_rect
            .right()
            .min(viewport.right() - half)
            .max(viewport.left() + half);
        let bottom_y = canvas_rect
            .bottom()
            .min(viewport.bottom() - half)
            .max(viewport.top() + half);
        let right = Rect::from_center_size(
            Pos2::new(right_x, visible.center().y),
            Vec2::new(size, visible.height().max(size)),
        );
        let bottom = Rect::from_center_size(
            Pos2::new(visible.center().x, bottom_y),
            Vec2::new(visible.width().max(size), size),
        );
        let corner = Rect::from_center_size(Pos2::new(right_x, bottom_y), Vec2::splat(size + 4.0));

        let right_used = self.handle_resize_drag(ui, ResizeHandle::Right, right, canvas_rect);
        let bottom_used = self.handle_resize_drag(ui, ResizeHandle::Bottom, bottom, canvas_rect);
        let corner_used = self.handle_resize_drag(ui, ResizeHandle::Corner, corner, canvas_rect);
        right_used || bottom_used || corner_used
    }

    fn handle_resize_drag(
        &mut self,
        ui: &mut egui::Ui,
        handle: ResizeHandle,
        rect: Rect,
        canvas_rect: Rect,
    ) -> bool {
        let id = egui::Id::new(("canvas_resize_area", handle));
        let response = ui.interact(rect, id, Sense::drag());
        let cursor = match handle {
            ResizeHandle::Right => CursorIcon::ResizeHorizontal,
            ResizeHandle::Bottom => CursorIcon::ResizeVertical,
            ResizeHandle::Corner => CursorIcon::ResizeNwSe,
        };
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(cursor);
        }

        let color = if response.hovered() || response.dragged() {
            Color32::from_rgb(0, 95, 184)
        } else {
            Color32::from_rgb(126, 134, 146)
        };
        let visual = Rect::from_center_size(rect.center(), Vec2::splat(10.0));
        ui.painter().rect_filled(visual, 1.0, Color32::WHITE);
        ui.painter()
            .rect_stroke(visual, 1.0, Stroke::new(1.0, color));

        if response.drag_started() {
            self.push_undo_snapshot();
            self.resize_start_size = Some((self.canvas.width, self.canvas.height));
        }
        if response.dragged() {
            let (start_width, start_height) = self
                .resize_start_size
                .unwrap_or((self.canvas.width, self.canvas.height));
            let pointer = response.interact_pointer_pos();
            let delta = response.drag_delta();
            let width = match handle {
                ResizeHandle::Right | ResizeHandle::Corner => {
                    if let Some(pointer) = pointer {
                        ((pointer.x - canvas_rect.left()) / self.zoom)
                            .round()
                            .max(1.0) as usize
                    } else {
                        (start_width as f32 + delta.x / self.zoom).round().max(1.0) as usize
                    }
                }
                ResizeHandle::Bottom => start_width,
            };
            let height = match handle {
                ResizeHandle::Bottom | ResizeHandle::Corner => {
                    if let Some(pointer) = pointer {
                        ((pointer.y - canvas_rect.top()) / self.zoom)
                            .round()
                            .max(1.0) as usize
                    } else {
                        (start_height as f32 + delta.y / self.zoom).round().max(1.0) as usize
                    }
                }
                ResizeHandle::Right => start_height,
            };
            self.resize_canvas_live(width, height);
        }
        if response.drag_stopped() {
            self.resize_start_size = None;
        }
        response.hovered() || response.dragged()
    }

    fn handle_canvas_drag(&mut self, rect: Rect, response: &egui::Response) {
        let pan_gesture = response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
            || response.ctx.input(|input| {
                (input.modifiers.command || input.key_down(egui::Key::Space))
                    && response.dragged_by(egui::PointerButton::Primary)
            });
        if pan_gesture {
            let start = *self.pan_start.get_or_insert(self.canvas_pan);
            self.set_view(self.zoom, start + response.drag_delta());
            response.ctx.set_cursor_icon(CursorIcon::Grab);
            if response.drag_stopped() {
                self.pan_start = None;
            }
            return;
        }
        if response.drag_stopped() {
            self.pan_start = None;
        }
        if self.resizing_selection.is_some() {
            return;
        }

        let Some(pointer) = response
            .interact_pointer_pos()
            .or_else(|| response.hover_pos())
        else {
            return;
        };
        let Some(point) = self.canvas_point_from_screen(rect, pointer) else {
            return;
        };
        let modifiers = response.ctx.input(|input| input.modifiers);

        if !self.active_tool_is_enabled() {
            if response.clicked() || response.drag_started() {
                self.status = self.tr(LanguageText::NoToolActive);
            }
            return;
        }

        if response.clicked()
            && self.dispatch_active_tool_event(CanvasToolEvent::Click {
                point,
                pointer,
                modifiers,
            })
        {
            return;
        }
        if response.drag_started()
            && self.dispatch_active_tool_event(CanvasToolEvent::DragStarted {
                point,
                pointer,
                modifiers,
            })
        {
            return;
        }
        if response.dragged()
            && self.dispatch_active_tool_event(CanvasToolEvent::Dragged {
                point,
                pointer,
                delta: response.drag_delta(),
                modifiers,
            })
        {
            return;
        }
        if response.drag_stopped()
            && self.dispatch_active_tool_event(CanvasToolEvent::DragStopped {
                point,
                pointer,
                modifiers,
            })
        {
            return;
        }
        if !(response.clicked()
            || response.drag_started()
            || response.dragged()
            || response.drag_stopped())
            && self.dispatch_active_tool_event(CanvasToolEvent::Hover {
                point,
                pointer,
                modifiers,
            })
        {
            return;
        }

        if response.clicked() {
            if self.active_text_box.is_some() {
                let inside_text_box = self
                    .active_text_box
                    .as_ref()
                    .map(|active| {
                        self.text_item_rect(rect, &active.item)
                            .expand(10.0)
                            .contains(pointer)
                    })
                    .unwrap_or(false);
                if !inside_text_box {
                    self.commit_active_text();
                    return;
                }
                return;
            }
            if self.active_tool == ToolKind::Select
                && self.selected_rect.is_some()
                && !self.point_in_selection(point)
            {
                self.commit_active_selection();
                return;
            }
            if self.active_tool == ToolKind::Text {
                self.begin_text_edit(point);
                return;
            }
        }

        if self.active_text_box.is_some() && self.active_tool == ToolKind::Text {
            return;
        }

        if self.active_tool == ToolKind::Shape && self.active_shape_kind() == Some(ShapeKind::Curve)
        {
            self.handle_curve_shape_input(point, response);
            return;
        }

        if response.drag_started() {
            self.drag_start = Some(point);
            self.last_point = Some(point);
            self.preview_point = Some(point);
            if matches!(
                self.active_tool,
                ToolKind::Brush | ToolKind::Pencil | ToolKind::Eraser | ToolKind::Shape
            ) {
                self.push_undo_snapshot();
            }
            if self.active_tool == ToolKind::Pencil {
                let primary = self.primary;
                self.active_canvas_mut()
                    .set_pixel(point.0, point.1, primary);
                self.mark_canvas_dirty();
            }
            if self.active_tool == ToolKind::Eraser {
                let brush_size = self.brush_size;
                let fill = if self.active_layer == 0 {
                    Color32::WHITE
                } else {
                    Color32::TRANSPARENT
                };
                crate::algorithm::draw_disc(
                    self.active_canvas_mut(),
                    point.0,
                    point.1,
                    (brush_size / 2).max(1),
                    fill,
                );
                self.mark_canvas_dirty();
            }
            if self.active_tool == ToolKind::Fill {
                self.fill_at(point);
            }
            if self.active_tool == ToolKind::Picker {
                if let Some(color) = self.pick_composited_pixel(point) {
                    self.primary = color;
                    self.remember_recent_color(color);
                    self.status = format!(
                        "{} #{:02X}{:02X}{:02X}",
                        self.tr(LanguageText::PickedColor),
                        color.r(),
                        color.g(),
                        color.b()
                    );
                    self.emit_event(AppEvent::ColorChanged {
                        primary: self.primary,
                        secondary: self.secondary,
                    });
                }
            }
            if self.active_tool == ToolKind::Magnifier {
                let magnifier = MagnifierTool::new();
                let _factor = magnifier.factor();
                let zoom = if response.ctx.input(|input| input.modifiers.shift) {
                    magnifier.zoom_out(self.zoom)
                } else {
                    magnifier.zoom_in(self.zoom)
                };
                self.set_view(zoom, self.canvas_pan);
            }
            if self.active_tool == ToolKind::Select {
                self.active_shape_rect = None;
                self.active_shape_snapshot = None;
                if self.point_in_selection(point) {
                    if let Some((start, end)) = self.normalized_selection() {
                        if let Some(content) = self.selection_content() {
                            self.push_undo_snapshot();
                            let fill = self.selection_clear_fill();
                            self.active_canvas_mut().clear_rect(start, end, fill);
                            self.remove_text_items_in_rect(start, end);
                            self.moving_selection = Some(MovingSelection {
                                content,
                                origin: start,
                                position: start,
                                drag_anchor: point,
                            });
                            self.status = self.tr(LanguageText::SelectionMove);
                            self.mark_canvas_dirty();
                        }
                    }
                } else {
                    self.moving_selection = None;
                    self.selected_rect = Some((point, point));
                }
            }
            if self.active_tool == ToolKind::Shape {
                if let Some(rect) = self.active_shape_rect {
                    if self.point_in_rect(point, rect) && self.active_shape_snapshot.is_some() {
                        self.moving_shape = Some((point, rect));
                    } else {
                        self.active_shape_snapshot = Some(self.active_canvas().snapshot());
                        self.active_shape_rect = None;
                        self.moving_shape = None;
                    }
                } else {
                    self.active_shape_snapshot = Some(self.active_canvas().snapshot());
                    self.moving_shape = None;
                }
            }
        }

        if response.dragged() && self.active_tool == ToolKind::Brush {
            let from = self.last_point.unwrap_or(point);
            if let Some(brush) = self.brush_group.active_brush_mut() {
                let active_canvas = if self.active_layer == 0 {
                    &mut self.canvas
                } else {
                    &mut self.pixel_layers[self.active_layer - 1].canvas
                };
                brush.draw_line(active_canvas, from, point, self.primary, self.brush_size);
                self.dirty_texture = true;
                self.document_dirty = true;
                self.emit_event(AppEvent::CanvasDirty);
            }
            self.last_point = Some(point);
        }

        if response.dragged() && self.active_tool == ToolKind::Pencil {
            let from = self.last_point.unwrap_or(point);
            let primary = self.primary;
            PencilTool::new().draw_line(
                &crate::algorithm::BresenhamLine::new(),
                self.active_canvas_mut(),
                from,
                point,
                primary,
                1,
            );
            self.last_point = Some(point);
            self.mark_canvas_dirty();
        }

        if response.dragged() && self.active_tool == ToolKind::Eraser {
            let from = self.last_point.unwrap_or(point);
            let brush_size = self.brush_size;
            let fill = if self.active_layer == 0 {
                Color32::WHITE
            } else {
                Color32::TRANSPARENT
            };
            crate::algorithm::BresenhamLine::new().draw_line_with_disc(
                self.active_canvas_mut(),
                from,
                point,
                fill,
                brush_size,
            );
            self.last_point = Some(point);
            self.mark_canvas_dirty();
        }

        if response.dragged() && self.active_tool == ToolKind::Shape {
            if let Some((anchor, (start, end))) = self.moving_shape {
                let dx = point.0 - anchor.0;
                let dy = point.1 - anchor.1;
                self.redraw_active_shape((start.0 + dx, start.1 + dy), (end.0 + dx, end.1 + dy));
            } else {
                self.preview_point = Some(point);
            }
        }

        if response.dragged() && self.active_tool == ToolKind::Select {
            if let Some(selection) = &mut self.moving_selection {
                let dx = point.0 - selection.drag_anchor.0;
                let dy = point.1 - selection.drag_anchor.1;
                selection.position = (selection.origin.0 + dx, selection.origin.1 + dy);
                self.selected_rect = Some((
                    selection.position,
                    (
                        selection.position.0 + selection.content.region.width as i32 - 1,
                        selection.position.1 + selection.content.region.height as i32 - 1,
                    ),
                ));
            } else if let Some(start) = self.drag_start {
                self.selected_rect = Some((start, point));
                self.status = self.tr(LanguageText::SelectionStatus);
            }
        }

        if response.drag_stopped() {
            if self.active_tool == ToolKind::Brush {
                self.emit_event(AppEvent::BrushStrokeCommitted);
            }
            if matches!(self.active_tool, ToolKind::Pencil | ToolKind::Eraser) {
                self.emit_event(AppEvent::BrushStrokeCommitted);
            }
            if self.active_tool == ToolKind::Shape {
                if self.moving_shape.is_some() {
                    self.moving_shape = None;
                } else if let Some(start) = self.drag_start {
                    let primary = self.primary;
                    let secondary = self.secondary;
                    let brush_size = self.brush_size;
                    let shape_mode = self.shape_mode;
                    let active_layer = self.active_layer;
                    let canvas = &mut self.canvas;
                    let pixel_layers = &mut self.pixel_layers;
                    let shape_group = &mut self.shape_group;
                    if let Some(shape) = shape_group.active_shape_mut() {
                        let active_canvas = if active_layer == 0 {
                            canvas
                        } else {
                            &mut pixel_layers[active_layer - 1].canvas
                        };
                        shape.draw(
                            active_canvas,
                            start,
                            point,
                            primary,
                            secondary,
                            brush_size,
                            shape_mode,
                        );
                        self.dirty_texture = true;
                        self.document_dirty = true;
                        self.active_shape_rect = Some((start, point));
                        self.selected_rect = None;
                        self.emit_event(AppEvent::ShapeCommitted);
                        self.emit_event(AppEvent::CanvasDirty);
                    }
                }
            }
            if self.active_tool == ToolKind::Select {
                if let Some(selection) = self.moving_selection.take() {
                    let position = selection.position;
                    let width = selection.content.region.width as i32;
                    let height = selection.content.region.height as i32;
                    let transparent = self.transparent_selection;
                    let transparent_color = self.secondary;
                    self.paste_selection_region(
                        position,
                        &selection.content.region,
                        transparent,
                        transparent_color,
                    );
                    self.paste_selection_text_items(position, &selection.content.text_items);
                    self.selected_rect =
                        Some((position, (position.0 + width - 1, position.1 + height - 1)));
                    self.status = self.tr(LanguageText::SelectionMove);
                    self.mark_canvas_dirty();
                } else if let Some(start) = self.drag_start {
                    self.selected_rect = Some((start, point));
                    self.status = self.tr(LanguageText::SelectionStatus);
                }
            }
            self.drag_start = None;
            self.last_point = None;
            self.preview_point = None;
        }
    }

    fn text_toolbar_window(&mut self, ctx: &Context) {
        if self.active_text_box.is_none() {
            return;
        }

        let font_label = self.tr(LanguageText::Font);
        let size_label = self.tr(LanguageText::FontSize);
        let background_label = self.tr(LanguageText::TextBackgroundFill);
        let commit_label = self.tr(LanguageText::CommitText);
        let cancel_label = self.tr(LanguageText::CancelText);
        let font_families = self.text_renderer.font_families();
        let mut commit = false;
        let mut cancel = false;

        egui::Window::new(self.tr(LanguageText::TextEditing))
            .id(egui::Id::new("text_toolbar_window"))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(
                egui::Align2::CENTER_TOP,
                Vec2::new(0.0, self.top_bar_height() + 8.0),
            )
            .show(ctx, |ui| {
                if let Some(active) = &mut self.active_text_box {
                    ui.horizontal(|ui| {
                        ui.label(font_label);
                        egui::ComboBox::from_id_source("text_font_family")
                            .selected_text(active.item.style.font_family.as_str())
                            .width(188.0)
                            .show_ui(ui, |ui| {
                                for font in &font_families {
                                    ui.selectable_value(
                                        &mut active.item.style.font_family,
                                        font.clone(),
                                        font.as_str(),
                                    );
                                }
                            });
                        ui.add_sized(
                            [72.0, 24.0],
                            egui::DragValue::new(&mut active.item.style.font_size)
                                .range(6.0..=144.0)
                                .speed(1.0)
                                .prefix(format!("{size_label} ")),
                        );
                        ui.separator();
                        if ui
                            .selectable_label(
                                active.item.style.bold,
                                egui::RichText::new("B").strong(),
                            )
                            .clicked()
                        {
                            active.item.style.bold = !active.item.style.bold;
                        }
                        if ui
                            .selectable_label(
                                active.item.style.italic,
                                egui::RichText::new("I").italics(),
                            )
                            .clicked()
                        {
                            active.item.style.italic = !active.item.style.italic;
                        }
                        if ui
                            .selectable_label(
                                active.item.style.underline,
                                egui::RichText::new("U").underline(),
                            )
                            .clicked()
                        {
                            active.item.style.underline = !active.item.style.underline;
                        }
                        if ui
                            .selectable_label(
                                active.item.style.strikethrough,
                                egui::RichText::new("S").strikethrough(),
                            )
                            .clicked()
                        {
                            active.item.style.strikethrough = !active.item.style.strikethrough;
                        }
                        ui.separator();
                        if text_align_button(ui, TextAlign::Left, active.item.style.align).clicked()
                        {
                            active.item.style.align = TextAlign::Left;
                        }
                        if text_align_button(ui, TextAlign::Center, active.item.style.align)
                            .clicked()
                        {
                            active.item.style.align = TextAlign::Center;
                        }
                        if text_align_button(ui, TextAlign::Right, active.item.style.align)
                            .clicked()
                        {
                            active.item.style.align = TextAlign::Right;
                        }
                        ui.separator();
                        ui.checkbox(&mut active.item.style.background_fill, background_label);
                        if ui.button(commit_label).clicked() {
                            commit = true;
                        }
                        if ui.button(cancel_label).clicked() {
                            cancel = true;
                        }
                    });
                }
            });

        if commit {
            self.commit_active_text();
        }
        if cancel {
            self.cancel_active_text();
        }
    }
}

impl AppHost for PaintApp {
    fn load_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
        self.tool_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    fn load_cursor_tool(&mut self, tool: Box<dyn CursorTool>) {
        self.cursor_tools.push(tool);
        self.cursor_tool_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    fn load_brush(&mut self, brush: Box<dyn Brush>) {
        self.brush_group.load_brush(brush);
        self.brush_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    fn load_shape(&mut self, shape: Box<dyn Shape>) {
        self.shape_group.load_shape(shape);
        self.shape_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    fn load_panel(&mut self, panel: Box<dyn Panel>) {
        self.panels.push(panel);
        self.panel_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    fn load_app_panel(&mut self, panel: Box<dyn AppPanel>) {
        PaintApp::load_app_panel(self, panel);
    }

    fn load_hook(&mut self, hook: Box<dyn AppHook>) {
        self.hooks.push(hook);
        self.hook_sources.push(
            self.loading_component_source
                .unwrap_or(EXTERNAL_COMPONENT_SOURCE),
        );
    }

    fn load_plugin(&mut self, mut plugin: Box<dyn Plugin>) {
        let source = plugin.plugin_name();
        self.plugin_sources.push(source);
        if self.plugin_source_enabled(source) {
            match self.activate_plugin_components(plugin.as_mut(), source) {
                Ok(()) => {
                    self.emit_event(AppEvent::PluginActivated);
                }
                Err(_) => {
                    self.disabled_plugins.insert(Self::plugin_key(source));
                }
            }
        }
        self.plugins.push(plugin);
    }

    fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    fn mark_canvas_dirty(&mut self) {
        self.dirty_texture = true;
        self.document_dirty = true;
        self.emit_event(AppEvent::CanvasDirty);
    }

    fn push_history_snapshot(&mut self) {
        self.push_undo_snapshot();
    }

    fn undo(&mut self) -> bool {
        self.undo_action()
    }

    fn redo(&mut self) -> bool {
        self.redo_action()
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn clear_history(&mut self) {
        self.clear_history_stack();
    }

    fn language(&self) -> &Language {
        &self.language
    }
}

impl Drop for PaintApp {
    fn drop(&mut self) {
        self.save_app_state();
        let mut plugins = std::mem::take(&mut self.plugins);
        let sources = std::mem::take(&mut self.plugin_sources);
        for (index, plugin) in plugins.iter_mut().enumerate() {
            let Some(source) = sources.get(index).copied() else {
                continue;
            };
            if !self.plugin_source_enabled(source) {
                continue;
            }
            let _ = self.deactivate_plugin_components(plugin.as_mut(), source);
        }
        self.plugins = plugins;
        self.plugin_sources = sources;
    }
}

impl eframe::App for PaintApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let color = Color32::from_rgb(242, 244, 248);
        [
            color.r() as f32 / 255.0,
            color.g() as f32 / 255.0,
            color.b() as f32 / 255.0,
            1.0,
        ]
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if ctx.input(|input| input.viewport().close_requested())
            && self.document_dirty
            && !self.confirm_discard_changes()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            return;
        }
        if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            if self.active_text_box.is_some() {
                self.cancel_active_text();
            }
        }
        if ctx.input_mut(|input| {
            input.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Z,
            ))
        }) {
            self.undo_action();
        }
        if ctx.input_mut(|input| {
            input.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::Y,
            ))
        }) {
            self.redo_action();
        }
        self.handle_selection_shortcuts(ctx);
        self.emit_event(AppEvent::BeforeUi);
        self.top_bar(ctx);
        self.status_bar(ctx);
        self.plugin_side_panels(ctx);
        self.canvas_panel(ctx);
        self.plugins_window(ctx);
        self.text_toolbar_window(ctx);
        self.color_editor_window(ctx);
        self.plugin_windows(ctx);
        self.active_tool_window(ctx);
        self.emit_event(AppEvent::AfterUi);
    }
}

fn system_font_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if cfg!(target_os = "windows") {
        dirs.push(PathBuf::from("C:/Windows/Fonts"));
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local_app_data).join("Microsoft/Windows/Fonts"));
        }
    } else if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/System/Library/Fonts"));
        dirs.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join("Library/Fonts"));
        }
    } else {
        dirs.push(PathBuf::from("/usr/share/fonts"));
        dirs.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = env::var_os("HOME") {
            let home = PathBuf::from(home);
            dirs.push(home.join(".fonts"));
            dirs.push(home.join(".local/share/fonts"));
        }
    }
    dirs
}

fn collect_font_files(
    dir: &Path,
    fonts: &mut Vec<SystemFont>,
    seen: &mut HashSet<String>,
    depth: usize,
) {
    if depth > 8 || fonts.len() >= 1024 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_font_files(&path, fonts, seen, depth + 1);
            continue;
        }
        if !is_supported_font_file(&path) {
            continue;
        }
        if load_font_arc(&path).is_none() {
            continue;
        }
        let Some(family) = font_family_from_file(&path) else {
            continue;
        };
        let family_key = family.to_lowercase();
        if !seen.insert(family_key) {
            continue;
        }
        let egui_key = format!("laydraw_system_font_{}", fonts.len());
        fonts.push(SystemFont {
            family,
            path,
            egui_key,
        });
    }
}

fn is_supported_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| matches!(extension.to_ascii_lowercase().as_str(), "ttf" | "otf"))
        .unwrap_or(false)
}

fn font_family_from_file(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let face_count = ttf_parser::fonts_in_collection(&bytes).unwrap_or(1).max(1);
    for face_index in 0..face_count.min(8) {
        let Ok(face) = ttf_parser::Face::parse(&bytes, face_index) else {
            continue;
        };
        if let Some(family) = font_family_from_face(&face) {
            return Some(family);
        }
    }
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.replace(['-', '_'], " "))
}

fn font_family_from_face(face: &ttf_parser::Face<'_>) -> Option<String> {
    for name_id in [
        ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
        ttf_parser::name_id::FAMILY,
        ttf_parser::name_id::FULL_NAME,
    ] {
        if let Some(name) = face
            .names()
            .into_iter()
            .filter(|name| name.name_id == name_id)
            .filter_map(|name| name.to_string())
            .find(|name| !name.trim().is_empty())
        {
            return Some(name);
        }
    }
    None
}

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

fn load_font_arc(path: &Path) -> Option<FontArc> {
    fs::read(path)
        .ok()
        .and_then(|bytes| FontArc::try_from_vec(bytes).ok())
}

fn render_text_item_to_pixels(
    pixels: &mut [Color32],
    width: usize,
    height: usize,
    item: &TextItem,
    font: &FontArc,
) {
    let x0 = item.position.0;
    let y0 = item.position.1;
    let text_w = item.size.0.max(1);
    let text_h = item.size.1.max(1);
    if item.style.background_fill {
        fill_export_rect(
            pixels,
            width,
            height,
            x0,
            y0,
            text_w,
            text_h,
            item.background,
        );
    }

    let scale = item.style.font_size.max(1.0);
    let scaled = font.as_scaled(scale);
    let ascent = scaled.ascent();
    let line_height = (scaled.ascent() - scaled.descent() + scaled.line_gap()).max(scale);
    let lines = wrap_export_text(font, scale, &item.text, (text_w - 8).max(1) as f32);
    let mut baseline_y = y0 as f32 + 4.0 + ascent;
    for line in lines {
        if baseline_y > (y0 + text_h) as f32 {
            break;
        }
        let line_width = measure_export_line(font, scale, &line);
        let start_x = match item.style.align {
            TextAlign::Left => x0 as f32 + 4.0,
            TextAlign::Center => x0 as f32 + (text_w as f32 - line_width) * 0.5,
            TextAlign::Right => x0 as f32 + text_w as f32 - 4.0 - line_width,
        };
        draw_export_line(
            pixels,
            width,
            height,
            font,
            scale,
            &line,
            start_x,
            baseline_y,
            item.color,
            item.style.bold,
        );
        if item.style.underline {
            draw_export_rule(
                pixels,
                width,
                height,
                start_x as i32,
                (baseline_y + 2.0) as i32,
                line_width as i32,
                item.color,
            );
        }
        if item.style.strikethrough {
            draw_export_rule(
                pixels,
                width,
                height,
                start_x as i32,
                (baseline_y - ascent * 0.35) as i32,
                line_width as i32,
                item.color,
            );
        }
        baseline_y += line_height;
    }
}

fn wrap_export_text(font: &FontArc, scale: f32, text: &str, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut current = String::new();
        for ch in source_line.chars() {
            let next = format!("{current}{ch}");
            if !current.is_empty() && measure_export_line(font, scale, &next) > max_width {
                lines.push(current);
                current = ch.to_string();
            } else {
                current = next;
            }
        }
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn measure_export_line(font: &FontArc, scale: f32, line: &str) -> f32 {
    let scaled = font.as_scaled(scale);
    let mut width = 0.0;
    let mut last = None;
    for ch in line.chars() {
        let id = scaled.glyph_id(ch);
        if let Some(last_id) = last {
            width += scaled.kern(last_id, id);
        }
        width += scaled.h_advance(id);
        last = Some(id);
    }
    width
}

fn draw_export_line(
    pixels: &mut [Color32],
    width: usize,
    height: usize,
    font: &FontArc,
    scale: f32,
    line: &str,
    x: f32,
    baseline_y: f32,
    color: Color32,
    bold: bool,
) {
    let scaled = font.as_scaled(scale);
    let mut cursor_x = x;
    let mut last = None;
    for ch in line.chars() {
        let id = scaled.glyph_id(ch);
        if let Some(last_id) = last {
            cursor_x += scaled.kern(last_id, id);
        }
        let glyph = id.with_scale_and_position(scale, point(cursor_x, baseline_y));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                let px = bounds.min.x as i32 + gx as i32;
                let py = bounds.min.y as i32 + gy as i32;
                blend_export_pixel(pixels, width, height, px, py, color, coverage);
                if bold {
                    blend_export_pixel(pixels, width, height, px + 1, py, color, coverage * 0.7);
                }
            });
        }
        cursor_x += scaled.h_advance(id);
        last = Some(id);
    }
}

fn fill_export_rect(
    pixels: &mut [Color32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    rect_w: i32,
    rect_h: i32,
    color: Color32,
) {
    for py in y.max(0)..(y + rect_h).min(height as i32) {
        for px in x.max(0)..(x + rect_w).min(width as i32) {
            blend_export_pixel(
                pixels,
                width,
                height,
                px,
                py,
                color,
                color.a() as f32 / 255.0,
            );
        }
    }
}

fn draw_export_rule(
    pixels: &mut [Color32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    rule_w: i32,
    color: Color32,
) {
    for px in x.max(0)..(x + rule_w).min(width as i32) {
        blend_export_pixel(pixels, width, height, px, y, color, 1.0);
    }
}

fn blend_export_pixel(
    pixels: &mut [Color32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    color: Color32,
    coverage: f32,
) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }
    let index = y as usize * width + x as usize;
    let alpha = (color.a() as f32 / 255.0 * coverage).clamp(0.0, 1.0);
    if alpha <= 0.0 {
        return;
    }
    let base = pixels[index];
    let inv = 1.0 - alpha;
    pixels[index] = Color32::from_rgba_unmultiplied(
        (base.r() as f32 * inv + color.r() as f32 * alpha).round() as u8,
        (base.g() as f32 * inv + color.g() as f32 * alpha).round() as u8,
        (base.b() as f32 * inv + color.b() as f32 * alpha).round() as u8,
        255,
    );
}

fn text_align_to_egui(align: TextAlign) -> egui::Align {
    match align {
        TextAlign::Left => egui::Align::Min,
        TextAlign::Center => egui::Align::Center,
        TextAlign::Right => egui::Align::Max,
    }
}

fn aligned_text_origin(rect: Rect, align: TextAlign) -> Pos2 {
    let y = rect.top() + 4.0;
    match align {
        TextAlign::Left => Pos2::new(rect.left() + 4.0, y),
        TextAlign::Center => Pos2::new(rect.center().x, y),
        TextAlign::Right => Pos2::new(rect.right() - 4.0, y),
    }
}

fn color_swatch(ui: &mut egui::Ui, color: Color32, selected: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
    ui.painter().rect_filled(rect, 14.0, color);
    if selected {
        ui.painter().circle_stroke(
            rect.center(),
            13.0,
            Stroke::new(1.5, Color32::from_rgb(0, 95, 184)),
        );
    }
    ui.painter().rect_stroke(
        rect,
        14.0,
        egui::Stroke::new(1.0, Color32::from_rgb(120, 128, 140)),
    );
    response
}

fn mini_color_dot(ui: &mut egui::Ui, color: Color32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::click());
    ui.painter().circle_filled(rect.center(), 7.0, color);
    ui.painter().circle_stroke(
        rect.center(),
        7.0,
        Stroke::new(0.8, Color32::from_rgb(130, 130, 130)),
    );
    response
}

fn color_number(
    ui: &mut egui::Ui,
    value: &mut i32,
    label: String,
    range: std::ops::RangeInclusive<i32>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        changed = ui
            .add_sized(
                [120.0, 28.0],
                egui::DragValue::new(value).range(range).speed(1),
            )
            .changed();
        ui.label(label);
    });
    changed
}

fn palette_dot(ui: &mut egui::Ui, color: Color32, empty: bool) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
    if empty {
        ui.painter().circle_stroke(
            rect.center(),
            9.0,
            Stroke::new(1.0, Color32::from_rgb(170, 170, 170)),
        );
    } else {
        ui.painter().circle_filled(rect.center(), 9.0, color);
        ui.painter().circle_stroke(
            rect.center(),
            9.0,
            Stroke::new(1.0, Color32::from_rgb(120, 120, 120)),
        );
    }
    response
}

fn text_align_button(ui: &mut egui::Ui, align: TextAlign, selected: TextAlign) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
    let active = align == selected;
    let visuals = ui.style().interact_selectable(&response, active);
    ui.painter().rect_filled(rect, 3.0, visuals.bg_fill);
    ui.painter().rect_stroke(rect, 3.0, visuals.bg_stroke);

    let color = visuals.text_color();
    let widths = [14.0, 10.0, 13.0, 8.0];
    let start_y = rect.center().y - 6.0;
    for (index, width) in widths.into_iter().enumerate() {
        let y = start_y + index as f32 * 4.0;
        let x0 = match align {
            TextAlign::Left => rect.left() + 5.0,
            TextAlign::Center => rect.center().x - width * 0.5,
            TextAlign::Right => rect.right() - 5.0 - width,
        };
        ui.painter().line_segment(
            [Pos2::new(x0, y), Pos2::new(x0 + width, y)],
            Stroke::new(1.4, color),
        );
    }
    response
}

fn parse_hex_color(hex: &str) -> Option<Color32> {
    let text = hex.trim().trim_start_matches('#');
    if text.len() != 6 {
        return None;
    }
    let value = u32::from_str_radix(text, 16).ok()?;
    Some(Color32::from_rgb(
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
    ))
}

fn ensure_image_extension(mut path: PathBuf) -> PathBuf {
    let supported = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "bmp"
            )
        })
        .unwrap_or(false);
    if !supported {
        path.set_extension("png");
    }
    path
}

fn display_path_label(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn format_color(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

fn parse_bool(value: &str, fallback: bool) -> bool {
    match value {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => fallback,
    }
}

fn rgb_to_hsv(color: Color32) -> (f32, f32, f32) {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta <= f32::EPSILON {
        0.0
    } else if (max - r).abs() <= f32::EPSILON {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if (max - g).abs() <= f32::EPSILON {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let saturation = if max <= f32::EPSILON {
        0.0
    } else {
        delta / max
    };
    (hue, saturation, max)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Color32 {
    let h = (hue.rem_euclid(360.0)) / 60.0;
    let c = value * saturation.clamp(0.0, 1.0);
    let x = c * (1.0 - (h.rem_euclid(2.0) - 1.0).abs());
    let m = value.clamp(0.0, 1.0) - c;
    let (r, g, b) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    Color32::from_rgb(
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
}

fn composite_layer(
    base: &mut [Color32],
    overlay: &[Color32],
    opacity: f32,
    blend_mode: LayerBlendMode,
) {
    let opacity = opacity.clamp(0.0, 1.0);
    if opacity <= 0.0 {
        return;
    }
    for (base, overlay) in base.iter_mut().zip(overlay.iter()) {
        let overlay_alpha = overlay.a();
        if overlay_alpha == 0 {
            continue;
        }
        if opacity >= 1.0 && overlay_alpha == 255 && blend_mode == LayerBlendMode::Normal {
            *base = *overlay;
            continue;
        }
        let alpha = overlay_alpha as f32 / 255.0 * opacity;
        if alpha <= 0.0 {
            continue;
        }
        let blended = blend_color(*base, *overlay, blend_mode);
        let inv = 1.0 - alpha;
        *base = Color32::from_rgba_unmultiplied(
            (base.r() as f32 * inv + blended.r() as f32 * alpha) as u8,
            (base.g() as f32 * inv + blended.g() as f32 * alpha) as u8,
            (base.b() as f32 * inv + blended.b() as f32 * alpha) as u8,
            255,
        );
    }
}

fn blend_color(base: Color32, overlay: Color32, blend_mode: LayerBlendMode) -> Color32 {
    let blend_channel = |base: u8, overlay: u8| -> u8 {
        match blend_mode {
            LayerBlendMode::Normal => overlay,
            LayerBlendMode::Multiply => ((base as u16 * overlay as u16) / 255) as u8,
            LayerBlendMode::Screen => {
                255 - (((255 - base as u16) * (255 - overlay as u16)) / 255) as u8
            }
        }
    };
    Color32::from_rgb(
        blend_channel(base.r(), overlay.r()),
        blend_channel(base.g(), overlay.g()),
        blend_channel(base.b(), overlay.b()),
    )
}

fn layer_blend_label<'a>(
    blend_mode: LayerBlendMode,
    normal: &'a str,
    multiply: &'a str,
    screen: &'a str,
) -> &'a str {
    match blend_mode {
        LayerBlendMode::Normal => normal,
        LayerBlendMode::Multiply => multiply,
        LayerBlendMode::Screen => screen,
    }
}

fn paint_layer_thumbnail(ui: &mut egui::Ui, canvas: &Canvas, background: bool) {
    let size = Vec2::new(42.0, 30.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, Color32::WHITE);
    if !background {
        let checker = 6.0;
        let cols = (rect.width() / checker).ceil() as i32;
        let rows = (rect.height() / checker).ceil() as i32;
        for row in 0..rows {
            for col in 0..cols {
                if (row + col) % 2 == 0 {
                    let cell = Rect::from_min_size(
                        Pos2::new(
                            rect.left() + col as f32 * checker,
                            rect.top() + row as f32 * checker,
                        ),
                        Vec2::splat(checker),
                    )
                    .intersect(rect);
                    painter.rect_filled(cell, 0.0, Color32::from_rgb(224, 224, 224));
                }
            }
        }
    }
    let sample_w = 14usize;
    let sample_h = 10usize;
    for sy in 0..sample_h {
        for sx in 0..sample_w {
            let source_x = sx * canvas.width / sample_w;
            let source_y = sy * canvas.height / sample_h;
            let color = canvas.pixels[source_y * canvas.width + source_x];
            if color.a() == 0 {
                continue;
            }
            let cell = Rect::from_min_max(
                Pos2::new(
                    rect.left() + sx as f32 * rect.width() / sample_w as f32,
                    rect.top() + sy as f32 * rect.height() / sample_h as f32,
                ),
                Pos2::new(
                    rect.left() + (sx + 1) as f32 * rect.width() / sample_w as f32,
                    rect.top() + (sy + 1) as f32 * rect.height() / sample_h as f32,
                ),
            );
            painter.rect_filled(cell, 0.0, color);
        }
    }
    painter.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, Color32::from_rgb(170, 176, 186)),
    );
}

fn swap_text_layers(text_items: &mut [TextItem], a: usize, b: usize) {
    for item in text_items {
        if item.layer == a {
            item.layer = b;
        } else if item.layer == b {
            item.layer = a;
        }
    }
}

fn draw_horizontal_ruler(painter: &egui::Painter, rect: Rect, width: usize, zoom: f32) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(245, 246, 248));
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(178, 184, 194)),
    );
    draw_ruler_ticks(painter, rect, width, zoom, true);
}

fn draw_vertical_ruler(painter: &egui::Painter, rect: Rect, height: usize, zoom: f32) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(245, 246, 248));
    painter.line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(178, 184, 194)),
    );
    draw_ruler_ticks(painter, rect, height, zoom, false);
}

fn draw_horizontal_ruler_cursor(painter: &egui::Painter, rect: Rect, x: i32, zoom: f32) {
    let marker_x = rect.left() + (x as f32 + 0.5) * zoom;
    if marker_x < rect.left() || marker_x > rect.right() {
        return;
    }
    painter.line_segment(
        [
            Pos2::new(marker_x, rect.top()),
            Pos2::new(marker_x, rect.bottom()),
        ],
        Stroke::new(1.5, Color32::from_rgb(230, 40, 45)),
    );
}

fn draw_vertical_ruler_cursor(painter: &egui::Painter, rect: Rect, y: i32, zoom: f32) {
    let marker_y = rect.top() + (y as f32 + 0.5) * zoom;
    if marker_y < rect.top() || marker_y > rect.bottom() {
        return;
    }
    painter.line_segment(
        [
            Pos2::new(rect.left(), marker_y),
            Pos2::new(rect.right(), marker_y),
        ],
        Stroke::new(1.5, Color32::from_rgb(230, 40, 45)),
    );
}

fn draw_ruler_ticks(
    painter: &egui::Painter,
    rect: Rect,
    pixels: usize,
    zoom: f32,
    horizontal: bool,
) {
    let major_step = if zoom >= 4.0 {
        10
    } else if zoom >= 1.0 {
        50
    } else {
        100
    };
    let minor_step = (major_step / 5).max(1);
    let font = FontId::monospace(10.0);
    let color = Color32::from_rgb(83, 91, 102);

    for value in (0..=pixels).step_by(minor_step) {
        let is_major = value % major_step == 0;
        let offset = value as f32 * zoom;
        if horizontal {
            let x = rect.left() + offset;
            if x > rect.right() {
                break;
            }
            let tick = if is_major { 12.0 } else { 6.0 };
            painter.line_segment(
                [
                    Pos2::new(x, rect.bottom()),
                    Pos2::new(x, rect.bottom() - tick),
                ],
                Stroke::new(1.0, color),
            );
            if is_major {
                painter.text(
                    Pos2::new(x + 2.0, rect.top() + 2.0),
                    egui::Align2::LEFT_TOP,
                    value.to_string(),
                    font.clone(),
                    color,
                );
            }
        } else {
            let y = rect.top() + offset;
            if y > rect.bottom() {
                break;
            }
            let tick = if is_major { 12.0 } else { 6.0 };
            painter.line_segment(
                [
                    Pos2::new(rect.right(), y),
                    Pos2::new(rect.right() - tick, y),
                ],
                Stroke::new(1.0, color),
            );
            if is_major {
                painter.text(
                    Pos2::new(rect.left() + 2.0, y + 2.0),
                    egui::Align2::LEFT_TOP,
                    value.to_string(),
                    font.clone(),
                    color,
                );
            }
        }
    }
}

fn consume_ctrl_or_command(input: &mut egui::InputState, key: egui::Key) -> bool {
    let modifiers = input.modifiers;
    let wanted = (modifiers.ctrl || modifiers.command) && !modifiers.alt && input.key_pressed(key);
    if wanted {
        input.consume_key(modifiers, key);
    }
    wanted
}

fn write_selection_to_system_clipboard(content: &SelectionContent) {
    let bytes = selection_region_rgba_bytes(&content.region);
    let image = arboard::ImageData {
        width: content.region.width,
        height: content.region.height,
        bytes: std::borrow::Cow::Owned(bytes),
    };
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_image(image);
    }
}

fn read_system_clipboard_selection() -> Option<SelectionContent> {
    read_clipboard_image_file()
        .or_else(read_clipboard_windows_bitmap)
        .or_else(read_clipboard_image)
        .or_else(read_clipboard_text_image_path)
}

fn system_clipboard_has_image() -> bool {
    #[cfg(windows)]
    {
        use clipboard_win::{Format, formats};
        if clipboard_win::Clipboard::new_attempts(2).is_ok()
            && (formats::FileList.is_format_avail()
                || formats::RawData(formats::CF_DIBV5).is_format_avail()
                || formats::RawData(formats::CF_DIB).is_format_avail()
                || formats::Bitmap.is_format_avail())
        {
            return true;
        }
    }

    let Ok(mut clipboard) = arboard::Clipboard::new() else {
        return false;
    };
    clipboard.get_image().is_ok()
        || clipboard
            .get_text()
            .is_ok_and(|text| text_has_image_path(&text))
}

fn read_clipboard_image() -> Option<SelectionContent> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let image = clipboard.get_image().ok()?;
    let pixels = image
        .bytes
        .chunks_exact(4)
        .map(|chunk| Color32::from_rgba_unmultiplied(chunk[0], chunk[1], chunk[2], chunk[3]))
        .collect();
    Some(SelectionContent {
        region: CanvasRegion {
            width: image.width,
            height: image.height,
            pixels,
        },
        text_items: Vec::new(),
    })
}

#[cfg(windows)]
fn read_clipboard_image_file() -> Option<SelectionContent> {
    use clipboard_win::{formats, get_clipboard};

    let paths: Vec<PathBuf> = get_clipboard(formats::FileList).ok()?;
    paths
        .into_iter()
        .find_map(|path| image_io::load_canvas(&path).ok())
        .map(canvas_to_selection_content)
}

#[cfg(not(windows))]
fn read_clipboard_image_file() -> Option<SelectionContent> {
    None
}

fn read_clipboard_text_image_path() -> Option<SelectionContent> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    let text = clipboard.get_text().ok()?;
    text.lines()
        .map(|line| line.trim().trim_matches('"'))
        .map(PathBuf::from)
        .find_map(|path| image_io::load_canvas(&path).ok())
        .map(canvas_to_selection_content)
}

#[cfg(windows)]
fn read_clipboard_windows_bitmap() -> Option<SelectionContent> {
    use clipboard_win::formats;

    read_clipboard_windows_dib(formats::CF_DIBV5)
        .or_else(|| read_clipboard_windows_dib(formats::CF_DIB))
        .or_else(read_clipboard_windows_bitmap_handle)
}

#[cfg(not(windows))]
fn read_clipboard_windows_bitmap() -> Option<SelectionContent> {
    None
}

#[cfg(windows)]
fn read_clipboard_windows_dib(format: u32) -> Option<SelectionContent> {
    use clipboard_win::{formats, get_clipboard};

    let bytes: Vec<u8> = get_clipboard(formats::RawData(format)).ok()?;
    selection_content_from_dib_bytes(&bytes)
}

#[cfg(windows)]
fn read_clipboard_windows_bitmap_handle() -> Option<SelectionContent> {
    use clipboard_win::{formats, get_clipboard};

    let bytes: Vec<u8> = get_clipboard(formats::Bitmap).ok()?;
    selection_content_from_encoded_image(&bytes)
}

#[cfg(windows)]
fn selection_content_from_dib_bytes(bytes: &[u8]) -> Option<SelectionContent> {
    if bytes.len() < 16 {
        return None;
    }
    let header_size = u32::from_le_bytes(bytes.get(0..4)?.try_into().ok()?) as usize;
    if header_size < 12 || bytes.len() < header_size {
        return None;
    }

    let file_header_size = 14usize;
    let mut bmp = Vec::with_capacity(file_header_size + bytes.len());
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&((file_header_size + bytes.len()) as u32).to_le_bytes());
    bmp.extend_from_slice(&[0, 0, 0, 0]);
    bmp.extend_from_slice(&(file_header_size as u32 + bitmap_pixel_offset(bytes)?).to_le_bytes());
    bmp.extend_from_slice(bytes);
    selection_content_from_encoded_image(&bmp)
}

#[cfg(windows)]
fn bitmap_pixel_offset(bytes: &[u8]) -> Option<u32> {
    let header_size = u32::from_le_bytes(bytes.get(0..4)?.try_into().ok()?) as usize;
    if header_size == 12 {
        let bit_count = u16::from_le_bytes(bytes.get(10..12)?.try_into().ok()?);
        let colors = if bit_count <= 8 { 1u32 << bit_count } else { 0 };
        return Some(header_size as u32 + colors * 3);
    }

    let bit_count = u16::from_le_bytes(bytes.get(14..16)?.try_into().ok()?);
    let compression = u32::from_le_bytes(bytes.get(16..20)?.try_into().ok()?);
    let clr_used = u32::from_le_bytes(bytes.get(32..36).unwrap_or(&[0, 0, 0, 0]).try_into().ok()?);
    let palette_colors = if clr_used != 0 {
        clr_used
    } else if bit_count <= 8 {
        1u32 << bit_count
    } else {
        0
    };
    let masks = if matches!(compression, 3 | 6) && header_size == 40 {
        if compression == 6 { 16 } else { 12 }
    } else {
        0
    };
    Some(header_size as u32 + masks + palette_colors * 4)
}

fn selection_content_from_encoded_image(bytes: &[u8]) -> Option<SelectionContent> {
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (width, height) = image.dimensions();
    let pixels = image
        .pixels()
        .map(|pixel| Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]))
        .collect();
    Some(SelectionContent {
        region: CanvasRegion {
            width: width as usize,
            height: height as usize,
            pixels,
        },
        text_items: Vec::new(),
    })
}

fn text_has_image_path(text: &str) -> bool {
    text.lines()
        .map(|line| line.trim().trim_matches('"'))
        .filter(|line| !line.is_empty())
        .map(Path::new)
        .any(|path| path.exists() && image_io::load_canvas(path).is_ok())
}

fn canvas_to_selection_content(canvas: Canvas) -> SelectionContent {
    SelectionContent {
        region: CanvasRegion {
            width: canvas.width,
            height: canvas.height,
            pixels: canvas.pixels,
        },
        text_items: Vec::new(),
    }
}

fn selection_region_rgba_bytes(region: &CanvasRegion) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(region.pixels.len() * 4);
    for color in &region.pixels {
        bytes.extend_from_slice(&[color.r(), color.g(), color.b(), color.a()]);
    }
    bytes
}

fn text_item_canvas_rect(item: &TextItem) -> ((i32, i32), (i32, i32)) {
    (
        item.position,
        (
            item.position.0 + item.size.0.max(1) - 1,
            item.position.1 + item.size.1.max(1) - 1,
        ),
    )
}

fn canvas_rects_intersect(a: ((i32, i32), (i32, i32)), b: ((i32, i32), (i32, i32))) -> bool {
    let (a_start, a_end) = a;
    let (b_start, b_end) = b;
    let a_left = a_start.0.min(a_end.0);
    let a_top = a_start.1.min(a_end.1);
    let a_right = a_start.0.max(a_end.0);
    let a_bottom = a_start.1.max(a_end.1);
    let b_left = b_start.0.min(b_end.0);
    let b_top = b_start.1.min(b_end.1);
    let b_right = b_start.0.max(b_end.0);
    let b_bottom = b_start.1.max(b_end.1);

    a_left <= b_right && a_right >= b_left && a_top <= b_bottom && a_bottom >= b_top
}

fn scale_selection_content(
    content: &SelectionContent,
    width: usize,
    height: usize,
) -> SelectionContent {
    let old_width = content.region.width.max(1) as f32;
    let old_height = content.region.height.max(1) as f32;
    let scale_x = width.max(1) as f32 / old_width;
    let scale_y = height.max(1) as f32 / old_height;
    let text_items = content
        .text_items
        .iter()
        .cloned()
        .map(|mut item| {
            item.position = (
                (item.position.0 as f32 * scale_x).round() as i32,
                (item.position.1 as f32 * scale_y).round() as i32,
            );
            item.size = (
                (item.size.0 as f32 * scale_x).round().max(1.0) as i32,
                (item.size.1 as f32 * scale_y).round().max(1.0) as i32,
            );
            item.style.font_size = (item.style.font_size * ((scale_x + scale_y) * 0.5)).max(1.0);
            item
        })
        .collect();
    SelectionContent {
        region: scale_region_nearest(&content.region, width, height),
        text_items,
    }
}

fn scale_region_nearest(region: &CanvasRegion, width: usize, height: usize) -> CanvasRegion {
    let width = width.max(1);
    let height = height.max(1);
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        let source_y = y * region.height / height;
        for x in 0..width {
            let source_x = x * region.width / width;
            pixels.push(region.pixels[source_y * region.width + source_x]);
        }
    }
    CanvasRegion {
        width,
        height,
        pixels,
    }
}

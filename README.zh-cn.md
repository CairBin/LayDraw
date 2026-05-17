# LayDraw

LayDraw 是一个轻量、可扩展的绘图与图像编辑框架及应用，使用 Rust + egui/eframe 构建。
采用类似「画图（Paint）」的工作流，整体为插件化架构：工具、画笔、图形、面板、钩子、应用面板均可扩展。
本项目并非要替代专业图像软件，而是为 macOS / Linux（Windows 也支持）提供一个简单、轻量、可扩展的绘图工具。

## 特性

- 画图风格画布：缩放、平移、标尺、网格、可调整画布大小
- 工具：选择、铅笔、画笔、橡皮、填充、取色器、文字、放大镜、形状 + 插件工具
- 形状绘制：轮廓/填充、可编辑外框、缩放/移动、复制/剪切/粘贴、右键菜单
- 文字工具：扫描系统字体、支持中日韩字体、行内编辑、文字框移动/缩放、样式控制、屏幕/导出共用渲染路径
- 图层：可见性、透明度、混合模式、重命名、缩略图、虚拟滚动、拖拽排序
- 撤销/重做栈：插件可调用命令
- 剪贴板：选区、位图、Windows 图片文件粘贴
- 图像导入导出：基于 `image` 库
- 插件管理器：启用/禁用插件与组件
- 可扩展面板：顶部栏、标签页、侧边栏、底部栏、浮动窗口


## 构建与运行

安装 Rust 工具链后执行：

```bash
cargo run
```

发布构建

```bash
cargo build --release
```

* 在工作空间根目录 cargo run 会运行 laydraw_app
* 根目录 laydraw 是框架 / 库；laydraw_app 负责把插件打包进可执行文件

## 插件

#### 插件模型

插件实现`src/plugins.rs`中的Plugin，并通过`AppHost`注册组件：

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

可注册项：
* `load_tool`：普通工具按钮
* `load_cursor_tool`：工具 + 自定义光标绘制
* `load_brush`：画笔下拉项（含预览）
* `load_shape`：图形面板项
* `load_panel`：通用面板（功能区 / 侧边栏 / 底部栏 / 窗口）
* `load_app_panel`：应用标签面板（Home / 视图 / 图层扩展）
* `load_hook`：生命周期 / 文档事件钩子
* `load_plugin`：嵌套插件注册


插件窗口显示信息：
* `plugin_name`：稳定 ID（组件归属、持久化启用状态）
* `plugin_title`：本地化显示名
* `plugin_author`：作者 / 组织
* `plugin_version`：版本
* `supported_laydraw_versions`：兼容范围
* `plugin_url`：项目主页
* `plugin_email`：联系邮箱



### 插件子包

框架通过`src/lib.rs`暴露模块，插件可作为独立的Cargo包依赖主框架，并随主程序一同编译

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


你需要在插件目录下的`laydraw-plugin.toml`中声明版本兼容

```toml
id = "my.plugin"
title = "My Plugin"
# 精确版本
support = "0.1.0"
# 最大版本
support = "<=0.1.0"
# 版本范围
support = ">=0.1.0, <0.2.0"
```

等效键：support / supported_laydraw / laydraw_version / max_laydraw_version。`max_laydraw_version = "0.1.0"` 等价于 `<=0.1.0`

模板请参考`./plugin_packages/example_package_plugin`

示例内容：
* 从 `laydraw::plugins` / `laydraw::ui::panel` 导入 trait
* 声明依赖 once_cell
* laydraw-plugin.toml 兼容声明
* 导出 `pub fn plugin() -> Box<dyn Plugin>`
* 通过 `AppHost` 注册面板

1. 如果你想要把插件编译到应用中，你必须在`./laydraw_app/Cargo.toml`添加

```toml
[dependencies]
my-laydraw-plugin = { path = "../plugin_packages/my_laydraw_plugin" }
```

2. 并在`./laydraw_app/src/main.rs`中加载

```rust
let mut app = ui::PaintApp::new(cc);
app.load_plugin(my_laydraw_plugin::plugin());
```

检查整个工作空间（含插件）：

```bash
cargo check --workspace
```

`./laydraw_app/build.rs`会扫描所有的`./plugin_packages/*/laydraw-plugin.toml`文件，并与框架版本进行对比，不兼容会造成编译失败

### 加载说明

Rust trait 对象插件为静态编译，保证类型安全。插件子包可独立编译，打包进主程序，需主应用依赖插件包并调用工厂函数。

由于Rust trait对象跨动态库ABI不稳定，目前**不支持动态加载**


### 插件i18n

插件通过 `PanelContext` / `CanvasToolContext` / `ToolUiContext` / `EventContext` 获取当前语言。

内置语言判断

```rust
if let Some(zh_cn) = context.language.as_zh_cn_simple() {
    let label = zh_cn.get_text(LanguageText::Extra("插件文本".to_owned()));
}
if let Some(en_us) = context.language.as_en_us() {
    let label = en_us.get_text(LanguageText::Extra("Plugin text".to_owned()));
}
```

简洁写法（推荐）：

```rust
let text = match context.language.plugin_locale_key() {
    "zh-CN" => "印章设置",
    "en-US" => "Stamp settings",
    other if other == "my-language-pack" => "自定义文本",
    _ => "Stamp settings",
};
let label = context.language.plugin_text(text);
```

* `LanguageText::Extra`: 内置语言包透传，插件可自管翻译
* 第三方语言包：`plugin_locale_key()` 返回包名，插件可匹配翻译表

面板标题、按钮、菜单、提示文字统一用此模式。示例包已包含国际化演示。

### 工具扩展

Tool 支持：画布事件、叠加层、右键菜单、浮动工具窗口。

常用方法：
* `tool_id`：稳定组件 ID
* `get_tool_kind`：内置 / ToolKind::Extra
* `tool_button`：自定义按钮渲染
* `tool_button_context_menu`：按钮右键菜单
* `wants_canvas_events`：声明接收画布事件
* `on_canvas_event`：处理点击 / 拖拽 / 悬停 / 结束
* `paint_canvas_overlay`：绘制临时叠加 UI
* `has_canvas_context_menu` + `canvas_context_menu`：画布右键菜单
* `has_tool_window` + `tool_window`：浮动设置窗口

不要直接改状态，通过`CanvasToolContext`发命令：

```rust
context.command(AppCommand::PushHistorySnapshot);
context.command(AppCommand::MarkCanvasDirty);
context.command(AppCommand::SetStatus("完成".to_owned()));
```


### 面板扩展

通用面板实现`Panel`:

* `PanelArea::TopBar`
* `PanelArea::RibbonTab("tab id")`
* `PanelArea::LeftBar`
* `PanelArea::RightBar`
* `PanelArea::BottomBar`
* `PanelArea::Window`

应用面板实现 `AppPanel`：可扩展 Home / 视图 / 图层 等内置标签页，适合深度集成。

### 钩子事件(Hook)

钩子实现`AppHook`，接收`AppEvent` + `EventContext`。

事件分类：
* 应用生命周期：Startup / BeforeUi / AfterUi
* 画布：BeforeCanvasPaint / AfterCanvasPaint / CanvasDirty / CanvasResized
* 工具：ActiveToolChanged / BrushSizeChanged / ColorChanged
* 图层：新增 / 删除 / 移动 / 合并 / 清空
* 文字 / 选区 / 图片导入
* 历史：撤销 / 重做 / 快照 / 清空
* 插件：加载 / 卸载 / 成功 / 失败
* 钩子改状态也要用 `context.command(...)`，保持宿主为唯一命令分发者。

最简钩子示例：

```rust
struct MyHook;
impl AppHook for MyHook {
    fn hook_id(&self) -> &'static str {
        "my.hook"
    }
    fn on_event(&mut self, event: &AppEvent, context: &mut EventContext<'_>) {
        match event {
            AppEvent::Startup => {
                context.command(AppCommand::SetStatus("应用已启动".to_owned()));
            }
            _ => {}
        }
    }
}
```

常用事件一览：

```rust
fn on_event(&mut self, event: &AppEvent, context: &mut EventContext<'_>) {
    match event {
        AppEvent::Startup => {
            context.command(AppCommand::SetStatus("启动中".to_owned()));
        }
        AppEvent::BeforeUi => { /* UI 绘制前 */ }
        AppEvent::AfterUi => { /* UI 绘制后 */ }
        AppEvent::BeforeCanvasPaint => { /* 画布绘制前 */ }
        AppEvent::AfterCanvasPaint => { /* 画布绘制后 */ }
        AppEvent::CanvasDirty => { *context.dirty_texture = true; }
        AppEvent::ColorChanged { primary, secondary } => {}
        AppEvent::BrushSizeChanged { size } => {}
        AppEvent::ActiveLayerChanged { layer } => {}
        AppEvent::LayerAdded { layer } => {}
        AppEvent::LayerDeleted { layer } => {}
        AppEvent::LayerMoved { layer } => {}
        AppEvent::LayerMerged => {}
        AppEvent::LayerCleared { layer } => {}
        AppEvent::SelectionChanged => {}
        AppEvent::TextCommitted => {}
        AppEvent::TextCanceled => {}
        AppEvent::ImageImported { width, height } => {}
        AppEvent::LanguageChanged => {}
        AppEvent::BrushStrokeCommitted => {}
        AppEvent::ShapeCommitted => {}
        AppEvent::Undo => {}
        AppEvent::Redo => {}
        AppEvent::HistorySnapshotPushed => {}
        AppEvent::HistoryCleared => {}
        AppEvent::PluginActivated => {}
        AppEvent::PluginDeactivated => {}
        AppEvent::PluginAfterLoad { plugin } => {}
        AppEvent::PluginBeforeUnload { plugin } => {}
        AppEvent::PluginLoadFailed { plugin, error } => {}
        AppEvent::PluginUnloadFailed { plugin, error } => {}
        _ => {}
    }
}
```

## 内部架构

### 命令系统

`AppCommand`是唯一修改路径。工具/面板/钩子发命令，并不直接修改字段:

```rust
context.command(AppCommand::PushHistorySnapshot);
context.command(AppCommand::SetPrimaryColor(egui::Color32::RED));
context.command(AppCommand::MarkCanvasDirty);
```
由`PaintApp::apply_commands`统一执行：历史、脏标记、图层、选区、视图更新。


### 事件系统

`PaintApp::emit_event`构造`EventContext`，广播给所有启用钩子；禁用插件 / 钩子跳过。钩子可读状态、发命令；所有钩子执行完再统一应用命令。

### 画布与纹理

`Canvas`: `width`/`height`/`pixels: Vec<egui::Color32>`

* 像素修改: 调用`mark_canvas_dirty` -> `dirty_texture = true`
* 渲染：仅当脏标记为真时，才上传纹理

### 图层

* 背景层为0层
* 像素图层：`pixel_layers`
* 图层属性： `name`/`canvas`/`visible`/`opacity`/`blend_mode`
* UI：虚拟列表，大量图层不卡顿
* 合成：不透明普通图层快速路径

### 历史操作

撤销/重做：`DocumentSnapshot`

* 保存：背景画布、像素图层、当前图层、图层面板、文字项
* 一次操作一次快照，并非每帧都推


### 选区
* `SelectionContent`：画布区域 + 选中文字项
* 移动 / 缩放 / 粘贴：优先保留文字元数据，非必要不栅格化


### 文字渲染
* `TextRenderer`：扫描系统字体、注册 TTF/OTF、屏幕 / 导出共用渲染
* 优先选择 CJK 兼容字体

### 工具

* 实现`Tool` trait；`wants_canvas_events=true` 时接收 `CanvasToolEvent`
* UI 工具：按钮 / 菜单 / 窗口，不处理拖拽

### 画笔

* 实现`Brush` trait；在画笔工具内选择
* `paint_preview`：预览绘制
* 大尺寸优化：稀疏采样、优化线条原语

### 图形

* 实现 `Shape` trait，通过 `ShapeGroup` / `AppHost::load_shape` 注册
* 内置：直线 / 曲线 / 矩形 / 椭圆 / 多边形 / 标注 / 闪电
* 绘制参数：轮廓色 / 填充色 / 粗细 / 模式


### 面板

* 通用面板：`Panel` + `PanelContext`
* 应用面板：`AppPanel` + `&mut PaintApp`（内置 / 高可信插件）

### 插件管理器

* 记录组件归属：`plugin_name()`
* 启用状态：持久化
* 显示名：`plugin_title(&Language)`（可随语言变化）

### 性能建议

性能敏感路径：
* 粗线：胶囊填充代替逐像素画圆
* 纹理画笔：按尺寸采样，超大尺寸降采样
* 图层列表：虚拟行
* 图层合成：不透明普通图层快速路径
* 画布上传：仅脏标记时更新

插件开发：

* 拖拽过程不频繁分配大数组
* 仅操作开始推一次撤销快照
* 优先用 CanvasToolContext 命令
* 仅像素 / 可见状态真变化时标记脏画布
* 面板高度稳定；宽内容用横向滚动或独立窗口

## 配置文件

运行时组件/设置状态保存到`laydraw_components.cfg`
Git 忽略，仅本地用户状态。
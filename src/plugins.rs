use crate::{
    canvas::Canvas,
    i18n::Language,
    tools::{CursorTool, Tool, ToolKind, brush::Brush, shape::Shape},
    ui::panel::{Panel, app::AppPanel},
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    Startup,
    BeforeUi,
    AfterUi,
    BeforeCanvasPaint,
    AfterCanvasPaint,
    ActiveToolChanged {
        tool: ToolKind,
    },
    ViewChanged {
        zoom: f32,
        pan: egui::Vec2,
    },
    CanvasResized {
        width: usize,
        height: usize,
    },
    CanvasDirty,
    ColorChanged {
        primary: egui::Color32,
        secondary: egui::Color32,
    },
    BrushSizeChanged {
        size: i32,
    },
    ActiveLayerChanged {
        layer: usize,
    },
    LayerAdded {
        layer: usize,
    },
    LayerDeleted {
        layer: usize,
    },
    LayerMoved {
        layer: usize,
    },
    LayerMerged,
    LayerCleared {
        layer: usize,
    },
    SelectionChanged,
    TextCommitted,
    TextCanceled,
    ImageImported {
        width: usize,
        height: usize,
    },
    LanguageChanged,
    BrushStrokeCommitted,
    ShapeCommitted,
    Undo,
    Redo,
    HistorySnapshotPushed,
    HistoryCleared,
    PluginActivated,
    PluginDeactivated,
    PluginAfterLoad {
        plugin: &'static str,
    },
    PluginBeforeUnload {
        plugin: &'static str,
    },
    PluginLoadFailed {
        plugin: &'static str,
        error: String,
    },
    PluginUnloadFailed {
        plugin: &'static str,
        error: String,
    },
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum AppCommand {
    MarkCanvasDirty,
    PushHistorySnapshot,
    Undo,
    Redo,
    ClearHistory,
    SetStatus(String),
    SetActiveTool(ToolKind),
    SetView { zoom: f32, pan: egui::Vec2 },
    ResizeCanvas { width: usize, height: usize },
    SetPrimaryColor(egui::Color32),
    SetSecondaryColor(egui::Color32),
    SwapColors,
    SetBrushSize(i32),
    SetActiveLayer(usize),
    AddLayer,
    DeleteActiveLayer,
    MoveActiveLayerUp,
    MoveActiveLayerDown,
    MergeActiveLayerDown,
    MergeVisibleLayers,
    ClearCurrentLayer,
    ClearSelection,
}

#[allow(dead_code)]
pub struct EventContext<'a> {
    pub canvas: &'a mut Canvas,
    pub dirty_texture: &'a mut bool,
    pub language: &'a Language,
    pub status: &'a mut String,
    pub active_tool: ToolKind,
    pub primary: egui::Color32,
    pub secondary: egui::Color32,
    pub brush_size: i32,
    pub active_layer: usize,
    pub layer_count: usize,
    pub selected_rect: Option<((i32, i32), (i32, i32))>,
    pub pointer_canvas_pos: Option<(i32, i32)>,
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub commands: Vec<AppCommand>,
}

impl<'a> EventContext<'a> {
    pub fn command(&mut self, command: AppCommand) {
        self.commands.push(command);
    }
}

pub trait AppHook {
    fn hook_id(&self) -> &'static str {
        "hook"
    }

    fn hook_title(&self) -> &'static str {
        self.hook_id()
    }

    fn on_event(&mut self, _event: &AppEvent, _context: &mut EventContext<'_>) {}
}

#[allow(dead_code)]
pub trait AppHost {
    fn load_tool(&mut self, tool: Box<dyn Tool>);
    fn load_cursor_tool(&mut self, tool: Box<dyn CursorTool>);
    fn load_brush(&mut self, brush: Box<dyn Brush>);
    fn load_shape(&mut self, shape: Box<dyn Shape>);
    fn load_panel(&mut self, panel: Box<dyn Panel>);
    fn load_app_panel(&mut self, panel: Box<dyn AppPanel>);
    fn load_hook(&mut self, hook: Box<dyn AppHook>);
    fn load_plugin(&mut self, plugin: Box<dyn Plugin>);
    fn canvas(&self) -> &Canvas;
    fn canvas_mut(&mut self) -> &mut Canvas;
    fn mark_canvas_dirty(&mut self);
    fn push_history_snapshot(&mut self);
    fn undo(&mut self) -> bool;
    fn redo(&mut self) -> bool;
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;
    fn clear_history(&mut self);
    fn language(&self) -> &Language;
}

pub trait Plugin {
    fn plugin_name(&self) -> &'static str {
        "Plugin"
    }

    fn plugin_title(&self, language: &Language) -> String {
        language.plugin_text(self.plugin_name())
    }

    fn plugin_author(&self) -> &'static str {
        ""
    }

    fn plugin_version(&self) -> &'static str {
        ""
    }

    fn supported_laydraw_versions(&self) -> &'static str {
        "*"
    }

    fn plugin_url(&self) -> &'static str {
        ""
    }

    fn plugin_email(&self) -> &'static str {
        ""
    }

    fn before_load(&mut self, _app_host: &mut dyn AppHost) -> Result<(), String> {
        Ok(())
    }

    fn active(&mut self, app_host: &mut dyn AppHost);

    fn after_load(&mut self, _app_host: &mut dyn AppHost) -> Result<(), String> {
        Ok(())
    }

    fn on_load_error(&mut self, _app_host: &mut dyn AppHost, _error: &str) {}

    fn before_unload(&mut self, _app_host: &mut dyn AppHost) -> Result<(), String> {
        Ok(())
    }

    fn inactive(&mut self, app_host: &mut dyn AppHost);

    fn after_unload(&mut self, _app_host: &mut dyn AppHost) -> Result<(), String> {
        Ok(())
    }

    fn on_unload_error(&mut self, _app_host: &mut dyn AppHost, _error: &str) {}
}

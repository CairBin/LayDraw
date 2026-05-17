use crate::{
    canvas::Canvas,
    i18n::{Language, LanguageText},
    plugins::AppCommand,
};

pub mod brush;
pub mod eraser;
pub mod fill;
pub mod magnifier;
pub mod pencil;
pub mod picker;
pub mod select;
pub mod shape;
pub mod text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolKind {
    Select,
    Pencil,
    Brush,
    Eraser,
    Fill,
    Picker,
    Text,
    Magnifier,
    Shape,

    Extra,
}

pub trait Tool {
    fn tool_id(&self) -> &'static str {
        match self.get_tool_kind() {
            ToolKind::Select => "tool.select",
            ToolKind::Pencil => "tool.pencil",
            ToolKind::Brush => "tool.brush",
            ToolKind::Eraser => "tool.eraser",
            ToolKind::Fill => "tool.fill",
            ToolKind::Picker => "tool.picker",
            ToolKind::Text => "tool.text",
            ToolKind::Magnifier => "tool.magnifier",
            ToolKind::Shape => "tool.shape",
            ToolKind::Extra => "tool.extra",
        }
    }

    fn get_tool_kind(&self) -> ToolKind;

    fn get_tool_label(&self, current_language: &Language) -> LanguageText;

    fn tool_button(
        &mut self,
        ui: &mut egui::Ui,
        current_language: &Language,
        selected: bool,
    ) -> egui::Response {
        ui.selectable_label(
            selected,
            current_language.get_text(self.get_tool_label(current_language)),
        )
    }

    fn tool_button_context_menu(&mut self, _ui: &mut egui::Ui, _context: &mut ToolUiContext<'_>) {}

    fn has_canvas_context_menu(&self) -> bool {
        false
    }

    fn canvas_context_menu(&mut self, _ui: &mut egui::Ui, _context: &mut CanvasToolContext<'_>) {}

    fn has_tool_window(&self) -> bool {
        false
    }

    fn tool_window(&mut self, _ctx: &egui::Context, _context: &mut CanvasToolContext<'_>) {}

    fn wants_canvas_events(&self) -> bool {
        false
    }

    fn on_canvas_event(
        &mut self,
        _event: CanvasToolEvent,
        _context: &mut CanvasToolContext<'_>,
    ) -> bool {
        false
    }

    fn paint_canvas_overlay(
        &mut self,
        _ui: &mut egui::Ui,
        _canvas_rect: egui::Rect,
        _context: &mut CanvasToolContext<'_>,
    ) {
    }
}

pub trait CursorTool: Tool + crate::ui::cursor::Cursor {}

impl<T> CursorTool for T where T: Tool + crate::ui::cursor::Cursor {}

#[allow(dead_code)]
pub struct ToolUiContext<'a> {
    pub language: &'a Language,
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

impl<'a> ToolUiContext<'a> {
    pub fn command(&mut self, command: AppCommand) {
        self.commands.push(command);
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum CanvasToolEvent {
    Hover {
        point: (i32, i32),
        pointer: egui::Pos2,
        modifiers: egui::Modifiers,
    },
    Click {
        point: (i32, i32),
        pointer: egui::Pos2,
        modifiers: egui::Modifiers,
    },
    DragStarted {
        point: (i32, i32),
        pointer: egui::Pos2,
        modifiers: egui::Modifiers,
    },
    Dragged {
        point: (i32, i32),
        pointer: egui::Pos2,
        delta: egui::Vec2,
        modifiers: egui::Modifiers,
    },
    DragStopped {
        point: (i32, i32),
        pointer: egui::Pos2,
        modifiers: egui::Modifiers,
    },
}

#[allow(dead_code)]
pub struct CanvasToolContext<'a> {
    pub canvas: &'a mut Canvas,
    pub language: &'a Language,
    pub primary: egui::Color32,
    pub secondary: egui::Color32,
    pub brush_size: i32,
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub active_layer: usize,
    pub layer_count: usize,
    pub commands: Vec<AppCommand>,
}

impl<'a> CanvasToolContext<'a> {
    pub fn command(&mut self, command: AppCommand) {
        self.commands.push(command);
    }
}

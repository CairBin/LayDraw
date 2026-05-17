pub mod app;
pub mod builtin;

use crate::i18n::Language;
use crate::plugins::AppCommand;
use crate::tools::ToolKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PanelArea {
    TopBar,
    RibbonTab(&'static str),
    LeftBar,
    RightBar,
    BottomBar,
    Window,
}

#[allow(dead_code)]
pub struct PanelContext<'a> {
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
    pub status: &'a str,
    pub commands: Vec<AppCommand>,
}

impl<'a> PanelContext<'a> {
    pub fn command(&mut self, command: AppCommand) {
        self.commands.push(command);
    }
}

pub trait Panel {
    fn panel_id(&self) -> &'static str;

    fn panel_title(&self, current_language: &Language) -> String;

    fn panel_area(&self) -> PanelArea {
        PanelArea::TopBar
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut PanelContext<'_>);
}

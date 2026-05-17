use crate::{
    i18n::{Language, LanguageText},
    tools::{Tool, ToolKind},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MagnifierTool {
    factor: f32,
}

impl MagnifierTool {
    pub fn new() -> Self {
        Self { factor: 1.25 }
    }

    pub fn factor(&self) -> f32 {
        self.factor
    }

    #[allow(dead_code)]
    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor.clamp(1.01, 8.0);
    }

    pub fn zoom_in(&self, zoom: f32) -> f32 {
        (zoom * self.factor).clamp(0.05, 64.0)
    }

    pub fn zoom_out(&self, zoom: f32) -> f32 {
        (zoom / self.factor).clamp(0.05, 64.0)
    }
}

impl Default for MagnifierTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for MagnifierTool {
    fn get_tool_kind(&self) -> ToolKind {
        ToolKind::Magnifier
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::Magnifier
    }
}

impl crate::ui::cursor::Cursor for MagnifierTool {
    fn cursor(&self) -> crate::ui::cursor::MyCursorIcon<'_> {
        crate::ui::cursor::MyCursorIcon::EguiCursorIcon(egui::CursorIcon::ZoomIn)
    }
}

use crate::{
    i18n::{Language, LanguageText},
    tools::{Tool, ToolKind},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PickerTool;

impl PickerTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for PickerTool {
    fn get_tool_kind(&self) -> ToolKind {
        ToolKind::Picker
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::Picker
    }
}

impl crate::ui::cursor::Cursor for PickerTool {
    fn cursor(&self) -> crate::ui::cursor::MyCursorIcon<'_> {
        crate::ui::cursor::MyCursorIcon::EguiCursorIcon(egui::CursorIcon::Crosshair)
    }
}

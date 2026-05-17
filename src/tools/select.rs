use crate::{
    i18n::{Language, LanguageText},
    tools::{Tool, ToolKind},
};

pub struct SelectTool;

impl SelectTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for SelectTool {
    fn get_tool_kind(&self) -> ToolKind {
        ToolKind::Select
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::Select
    }
}

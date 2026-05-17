use crate::{
    i18n::{Language, LanguageText},
    tools::{Tool, ToolKind},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextTool {
    text: String,
    font_size: i32,
}

impl TextTool {
    pub fn new() -> Self {
        Self {
            text: "Text".to_owned(),
            font_size: 18,
        }
    }

    #[allow(dead_code)]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[allow(dead_code)]
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    #[allow(dead_code)]
    pub fn font_size(&self) -> i32 {
        self.font_size
    }

    #[allow(dead_code)]
    pub fn set_font_size(&mut self, font_size: i32) {
        self.font_size = font_size.max(1);
    }
}

impl Default for TextTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for TextTool {
    fn get_tool_kind(&self) -> ToolKind {
        ToolKind::Text
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::TextTool
    }
}

impl crate::ui::cursor::Cursor for TextTool {
    fn cursor(&self) -> crate::ui::cursor::MyCursorIcon<'_> {
        crate::ui::cursor::MyCursorIcon::EguiCursorIcon(egui::CursorIcon::Text)
    }
}

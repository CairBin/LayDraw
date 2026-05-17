#![allow(dead_code)]

use crate::{
    i18n::{Language, LanguageText},
    ui::panel::{Panel, PanelArea, PanelContext},
};

pub struct StatusHintPanel {
    id: &'static str,
}

impl StatusHintPanel {
    pub fn new(id: &'static str) -> Self {
        Self { id }
    }
}

impl Panel for StatusHintPanel {
    fn panel_id(&self) -> &'static str {
        self.id
    }

    fn panel_title(&self, current_language: &Language) -> String {
        current_language.get_text(LanguageText::StatusBar)
    }

    fn panel_area(&self) -> PanelArea {
        PanelArea::BottomBar
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut PanelContext<'_>) {
        ui.label(context.language.get_text(LanguageText::Ready));
    }
}

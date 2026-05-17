use crate::ui::PaintApp;

pub mod builtin;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppPanelArea {
    Home,
    View,
    Layers,
}

pub trait AppPanel {
    fn panel_id(&self) -> &'static str;

    fn panel_area(&self) -> AppPanelArea;

    fn ui(&mut self, app: &mut PaintApp, ui: &mut egui::Ui);
}

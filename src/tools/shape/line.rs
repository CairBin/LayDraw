use crate::{
    algorithm::{BresenhamLine, LineAlgorithm},
    canvas::Canvas,
    i18n::{Language, LanguageText},
    tools::shape::{Shape, ShapeKind, ShapeMode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineShape;

impl LineShape {
    pub fn new() -> Self {
        Self
    }
}

impl Shape for LineShape {
    fn get_shape_kind(&self) -> ShapeKind {
        ShapeKind::Line
    }

    fn get_shape_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::Line
    }

    fn draw(
        &mut self,
        canvas: &mut Canvas,
        start: (i32, i32),
        end: (i32, i32),
        outline: egui::Color32,
        _fill: egui::Color32,
        thickness: i32,
        _mode: ShapeMode,
    ) {
        BresenhamLine::new().draw_line_with_disc(canvas, start, end, outline, thickness.max(1));
    }
}

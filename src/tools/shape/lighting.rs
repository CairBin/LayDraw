use crate::{ordered_pair, tools::shape::Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightingShape;

impl LightingShape {
    pub fn new() -> Self {
        Self
    }

    fn lightning_points(&self, start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32)> {
        let (left, right) = ordered_pair!(start.0, end.0);
        let (top, bottom) = ordered_pair!(start.1, end.1);
        let w = (right - left).max(1);
        let h = (bottom - top).max(1);
        vec![
            (left + w * 3 / 5, top),
            (left + w / 4, top + h / 2),
            (left + w / 2, top + h / 2),
            (left + w * 2 / 5, bottom),
            (right - w / 5, top + h / 3),
            (left + w / 2, top + h / 3),
        ]
    }
}

// impl Tool for LightingShape {
//     fn get_tool_kind(&self) -> crate::tools::ToolKind {
//         crate::tools::ToolKind::Shape(self.get_shape_kind())
//     }

//     fn get_tool_label(
//         &self,
//         _current_language: &crate::i18n::Language,
//     ) -> crate::i18n::LanguageText {
//         crate::i18n::LanguageText::Shapes
//     }

//     fn cursor(&self) -> crate::tools::MyCursorIcon<'_> {
//         crate::tools::MyCursorIcon::EguiCursorIcon(egui::CursorIcon::Crosshair)
//     }
// }

impl Shape for LightingShape {
    fn get_shape_kind(&self) -> super::ShapeKind {
        super::ShapeKind::Lightning
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::Lightning
    }

    fn draw(
        &mut self,
        canvas: &mut crate::canvas::Canvas,
        start: (i32, i32),
        end: (i32, i32),
        outline: egui::Color32,
        fill: egui::Color32,
        thickness: i32,
        mode: super::ShapeMode,
    ) {
        let points = self.lightning_points(start, end);
        crate::algorithm::draw_polygon_shape(canvas, &points, outline, fill, thickness, mode);
    }
}

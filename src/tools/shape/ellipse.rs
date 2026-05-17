use super::ShapeMode;
use crate::{ordered_pair, tools::shape::Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EllipseShape;

impl EllipseShape {
    pub fn new() -> Self {
        Self
    }
}

// impl Tool for EllipseShape {
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

impl Shape for EllipseShape {
    fn get_shape_kind(&self) -> super::ShapeKind {
        super::ShapeKind::Ellipse
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::Oval
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
        let (left, right) = ordered_pair!(start.0, end.0);
        let (top, bottom) = ordered_pair!(start.1, end.1);
        let width = (right - left).max(1) as f32;
        let height = (bottom - top).max(1) as f32;
        let cx = (left + right) as f32 / 2.0;
        let cy = (top + bottom) as f32 / 2.0;
        let rx = width / 2.0;
        let ry = height / 2.0;
        let outline_band = (thickness.max(1) as f32 / rx.min(ry).max(1.0)).min(0.35);

        for y in top..=bottom {
            for x in left..=right {
                let nx = (x as f32 - cx) / rx.max(1.0);
                let ny = (y as f32 - cy) / ry.max(1.0);
                let d = nx * nx + ny * ny;

                if matches!(mode, ShapeMode::Filled | ShapeMode::FilledOutline) && d <= 1.0 {
                    canvas.set_pixel(x, y, fill);
                }

                if matches!(mode, ShapeMode::Outline | ShapeMode::FilledOutline)
                    && d <= 1.0
                    && d >= 1.0 - outline_band
                {
                    canvas.set_pixel(x, y, outline);
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EllipseCallout;

impl EllipseCallout {
    pub fn new() -> Self {
        Self
    }
}

// impl Tool for EllipseCallout {
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

impl Shape for EllipseCallout {
    fn get_shape_kind(&self) -> super::ShapeKind {
        super::ShapeKind::EllipseCallout
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::OvalCallout
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
        EllipseShape::new().draw(canvas, start, end, outline, fill, thickness, mode);
        crate::algorithm::draw_polyline(
            canvas,
            &crate::callout_tail!(start, end),
            outline,
            thickness,
            false,
        );
    }
}

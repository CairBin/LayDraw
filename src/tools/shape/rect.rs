use crate::{callout_tail, ordered_pair, tools::shape::Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectShape;

impl RectShape {
    pub fn new() -> Self {
        RectShape
    }
}

// impl Tool for RectShape {
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

impl Shape for RectShape {
    fn get_shape_kind(&self) -> super::ShapeKind {
        super::ShapeKind::Rectangle
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::Rectangle
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
        let thickness = thickness.max(1);

        if matches!(
            mode,
            super::ShapeMode::Filled | super::ShapeMode::FilledOutline
        ) {
            for y in top..=bottom {
                for x in left..=right {
                    canvas.set_pixel(x, y, fill);
                }
            }
        }

        if matches!(
            mode,
            super::ShapeMode::Outline | super::ShapeMode::FilledOutline
        ) {
            for offset in 0..thickness {
                for x in left - offset..=right + offset {
                    canvas.set_pixel(x, top - offset, outline);
                    canvas.set_pixel(x, bottom + offset, outline);
                }
                for y in top - offset..=bottom + offset {
                    canvas.set_pixel(left - offset, y, outline);
                    canvas.set_pixel(right + offset, y, outline);
                }
            }
        }
    }
}

/// 圆角矩形
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundedRectShape;

impl RoundedRectShape {
    pub fn new() -> Self {
        RoundedRectShape
    }
}

// impl Tool for RoundedRectShape {
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

impl Shape for RoundedRectShape {
    fn get_shape_kind(&self) -> super::ShapeKind {
        super::ShapeKind::RoundedRectangle
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::RoundedRectangle
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
        let radius = ((right - left).min(bottom - top) / 5).max(2);
        let thickness = thickness.max(1);

        if matches!(
            mode,
            super::ShapeMode::Filled | super::ShapeMode::FilledOutline
        ) {
            for y in top..=bottom {
                for x in left..=right {
                    if crate::algorithm::rounded_rect_contains(
                        x, y, left, top, right, bottom, radius,
                    ) {
                        canvas.set_pixel(x, y, fill);
                    }
                }
            }
        }

        if matches!(
            mode,
            super::ShapeMode::Outline | super::ShapeMode::FilledOutline
        ) {
            for y in top - thickness..=bottom + thickness {
                for x in left - thickness..=right + thickness {
                    let outer = crate::algorithm::rounded_rect_contains(
                        x, y, left, top, right, bottom, radius,
                    );
                    let inner = crate::algorithm::rounded_rect_contains(
                        x,
                        y,
                        left + thickness,
                        top + thickness,
                        right - thickness,
                        bottom - thickness,
                        (radius - thickness).max(1),
                    );
                    if outer && !inner {
                        canvas.set_pixel(x, y, outline);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectCallout;

impl RectCallout {
    pub fn new() -> Self {
        Self
    }
}

// impl Tool for RectCallout {
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
impl Shape for RectCallout {
    fn get_shape_kind(&self) -> super::ShapeKind {
        super::ShapeKind::RectCallout
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::RectCallout
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
        RectShape::new().draw(canvas, start, end, outline, fill, thickness, mode);
        crate::algorithm::draw_polyline(
            canvas,
            &callout_tail!(start, end),
            outline,
            thickness,
            false,
        );
    }
}

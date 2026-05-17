use crate::{ordered_pair, tools::shape::Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolygonShape {
    kind: super::ShapeKind,
}

impl PolygonShape {
    pub fn new(kind: super::ShapeKind) -> Self {
        Self { kind }
    }

    pub fn polygon_points(
        kind: super::ShapeKind,
        start: (i32, i32),
        end: (i32, i32),
    ) -> Vec<(i32, i32)> {
        let (left, right) = ordered_pair!(start.0, end.0);
        let (top, bottom) = ordered_pair!(start.1, end.1);
        let w = (right - left).max(1);
        let h = (bottom - top).max(1);
        let cx = left + w / 2;
        let cy = top + h / 2;

        match kind {
            super::ShapeKind::Triangle => vec![(cx, top), (right, bottom), (left, bottom)],
            super::ShapeKind::RightTriangle => vec![(left, top), (right, bottom), (left, bottom)],
            super::ShapeKind::Diamond => vec![(cx, top), (right, cy), (cx, bottom), (left, cy)],
            super::ShapeKind::Pentagon => Self::regular_polygon(cx, cy, w, h, 5, -90.0),
            super::ShapeKind::Hexagon => Self::regular_polygon(cx, cy, w, h, 6, 30.0),
            super::ShapeKind::Polygon => vec![
                (left + w / 4, top),
                (right, top + h / 3),
                (right - w / 5, bottom),
                (left, bottom - h / 4),
                (left, top + h / 3),
            ],
            super::ShapeKind::RightArrow => vec![
                (left, top + h / 3),
                (left + w * 2 / 3, top + h / 3),
                (left + w * 2 / 3, top),
                (right, cy),
                (left + w * 2 / 3, bottom),
                (left + w * 2 / 3, bottom - h / 3),
                (left, bottom - h / 3),
            ],
            super::ShapeKind::LeftArrow => vec![
                (right, top + h / 3),
                (left + w / 3, top + h / 3),
                (left + w / 3, top),
                (left, cy),
                (left + w / 3, bottom),
                (left + w / 3, bottom - h / 3),
                (right, bottom - h / 3),
            ],
            super::ShapeKind::UpArrow => vec![
                (left + w / 3, bottom),
                (left + w / 3, top + h / 3),
                (left, top + h / 3),
                (cx, top),
                (right, top + h / 3),
                (right - w / 3, top + h / 3),
                (right - w / 3, bottom),
            ],
            super::ShapeKind::DownArrow => vec![
                (left + w / 3, top),
                (right - w / 3, top),
                (right - w / 3, bottom - h / 3),
                (right, bottom - h / 3),
                (cx, bottom),
                (left, bottom - h / 3),
                (left + w / 3, bottom - h / 3),
            ],
            super::ShapeKind::FourPointStar => Self::star_points(cx, cy, w, h, 4),
            super::ShapeKind::FivePointStar => Self::star_points(cx, cy, w, h, 5),
            super::ShapeKind::SixPointStar => Self::star_points(cx, cy, w, h, 6),
            _ => vec![(left, top), (right, top), (right, bottom), (left, bottom)],
        }
    }

    pub fn regular_polygon(
        cx: i32,
        cy: i32,
        w: i32,
        h: i32,
        sides: usize,
        rotation_deg: f32,
    ) -> Vec<(i32, i32)> {
        let rx = w as f32 / 2.0;
        let ry = h as f32 / 2.0;
        (0..sides)
            .map(|i| {
                let angle = (rotation_deg + i as f32 * 360.0 / sides as f32).to_radians();
                (
                    cx + (angle.cos() * rx) as i32,
                    cy + (angle.sin() * ry) as i32,
                )
            })
            .collect()
    }

    pub fn star_points(cx: i32, cy: i32, w: i32, h: i32, points: usize) -> Vec<(i32, i32)> {
        let outer_rx = w as f32 / 2.0;
        let outer_ry = h as f32 / 2.0;
        let inner_rx = outer_rx * 0.42;
        let inner_ry = outer_ry * 0.42;

        (0..points * 2)
            .map(|i| {
                let angle = (-90.0 + i as f32 * 180.0 / points as f32).to_radians();
                let (rx, ry) = if i % 2 == 0 {
                    (outer_rx, outer_ry)
                } else {
                    (inner_rx, inner_ry)
                };
                (
                    cx + (angle.cos() * rx) as i32,
                    cy + (angle.sin() * ry) as i32,
                )
            })
            .collect()
    }
}

// impl Tool for PolygonShape {
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

impl Shape for PolygonShape {
    fn get_shape_kind(&self) -> super::ShapeKind {
        self.kind.clone()
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        match self.kind {
            super::ShapeKind::Polygon => crate::i18n::LanguageText::Polygon,
            super::ShapeKind::Triangle => crate::i18n::LanguageText::Triangle,
            super::ShapeKind::RightTriangle => crate::i18n::LanguageText::RightTriangle,
            super::ShapeKind::Diamond => crate::i18n::LanguageText::Diamond,
            super::ShapeKind::Pentagon => crate::i18n::LanguageText::Pentagon,
            super::ShapeKind::Hexagon => crate::i18n::LanguageText::Hexagon,
            super::ShapeKind::RightArrow => crate::i18n::LanguageText::RightArrow,
            super::ShapeKind::LeftArrow => crate::i18n::LanguageText::LeftArrow,
            super::ShapeKind::UpArrow => crate::i18n::LanguageText::UpArrow,
            super::ShapeKind::DownArrow => crate::i18n::LanguageText::DownArrow,
            super::ShapeKind::FourPointStar => crate::i18n::LanguageText::FourPointStar,
            super::ShapeKind::FivePointStar => crate::i18n::LanguageText::FivePointStar,
            super::ShapeKind::SixPointStar => crate::i18n::LanguageText::SixPointStar,
            _ => panic!("Unknown shape kind: {:?}", self.kind),
        }
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
        let points = Self::polygon_points(self.kind, start, end);
        crate::algorithm::draw_polygon_shape(canvas, &points, outline, fill, thickness, mode);
    }
}

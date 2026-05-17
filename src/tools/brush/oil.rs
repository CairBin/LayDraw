use super::{Brush, BrushKind};
use crate::i18n::{Language, LanguageText};
use crate::{
    algorithm::{LineAlgorithm, brush_sample_stride},
    canvas::Canvas,
};
use crate::{pseudo_noise, scale_color};
use egui::Color32;

/// 油画笔
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OilBrush;
impl OilBrush {
    pub fn new() -> Self {
        Self
    }

    fn draw_oil_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
        step: i32,
    ) {
        let radius = (size / 2).max(1);
        for band in -3..=3 {
            let wobble = (pseudo_noise!(x + band, y - band, step) * 4.0 - 2.0) as i32;
            let shade = 0.55 + pseudo_noise!(x - band * 5, y + band, step) * 0.42;
            let stroke = (size / 5).max(1);
            crate::algorithm::BresenhamLine::new().draw_line_with_disc(
                canvas,
                (x - radius, y + band + wobble),
                (x + radius, y + band - wobble / 2),
                scale_color!(color, shade),
                stroke,
            );
            if band.abs() <= 1 {
                crate::algorithm::BresenhamLine::new().draw_line_with_disc(
                    canvas,
                    (x - radius / 2, y + band),
                    (x + radius / 2, y + band),
                    scale_color!(color, 1.12),
                    1,
                );
            }
        }
    }

    fn draw_brush_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
        step: i32,
    ) {
        let size = size.max(1);
        self.draw_oil_stamp(canvas, x, y, color, size, step);
    }
}

// impl Tool for OilBrush {
//     fn get_tool_kind(&self) -> crate::tools::ToolKind {
//         crate::tools::ToolKind::Brush(self.get_brush_kind())
//     }

//     fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
//         LanguageText::Brush
//     }

//     fn cursor(&self) -> crate::tools::MyCursorIcon<'_> {
//         crate::tools::MyCursorIcon::EguiCursorIcon(egui::CursorIcon::Crosshair)
//     }
// }

impl Brush for OilBrush {
    fn get_brush_kind(&self) -> BrushKind {
        BrushKind::Oil
    }

    fn get_brush_label(&self, _curren_language: &Language) -> LanguageText {
        LanguageText::BrushKindOil
    }

    fn draw_line(
        &mut self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: Color32,
        size: i32,
    ) {
        let stride = brush_sample_stride(size, 5);
        crate::algorithm::for_each_line_point(from, to, |x, y, step| {
            if step % stride == 0 {
                self.draw_brush_stamp(canvas, x, y, color, size, step as i32);
            }
        });
        self.draw_brush_stamp(canvas, to.0, to.1, color, size, 0);
    }
}

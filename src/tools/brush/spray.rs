use super::{Brush, BrushKind};
use crate::i18n::LanguageText;
use crate::{algorithm::brush_sample_stride, canvas::Canvas, i18n::Language, pseudo_noise};
use egui::Color32;

/// 喷雾画笔
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SprayBrush;

impl SprayBrush {
    pub fn new() -> Self {
        Self
    }

    fn draw_spray_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
        step: i32,
    ) {
        let radius = size.max(3);
        let density = (size * 5).max(28);
        for i in 0..density {
            let seed = pseudo_noise!(x + i, y - i, step + i);
            let angle = seed * std::f32::consts::TAU;
            let dist = pseudo_noise!(x - i * 3, y + i * 7, step).sqrt() * radius as f32;
            let px = x + (angle.cos() * dist) as i32;
            let py = y + (angle.sin() * dist) as i32;
            let speck = 0.28 + pseudo_noise!(px, py, step + i) * 0.45;
            canvas.blend_pixel(px, py, color, speck);
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
        self.draw_spray_stamp(canvas, x, y, color, size, step);
    }
}

// impl Tool for SprayBrush {
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

impl Brush for SprayBrush {
    fn get_brush_kind(&self) -> BrushKind {
        BrushKind::Spray
    }

    fn get_brush_label(&self, _curren_language: &Language) -> LanguageText {
        LanguageText::BrushKindSpray
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

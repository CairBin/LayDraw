use super::{Brush, BrushKind};
use crate::i18n::LanguageText;
use crate::{algorithm::brush_sample_stride, canvas::Canvas, i18n::Language};
use egui::Color32;

/// 毛笔
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalligraphyBrush;

impl CalligraphyBrush {
    pub fn new() -> Self {
        Self
    }

    fn draw_slanted_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
    ) {
        let radius = (size / 2).max(1);
        for py in y - radius..=y + radius {
            for px in x - radius..=x + radius {
                let dx = px - x;
                let dy = py - y;
                let chisel = (dx * 2 + dy).abs();
                if chisel <= radius && dy.abs() <= radius / 2 + 1 {
                    let alpha = if chisel < radius / 2 { 1.0 } else { 0.78 };
                    canvas.blend_pixel(px, py, color, alpha);
                }
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
        _step: i32,
    ) {
        let size = size.max(1);
        self.draw_slanted_stamp(canvas, x, y, color, size);
    }
}

// impl Tool for CalligraphyBrush {
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

impl Brush for CalligraphyBrush {
    fn get_brush_kind(&self) -> BrushKind {
        BrushKind::Calligraphy
    }

    fn get_brush_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::BrushKindCalligraphy
    }

    fn draw_line(
        &mut self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: Color32,
        size: i32,
    ) {
        let stride = brush_sample_stride(size, 3);
        crate::algorithm::for_each_line_point(from, to, |x, y, step| {
            if step % stride == 0 {
                self.draw_brush_stamp(canvas, x, y, color, size, step as i32);
            }
        });
        self.draw_brush_stamp(canvas, to.0, to.1, color, size, 0);
    }
}

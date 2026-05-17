use super::{Brush, BrushKind};
use crate::i18n::{Language, LanguageText};
use crate::pseudo_noise;
use crate::{algorithm::brush_sample_stride, canvas::Canvas};
use egui::Color32;
/// 蜡笔
pub struct CrayonBrush;

impl CrayonBrush {
    pub fn new() -> Self {
        Self
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
        self.draw_crayon_stamp(canvas, x, y, color, size, step);
    }

    fn draw_crayon_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
        step: i32,
    ) {
        let radius = (size / 2).max(1);
        for py in y - radius / 2..=y + radius / 2 {
            for px in x - radius..=x + radius {
                let dx = px - x;
                let dy = (py - y) * 2;
                if dx * dx + dy * dy <= radius * radius {
                    let grain = pseudo_noise!(px / 2, py / 2, step);
                    let scratch = pseudo_noise!(px + step * 3, py - step, 11);
                    if grain > 0.24 || scratch > 0.82 {
                        let alpha = (0.5 + grain * 0.5).min(0.95);
                        canvas.blend_pixel(px, py, color, alpha);
                    }
                }
            }
        }
    }
}

// impl Tool for CrayonBrush {
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

impl Brush for CrayonBrush {
    fn get_brush_kind(&self) -> BrushKind {
        super::BrushKind::Crayon
    }

    fn get_brush_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::BrushKindCrayon
    }
    fn draw_line(
        &mut self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: egui::Color32,
        size: i32,
    ) {
        let stride = brush_sample_stride(size, 4);
        crate::algorithm::for_each_line_point(from, to, |x, y, step| {
            if step % stride == 0 {
                self.draw_brush_stamp(canvas, x, y, color, size, step as i32);
            }
        });
        self.draw_brush_stamp(canvas, to.0, to.1, color, size, 0);
    }
}

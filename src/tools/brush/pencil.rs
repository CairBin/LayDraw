use super::{Brush, BrushKind};
use crate::i18n::LanguageText;
use crate::{algorithm::brush_sample_stride, canvas::Canvas, i18n::Language};
use egui::Color32;

/// 铅笔画笔
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PencilBrush;

impl PencilBrush {
    pub fn new() -> Self {
        Self
    }

    fn draw_tapered_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
        step: i32,
        alpha: f32,
    ) {
        let radius = (size / 3).max(1);
        let offset = step.rem_euclid((radius * 2).max(1)) - radius;
        for py in y - radius..=y + radius {
            for px in x - radius..=x + radius {
                let dx = px - x;
                let dy = py - y - offset / 3;
                let width = radius + offset.abs() / 4;
                if dx * dx + dy * dy <= width * radius {
                    let edge = 1.0 - ((dx * dx + dy * dy) as f32 / (width * radius).max(1) as f32);
                    canvas.blend_pixel(px, py, color, alpha * edge.clamp(0.35, 1.0));
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
        step: i32,
    ) {
        let size = size.max(1);
        self.draw_tapered_stamp(canvas, x, y, color, size, step as i32, 1.0);
    }
}

// impl Tool for PencilBrush {
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

impl Brush for PencilBrush {
    fn get_brush_kind(&self) -> BrushKind {
        BrushKind::PencilBrush
    }

    fn get_brush_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::BrushKindPencil
    }

    fn draw_line(
        &mut self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: Color32,
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

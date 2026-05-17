use super::{Brush, BrushKind};
use crate::i18n::LanguageText;
use crate::{
    algorithm::brush_sample_stride, canvas::Canvas, i18n::Language, pseudo_noise, scale_color,
};
use egui::Color32;

/// 自然铅笔笔刷
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NaturalPencilBrush;

impl NaturalPencilBrush {
    pub fn new() -> Self {
        NaturalPencilBrush
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
        self.draw_natural_pencil_stamp(canvas, x, y, color, size, step);
    }

    fn draw_natural_pencil_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
        step: i32,
    ) {
        let radius = (size / 4).max(1);
        let pencil_color = scale_color!(color, 0.58);
        for py in y - radius..=y + radius {
            for px in x - radius..=x + radius {
                let dx = px - x;
                let dy = py - y;
                if dx * dx + dy * dy <= radius * radius {
                    let grain = pseudo_noise!(px, py, step);
                    if grain > 0.33 {
                        canvas.blend_pixel(px, py, pencil_color, 0.28 + grain * 0.34);
                    }
                }
            }
        }
    }
}

// impl Tool for NaturalPencilBrush {
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

impl Brush for NaturalPencilBrush {
    fn get_brush_kind(&self) -> BrushKind {
        BrushKind::NaturalPencil
    }

    fn get_brush_label(&self, _curren_language: &Language) -> LanguageText {
        LanguageText::BrushKindNaturalPencil
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

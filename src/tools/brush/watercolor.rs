use super::{Brush, BrushKind};
use crate::i18n::LanguageText;
use crate::{algorithm::brush_sample_stride, canvas::Canvas, i18n::Language};
use crate::{pseudo_noise, scale_color};
use egui::Color32;

/// 水彩笔刷
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatercolorBrush;

impl WatercolorBrush {
    pub fn new() -> Self {
        WatercolorBrush
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
        self.draw_watercolor_stamp(canvas, x, y, color, size, step);
    }

    fn draw_watercolor_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
        step: i32,
    ) {
        let radius = (size / 2).max(2);
        for py in y - radius..=y + radius {
            for px in x - radius..=x + radius {
                let dx = px - x;
                let dy = py - y;
                let distance_sq = dx * dx + dy * dy;
                if distance_sq <= radius * radius {
                    let fade = 1.0 - distance_sq as f32 / (radius * radius).max(1) as f32;
                    let paper = pseudo_noise!(px / 3, py / 3, step);
                    let alpha = (0.12 + fade * 0.38) * (0.65 + paper * 0.5);
                    canvas.blend_pixel(px, py, scale_color!(color, 0.52 + fade * 0.38), alpha);
                }
            }
        }
    }
}

// impl Tool for WatercolorBrush {
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

impl Brush for WatercolorBrush {
    fn get_brush_kind(&self) -> BrushKind {
        BrushKind::Watercolor
    }

    fn get_brush_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::BrushKindWatercolor
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

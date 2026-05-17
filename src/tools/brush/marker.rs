use super::{Brush, BrushKind};
use crate::i18n::LanguageText;
use crate::scale_color;
use crate::{algorithm::brush_sample_stride, canvas::Canvas, i18n::Language};
use egui::Color32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkerBrush;

impl MarkerBrush {
    pub fn new() -> Self {
        MarkerBrush
    }

    fn draw_marker_stamp(
        &mut self,
        canvas: &mut Canvas,
        x: i32,
        y: i32,
        color: Color32,
        size: i32,
    ) {
        let radius = (size / 2).max(1);
        let marker_color = scale_color!(color, 0.32);
        for py in y - radius / 2..=y + radius / 2 {
            for px in x - radius..=x + radius {
                let dx = px - x;
                let dy = py - y;
                if dx.abs() <= radius && (dy + dx / 8).abs() <= radius / 2 {
                    canvas.blend_pixel(px, py, marker_color, 0.96);
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
        self.draw_marker_stamp(canvas, x, y, color, size);
    }
}

// impl Tool for MarkerBrush {
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

impl Brush for MarkerBrush {
    fn get_brush_kind(&self) -> BrushKind {
        BrushKind::Marker
    }

    fn get_brush_label(&self, _curren_language: &Language) -> LanguageText {
        LanguageText::BrushKindMarker
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

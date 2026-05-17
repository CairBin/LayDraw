use crate::tools::shape::Shape;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurveShape;

impl CurveShape {
    pub fn new() -> Self {
        Self
    }
}

pub fn draw_cubic_curve(
    canvas: &mut crate::canvas::Canvas,
    start: (i32, i32),
    control1: (i32, i32),
    control2: (i32, i32),
    end: (i32, i32),
    color: egui::Color32,
    thickness: i32,
) {
    let dx = (end.0 - start.0) as f32;
    let dy = (end.1 - start.1) as f32;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 1.0 {
        canvas.set_pixel(start.0, start.1, color);
        return;
    }

    let p0 = (start.0 as f32, start.1 as f32);
    let p1 = (control1.0 as f32, control1.1 as f32);
    let p2 = (control2.0 as f32, control2.1 as f32);
    let p3 = (end.0 as f32, end.1 as f32);
    let steps = ((length / 3.0).ceil() as usize).clamp(16, 160);
    let mut points = Vec::with_capacity(steps + 1);

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let inv = 1.0 - t;
        let x = inv.powi(3) * p0.0
            + 3.0 * inv * inv * t * p1.0
            + 3.0 * inv * t * t * p2.0
            + t.powi(3) * p3.0;
        let y = inv.powi(3) * p0.1
            + 3.0 * inv * inv * t * p1.1
            + 3.0 * inv * t * t * p2.1
            + t.powi(3) * p3.1;
        let point = (x.round() as i32, y.round() as i32);
        if points.last().copied() != Some(point) {
            points.push(point);
        }
    }

    crate::algorithm::draw_polyline(canvas, &points, color, thickness.max(1), false);
}

// impl Tool for CurveShape {
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

impl Shape for CurveShape {
    fn get_shape_kind(&self) -> super::ShapeKind {
        super::ShapeKind::Curve
    }

    fn get_shape_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::Curve
    }

    fn draw(
        &mut self,
        canvas: &mut crate::canvas::Canvas,
        start: (i32, i32),
        end: (i32, i32),
        outline: egui::Color32,
        _fill: egui::Color32,
        thickness: i32,
        _mode: super::ShapeMode,
    ) {
        let dx = (end.0 - start.0) as f32;
        let dy = (end.1 - start.1) as f32;
        let length = (dx * dx + dy * dy).sqrt();
        if length < 1.0 {
            canvas.set_pixel(start.0, start.1, outline);
            return;
        }

        let bend = (length * 0.28).clamp(6.0, 120.0);
        let nx = -dy / length;
        let ny = dx / length;
        let p1 = (
            (start.0 as f32 + dx * 0.25 + nx * bend).round() as i32,
            (start.1 as f32 + dy * 0.25 + ny * bend).round() as i32,
        );
        let p2 = (
            (start.0 as f32 + dx * 0.75 - nx * bend).round() as i32,
            (start.1 as f32 + dy * 0.75 - ny * bend).round() as i32,
        );
        draw_cubic_curve(canvas, start, p1, p2, end, outline, thickness);
    }
}

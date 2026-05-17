use egui::Color32;

use crate::{canvas::Canvas, tools::shape::ShapeMode};

pub trait LineAlgorithm {
    /// 计算两点之间的离散点
    fn line_points(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)>;

    /// 画线并填充圆，用于实现油画笔等效果
    fn draw_line_with_disc(
        &self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: Color32,
        size: i32,
    );
}

pub fn for_each_line_point(
    from: (i32, i32),
    to: (i32, i32),
    mut visit: impl FnMut(i32, i32, usize),
) {
    let (mut x0, mut y0) = from;
    let (x1, y1) = to;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut step = 0usize;

    loop {
        visit(x0, y0, step);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
        step += 1;
    }
}

pub fn brush_sample_stride(size: i32, divisor: i32) -> usize {
    (size.max(1) / divisor.max(1)).max(1) as usize
}

pub fn draw_disc(canvas: &mut Canvas, center_x: i32, center_y: i32, radius: i32, color: Color32) {
    let radius: i32 = radius.max(0);
    let top = (center_y - radius).max(0);
    let bottom = (center_y + radius).min(canvas.height as i32 - 1);
    let radius_sq = radius * radius;

    for y in top..=bottom {
        let dy = y - center_y;
        let span = ((radius_sq - dy * dy) as f32).sqrt() as i32;
        fill_span(canvas, center_x - span, center_x + span, y, color);
    }
}

fn fill_span(canvas: &mut Canvas, left: i32, right: i32, y: i32, color: Color32) {
    if y < 0 || y >= canvas.height as i32 {
        return;
    }
    let left = left.max(0);
    let right = right.min(canvas.width as i32 - 1);
    if left > right {
        return;
    }
    let row = y as usize * canvas.width;
    let start = row + left as usize;
    let end = row + right as usize + 1;
    canvas.pixels[start..end].fill(color);
}

pub fn draw_thick_line(
    canvas: &mut Canvas,
    from: (i32, i32),
    to: (i32, i32),
    color: Color32,
    size: i32,
) {
    let size = size.max(1);
    if size <= 1 {
        for (x, y) in BresenhamLine::new().line_points(from, to) {
            canvas.set_pixel(x, y, color);
        }
        return;
    }

    let radius = size as f32 / 2.0;
    let radius_sq = radius * radius;
    let (x0, y0) = (from.0 as f32, from.1 as f32);
    let (x1, y1) = (to.0 as f32, to.1 as f32);
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len_sq = dx * dx + dy * dy;
    if len_sq <= f32::EPSILON {
        draw_disc(canvas, from.0, from.1, radius.ceil() as i32, color);
        return;
    }

    let margin = radius.ceil() as i32;
    let left = (from.0.min(to.0) - margin).max(0);
    let right = (from.0.max(to.0) + margin).min(canvas.width as i32 - 1);
    let top = (from.1.min(to.1) - margin).max(0);
    let bottom = (from.1.max(to.1) + margin).min(canvas.height as i32 - 1);

    for y in top..=bottom {
        let py = y as f32 + 0.5;
        let row = y as usize * canvas.width;
        for x in left..=right {
            let px = x as f32 + 0.5;
            let t = (((px - x0) * dx + (py - y0) * dy) / len_sq).clamp(0.0, 1.0);
            let closest_x = x0 + t * dx;
            let closest_y = y0 + t * dy;
            let dist_x = px - closest_x;
            let dist_y = py - closest_y;
            if dist_x * dist_x + dist_y * dist_y <= radius_sq {
                canvas.pixels[row + x as usize] = color;
            }
        }
    }
}

// Bresenham 画线算法
pub struct BresenhamLine;

impl BresenhamLine {
    pub fn new() -> Self {
        Self
    }
}

impl LineAlgorithm for BresenhamLine {
    fn line_points(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
        let (mut x0, mut y0) = from;
        let (x1, y1) = to;
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut points = Vec::new();

        loop {
            points.push((x0, y0));
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }

        points
    }

    fn draw_line_with_disc(
        &self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: Color32,
        size: i32,
    ) {
        draw_thick_line(canvas, from, to, color, size);
    }
}

pub fn draw_polyline(
    canvas: &mut Canvas,
    points: &[(i32, i32)],
    color: Color32,
    thickness: i32,
    closed: bool,
) {
    let bl = BresenhamLine::new();
    for pair in points.windows(2) {
        bl.draw_line_with_disc(canvas, pair[0], pair[1], color, thickness);
    }

    if closed && points.len() > 2 {
        bl.draw_line_with_disc(
            canvas,
            points[points.len() - 1],
            points[0],
            color,
            thickness,
        );
    }
}

pub fn rounded_rect_contains(
    x: i32,
    y: i32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    radius: i32,
) -> bool {
    if left > right || top > bottom {
        return false;
    }
    let cx = if x < left + radius {
        left + radius
    } else if x > right - radius {
        right - radius
    } else {
        x
    };
    let cy = if y < top + radius {
        top + radius
    } else if y > bottom - radius {
        bottom - radius
    } else {
        y
    };
    let dx = x - cx;
    let dy = y - cy;
    x >= left && x <= right && y >= top && y <= bottom && dx * dx + dy * dy <= radius * radius
}

pub fn draw_polygon_shape(
    canvas: &mut Canvas,
    points: &[(i32, i32)],
    outline: Color32,
    fill: Color32,
    thickness: i32,
    mode: ShapeMode,
) {
    if points.len() < 2 {
        return;
    }

    if matches!(mode, ShapeMode::Filled | ShapeMode::FilledOutline) {
        fill_polygon(canvas, points, fill);
    }

    if matches!(mode, ShapeMode::Outline | ShapeMode::FilledOutline) {
        draw_polyline(canvas, points, outline, thickness, true);
    }
}

fn fill_polygon(canvas: &mut Canvas, points: &[(i32, i32)], color: Color32) {
    let min_y = points.iter().map(|p| p.1).min().unwrap_or(0).max(0);
    let max_y = points
        .iter()
        .map(|p| p.1)
        .max()
        .unwrap_or(0)
        .min(canvas.height as i32 - 1);

    for y in min_y..=max_y {
        let mut nodes = Vec::new();
        let mut previous = points[points.len() - 1];
        for &current in points {
            if (current.1 < y && previous.1 >= y) || (previous.1 < y && current.1 >= y) {
                let dy = previous.1 - current.1;
                if dy != 0 {
                    let x = current.0 + (y - current.1) * (previous.0 - current.0) / dy;
                    nodes.push(x);
                }
            }
            previous = current;
        }

        nodes.sort_unstable();
        for pair in nodes.chunks(2) {
            if pair.len() != 2 {
                continue;
            }
            let from = pair[0].max(0);
            let to = pair[1].min(canvas.width as i32 - 1);
            for x in from..=to {
                canvas.set_pixel(x, y, color);
            }
        }
    }
}

use crate::constants::{MAX_CANVAS_SIDE, MIN_CANVAS_SIDE};
use egui::Color32;

#[macro_export]
macro_rules! mix_color {
    ($base:expr, $paint:expr, $alpha:expr) => {{
        let base = $base;
        let paint = $paint;
        let alpha = ($alpha).clamp(0.0, 1.0) * (paint.a() as f32 / 255.0);
        let inv = 1.0 - alpha;
        Color32::from_rgba_unmultiplied(
            (base.r() as f32 * inv + paint.r() as f32 * alpha) as u8,
            (base.g() as f32 * inv + paint.g() as f32 * alpha) as u8,
            (base.b() as f32 * inv + paint.b() as f32 * alpha) as u8,
            255,
        )
    }};
}

#[derive(Clone)]
pub struct CanvasSnapshot {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color32>,
}

#[derive(Clone)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color32>,
}

#[derive(Clone)]
pub struct CanvasRegion {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Color32>,
}

// 画布尺寸限制
fn clamp_side(value: usize) -> usize {
    value.clamp(MIN_CANVAS_SIDE, MAX_CANVAS_SIDE)
}

impl Canvas {
    pub fn new(width: usize, height: usize, color: Color32) -> Self {
        let width = clamp_side(width);
        let height = clamp_side(height);
        Self {
            width,
            height,
            pixels: vec![color; width * height],
        }
    }

    pub fn scratch(width: usize, height: usize, color: Color32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        Self {
            width,
            height,
            pixels: vec![color; width * height],
        }
    }

    pub fn from_pixels(width: usize, height: usize, pixels: Vec<Color32>) -> Self {
        let source_width = width;
        let source_height = height;
        let width = clamp_side(width);
        let height = clamp_side(height);
        let mut normalized = vec![Color32::WHITE; width * height];
        let copy_width = source_width.min(width);
        let copy_height = source_height.min(height);
        for y in 0..copy_height {
            let source_row = y * source_width;
            let target_row = y * width;
            normalized[target_row..target_row + copy_width]
                .copy_from_slice(&pixels[source_row..source_row + copy_width]);
        }

        Self {
            width,
            height,
            pixels: normalized,
        }
    }

    /// 创建快照
    pub fn snapshot(&self) -> CanvasSnapshot {
        CanvasSnapshot {
            width: self.width,
            height: self.height,
            pixels: self.pixels.clone(),
        }
    }

    /// 用快照来恢复画布
    pub fn restore(&mut self, snapshot: CanvasSnapshot) {
        self.width = snapshot.width;
        self.height = snapshot.height;
        self.pixels = snapshot.pixels;
    }

    /// 清除画布
    pub fn clear(&mut self, width: usize, height: usize, color: Color32) {
        let width = clamp_side(width);
        let height = clamp_side(height);
        self.width = width;
        self.height = height;
        self.pixels = vec![color; width * height];
    }

    pub fn resize(&mut self, width: usize, height: usize, fill: Color32) {
        let width = clamp_side(width);
        let height = clamp_side(height);
        if width == self.width && height == self.height {
            return;
        }

        let mut pixels = vec![fill; width * height];
        let copy_width = self.width.min(width);
        let copy_height = self.height.min(height);
        for y in 0..copy_height {
            let old_row = y * self.width;
            let new_row = y * width;
            pixels[new_row..new_row + copy_width]
                .copy_from_slice(&self.pixels[old_row..old_row + copy_width]);
        }

        self.width = width;
        self.height = height;
        self.pixels = pixels;
    }

    pub fn flip_horizontal(&mut self) {
        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..self.width / 2 {
                self.pixels.swap(row + x, row + self.width - 1 - x);
            }
        }
    }

    pub fn flip_vertical(&mut self) {
        for y in 0..self.height / 2 {
            let opposite = self.height - 1 - y;
            for x in 0..self.width {
                self.pixels
                    .swap(y * self.width + x, opposite * self.width + x);
            }
        }
    }

    pub fn crop_rect(&mut self, start: (i32, i32), end: (i32, i32)) -> bool {
        let Some(region) = self.copy_rect(start, end) else {
            return false;
        };

        let width = clamp_side(region.width);
        let height = clamp_side(region.height);
        let mut pixels = vec![Color32::WHITE; width * height];
        for y in 0..region.height.min(height) {
            for x in 0..region.width.min(width) {
                pixels[y * width + x] = region.pixels[y * region.width + x];
            }
        }

        self.width = width;
        self.height = height;
        self.pixels = pixels;
        true
    }

    pub fn copy_rect(&self, start: (i32, i32), end: (i32, i32)) -> Option<CanvasRegion> {
        let (left, top, right, bottom) = self.clamped_rect(start, end)?;
        let width = (right - left + 1) as usize;
        let height = (bottom - top + 1) as usize;
        let mut pixels = Vec::with_capacity(width * height);
        for y in top..=bottom {
            let row = y as usize * self.width;
            pixels.extend_from_slice(&self.pixels[row + left as usize..=row + right as usize]);
        }
        Some(CanvasRegion {
            width,
            height,
            pixels,
        })
    }

    pub fn clear_rect(&mut self, start: (i32, i32), end: (i32, i32), fill: Color32) {
        let Some((left, top, right, bottom)) = self.clamped_rect(start, end) else {
            return;
        };
        for y in top..=bottom {
            for x in left..=right {
                self.set_pixel(x, y, fill);
            }
        }
    }

    pub fn paste_region(&mut self, left: i32, top: i32, region: &CanvasRegion) {
        for y in 0..region.height {
            for x in 0..region.width {
                let target_x = left + x as i32;
                let target_y = top + y as i32;
                if self.index(target_x, target_y).is_some() {
                    self.set_pixel(target_x, target_y, region.pixels[y * region.width + x]);
                }
            }
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color32> {
        self.index(x, y).map(|index| self.pixels[index])
    }

    fn clamped_rect(&self, start: (i32, i32), end: (i32, i32)) -> Option<(i32, i32, i32, i32)> {
        let left = start.0.min(end.0).max(0);
        let top = start.1.min(end.1).max(0);
        let right = start.0.max(end.0).min(self.width as i32 - 1);
        let bottom = start.1.max(end.1).min(self.height as i32 - 1);
        if left > right || top > bottom {
            None
        } else {
            Some((left, top, right, bottom))
        }
    }

    /// 获取像素点索引
    pub fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            None
        } else {
            Some(y as usize * self.width + x as usize)
        }
    }

    /// 设置像素点
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color32) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = color;
        }
    }

    /// 混合像素点
    pub fn blend_pixel(&mut self, x: i32, y: i32, color: Color32, alpha: f32) {
        if let Some(index) = self.index(x, y) {
            self.pixels[index] = mix_color!(self.pixels[index], color, alpha);
        }
    }
}

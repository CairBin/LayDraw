use std::path::Path;

use eframe::egui::Color32;
use image::{ImageBuffer, Rgb, Rgba};

use crate::canvas::Canvas;

pub fn load_canvas(path: &Path) -> image::ImageResult<Canvas> {
    let image = image::open(path)?.to_rgba8();
    let (width, height) = image.dimensions();
    let pixels = image
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    Ok(Canvas::from_pixels(width as usize, height as usize, pixels))
}

pub fn save_canvas(canvas: &Canvas, path: &Path) -> image::ImageResult<()> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(extension.as_str(), "jpg" | "jpeg") {
        return save_canvas_rgb(canvas, path);
    }

    let mut bytes = Vec::with_capacity(canvas.pixels.len() * 4);
    for color in &canvas.pixels {
        bytes.extend_from_slice(&[color.r(), color.g(), color.b(), color.a()]);
    }
    let Some(buffer): Option<ImageBuffer<Rgba<u8>, Vec<u8>>> =
        ImageBuffer::from_vec(canvas.width as u32, canvas.height as u32, bytes)
    else {
        return Err(image::ImageError::Parameter(
            image::error::ParameterError::from_kind(
                image::error::ParameterErrorKind::DimensionMismatch,
            ),
        ));
    };

    buffer.save(path)
}

fn save_canvas_rgb(canvas: &Canvas, path: &Path) -> image::ImageResult<()> {
    let mut bytes = Vec::with_capacity(canvas.pixels.len() * 3);
    for color in &canvas.pixels {
        let alpha = color.a() as f32 / 255.0;
        let inv = 1.0 - alpha;
        bytes.extend_from_slice(&[
            (color.r() as f32 * alpha + 255.0 * inv).round() as u8,
            (color.g() as f32 * alpha + 255.0 * inv).round() as u8,
            (color.b() as f32 * alpha + 255.0 * inv).round() as u8,
        ]);
    }
    let Some(buffer): Option<ImageBuffer<Rgb<u8>, Vec<u8>>> =
        ImageBuffer::from_vec(canvas.width as u32, canvas.height as u32, bytes)
    else {
        return Err(image::ImageError::Parameter(
            image::error::ParameterError::from_kind(
                image::error::ParameterErrorKind::DimensionMismatch,
            ),
        ));
    };

    buffer.save(path)
}

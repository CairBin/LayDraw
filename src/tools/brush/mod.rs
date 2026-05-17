use crate::canvas::Canvas;
use crate::i18n::{Language, LanguageText};
use crate::tools::{Tool, ToolKind};
use egui::Color32;
pub mod calligraphy;
pub mod common;
pub mod crayon;
pub mod marker;
pub mod natural_pencil;
pub mod oil;
pub mod pencil;
pub mod spray;
pub mod watercolor;

#[macro_export]
macro_rules! pseudo_noise {
    ($x:expr, $y:expr, $seed:expr) => {{
        // 强制转换为 i32，确保运算类型正确（若已为 i32 则无额外开销）
        let x = $x as i32;
        let y = $y as i32;
        let seed = $seed as i32;

        let n = x
            .wrapping_mul(374_761_393)
            .wrapping_add(y.wrapping_mul(668_265_263))
            .wrapping_add(seed.wrapping_mul(2_147_483));

        let n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);

        ((n ^ (n >> 16)) & 0x7fff) as f32 / 0x7fff as f32
    }};
}

#[macro_export]
macro_rules! scale_color {
    ($color:expr, $scale:expr) => {{
        let c = $color;
        let s = $scale;
        Color32::from_rgba_unmultiplied(
            ((c.r() as f32 * s).clamp(0.0, 255.0) as u8),
            ((c.g() as f32 * s).clamp(0.0, 255.0) as u8),
            ((c.b() as f32 * s).clamp(0.0, 255.0) as u8),
            c.a(),
        )
    }};
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BrushKind {
    // 普通画笔
    Brush,

    // 铅笔画笔
    PencilBrush,

    // 毛笔
    Calligraphy,

    // 喷雾画笔
    Spray,

    // 油画笔
    Oil,

    // 蜡笔
    Crayon,

    // 马克笔
    Marker,

    // 自然铅笔画笔
    NaturalPencil,

    // 水彩画笔
    Watercolor,

    Extra, // 作为扩展
}

/// 画笔类型
#[allow(dead_code)]
pub trait Brush {
    fn brush_id(&self) -> &'static str {
        match self.get_brush_kind() {
            BrushKind::Brush => "brush.common",
            BrushKind::PencilBrush => "brush.ink_pencil",
            BrushKind::Calligraphy => "brush.calligraphy",
            BrushKind::Spray => "brush.spray",
            BrushKind::Oil => "brush.oil",
            BrushKind::Crayon => "brush.crayon",
            BrushKind::Marker => "brush.marker",
            BrushKind::NaturalPencil => "brush.natural_pencil",
            BrushKind::Watercolor => "brush.watercolor",
            BrushKind::Extra => "brush.extra",
        }
    }

    /// 获取笔刷类型
    fn get_brush_kind(&self) -> BrushKind;

    /// 获取笔刷对应当前语言的标签
    fn get_brush_label(&self, current_language: &Language) -> LanguageText;

    /// 使用笔刷绘制线条
    /// - `algo` 画线算法
    /// - `canvas` 画布
    /// - `from` 起始点
    /// - `to` 终点
    /// - `color` 颜色
    /// - `size` 大小
    fn draw_line(
        &mut self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: Color32,
        size: i32,
    );

    fn paint_preview(&self, ui: &egui::Ui, rect: egui::Rect, color: Color32, size: i32) {
        let painter = ui.painter();
        let y = rect.center().y;
        let stroke_width = (size as f32 / 4.0).clamp(1.0, 6.0);
        painter.line_segment(
            [
                egui::pos2(rect.left() + 6.0, y),
                egui::pos2(rect.right() - 6.0, y),
            ],
            egui::Stroke::new(stroke_width, color),
        );
    }

    fn brush_button(
        &mut self,
        ui: &mut egui::Ui,
        current_language: &Language,
        selected: bool,
        color: Color32,
        size: i32,
    ) -> egui::Response {
        let label = current_language.get_text(self.get_brush_label(current_language));
        let desired_size = egui::vec2(104.0, 42.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let visuals = ui.style().interact_selectable(&response, selected);

        ui.painter().rect_filled(rect, 4.0, visuals.bg_fill);
        ui.painter()
            .rect_stroke(rect, 4.0, egui::Stroke::new(1.0, visuals.bg_stroke.color));

        let preview_rect = egui::Rect::from_min_max(
            rect.left_top() + egui::vec2(6.0, 5.0),
            rect.right_top() + egui::vec2(-6.0, 23.0),
        );
        self.paint_preview(ui, preview_rect, color, size);

        ui.painter().text(
            egui::pos2(rect.left() + 6.0, rect.bottom() - 15.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(12.0),
            visuals.text_color(),
        );

        response
    }
}

pub struct BrushGroup {
    brush: Vec<Box<dyn Brush>>,
    active: usize,
}

impl BrushGroup {
    pub fn new() -> Self {
        let mut brush_vec: Vec<Box<dyn Brush>> = Vec::new();
        brush_vec.push(Box::new(common::CommonBrush::new()));
        brush_vec.push(Box::new(pencil::PencilBrush::new()));
        brush_vec.push(Box::new(calligraphy::CalligraphyBrush::new()));
        brush_vec.push(Box::new(spray::SprayBrush::new()));
        brush_vec.push(Box::new(oil::OilBrush::new()));
        brush_vec.push(Box::new(crayon::CrayonBrush::new()));
        brush_vec.push(Box::new(marker::MarkerBrush::new()));
        brush_vec.push(Box::new(natural_pencil::NaturalPencilBrush::new()));
        brush_vec.push(Box::new(watercolor::WatercolorBrush::new()));

        Self {
            brush: brush_vec,
            active: 0,
        }
    }

    pub fn load_brush(&mut self, brush: Box<dyn Brush>) {
        self.brush.push(brush);
    }

    pub fn remove_brush(&mut self, index: usize) {
        if index >= self.brush.len() {
            return;
        }
        self.brush.remove(index);
        if self.brush.is_empty() {
            self.active = 0;
        } else if self.active == index {
            self.active = self.active.min(self.brush.len() - 1);
        } else if self.active > index {
            self.active -= 1;
        }
    }

    pub fn brushes(&self) -> &[Box<dyn Brush>] {
        &self.brush
    }

    #[allow(dead_code)]
    pub fn brushes_mut(&mut self) -> &mut [Box<dyn Brush>] {
        &mut self.brush
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn active_brush(&self) -> Option<&(dyn Brush + '_)> {
        self.brush.get(self.active).map(Box::as_ref)
    }

    pub fn active_brush_mut(&mut self) -> Option<&mut (dyn Brush + '_)> {
        if let Some(brush) = self.brush.get_mut(self.active) {
            Some(brush.as_mut())
        } else {
            None
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.brush.len() {
            self.active = index;
        }
    }

    #[allow(dead_code)]
    pub fn ribbon_ui(
        &mut self,
        ui: &mut egui::Ui,
        current_language: &Language,
        color: Color32,
        size: i32,
    ) {
        ui.vertical(|ui| {
            ui.label(current_language.get_text(LanguageText::Brushes));
            let selected_text = self
                .active_brush()
                .map(|brush| current_language.get_text(brush.get_brush_label(current_language)))
                .unwrap_or_else(|| current_language.get_text(LanguageText::Brushes));

            egui::ComboBox::from_id_source("brush_kind")
                .width(136.0)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    let mut selected = None;
                    for (index, brush) in self.brush.iter().enumerate() {
                        let label =
                            current_language.get_text(brush.get_brush_label(current_language));
                        if ui.selectable_label(self.active == index, label).clicked() {
                            selected = Some(index);
                        }
                    }
                    if let Some(index) = selected {
                        self.select(index);
                    }
                });

            let (preview_rect, _) =
                ui.allocate_exact_size(egui::vec2(136.0, 30.0), egui::Sense::hover());
            if let Some(brush) = self.active_brush_mut() {
                paint_live_brush_preview(brush, ui, preview_rect.shrink(4.0), color, size);
            }
        });
    }
}

pub(crate) fn paint_live_brush_preview(
    brush: &mut dyn Brush,
    ui: &egui::Ui,
    rect: egui::Rect,
    color: Color32,
    size: i32,
) {
    let width = 96usize;
    let height = 18usize;
    let mut canvas = Canvas::new(width, height, Color32::TRANSPARENT);
    brush.draw_line(
        &mut canvas,
        (6, height as i32 / 2),
        (width as i32 - 7, height as i32 / 2),
        color,
        size,
    );

    let pixel_w = rect.width() / width as f32;
    let pixel_h = rect.height() / height as f32;
    for y in 0..height {
        for x in 0..width {
            let color = canvas.pixels[y * width + x];
            if color.a() == 0 {
                continue;
            }
            let cell = egui::Rect::from_min_size(
                egui::pos2(
                    rect.left() + x as f32 * pixel_w,
                    rect.top() + y as f32 * pixel_h,
                ),
                egui::vec2(pixel_w.max(1.0), pixel_h.max(1.0)),
            );
            ui.painter().rect_filled(cell, 0.0, color);
        }
    }
}

impl Default for BrushGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for BrushGroup {
    fn get_tool_kind(&self) -> ToolKind {
        ToolKind::Brush
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::Brush
    }
}

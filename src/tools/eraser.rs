use egui::Vec2;

#[derive(Debug)]
pub struct Eraser {
    brush_size: i32,
    zoom: f32,
}

impl Eraser {
    pub fn new(brush_size: i32, zoom: f32) -> Self {
        Self { brush_size, zoom }
    }

    fn draw_eraser_cursor(&self, ui: &egui::Ui, canvas_rect: egui::Rect, pointer: egui::Pos2) {
        let size = (self.brush_size.max(1) as f32 * self.zoom).max(4.0);
        let cursor_rect =
            egui::Rect::from_center_size(pointer, Vec2::splat(size)).intersect(canvas_rect);
        ui.painter().rect_stroke(
            cursor_rect,
            0.0,
            egui::Stroke::new(
                3.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 210),
            ),
        );
        ui.painter().rect_stroke(
            cursor_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(36, 36, 36)),
        );
    }
}

impl super::Tool for Eraser {
    fn get_tool_kind(&self) -> super::ToolKind {
        super::ToolKind::Eraser
    }

    fn get_tool_label(
        &self,
        _current_language: &crate::i18n::Language,
    ) -> crate::i18n::LanguageText {
        crate::i18n::LanguageText::Eraser
    }
}

impl crate::ui::cursor::Cursor for Eraser {
    fn cursor(&self) -> crate::ui::cursor::MyCursorIcon<'_> {
        crate::ui::cursor::MyCursorIcon::Custom(Box::new(|ui, canvas_rect, pointer| {
            self.draw_eraser_cursor(ui, canvas_rect, pointer);
        }))
    }
}

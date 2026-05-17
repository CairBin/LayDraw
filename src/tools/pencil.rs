use egui::Color32;

use crate::{
    algorithm::LineAlgorithm,
    canvas::Canvas,
    i18n::{Language, LanguageText},
    tools::{Tool, ToolKind},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PencilTool;

impl PencilTool {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_line<A: LineAlgorithm>(
        &mut self,
        algo: &A,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: Color32,
        size: i32,
    ) {
        let size = size.max(1);
        if size <= 1 {
            crate::algorithm::for_each_line_point(from, to, |x, y, _| {
                canvas.set_pixel(x, y, color);
            });
        } else {
            algo.draw_line_with_disc(canvas, from, to, color, size);
        }
    }
}

impl Tool for PencilTool {
    fn get_tool_kind(&self) -> ToolKind {
        ToolKind::Pencil
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::Pencil
    }
}

impl crate::ui::cursor::Cursor for PencilTool {
    fn cursor(&self) -> crate::ui::cursor::MyCursorIcon<'_> {
        crate::ui::cursor::MyCursorIcon::EguiCursorIcon(egui::CursorIcon::Crosshair)
    }
}

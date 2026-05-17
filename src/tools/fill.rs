use std::collections::VecDeque;

use egui::Color32;

use crate::{
    canvas::Canvas,
    i18n::{Language, LanguageText},
    tools::{Tool, ToolKind},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FillTool;

impl FillTool {
    pub fn new() -> Self {
        Self
    }

    pub fn fill(&mut self, canvas: &mut Canvas, x: i32, y: i32, color: Color32) {
        let Some(start_index) = canvas.index(x, y) else {
            return;
        };
        let target = canvas.pixels[start_index];
        if target == color {
            return;
        }

        let mut queue = VecDeque::from([(x, y)]);
        while let Some((cx, cy)) = queue.pop_front() {
            let Some(index) = canvas.index(cx, cy) else {
                continue;
            };
            if canvas.pixels[index] != target {
                continue;
            }

            canvas.pixels[index] = color;
            queue.push_back((cx + 1, cy));
            queue.push_back((cx - 1, cy));
            queue.push_back((cx, cy + 1));
            queue.push_back((cx, cy - 1));
        }
    }
}

impl Tool for FillTool {
    fn get_tool_kind(&self) -> ToolKind {
        ToolKind::Fill
    }

    fn get_tool_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::Fill
    }
}

impl crate::ui::cursor::Cursor for FillTool {
    fn cursor(&self) -> crate::ui::cursor::MyCursorIcon<'_> {
        crate::ui::cursor::MyCursorIcon::EguiCursorIcon(egui::CursorIcon::PointingHand)
    }
}

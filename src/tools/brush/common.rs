use super::{Brush, BrushKind};
use crate::i18n::LanguageText;
use crate::{algorithm::LineAlgorithm, canvas::Canvas, i18n::Language};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommonBrush;

impl CommonBrush {
    pub fn new() -> Self {
        CommonBrush
    }
}

// impl Tool for CommonBrush {
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

impl Brush for CommonBrush {
    fn get_brush_kind(&self) -> BrushKind {
        super::BrushKind::Brush
    }

    fn get_brush_label(&self, _current_language: &Language) -> LanguageText {
        LanguageText::BrushKindBrush
    }
    fn draw_line(
        &mut self,
        canvas: &mut Canvas,
        from: (i32, i32),
        to: (i32, i32),
        color: egui::Color32,
        size: i32,
    ) {
        crate::algorithm::BresenhamLine::new().draw_line_with_disc(canvas, from, to, color, size);
    }
}

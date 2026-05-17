/// 光标类型
pub enum MyCursorIcon<'a> {
    // EGUI光标
    EguiCursorIcon(egui::CursorIcon),

    // 自定义光标，要求传入绘制函数
    Custom(Box<dyn Fn(&egui::Ui, egui::Rect, egui::Pos2) + 'a>),
}

// 用于实现光标类型的trait
pub trait Cursor {
    fn cursor(&self) -> MyCursorIcon<'_> {
        MyCursorIcon::EguiCursorIcon(egui::CursorIcon::Crosshair)
    }
}

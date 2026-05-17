#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RibbonTab {
    File,
    Home,
    View,
    Plugins,
    Plugin(&'static str),

    Extra,
}

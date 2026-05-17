use crate::ui::{
    PaintApp,
    panel::app::{AppPanel, AppPanelArea},
};

pub struct ToolsPanel;
pub struct HandlePanel;
pub struct ShapesPanel;
pub struct OutlinePanel;
pub struct BrushesPanel;
pub struct SizePanel;
pub struct ColorsPanel;
pub struct ViewPanel;
pub struct LayersPanel;

macro_rules! impl_panel {
    ($ty:ty, $id:literal, $area:expr, $body:expr) => {
        impl AppPanel for $ty {
            fn panel_id(&self) -> &'static str {
                $id
            }

            fn panel_area(&self) -> AppPanelArea {
                $area
            }

            fn ui(&mut self, app: &mut PaintApp, ui: &mut egui::Ui) {
                $body(app, ui)
            }
        }
    };
}

impl_panel!(
    ToolsPanel,
    "builtin.tools",
    AppPanelArea::Home,
    PaintApp::tools_panel
);
impl_panel!(
    HandlePanel,
    "builtin.handle",
    AppPanelArea::Home,
    PaintApp::handle_panel
);
impl_panel!(
    ShapesPanel,
    "builtin.shapes",
    AppPanelArea::Home,
    PaintApp::shapes_panel
);
impl_panel!(
    OutlinePanel,
    "builtin.outline",
    AppPanelArea::Home,
    PaintApp::outline_panel
);
impl_panel!(
    BrushesPanel,
    "builtin.brushes",
    AppPanelArea::Home,
    PaintApp::brushes_panel
);
impl_panel!(
    SizePanel,
    "builtin.size",
    AppPanelArea::Home,
    PaintApp::size_panel
);
impl_panel!(
    ColorsPanel,
    "builtin.colors",
    AppPanelArea::Home,
    PaintApp::color_panel
);
impl_panel!(
    ViewPanel,
    "builtin.view",
    AppPanelArea::View,
    PaintApp::view_panel
);
impl_panel!(
    LayersPanel,
    "builtin.layers",
    AppPanelArea::Layers,
    PaintApp::layers_panel_contents
);

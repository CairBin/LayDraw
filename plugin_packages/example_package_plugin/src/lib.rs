use std::collections::HashMap;

use laydraw::{
    i18n::Language,
    plugins::{AppCommand, AppHost, Plugin},
    ui::panel::{Panel, PanelArea, PanelContext},
};
use once_cell::sync::Lazy;

static LABELS: Lazy<HashMap<&'static str, (&'static str, &'static str)>> = Lazy::new(|| {
    HashMap::from([
        ("title", ("Package Plugin", "子包插件")),
        ("panel", ("Package", "子包")),
        ("dirty", ("Mark dirty", "标记修改")),
        ("ready", ("Package plugin ready", "子包插件已就绪")),
    ])
});

fn tr(language: &Language, key: &'static str) -> String {
    let (en_us, zh_cn) = LABELS.get(key).copied().unwrap_or((key, key));
    let text = match language.plugin_locale_key() {
        "zh-CN" => zh_cn,
        _ => en_us,
    };
    language.plugin_text(text)
}

pub fn plugin() -> Box<dyn Plugin> {
    Box::new(PackagePlugin)
}

pub struct PackagePlugin;

impl Plugin for PackagePlugin {
    fn plugin_name(&self) -> &'static str {
        "example.package_plugin"
    }

    fn supported_laydraw_versions(&self) -> &'static str {
        ">=0.1.0, <0.2.0"
    }

    fn plugin_title(&self, language: &Language) -> String {
        tr(language, "title")
    }

    fn plugin_author(&self) -> &'static str {
        "LayDraw"
    }

    fn plugin_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn plugin_url(&self) -> &'static str {
        "https://example.com/laydraw/package-plugin"
    }

    fn plugin_email(&self) -> &'static str {
        "plugins@example.com"
    }

    fn active(&mut self, app_host: &mut dyn AppHost) {
        app_host.load_panel(Box::new(PackagePanel));
    }

    fn inactive(&mut self, app_host: &mut dyn AppHost) {
        app_host.mark_canvas_dirty();
    }
}

struct PackagePanel;

impl Panel for PackagePanel {
    fn panel_id(&self) -> &'static str {
        "example.package_panel"
    }

    fn panel_title(&self, current_language: &Language) -> String {
        tr(current_language, "panel")
    }

    fn panel_area(&self) -> PanelArea {
        PanelArea::Window
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut PanelContext<'_>) {
        ui.label(tr(context.language, "ready"));
        if ui.button(tr(context.language, "dirty")).clicked() {
            context.command(AppCommand::MarkCanvasDirty);
            context.command(AppCommand::SetStatus(tr(context.language, "ready")));
        }
    }
}

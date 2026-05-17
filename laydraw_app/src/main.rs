use laydraw::{
    i18n::{I18n, LanguageText, en_us::EnUs},
    plugins::AppHost,
    ui,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        &EnUs.get_text(LanguageText::AppTitle),
        options,
        Box::new(|cc| {
            let mut app = ui::PaintApp::new(cc);
            app.load_plugin(laydraw_example_package_plugin::plugin());
            Ok(Box::new(app))
        }),
    )
}

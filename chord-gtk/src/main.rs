//! `chord-gtk` — the GNOME (GTK4/VTE) frontend for Chord. Thin: everything that isn't
//! rendering UI lives in `chord-core` (PROMPT-CHORD.md §2.1).

mod tab_bar;
mod terminal_pane;
mod window;

use adw::prelude::*;

const APP_ID: &str = "org.lyraos.Chord";

fn main() -> gtk4::glib::ExitCode {
    chord_core::i18n::init("/usr/share/locale");

    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default));
    window::register_accels(&app);
    app.connect_activate(|app| {
        let config = chord_core::Config::load().unwrap_or_default();
        window::build_window(app, &config);
    });

    app.run()
}

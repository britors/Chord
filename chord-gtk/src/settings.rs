//! Application preferences backed by chord-core's toolkit-neutral TOML config.

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use chord_core::i18n::tr;
use chord_core::{Config, Theme};

use crate::terminal_pane::apply_config;

fn save(config: &Rc<RefCell<Config>>) {
    if let Err(err) = config.borrow().save() {
        eprintln!("chord-gtk: failed to save settings: {err}");
    }
}

fn apply_to_open_terminals(root: &gtk4::Widget, theme: &Theme, config: &Config) {
    if let Some(terminal) = root.downcast_ref::<vte4::Terminal>() {
        apply_config(terminal, theme, config);
    }

    let mut child = root.first_child();
    while let Some(widget) = child {
        apply_to_open_terminals(&widget, theme, config);
        child = widget.next_sibling();
    }
}

fn update_open_terminals(notebook: &gtk4::Notebook, theme: &Theme, config: &Config) {
    apply_to_open_terminals(notebook.upcast_ref(), theme, config);
}

pub fn show(
    parent: &adw::ApplicationWindow,
    config: Rc<RefCell<Config>>,
    theme: Rc<Theme>,
    notebook: gtk4::Notebook,
) {
    let dialog = adw::PreferencesDialog::builder()
        .title(tr("Settings"))
        .content_width(560)
        .content_height(520)
        .build();
    let page = adw::PreferencesPage::builder()
        .title(tr("Console"))
        .icon_name("utilities-terminal-symbolic")
        .build();

    let appearance = adw::PreferencesGroup::builder()
        .title(tr("Appearance"))
        .description(tr("Changes apply immediately to open terminals."))
        .build();

    let font_row = adw::EntryRow::builder().title(tr("Font family")).build();
    font_row.set_text(&config.borrow().font);
    {
        let config = config.clone();
        let theme = theme.clone();
        let notebook = notebook.clone();
        font_row.connect_changed(move |row| {
            let font = row.text();
            if font.trim().is_empty() {
                return;
            }
            config.borrow_mut().font = font.trim().to_string();
            save(&config);
            update_open_terminals(&notebook, &theme, &config.borrow());
        });
    }

    let font_size_row = adw::SpinRow::with_range(6.0, 72.0, 1.0);
    font_size_row.set_title(&tr("Font size"));
    font_size_row.set_value(config.borrow().font_size as f64);
    {
        let config = config.clone();
        let theme = theme.clone();
        let notebook = notebook.clone();
        font_size_row.connect_value_notify(move |row| {
            config.borrow_mut().font_size = row.value() as u32;
            save(&config);
            update_open_terminals(&notebook, &theme, &config.borrow());
        });
    }

    let opacity_row = adw::SpinRow::with_range(10.0, 100.0, 1.0);
    opacity_row.set_title(&tr("Background opacity"));
    opacity_row.set_subtitle(&tr("Percentage"));
    opacity_row.set_value(config.borrow().background_opacity_percent as f64);
    {
        let config = config.clone();
        let theme = theme.clone();
        let notebook = notebook.clone();
        opacity_row.connect_value_notify(move |row| {
            config.borrow_mut().background_opacity_percent = row.value() as u8;
            save(&config);
            update_open_terminals(&notebook, &theme, &config.borrow());
        });
    }

    let cursor_row = adw::SwitchRow::builder()
        .title(tr("Blinking cursor"))
        .active(config.borrow().cursor_blink)
        .build();
    {
        let config = config.clone();
        let theme = theme.clone();
        let notebook = notebook.clone();
        cursor_row.connect_active_notify(move |row| {
            config.borrow_mut().cursor_blink = row.is_active();
            save(&config);
            update_open_terminals(&notebook, &theme, &config.borrow());
        });
    }

    appearance.add(&font_row);
    appearance.add(&font_size_row);
    appearance.add(&opacity_row);
    appearance.add(&cursor_row);

    let shell = adw::PreferencesGroup::builder()
        .title(tr("Shell"))
        .description(tr("Used by new tabs and panes."))
        .build();
    let shell_row = adw::EntryRow::builder().title(tr("Shell command")).build();
    shell_row.set_text(
        config
            .borrow()
            .profiles
            .first()
            .map(|profile| profile.command.as_str())
            .unwrap_or("/bin/sh"),
    );
    {
        let config = config.clone();
        shell_row.connect_changed(move |row| {
            let command = row.text();
            if command.trim().is_empty() {
                return;
            }
            if let Some(profile) = config.borrow_mut().profiles.first_mut() {
                profile.command = command.trim().to_string();
            }
            save(&config);
        });
    }
    shell.add(&shell_row);

    page.add(&appearance);
    page.add(&shell);
    dialog.add(&page);
    dialog.present(Some(parent));
}

//! The Chord window: tabs, splits, and the v1 shortcuts (PROMPT-CHORD.md §3.1).

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use chord_core::i18n::tr;
use chord_core::{Config, Theme};
use vte4::prelude::*;

use crate::settings;
use crate::tab_bar;
use crate::terminal_pane::TerminalPane;

/// Registers the app-wide accelerators for every `win.*` action (PROMPT-CHORD.md §3.1).
pub fn register_accels(app: &adw::Application) {
    app.set_accels_for_action("win.new-tab", &["<Ctrl><Shift>t"]);
    app.set_accels_for_action("win.close-tab", &["<Ctrl><Shift>w"]);
    app.set_accels_for_action("win.next-tab", &["<Ctrl>Tab"]);
    app.set_accels_for_action("win.prev-tab", &["<Ctrl><Shift>Tab"]);
    app.set_accels_for_action("win.split-horizontal", &["<Ctrl><Shift>o"]);
    app.set_accels_for_action("win.split-vertical", &["<Ctrl><Shift>e"]);
    app.set_accels_for_action("win.copy", &["<Ctrl><Shift>c"]);
    app.set_accels_for_action("win.paste", &["<Ctrl><Shift>v"]);
    app.set_accels_for_action("win.search", &["<Ctrl><Shift>f"]);
    app.set_accels_for_action("win.zoom-in", &["<Ctrl>plus", "<Ctrl>equal"]);
    app.set_accels_for_action("win.zoom-out", &["<Ctrl>minus"]);
    app.set_accels_for_action("win.zoom-reset", &["<Ctrl>0"]);
}

/// Shared state for a single Chord window. `focused` is the terminal that pane splits,
/// copy/paste, search and zoom actions apply to.
#[derive(Clone)]
struct WindowState {
    app: adw::Application,
    notebook: gtk4::Notebook,
    search_bar: gtk4::SearchBar,
    search_entry: gtk4::SearchEntry,
    theme: Rc<Theme>,
    config: Rc<RefCell<Config>>,
    focused: Rc<RefCell<Option<vte4::Terminal>>>,
}

pub fn build_window(app: &adw::Application, config: &Config) -> adw::ApplicationWindow {
    let theme = Rc::new(Theme::chord_dark());
    let config = Rc::new(RefCell::new(config.clone()));

    let notebook = gtk4::Notebook::builder().scrollable(true).build();

    let search_entry = gtk4::SearchEntry::builder()
        .placeholder_text(tr("Search scrollback…"))
        .build();
    let search_bar = gtk4::SearchBar::builder().child(&search_entry).build();
    search_bar.connect_entry(&search_entry);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content.append(&search_bar);
    content.append(&notebook);

    let state = WindowState {
        app: app.clone(),
        notebook: notebook.clone(),
        search_bar,
        search_entry,
        theme,
        config,
        focused: Rc::new(RefCell::new(None)),
    };

    notebook.connect_switch_page(|_, page, _| {
        let page = page.clone();
        gtk4::glib::idle_add_local_once(move || {
            if let Some(terminal) = find_terminal(&page) {
                terminal.grab_focus();
            }
        });
    });

    // Standard GNOME/libadwaita window chrome (same pattern as Vega): an
    // `AdwHeaderBar` stacked above the content in a plain box, set as the
    // window's `content` rather than via `set_titlebar`. This gives the
    // normal opaque GNOME headerbar instead of the plain GTK4 `HeaderBar`,
    // which some Wayland compositors render with a translucent backdrop.
    let header_bar = adw::HeaderBar::new();
    let new_tab_button = gtk4::Button::from_icon_name("tab-new-symbolic");
    new_tab_button.set_action_name(Some("win.new-tab"));
    new_tab_button.set_tooltip_text(Some(&tr("New tab")));
    header_bar.pack_start(&new_tab_button);

    let menu = gtk4::gio::Menu::new();
    menu.append(Some(&tr("Settings")), Some("win.settings"));
    menu.append(Some(&tr("About Chord")), Some("win.about"));
    let menu_button = gtk4::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .tooltip_text(tr("Main menu"))
        .menu_model(&menu)
        .build();
    header_bar.pack_end(&menu_button);

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    root.append(&header_bar);
    root.append(&content);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Chord")
        .default_width(960)
        .default_height(600)
        .content(&root)
        .build();

    install_actions(&window, &state);
    wire_search(&state);

    add_tab(&state);

    window.present();
    window
}

fn install_actions(window: &adw::ApplicationWindow, state: &WindowState) {
    let action = |name: &str, state: WindowState, f: fn(&WindowState)| {
        let action = gtk4::gio::SimpleAction::new(name, None);
        action.connect_activate(move |_, _| f(&state));
        action
    };

    window.add_action(&action("new-tab", state.clone(), |s| add_tab(s)));
    window.add_action(&action("close-tab", state.clone(), |s| {
        close_current_tab(s)
    }));
    window.add_action(&action("next-tab", state.clone(), |s| {
        select_relative_tab(s, 1)
    }));
    window.add_action(&action("prev-tab", state.clone(), |s| {
        select_relative_tab(s, -1)
    }));
    window.add_action(&action("split-horizontal", state.clone(), |s| {
        split_focused(s, gtk4::Orientation::Horizontal)
    }));
    window.add_action(&action("split-vertical", state.clone(), |s| {
        split_focused(s, gtk4::Orientation::Vertical)
    }));
    window.add_action(&action("copy", state.clone(), |s| {
        if let Some(t) = s.focused.borrow().as_ref() {
            t.copy_clipboard_format(vte4::Format::Text);
        }
    }));
    window.add_action(&action("paste", state.clone(), |s| {
        if let Some(t) = s.focused.borrow().as_ref() {
            t.paste_clipboard();
        }
    }));
    window.add_action(&action("search", state.clone(), |s| {
        let active = s.search_bar.is_search_mode();
        s.search_bar.set_search_mode(!active);
        if !active {
            s.search_entry.grab_focus();
        }
    }));
    window.add_action(&action("zoom-in", state.clone(), |s| zoom(s, 1.1)));
    window.add_action(&action("zoom-out", state.clone(), |s| zoom(s, 1.0 / 1.1)));
    window.add_action(&action("zoom-reset", state.clone(), |s| {
        if let Some(t) = s.focused.borrow().as_ref() {
            t.set_font_scale(1.0);
        }
    }));

    let settings_action = gtk4::gio::SimpleAction::new("settings", None);
    let window_for_settings = window.clone();
    let state_for_settings = state.clone();
    settings_action.connect_activate(move |_, _| {
        settings::show(
            &window_for_settings,
            state_for_settings.config.clone(),
            state_for_settings.theme.clone(),
            state_for_settings.notebook.clone(),
        );
    });
    window.add_action(&settings_action);

    let about_action = gtk4::gio::SimpleAction::new("about", None);
    let window_for_about = window.clone();
    about_action.connect_activate(move |_, _| {
        let dialog = adw::AboutDialog::builder()
            .application_name("Chord")
            .application_icon("chord-icon-mark")
            .version(env!("CARGO_PKG_VERSION"))
            .comments(tr(
                "Terminal emulator for the Lyra Enterprise Linux ecosystem.",
            ))
            .developers(["Rodrigo Brito <rodrigo@w3ti.com.br>"])
            .copyright("© 2026 Rodrigo Brito")
            .license_type(gtk4::License::Gpl30)
            .build();
        dialog.present(Some(&window_for_about));
    });
    window.add_action(&about_action);
}

fn wire_search(state: &WindowState) {
    let state_for_search = state.clone();
    state.search_entry.connect_search_changed(move |entry| {
        let Some(terminal) = state_for_search.focused.borrow().clone() else {
            return;
        };
        let text = entry.text();
        if text.is_empty() {
            terminal.search_set_regex(None, 0);
            return;
        }
        match vte4::Regex::for_search(&text, 0) {
            Ok(regex) => {
                terminal.search_set_regex(Some(&regex), 0);
                terminal.search_find_next();
            }
            Err(err) => eprintln!("chord-gtk: invalid search pattern: {err}"),
        }
    });

    let state_for_activate = state.clone();
    state.search_entry.connect_activate(move |_| {
        if let Some(terminal) = state_for_activate.focused.borrow().clone() {
            terminal.search_find_next();
        }
    });
}

fn zoom(state: &WindowState, factor: f64) {
    if let Some(terminal) = state.focused.borrow().as_ref() {
        let scale = (terminal.font_scale() * factor).clamp(0.25, 4.0);
        terminal.set_font_scale(scale);
    }
}

fn select_relative_tab(state: &WindowState, offset: i32) {
    let page_count = state.notebook.n_pages();
    if page_count < 2 {
        return;
    }

    let current = state.notebook.current_page().unwrap_or(0) as i32;
    let target = (current + offset).rem_euclid(page_count as i32) as u32;
    state.notebook.set_current_page(Some(target));
}

fn find_terminal(widget: &gtk4::Widget) -> Option<vte4::Terminal> {
    if let Some(terminal) = widget.downcast_ref::<vte4::Terminal>() {
        return Some(terminal.clone());
    }

    let mut child = widget.first_child();
    while let Some(widget) = child {
        if let Some(terminal) = find_terminal(&widget) {
            return Some(terminal);
        }
        child = widget.next_sibling();
    }
    None
}

/// Tracks focus-enter on `terminal` so window-level actions (copy, split, search, zoom)
/// always apply to whichever pane the user last interacted with.
fn track_focus(terminal: &vte4::Terminal, state: &WindowState) {
    let controller = gtk4::EventControllerFocus::new();
    let terminal_for_focus = terminal.clone();
    let focused = state.focused.clone();
    controller.connect_enter(move |_| {
        *focused.borrow_mut() = Some(terminal_for_focus.clone());
    });
    terminal.add_controller(controller);

    let app = state.app.clone();
    terminal.connect_child_exited(move |_, _| {
        app.quit();
    });
}

fn add_tab(state: &WindowState) {
    let profile = state
        .config
        .borrow()
        .profiles
        .first()
        .cloned()
        .unwrap_or_default();
    let pane = TerminalPane::new(&profile, &state.theme, &state.config.borrow());
    track_focus(&pane.terminal, state);

    let (label, close_button) = tab_bar::build_tab_label(&profile.name);
    let notebook = state.notebook.clone();
    let root_for_close = pane.root.clone();
    close_button.connect_clicked(move |_| {
        if let Some(page_num) = notebook.page_num(&root_for_close) {
            notebook.remove_page(Some(page_num));
        }
    });

    let page_index = state.notebook.append_page(&pane.root, Some(&label));
    state.notebook.set_tab_reorderable(&pane.root, true);
    state.notebook.set_current_page(Some(page_index));
    pane.terminal.grab_focus();
}

fn close_current_tab(state: &WindowState) {
    if let Some(page_num) = state.notebook.current_page() {
        state.notebook.remove_page(Some(page_num));
    }
}

/// Splits the pane containing the focused terminal, placing a freshly spawned terminal
/// alongside it in a new `gtk4::Paned` (PROMPT-CHORD.md §3.1).
fn split_focused(state: &WindowState, orientation: gtk4::Orientation) {
    let Some(focused_terminal) = state.focused.borrow().clone() else {
        return;
    };
    let Some(old_root) = focused_terminal
        .parent()
        .and_downcast::<gtk4::ScrolledWindow>()
    else {
        return;
    };
    let Some(container) = old_root.parent() else {
        return;
    };

    let profile = state
        .config
        .borrow()
        .profiles
        .first()
        .cloned()
        .unwrap_or_default();
    let new_pane = TerminalPane::new(&profile, &state.theme, &state.config.borrow());
    track_focus(&new_pane.terminal, state);

    if let Some(notebook) = container.clone().downcast::<gtk4::Notebook>().ok() {
        let page_num = notebook.page_num(&old_root);
        let tab_label = notebook.tab_label(&old_root);
        if let Some(page_num) = page_num {
            notebook.remove_page(Some(page_num));

            let paned = gtk4::Paned::new(orientation);
            paned.set_start_child(Some(&old_root));
            paned.set_end_child(Some(&new_pane.root));
            paned.set_resize_start_child(true);
            paned.set_resize_end_child(true);

            let inserted = notebook.insert_page(&paned, tab_label.as_ref(), Some(page_num));
            notebook.set_current_page(Some(inserted));
        }
    } else if let Some(parent_paned) = container.downcast::<gtk4::Paned>().ok() {
        let old_was_start = parent_paned.start_child().as_ref() == Some(old_root.upcast_ref());

        if old_was_start {
            parent_paned.set_start_child(gtk4::Widget::NONE);
        } else {
            parent_paned.set_end_child(gtk4::Widget::NONE);
        }

        let paned = gtk4::Paned::new(orientation);
        paned.set_start_child(Some(&old_root));
        paned.set_end_child(Some(&new_pane.root));
        paned.set_resize_start_child(true);
        paned.set_resize_end_child(true);

        if old_was_start {
            parent_paned.set_start_child(Some(&paned));
        } else {
            parent_paned.set_end_child(Some(&paned));
        }
    }

    new_pane.terminal.grab_focus();
}

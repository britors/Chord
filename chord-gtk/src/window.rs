//! The Chord window: tabs, splits, and the v1 shortcuts (PROMPT-CHORD.md §3.1).

use std::cell::RefCell;
use std::rc::Rc;

use chord_core::i18n::tr;
use chord_core::{Config, Theme};
use gtk4::prelude::*;
use vte4::prelude::*;

use crate::tab_bar;
use crate::terminal_pane::TerminalPane;

/// Registers the app-wide accelerators for every `win.*` action (PROMPT-CHORD.md §3.1).
pub fn register_accels(app: &gtk4::Application) {
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
    notebook: gtk4::Notebook,
    search_bar: gtk4::SearchBar,
    search_entry: gtk4::SearchEntry,
    theme: Rc<Theme>,
    config: Rc<Config>,
    focused: Rc<RefCell<Option<vte4::Terminal>>>,
}

pub fn build_window(app: &gtk4::Application, config: &Config) -> gtk4::ApplicationWindow {
    let theme = Rc::new(Theme::chord_dark());
    let config = Rc::new(config.clone());

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
        notebook: notebook.clone(),
        search_bar,
        search_entry,
        theme,
        config,
        focused: Rc::new(RefCell::new(None)),
    };

    let header = gtk4::HeaderBar::new();
    let new_tab_button = gtk4::Button::from_icon_name("tab-new-symbolic");
    new_tab_button.set_action_name(Some("win.new-tab"));
    new_tab_button.set_tooltip_text(Some(&tr("New tab")));
    header.pack_start(&new_tab_button);

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Chord")
        .default_width(960)
        .default_height(600)
        .build();
    window.set_titlebar(Some(&header));
    window.set_child(Some(&content));

    install_actions(&window, &state);
    wire_search(&state);

    add_tab(&state);

    window.present();
    window
}

fn install_actions(window: &gtk4::ApplicationWindow, state: &WindowState) {
    let action = |name: &str, state: WindowState, f: fn(&WindowState)| {
        let action = gtk4::gio::SimpleAction::new(name, None);
        action.connect_activate(move |_, _| f(&state));
        action
    };

    window.add_action(&action("new-tab", state.clone(), |s| add_tab(s)));
    window.add_action(&action("close-tab", state.clone(), |s| close_current_tab(s)));
    window.add_action(&action("next-tab", state.clone(), |s| s.notebook.next_page()));
    window.add_action(&action("prev-tab", state.clone(), |s| s.notebook.prev_page()));
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
}

fn add_tab(state: &WindowState) {
    let profile = state
        .config
        .profiles
        .first()
        .cloned()
        .unwrap_or_default();
    let pane = TerminalPane::new(&profile, &state.theme);
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
        .profiles
        .first()
        .cloned()
        .unwrap_or_default();
    let new_pane = TerminalPane::new(&profile, &state.theme);
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

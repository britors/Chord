//! Builds tab label widgets (title + close button) for the window's `gtk4::Notebook`.

use chord_core::i18n::tr;
use gtk4::prelude::*;

fn finish_edit(entry: &gtk4::Entry, label: &gtk4::Label, stack: &gtk4::Stack) {
    let title = entry.text();
    let title = title.trim();
    if !title.is_empty() {
        label.set_text(title);
    }
    entry.set_text(&label.text());
    stack.set_visible_child(label);
}

/// A tab label: a title and a close button, packed in a row. Returns the row plus the
/// close button, so callers can wire up their own removal logic.
pub fn build_tab_label(title: &str) -> (gtk4::Box, gtk4::Button) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let label = gtk4::Label::new(Some(title));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_width_chars(12);
    label.set_tooltip_text(Some(&tr("Double-click to rename")));

    let entry = gtk4::Entry::builder()
        .text(title)
        .width_chars(12)
        .has_frame(false)
        .build();

    let title_stack = gtk4::Stack::new();
    title_stack.add_child(&label);
    title_stack.add_child(&entry);
    title_stack.set_visible_child(&label);

    let edit_gesture = gtk4::GestureClick::new();
    let entry_for_edit = entry.clone();
    let label_for_edit = label.clone();
    let stack_for_edit = title_stack.clone();
    edit_gesture.connect_pressed(move |_, press_count, _, _| {
        if press_count == 2 {
            entry_for_edit.set_text(&label_for_edit.text());
            stack_for_edit.set_visible_child(&entry_for_edit);
            entry_for_edit.grab_focus();
            entry_for_edit.select_region(0, -1);
        }
    });
    label.add_controller(edit_gesture);

    let label_for_activate = label.clone();
    let stack_for_activate = title_stack.clone();
    entry.connect_activate(move |entry| {
        finish_edit(entry, &label_for_activate, &stack_for_activate);
    });

    let focus_controller = gtk4::EventControllerFocus::new();
    let entry_for_focus = entry.clone();
    let label_for_focus = label.clone();
    let stack_for_focus = title_stack.clone();
    focus_controller.connect_leave(move |_| {
        finish_edit(&entry_for_focus, &label_for_focus, &stack_for_focus);
    });
    entry.add_controller(focus_controller);

    let key_controller = gtk4::EventControllerKey::new();
    let entry_for_key = entry.clone();
    let label_for_key = label.clone();
    let stack_for_key = title_stack.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            entry_for_key.set_text(&label_for_key.text());
            stack_for_key.set_visible_child(&label_for_key);
            return gtk4::glib::Propagation::Stop;
        }
        gtk4::glib::Propagation::Proceed
    });
    entry.add_controller(key_controller);

    let close_button = gtk4::Button::from_icon_name("window-close-symbolic");
    close_button.set_has_frame(false);
    close_button.set_tooltip_text(Some(&tr("Close tab")));

    row.append(&title_stack);
    row.append(&close_button);

    (row, close_button)
}

//! Builds tab label widgets (title + close button) for the window's `gtk4::Notebook`.

use chord_core::i18n::tr;
use gtk4::prelude::*;

/// A tab label: a title and a close button, packed in a row. Returns the row plus the
/// close button, so callers can wire up their own removal logic.
pub fn build_tab_label(title: &str) -> (gtk4::Box, gtk4::Button) {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let label = gtk4::Label::new(Some(title));
    label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    label.set_width_chars(12);

    let close_button = gtk4::Button::from_icon_name("window-close-symbolic");
    close_button.set_has_frame(false);
    close_button.set_tooltip_text(Some(&tr("Close tab")));

    row.append(&label);
    row.append(&close_button);

    (row, close_button)
}

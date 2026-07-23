//! Wraps a single [`vte4::Terminal`] with the shared theme applied and a shell spawned
//! (PROMPT-CHORD.md §2.3: the ANSI palette always comes from `chord_core::theme`,
//! never hardcoded here).

use chord_core::profile::ShellProfile;
use chord_core::theme::{Color, Theme};
use chord_core::Config;
use vte4::prelude::*;

/// A single terminal, plus the scrollable container that actually gets placed into a
/// tab or a split — this is the widget callers reparent when splitting panes.
pub struct TerminalPane {
    pub terminal: vte4::Terminal,
    pub root: gtk4::ScrolledWindow,
    pub shell_pid: Rc<Cell<Option<i32>>>,
}

impl TerminalPane {
    pub fn new(profile: &ShellProfile, theme: &Theme, config: &Config) -> Self {
        let terminal = vte4::Terminal::new();
        terminal.set_scrollback_lines(10_000);
        terminal.set_bold_is_bright(true);
        apply_config(&terminal, theme, config);
        let shell_pid = spawn_shell(&terminal, profile);

        let root = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .hexpand(true)
            .child(&terminal)
            .build();

        Self {
            terminal,
            root,
            shell_pid,
        }
    }
}

/// Applies the user-configurable visual settings on top of the selected theme.
pub fn apply_config(terminal: &vte4::Terminal, theme: &Theme, config: &Config) {
    let mut configured_theme = theme.clone();
    configured_theme.background_opacity_percent = config.background_opacity_percent;
    configured_theme.cursor_blink = config.cursor_blink;
    apply_theme(terminal, &configured_theme);

    let font =
        gtk4::pango::FontDescription::from_string(&format!("{} {}", config.font, config.font_size));
    terminal.set_font(Some(&font));
}

fn to_rgba(color: &Color, alpha: f32) -> gtk4::gdk::RGBA {
    let (r, g, b) = color.to_rgb8().unwrap_or((0, 0, 0));
    gtk4::gdk::RGBA::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, alpha)
}

/// Applies every color and cursor setting from `theme` to `terminal`. The single place
/// where a `chord_core::Theme` becomes concrete VTE/GDK colors.
pub fn apply_theme(terminal: &vte4::Terminal, theme: &Theme) {
    let foreground = to_rgba(&theme.foreground, 1.0);
    let background_alpha = theme.background_opacity_percent as f32 / 100.0;
    let background = to_rgba(&theme.background, background_alpha);
    let palette: Vec<gtk4::gdk::RGBA> = theme
        .ansi
        .as_array()
        .iter()
        .map(|c| to_rgba(c, 1.0))
        .collect();
    let palette_refs: Vec<&gtk4::gdk::RGBA> = palette.iter().collect();

    terminal.set_colors(Some(&foreground), Some(&background), &palette_refs);
    terminal.set_color_cursor(Some(&to_rgba(&theme.cursor, 1.0)));

    let selection_alpha = theme.selection_opacity_percent as f32 / 100.0;
    terminal.set_color_highlight(Some(&to_rgba(&theme.selection, selection_alpha)));

    terminal.set_cursor_blink_mode(if theme.cursor_blink {
        vte4::CursorBlinkMode::On
    } else {
        vte4::CursorBlinkMode::Off
    });
}

/// Spawns `profile`'s shell command asynchronously inside `terminal`'s PTY.
fn spawn_shell(terminal: &vte4::Terminal, profile: &ShellProfile) -> Rc<Cell<Option<i32>>> {
    let mut argv: Vec<&str> = vec![profile.command.as_str()];
    argv.extend(profile.args.iter().map(String::as_str));
    let shell_pid = Rc::new(Cell::new(None));
    let shell_pid_for_spawn = shell_pid.clone();

    terminal.spawn_async(
        vte4::PtyFlags::DEFAULT,
        None,
        &argv,
        &[],
        gtk4::glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        gtk4::gio::Cancellable::NONE,
        move |result| match result {
            Ok(pid) => shell_pid_for_spawn.set(Some(pid.0)),
            Err(err) => {
                eprintln!("chord-gtk: failed to spawn shell: {err}");
            }
        },
    );
    shell_pid
}
use std::cell::Cell;
use std::rc::Rc;

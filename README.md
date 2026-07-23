# Chord

Terminal emulator for the Lyra Enterprise Linux ecosystem. Two native frontends (GTK4
for GNOME, Qt for KDE) share a single UI-agnostic Rust core, `chord-core`. See
`PROMPT-CHORD.md` for the full build spec.

## Features

- Tabs with inline renaming (double-click the tab title)
- Horizontal and vertical pane splits
- Copy, paste, scrollback search and font zoom
- Automatic application exit when the shell finishes
- Preferences for system monospace font, font size, background opacity, blinking cursor
  and the shell command used by new terminals
- About dialog with application identity, maintainer and GPLv3 license
- Built-in enterprise "Chord Dark" theme plus base16 theme import
- Shared TOML configuration at `~/.config/chord/config.toml`

The GTK4 frontend is functional. The Qt frontend described in the architecture has not
been started yet.

## Default theme

The default theme follows the Lyra Enterprise Linux palette. `data/palette.json` is the
single color source consumed by `chord-core`; `data/themes/chord-dark.json` is a
generated artifact. The theme uses a 95% opaque slate background, the system's
configured monospace font and a subdued ANSI palette.

## Building

```sh
cargo build --workspace
cargo run -p chord-gtk
```

Requires Rust, GTK4, libadwaita and VTE's GTK4 development files (Fedora:
`gtk4-devel`, `libadwaita-devel` and `vte291-gtk4-devel`).

## Configuration

Open the main menu in the window header and select **Settings**. Visual changes apply
to open terminals immediately; shell command changes apply to newly created tabs and
panes. Settings are saved in `~/.config/chord/config.toml`.

The main shortcuts are:

| Action | Shortcut |
|---|---|
| New tab | `Ctrl+Shift+T` |
| Close tab | `Ctrl+Shift+W` |
| Next/previous tab | `Ctrl+Tab` / `Ctrl+Shift+Tab` |
| Horizontal/vertical split | `Ctrl+Shift+O` / `Ctrl+Shift+E` |
| Copy/paste | `Ctrl+Shift+C` / `Ctrl+Shift+V` |
| Search scrollback | `Ctrl+Shift+F` |
| Zoom | `Ctrl++` / `Ctrl+-` / `Ctrl+0` |

## Regenerating the default theme artifact

`data/themes/chord-dark.json` is generated, not hand-edited:

```sh
cargo run -p chord-core --example export_chord_dark > data/themes/chord-dark.json
```

## License

GPLv3.

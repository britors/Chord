# Chord

Terminal emulator for the Lyra OS ecosystem. Two native frontends (GTK4 for GNOME, Qt
for KDE) share a single UI-agnostic Rust core, `chord-core`. See `PROMPT-CHORD.md` for
the full build spec.

## Status

- `chord-core` — done: theming (built-in "Chord Dark" + base16 import), shell profile
  detection, `~/.config/chord/config.toml` read/write, gettext i18n scaffolding.
- `chord-gtk` — v1 essentials in place: tabs, horizontal/vertical splits, copy/paste,
  scrollback search, font zoom, all wired to the shortcuts in the spec (§3.1).
- `chord-qt` — not started.

## Building

```sh
cargo build --workspace
cargo run -p chord-gtk
```

Requires GTK4 and VTE's GTK4 widget development files (Fedora: `vte291-gtk4-devel`).

## Regenerating the default theme artifact

`data/themes/chord-dark.json` is generated, not hand-edited:

```sh
cargo run -p chord-core --example export_chord_dark > data/themes/chord-dark.json
```

## License

GPLv3.

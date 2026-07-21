//! Regenerates `data/themes/chord-dark.json` from `chord_core::theme::Theme::chord_dark()`.
//! Run with `cargo run -p chord-core --example export_chord_dark > data/themes/chord-dark.json`.

fn main() {
    let theme = chord_core::Theme::chord_dark();
    println!("{}", theme.to_json_string_pretty().unwrap());
}

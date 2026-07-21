//! `chord-core` — shared, UI-toolkit-agnostic logic for the Chord terminal emulator.
//!
//! This crate has no GTK or Qt dependency. Both `chord-gtk` and `chord-qt` depend on it
//! for theming, shell profiles, configuration, and translations, so a fix here applies
//! to both frontends by construction (see PROMPT-CHORD.md §2.2).

pub mod config;
pub mod i18n;
pub mod profile;
pub mod theme;

pub use config::Config;
pub use profile::ShellProfile;
pub use theme::Theme;

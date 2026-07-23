//! Theme parsing and the "Chord Dark" default theme (PROMPT-CHORD.md §3.2).
//!
//! A [`Theme`] is the single source of truth for color used by both frontends: neither
//! `chord-gtk` nor `chord-qt` is allowed to hardcode a palette of its own.

use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("failed to read theme file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse theme JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("base16 scheme is missing required key `{0}`")]
    MissingBase16Key(&'static str),
}

/// An RGB color stored as `#rrggbb`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub String);

impl Color {
    pub fn new(hex: impl Into<String>) -> Self {
        Self(hex.into())
    }

    /// Parses `#rrggbb` or `#rgb` into 8-bit RGB components.
    pub fn to_rgb8(&self) -> Option<(u8, u8, u8)> {
        let hex = self.0.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some((r, g, b))
            }
            3 => {
                let expand = |c: &str| u8::from_str_radix(&c.repeat(2), 16).ok();
                Some((
                    expand(&hex[0..1])?,
                    expand(&hex[1..2])?,
                    expand(&hex[2..3])?,
                ))
            }
            _ => None,
        }
    }
}

/// The 16-color ANSI palette (standard + bright variants).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnsiPalette {
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub bright_black: Color,
    pub bright_red: Color,
    pub bright_green: Color,
    pub bright_yellow: Color,
    pub bright_blue: Color,
    pub bright_magenta: Color,
    pub bright_cyan: Color,
    pub bright_white: Color,
}

/// Branding colors supplied by `lyra-branding`'s `palette.json`.
///
/// The file is embedded at compile time so every frontend receives exactly the same
/// values through `chord-core`, without maintaining a second hardcoded palette.
#[derive(Debug, Deserialize)]
struct BrandingPalette {
    #[serde(rename = "lyra-slate")]
    slate: Color,
    #[serde(rename = "lyra-slate-alt")]
    slate_alt: Color,
    #[serde(rename = "lyra-mist")]
    mist: Color,
    #[serde(rename = "lyra-haze")]
    haze: Color,
    #[serde(rename = "lyra-fog")]
    fog: Color,
    #[serde(rename = "lyra-fog-alt")]
    fog_alt: Color,
    #[serde(rename = "lyra-lavender")]
    lavender: Color,
    #[serde(rename = "lyra-star")]
    star: Color,
    ansi: AnsiPalette,
}

fn branding_palette() -> BrandingPalette {
    serde_json::from_str(include_str!("../../data/palette.json"))
        .expect("the embedded Lyra palette.json must be valid")
}

impl AnsiPalette {
    /// The 16 colors in canonical VT100 ANSI order (index 0-15), as consumed by
    /// terminal widgets such as VTE's `set_colors`.
    pub fn as_array(&self) -> [Color; 16] {
        [
            self.black.clone(),
            self.red.clone(),
            self.green.clone(),
            self.yellow.clone(),
            self.blue.clone(),
            self.magenta.clone(),
            self.cyan.clone(),
            self.white.clone(),
            self.bright_black.clone(),
            self.bright_red.clone(),
            self.bright_green.clone(),
            self.bright_yellow.clone(),
            self.bright_blue.clone(),
            self.bright_magenta.clone(),
            self.bright_cyan.clone(),
            self.bright_white.clone(),
        ]
    }
}

/// A full terminal theme: background/foreground/cursor/selection plus the 16-color
/// ANSI palette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub cursor_blink: bool,
    pub selection: Color,
    /// Selection highlight opacity, 0-100 (percent).
    pub selection_opacity_percent: u8,
    /// Background opacity, 0-100 (percent opaque). Spec default: 100.
    pub background_opacity_percent: u8,
    pub ansi: AnsiPalette,
}

impl Theme {
    /// The built-in "Chord Dark" theme, derived exclusively from the embedded
    /// `lyra-branding` `palette.json`.
    pub fn chord_dark() -> Self {
        let palette = branding_palette();
        // Keep light-mode tokens parsed and validated even though Chord Light is
        // outside the v1 scope.
        let _light_tokens = (
            &palette.fog,
            &palette.fog_alt,
            &palette.lavender,
            &palette.haze,
            &palette.slate_alt,
        );
        Self {
            name: "Chord Dark".to_string(),
            background: palette.slate,
            foreground: palette.star,
            cursor: palette.mist.clone(),
            cursor_blink: true,
            selection: palette.mist,
            selection_opacity_percent: 40,
            background_opacity_percent: 95,
            ansi: palette.ansi,
        }
    }

    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, ThemeError> {
        let raw = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn to_json_string_pretty(&self) -> Result<String, ThemeError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Parses a base16 scheme (https://github.com/chriskempson/base16) given as JSON
    /// with `base00`.."base0F"` keys (hex, no leading `#`), mapping it onto our
    /// [`Theme`] shape via the widely-used base16-shell ANSI convention. Lets popular
    /// schemes (Catppuccin, Nord, Dracula, Tokyo Night) load without manual conversion,
    /// per PROMPT-CHORD.md §3.2.
    pub fn from_base16_file(path: impl AsRef<Path>) -> Result<Self, ThemeError> {
        let raw = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&raw)?;
        Self::from_base16_value(&value)
    }

    fn from_base16_value(value: &serde_json::Value) -> Result<Self, ThemeError> {
        let base = |key: &'static str| -> Result<Color, ThemeError> {
            let hex = value
                .get(key)
                .and_then(|v| v.as_str())
                .ok_or(ThemeError::MissingBase16Key(key))?;
            Ok(Color::new(format!("#{hex}")))
        };

        let name = value
            .get("scheme")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled base16 scheme")
            .to_string();

        Ok(Self {
            name,
            background: base("base00")?,
            foreground: base("base05")?,
            cursor: base("base05")?,
            cursor_blink: true,
            selection: base("base02")?,
            selection_opacity_percent: 40,
            background_opacity_percent: 100,
            ansi: AnsiPalette {
                black: base("base00")?,
                red: base("base08")?,
                green: base("base0B")?,
                yellow: base("base0A")?,
                blue: base("base0D")?,
                magenta: base("base0E")?,
                cyan: base("base0C")?,
                white: base("base05")?,
                bright_black: base("base03")?,
                bright_red: base("base08")?,
                bright_green: base("base0B")?,
                bright_yellow: base("base0A")?,
                bright_blue: base("base0D")?,
                bright_magenta: base("base0E")?,
                bright_cyan: base("base0C")?,
                bright_white: base("base07")?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relative_luminance(color: &Color) -> f64 {
        let (r, g, b) = color.to_rgb8().unwrap();
        let linear = |component: u8| {
            let value = component as f64 / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
    }

    fn contrast_ratio(a: &Color, b: &Color) -> f64 {
        let (lighter, darker) = {
            let a = relative_luminance(a);
            let b = relative_luminance(b);
            if a > b {
                (a, b)
            } else {
                (b, a)
            }
        };
        (lighter + 0.05) / (darker + 0.05)
    }

    #[test]
    fn color_hex_parsing() {
        assert_eq!(Color::new("#0D0D1F").to_rgb8(), Some((0x0D, 0x0D, 0x1F)));
        assert_eq!(Color::new("#fff").to_rgb8(), Some((0xFF, 0xFF, 0xFF)));
        assert_eq!(Color::new("nonsense").to_rgb8(), None);
    }

    #[test]
    fn chord_dark_roundtrips_through_json() {
        let theme = Theme::chord_dark();
        let json = theme.to_json_string_pretty().unwrap();
        let parsed: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(theme, parsed);
    }

    #[test]
    fn chord_dark_uses_enterprise_branding_tokens() {
        let theme = Theme::chord_dark();
        assert_eq!(theme.background, Color::new("#16191D"));
        assert_eq!(theme.foreground, Color::new("#E8ECFF"));
        assert_eq!(theme.cursor, Color::new("#262B3D"));
        assert_eq!(theme.selection, Color::new("#262B3D"));
        assert_eq!(theme.background_opacity_percent, 95);
    }

    #[test]
    fn chord_dark_text_passes_wcag_aa() {
        let theme = Theme::chord_dark();
        assert!(contrast_ratio(&theme.foreground, &theme.background) >= 4.5);
    }

    #[test]
    fn base16_scheme_parses() {
        let value = serde_json::json!({
            "scheme": "Test Scheme",
            "base00": "000000", "base01": "111111", "base02": "222222", "base03": "333333",
            "base04": "444444", "base05": "eeeeee", "base06": "fefefe", "base07": "ffffff",
            "base08": "ff0000", "base09": "ff8800", "base0A": "ffff00", "base0B": "00ff00",
            "base0C": "00ffff", "base0D": "0000ff", "base0E": "ff00ff", "base0F": "884400",
        });
        let theme = Theme::from_base16_value(&value).unwrap();
        assert_eq!(theme.name, "Test Scheme");
        assert_eq!(theme.background.0, "#000000");
        assert_eq!(theme.ansi.red.0, "#ff0000");
    }
}

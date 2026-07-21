//! Neutral, cross-toolkit configuration (PROMPT-CHORD.md §2.4): a single TOML file at
//! `~/.config/chord/config.toml`, read and written identically by both frontends —
//! never GSettings/dconf on one side and KConfig on the other.

use crate::profile::ShellProfile;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not determine the user config directory")]
    NoConfigDir,
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config.toml: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize config.toml: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: String,
    pub background_opacity_percent: u8,
    pub cursor_blink: bool,
    pub font: String,
    pub font_size: u32,
    pub profiles: Vec<ShellProfile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "chord-dark".to_string(),
            background_opacity_percent: 100,
            cursor_blink: true,
            font: "JetBrains Mono".to_string(),
            font_size: 11,
            profiles: vec![ShellProfile::default_for_system()],
        }
    }
}

impl Config {
    /// `~/.config/chord/config.toml`.
    pub fn path() -> Result<PathBuf, ConfigError> {
        let base = dirs::config_dir().ok_or(ConfigError::NoConfigDir)?;
        Ok(base.join("chord").join("config.toml"))
    }

    /// Loads the config, falling back to [`Config::default`] if no file exists yet.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self)?;
        std::fs::write(path, raw)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_roundtrips_through_toml() {
        let config = Config::default();
        let raw = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&raw).unwrap();
        assert_eq!(config, parsed);
    }
}

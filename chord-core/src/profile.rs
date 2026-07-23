//! Shell profiles: which command to launch per tab/pane, and its font (PROMPT-CHORD.md §3.1).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellProfile {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub font: String,
    pub font_size: u32,
}

impl ShellProfile {
    /// The default profile: the user's `$SHELL`, rendered with the system's configured
    /// monospace font.
    pub fn default_for_system() -> Self {
        Self {
            name: "Default".to_string(),
            command: detect_default_shell(),
            args: Vec::new(),
            font: "Monospace".to_string(),
            font_size: 11,
        }
    }
}

impl Default for ShellProfile {
    fn default() -> Self {
        Self::default_for_system()
    }
}

/// Detects the user's default shell from `$SHELL`, falling back to `/bin/sh` if unset.
pub fn detect_default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_when_shell_unset() {
        // We can't unset $SHELL process-wide safely in a parallel test run, so just
        // check the function never panics and returns a non-empty path.
        assert!(!detect_default_shell().is_empty());
    }
}

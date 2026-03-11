use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

/// Parsed dependency graph rules configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Per-crate allowlists: crate name -> list of allowed internal dependencies.
    #[serde(default)]
    pub rules: BTreeMap<String, Vec<String>>,

    /// Global options controlling validation behavior.
    #[serde(default)]
    pub options: Options,
}

/// Global options for the validation engine.
#[derive(Debug, Deserialize)]
pub struct Options {
    /// If true, workspace members not listed in [rules] are errors.
    /// If false, they are warnings.
    #[serde(default = "default_true")]
    pub strict: bool,

    /// Whether to validate dev-dependencies against the allowlist.
    #[serde(default)]
    pub check_dev_deps: bool,

    /// Behavior when a config entry names a crate not in the workspace.
    #[serde(default)]
    pub unmatched_config_entries: UnmatchedPolicy,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            strict: true,
            check_dev_deps: false,
            unmatched_config_entries: UnmatchedPolicy::default(),
        }
    }
}

/// What to do when a config entry names a crate not present in the workspace.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnmatchedPolicy {
    /// Print a warning and continue.
    #[default]
    Warn,
    /// Treat as a violation (fail the check).
    Error,
    /// Silently skip.
    Ignore,
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Read and parse a config from a TOML file.
    pub fn from_path(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|source| Error::ConfigRead {
            path: path.to_path_buf(),
            source,
        })?;
        let config: Config = toml::from_str(&contents).map_err(|source| Error::ConfigParse {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_parses_valid_toml() {
        let toml = r#"
            [rules]
            core = []
            api = ["core"]

            [options]
            strict = true
            check_dev_deps = false
            unmatched_config_entries = "warn"
        "#;
        let config: Config = toml::from_str(toml).expect("should parse");
        assert_eq!(config.rules.len(), 2);
        assert_eq!(config.rules["core"], Vec::<String>::new());
        assert_eq!(config.rules["api"], vec!["core"]);
        assert!(config.options.strict);
        assert!(!config.options.check_dev_deps);
        assert_eq!(
            config.options.unmatched_config_entries,
            UnmatchedPolicy::Warn
        );
    }

    #[test]
    fn config_defaults_options_when_omitted() {
        let toml = r#"
            [rules]
            core = []
        "#;
        let config: Config = toml::from_str(toml).expect("should parse");
        assert!(config.options.strict);
        assert!(!config.options.check_dev_deps);
        assert_eq!(
            config.options.unmatched_config_entries,
            UnmatchedPolicy::Warn
        );
    }

    #[test]
    fn config_defaults_strict_to_true() {
        let toml = r#"
            [rules]
            core = []

            [options]
            check_dev_deps = true
        "#;
        let config: Config = toml::from_str(toml).expect("should parse");
        assert!(config.options.strict);
    }

    #[test]
    fn config_deserializes_unmatched_policy_variants() {
        for (input, expected) in [
            ("\"warn\"", UnmatchedPolicy::Warn),
            ("\"error\"", UnmatchedPolicy::Error),
            ("\"ignore\"", UnmatchedPolicy::Ignore),
        ] {
            let toml = format!("[rules]\n[options]\nunmatched_config_entries = {input}");
            let config: Config = toml::from_str(&toml).expect("should parse");
            assert_eq!(config.options.unmatched_config_entries, expected);
        }
    }

    #[test]
    fn config_rejects_invalid_toml() {
        let toml = "this is not valid toml {{{}}}";
        let result: std::result::Result<Config, _> = toml::from_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn config_parses_empty_rules_section() {
        let toml = "[rules]\n[options]\n";
        let config: Config = toml::from_str(toml).expect("should parse");
        assert!(config.rules.is_empty());
    }
}

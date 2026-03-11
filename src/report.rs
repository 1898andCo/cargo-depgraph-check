use crate::validate::{Violation, ViolationKind};

/// Whether to use colors in output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    On,
    Off,
}

impl ColorMode {
    /// Determine color mode from CLI flag and environment.
    pub fn from_choice(choice: &str, is_terminal: bool) -> Self {
        match choice {
            "always" => Self::On,
            "never" => Self::Off,
            _ => {
                // "auto" — check NO_COLOR env var and terminal status
                if std::env::var("NO_COLOR").is_ok() {
                    Self::Off
                } else if is_terminal {
                    Self::On
                } else {
                    Self::Off
                }
            }
        }
    }
}

/// Format violations and warnings as human-readable text.
pub fn report_text(errors: &[Violation], warnings: &[Violation], color: ColorMode) -> String {
    let mut output = String::new();

    for w in warnings {
        let msg = format_violation(w, "WARNING");
        if color == ColorMode::On {
            output.push_str(&format!("\x1b[33m{msg}\x1b[0m\n"));
        } else {
            output.push_str(&format!("{msg}\n"));
        }
    }

    for e in errors {
        let msg = format_violation(e, "ERROR");
        if color == ColorMode::On {
            output.push_str(&format!("\x1b[31m{msg}\x1b[0m\n"));
        } else {
            output.push_str(&format!("{msg}\n"));
        }
    }

    // Summary line.
    if errors.is_empty() && warnings.is_empty() {
        let msg = "All dependency rules pass";
        if color == ColorMode::On {
            output.push_str(&format!("\n\x1b[32m{msg}\x1b[0m\n"));
        } else {
            output.push_str(&format!("\n{msg}\n"));
        }
    } else if errors.is_empty() {
        let msg = format!("\n{} warning(s), no violations", warnings.len());
        if color == ColorMode::On {
            output.push_str(&format!("\x1b[33m{msg}\x1b[0m\n"));
        } else {
            output.push_str(&format!("{msg}\n"));
        }
    } else {
        let crate_names: std::collections::BTreeSet<_> =
            errors.iter().map(|v| &v.crate_name).collect();
        let msg = format!(
            "\n{} violation(s) found across {} crate(s)",
            errors.len(),
            crate_names.len()
        );
        if color == ColorMode::On {
            output.push_str(&format!("\x1b[31m{msg}\x1b[0m\n"));
        } else {
            output.push_str(&format!("{msg}\n"));
        }
    }

    output
}

/// Format violations and warnings as JSON.
pub fn report_json(errors: &[Violation], warnings: &[Violation]) -> String {
    let error_entries: Vec<serde_json::Value> = errors.iter().map(violation_to_json).collect();
    let warning_entries: Vec<serde_json::Value> = warnings.iter().map(violation_to_json).collect();

    let crate_names: std::collections::BTreeSet<_> = errors.iter().map(|v| &v.crate_name).collect();

    let result = serde_json::json!({
        "violations": error_entries,
        "warnings": warning_entries,
        "summary": {
            "violation_count": errors.len(),
            "warning_count": warnings.len(),
            "crate_count": crate_names.len(),
        }
    });

    serde_json::to_string_pretty(&result).expect("JSON serialization should not fail")
}

fn format_violation(v: &Violation, level: &str) -> String {
    match &v.kind {
        ViolationKind::ForbiddenDependency { dep, allowed } => {
            let allowed_str = format!("[{}]", allowed.join(", "));
            format!(
                "{level}: {} depends on {dep}, but allowed deps are: {allowed_str}",
                v.crate_name
            )
        }
        ViolationKind::UnknownCrate => {
            format!(
                "{level}: workspace member '{}' is not listed in the config",
                v.crate_name
            )
        }
        ViolationKind::UnmatchedConfigEntry => {
            format!(
                "{level}: config entry '{}' has no matching workspace member",
                v.crate_name
            )
        }
    }
}

fn violation_to_json(v: &Violation) -> serde_json::Value {
    match &v.kind {
        ViolationKind::ForbiddenDependency { dep, allowed } => {
            serde_json::json!({
                "crate": v.crate_name,
                "kind": "forbidden_dependency",
                "dependency": dep,
                "allowed": allowed,
            })
        }
        ViolationKind::UnknownCrate => {
            serde_json::json!({
                "crate": v.crate_name,
                "kind": "unknown_crate",
            })
        }
        ViolationKind::UnmatchedConfigEntry => {
            serde_json::json!({
                "crate": v.crate_name,
                "kind": "unmatched_config_entry",
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::{Violation, ViolationKind};

    #[test]
    fn report_text_no_violations() {
        let output = report_text(&[], &[], ColorMode::Off);
        assert!(output.contains("All dependency rules pass"));
    }

    #[test]
    fn report_text_with_violation() {
        let errors = vec![Violation {
            crate_name: "api".to_string(),
            kind: ViolationKind::ForbiddenDependency {
                dep: "server".to_string(),
                allowed: vec!["core".to_string()],
            },
        }];
        let output = report_text(&errors, &[], ColorMode::Off);
        assert!(output.contains("ERROR: api depends on server, but allowed deps are: [core]"));
        assert!(output.contains("1 violation(s) found across 1 crate(s)"));
    }

    #[test]
    fn report_text_with_warning() {
        let warnings = vec![Violation {
            crate_name: "ghost".to_string(),
            kind: ViolationKind::UnmatchedConfigEntry,
        }];
        let output = report_text(&[], &warnings, ColorMode::Off);
        assert!(output.contains("WARNING: config entry 'ghost' has no matching workspace member"));
        assert!(output.contains("1 warning(s), no violations"));
    }

    #[test]
    fn report_json_structure() {
        let errors = vec![Violation {
            crate_name: "api".to_string(),
            kind: ViolationKind::ForbiddenDependency {
                dep: "server".to_string(),
                allowed: vec!["core".to_string()],
            },
        }];
        let output = report_json(&errors, &[]);
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("should be valid JSON");
        assert_eq!(parsed["summary"]["violation_count"], 1);
        assert_eq!(parsed["violations"][0]["crate"], "api");
        assert_eq!(parsed["violations"][0]["kind"], "forbidden_dependency");
        assert_eq!(parsed["violations"][0]["dependency"], "server");
    }

    #[test]
    fn report_text_colors_disabled_with_no_color() {
        let output = report_text(&[], &[], ColorMode::Off);
        assert!(!output.contains("\x1b["));
    }
}

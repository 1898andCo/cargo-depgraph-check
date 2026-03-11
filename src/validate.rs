use crate::config::{Config, UnmatchedPolicy};
use crate::metadata::WorkspaceGraph;

/// A single dependency graph violation or warning.
#[derive(Debug, Clone)]
pub struct Violation {
    /// The crate that has the issue.
    pub crate_name: String,
    /// What kind of issue was found.
    pub kind: ViolationKind,
}

/// The kind of dependency graph violation.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ViolationKind {
    /// A crate depends on another crate not in its allowlist.
    ForbiddenDependency { dep: String, allowed: Vec<String> },
    /// A workspace member is not listed in the config (strict mode).
    UnknownCrate,
    /// A config entry references a crate not in the workspace.
    UnmatchedConfigEntry,
}

/// Validation result separating errors (affect exit code) from warnings (informational).
pub struct ValidationResult {
    pub errors: Vec<Violation>,
    pub warnings: Vec<Violation>,
}

/// Validate the workspace dependency graph against the config rules.
///
/// Returns errors (violations that should fail the check) and warnings (informational).
pub fn validate(config: &Config, graph: &WorkspaceGraph) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check each workspace member against the allowlist.
    for member in &graph.members {
        match config.rules.get(member) {
            Some(allowed) => {
                // Check each actual dependency against the allowlist.
                if let Some(actual_deps) = graph.dependencies.get(member) {
                    for dep in actual_deps {
                        if !allowed.contains(dep) {
                            errors.push(Violation {
                                crate_name: member.clone(),
                                kind: ViolationKind::ForbiddenDependency {
                                    dep: dep.clone(),
                                    allowed: allowed.clone(),
                                },
                            });
                        }
                    }
                }
            }
            None => {
                // Workspace member not in config.
                let violation = Violation {
                    crate_name: member.clone(),
                    kind: ViolationKind::UnknownCrate,
                };
                if config.options.strict {
                    errors.push(violation);
                } else {
                    warnings.push(violation);
                }
            }
        }
    }

    // Check for config entries that don't match any workspace member.
    for config_crate in config.rules.keys() {
        if !graph.members.contains(config_crate) {
            let violation = Violation {
                crate_name: config_crate.clone(),
                kind: ViolationKind::UnmatchedConfigEntry,
            };
            match config.options.unmatched_config_entries {
                UnmatchedPolicy::Error => errors.push(violation),
                UnmatchedPolicy::Warn => warnings.push(violation),
                UnmatchedPolicy::Ignore => {}
            }
        }
    }

    ValidationResult { errors, warnings }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::config::{Config, Options, UnmatchedPolicy};
    use crate::metadata::WorkspaceGraph;

    fn make_config(
        rules: Vec<(&str, Vec<&str>)>,
        strict: bool,
        unmatched: UnmatchedPolicy,
    ) -> Config {
        Config {
            rules: rules
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.into_iter().map(String::from).collect()))
                .collect(),
            options: Options {
                strict,
                check_dev_deps: false,
                unmatched_config_entries: unmatched,
            },
        }
    }

    fn make_graph(deps: Vec<(&str, Vec<&str>)>) -> WorkspaceGraph {
        let mut members = BTreeSet::new();
        let mut dependencies = BTreeMap::new();
        for (name, dep_list) in deps {
            members.insert(name.to_string());
            dependencies.insert(
                name.to_string(),
                dep_list.into_iter().map(String::from).collect(),
            );
        }
        WorkspaceGraph {
            members,
            dependencies,
        }
    }

    #[test]
    fn validate_passes_valid_config() {
        let config = make_config(
            vec![("core", vec![]), ("api", vec!["core"])],
            true,
            UnmatchedPolicy::Warn,
        );
        let graph = make_graph(vec![("core", vec![]), ("api", vec!["core"])]);
        let result = validate(&config, &graph);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn validate_catches_forbidden_dep() {
        let config = make_config(
            vec![
                ("core", vec![]),
                ("api", vec!["core"]),
                ("server", vec!["core", "api"]),
            ],
            true,
            UnmatchedPolicy::Warn,
        );
        // api depends on server, which is forbidden
        let graph = make_graph(vec![
            ("core", vec![]),
            ("api", vec!["core", "server"]),
            ("server", vec!["core", "api"]),
        ]);
        let result = validate(&config, &graph);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].crate_name, "api");
        assert!(matches!(
            &result.errors[0].kind,
            ViolationKind::ForbiddenDependency { dep, .. } if dep == "server"
        ));
    }

    #[test]
    fn validate_catches_unknown_crate_strict() {
        let config = make_config(vec![("core", vec![])], true, UnmatchedPolicy::Warn);
        let graph = make_graph(vec![("core", vec![]), ("api", vec!["core"])]);
        let result = validate(&config, &graph);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].crate_name, "api");
        assert!(matches!(result.errors[0].kind, ViolationKind::UnknownCrate));
    }

    #[test]
    fn validate_warns_unknown_crate_permissive() {
        let config = make_config(vec![("core", vec![])], false, UnmatchedPolicy::Warn);
        let graph = make_graph(vec![("core", vec![]), ("api", vec!["core"])]);
        let result = validate(&config, &graph);
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].crate_name, "api");
    }

    #[test]
    fn validate_catches_stale_config_entry_error_policy() {
        let config = make_config(
            vec![("core", vec![]), ("ghost", vec!["core"])],
            true,
            UnmatchedPolicy::Error,
        );
        let graph = make_graph(vec![("core", vec![])]);
        let result = validate(&config, &graph);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].crate_name, "ghost");
        assert!(matches!(
            result.errors[0].kind,
            ViolationKind::UnmatchedConfigEntry
        ));
    }

    #[test]
    fn validate_warns_stale_config_entry_warn_policy() {
        let config = make_config(
            vec![("core", vec![]), ("ghost", vec!["core"])],
            true,
            UnmatchedPolicy::Warn,
        );
        let graph = make_graph(vec![("core", vec![])]);
        let result = validate(&config, &graph);
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].crate_name, "ghost");
    }

    #[test]
    fn validate_ignores_stale_config_entry_ignore_policy() {
        let config = make_config(
            vec![("core", vec![]), ("ghost", vec!["core"])],
            true,
            UnmatchedPolicy::Ignore,
        );
        let graph = make_graph(vec![("core", vec![])]);
        let result = validate(&config, &graph);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn validate_handles_crate_with_zero_deps() {
        let config = make_config(vec![("core", vec![])], true, UnmatchedPolicy::Warn);
        let graph = make_graph(vec![("core", vec![])]);
        let result = validate(&config, &graph);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn validate_handles_multiple_violations() {
        let config = make_config(
            vec![("core", vec![]), ("api", vec!["core"])],
            true,
            UnmatchedPolicy::Warn,
        );
        // api depends on both server and db, neither allowed
        let graph = make_graph(vec![
            ("core", vec![]),
            ("api", vec!["core", "server", "db"]),
            ("server", vec![]),
            ("db", vec![]),
        ]);
        let result = validate(&config, &graph);
        // 2 forbidden deps for api + 2 unknown crates (server, db) in strict mode
        assert_eq!(result.errors.len(), 4);
    }
}

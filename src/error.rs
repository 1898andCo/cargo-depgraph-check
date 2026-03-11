use std::path::PathBuf;

/// Crate-level error type.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors produced by cargo-depgraph-check.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read config from {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse config from {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("failed to run cargo metadata: {source}")]
    Metadata { source: cargo_metadata::Error },

    #[error("cargo metadata did not return a resolved dependency graph")]
    NoResolve,

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

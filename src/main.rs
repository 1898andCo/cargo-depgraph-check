#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "cargo-depgraph-check",
    version,
    about = "Enforce workspace crate dependency graph rules via allowlist configuration",
    multicall = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate workspace dependencies against the allowlist config
    Check {
        /// Path to the workspace Cargo.toml
        #[arg(long)]
        manifest_path: Option<PathBuf>,

        /// Path to the dependency rules config file
        #[arg(long, default_value = "depgraph-rules.toml")]
        config: PathBuf,

        /// Output format
        #[arg(long, default_value = "text", value_enum)]
        format: OutputFormat,

        /// Color output control
        #[arg(long, default_value = "auto", value_enum)]
        color: ColorChoice,
    },

    /// Generate a baseline config from the current workspace's dependency graph
    Generate {
        /// Path to the workspace Cargo.toml
        #[arg(long)]
        manifest_path: Option<PathBuf>,

        /// Write config to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, ValueEnum)]
enum ColorChoice {
    Auto,
    Always,
    Never,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Check { .. }) => {
            // 1. Parse config from `config` path
            // 2. Run cargo metadata with optional `manifest_path`
            // 3. Validate graph against rules
            // 4. Report results in `format`, respecting `color`
            // 5. Return ExitCode::SUCCESS (0) or ExitCode::from(1) for violations
            todo!("check subcommand not yet implemented")
        }
        Some(Commands::Generate { .. }) => {
            // 1. Run cargo metadata with optional `manifest_path`
            // 2. Extract current workspace dependency graph
            // 3. Write baseline TOML config to `output` or stdout
            todo!("generate subcommand not yet implemented")
        }
        None => {
            // No subcommand — print help
            // This handles the multicall case where only "depgraph-check" is passed
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            ExitCode::from(2)
        }
    }
}

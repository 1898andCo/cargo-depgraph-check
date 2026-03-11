#![forbid(unsafe_code)]

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};

use cargo_depgraph_check::config::Config;
use cargo_depgraph_check::metadata::{WorkspaceGraph, generate_config};
use cargo_depgraph_check::report::{ColorMode, report_json, report_text};
use cargo_depgraph_check::validate::validate;

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
        Some(Commands::Check {
            manifest_path,
            config: config_path,
            format,
            color,
        }) => run_check(manifest_path, config_path, format, color),
        Some(Commands::Generate {
            manifest_path,
            output,
        }) => run_generate(manifest_path, output),
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            ExitCode::from(2)
        }
    }
}

fn run_check(
    manifest_path: Option<PathBuf>,
    config_path: PathBuf,
    format: OutputFormat,
    color: ColorChoice,
) -> ExitCode {
    let config = match Config::from_path(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return ExitCode::from(2);
        }
    };

    let graph = match WorkspaceGraph::from_metadata(
        manifest_path.as_deref(),
        config.options.check_dev_deps,
    ) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return ExitCode::from(2);
        }
    };

    let result = validate(&config, &graph);

    let color_str = match color {
        ColorChoice::Auto => "auto",
        ColorChoice::Always => "always",
        ColorChoice::Never => "never",
    };
    let color_mode = ColorMode::from_choice(color_str, std::io::stderr().is_terminal());

    let output = match format {
        OutputFormat::Text => report_text(&result.errors, &result.warnings, color_mode),
        OutputFormat::Json => report_json(&result.errors, &result.warnings),
    };

    if !output.is_empty() {
        eprint!("{output}");
    }

    if result.errors.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_generate(manifest_path: Option<PathBuf>, output: Option<PathBuf>) -> ExitCode {
    let graph = match WorkspaceGraph::from_metadata(manifest_path.as_deref(), false) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ERROR: {e}");
            return ExitCode::from(2);
        }
    };

    let config_text = generate_config(&graph);

    match output {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, &config_text) {
                eprintln!("ERROR: failed to write config to {}: {e}", path.display());
                return ExitCode::from(2);
            }
            eprintln!("Config written to {}", path.display());
        }
        None => {
            print!("{config_text}");
        }
    }

    ExitCode::SUCCESS
}

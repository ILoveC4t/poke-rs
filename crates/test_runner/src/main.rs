//! AI-friendly test summary runner.
//!
//! Runs `cargo test` and produces a structured JSON summary suitable for AI agents.
//! Results are saved to `.test_runs/` with versioning.
//!
//! Usage:
//!   cargo run -p test_runner -- run
//!   cargo run -p test_runner -- run --package poke_engine
//!   cargo run -p test_runner -- run --filter damage

mod cmd;
mod models;
mod utils;

use clap::{Parser, Subcommand};
use cmd::{analyze, run};

#[derive(Parser)]
#[command(name = "test_runner")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests and generate summary
    Run(run::RunArgs),

    /// Analyze regressions between runs
    Analyze(analyze::AnalyzeArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run(args)) => {
            run::execute(args);
        }
        Some(Commands::Analyze(args)) => {
            analyze::execute(args);
        }
        None => {
            // Require explicit subcommand to avoid flag ambiguity at the root.
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help().unwrap();
        }
    }
}

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
use cmd::{run, analyze};

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
            // Default to 'run' with empty args if no subcommand provided
            // This maintains backward compatibility for simple "cargo run -p test_runner" usage if needed,
            // though it's better to be explicit.
            // Let's print help if no command, or default to run?
            // The prompt asked for "proper extensible cli", so forcing subcommand is better practice.
            // But for ease of use let's default to run if user didn't specify.
            // Actually, if we use just `cargo run`, we might want to run everything.
            
            // However, clap will fail if it sees arguments it doesn't recognize as a subcommand.
            // So if user does `cargo run -- --filter foo`, clap will fail because --filter is a flag of `run`, NOT root.
            // To fix this we would need to flatten `RunArgs` into `Cli`.
            // But we want subcommands.
            
            // Let's force usage of `run`.
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help().unwrap();
        }
    }
}

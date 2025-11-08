mod commands;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lean-playground")]
#[command(about = "A multifunctional CLI tool for Lean playground", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// A todo command that panics
    Todo,
    /// Extract all theorem names from a Lean file
    AllTheoremNames {
        /// Path to the Lean file
        file: PathBuf,
    },
    /// Compare theorems starting with "prop:" between .lean and .Decl.lean files
    CompareTheorems {
        /// Directory containing the .lean and .Decl.lean files
        dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Todo => {
            todo!()
        }
        Commands::AllTheoremNames { file } => {
            commands::theorem_names::extract_and_print_theorem_names(&file);
        }
        Commands::CompareTheorems { dir } => {
            commands::compare_theorems::compare_theorem_files(&dir);
        }
    }
}

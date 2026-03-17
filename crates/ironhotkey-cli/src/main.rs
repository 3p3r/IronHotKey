use std::fs;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ironhotkey")]
#[command(about = "AHK v1 parser/codegen/runtime pipeline")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run { script: String },
    Parse { script: String },
    Codegen { script: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { script } => {
            let source = fs::read_to_string(script)?;
            let ast = ironhotkey_parser::parse(&source)?;
            let js = ironhotkey_codegen::codegen(&ast)?;
            ironhotkey_runtime::run(&js)?;
        }
        Commands::Parse { script } => {
            let source = fs::read_to_string(script)?;
            let ast = ironhotkey_parser::parse(&source)?;
            println!("{}", serde_json::to_string_pretty(&ast)?);
        }
        Commands::Codegen { script } => {
            let source = fs::read_to_string(script)?;
            let ast = ironhotkey_parser::parse(&source)?;
            let js = ironhotkey_codegen::codegen(&ast)?;
            println!("{js}");
        }
    }

    Ok(())
}

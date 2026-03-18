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
    Run {
        script: String,
    },
    Parse {
        script: String,
    },
    Codegen {
        script: String,
        #[arg(long, conflicts_with = "javascript")]
        typescript: bool,
        #[arg(long, conflicts_with = "typescript")]
        javascript: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { script } => {
            let source = fs::read_to_string(script)?;
            let ast = ironhotkey_parser::parse(&source)?;
            let ts = ironhotkey_codegen::codegen(&ast)?;
            let js = ironhotkey_codegen::transpile(&ts)?;
            ironhotkey_runtime::run(&js)?;
        }
        Commands::Parse { script } => {
            let source = fs::read_to_string(script)?;
            let ast = ironhotkey_parser::parse(&source)?;
            println!("{}", serde_json::to_string_pretty(&ast)?);
        }
        Commands::Codegen {
            script,
            typescript,
            javascript,
        } => {
            let source = fs::read_to_string(script)?;
            let ast = ironhotkey_parser::parse(&source)?;
            let ts = ironhotkey_codegen::codegen(&ast)?;
            if javascript {
                let js = ironhotkey_codegen::transpile(&ts)?;
                println!("{js}");
            } else {
                let _ = typescript;
                println!("{ts}");
            }
        }
    }

    Ok(())
}

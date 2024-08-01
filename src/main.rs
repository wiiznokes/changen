use core::str;
use std::process::Command;

use clap::{Parser, Subcommand};

mod commit_parser;
mod change_log;


#[derive(Parser)]
#[command(name = "changelog")]
#[command(about = "Changelog generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[arg(short, long, help = "Path to the output file")]
        output_file: Option<String>,
    },
    Release {
        #[arg(short, long, help = "Version number for the release")]
        version: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { output_file } => {
            
        }
        Commands::Release { version } => {
        }
    }
}

fn last_commit_title() -> String {
    let output = Command::new("git")
        .args(&["log", "-1", "--pretty=%s"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap()
}

#[cfg(test)]
mod test {

    #[test]
    fn a() {
    }
}

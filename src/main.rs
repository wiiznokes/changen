use core::str;

use clap::{Parser, Subcommand};

mod config;
mod note_generator;

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
        Commands::Generate { output_file } => {}
        Commands::Release { version } => {}
    }
}

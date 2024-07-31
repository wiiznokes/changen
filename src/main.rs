use core::str;
use std::process::Command;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "changelog")]
#[command(about = "Changelog generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates a changelog based on commit history
    Generate {
        #[arg(short, long, help = "Path to the output file")]
        output: Option<String>,
    },
    /// Creates a new release with the generated changelog
    Release {
        #[arg(short, long, help = "Version number for the release")]
        version: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { output } => {
            // Here you would implement the logic for generating the changelog
            match output {
                Some(path) => println!("Generating changelog and saving to: {}", path),
                None => println!("Generating changelog..."),
            }
            // Add your changelog generation logic here
        }
        Commands::Release { version } => {
            // Here you would implement the logic for creating a release
            println!("Creating release with version: {}", version);
            // Add your release logic here
        }
    }
}

fn get_commit_title() {
    // git log -1 --pretty=%s
    // or
    // git log -1 --pretty=%b

    let output = Command::new("git")
        .args(&["log", "-1", "--pretty=%s"])
        .output()
        .expect("Failed to execute git command");

    // Convert the output from bytes to a string
    let commit_title = str::from_utf8(&output.stdout)
        .expect("Failed to convert output to string")
        .trim(); // Trim any trailing newline characters

    // Print the commit title
    println!("Latest commit title: {}", commit_title);
    //
}

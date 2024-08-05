#![allow(unused_variables)]
#![allow(dead_code)]

use core::str;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use config::{CommitMessageParsing, GitProvider};

mod config;
mod note_generator;

#[derive(Parser)]
#[command(name = "changelog", about = "Changelog generator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate the release note of the last commit
    Generate {
        #[arg(
            short,
            long,
            help = "Path to the changelog file.",
            default_value = "CHANGELOG.md"
        )]
        file: Option<String>,
        #[arg(long, help = "Path to the commit type to changelog section map.")]
        map: Option<PathBuf>,
        #[arg(long, help = "Parsing of the commit message.", default_value_t = CommitMessageParsing::Smart)]
        parsing: CommitMessageParsing,
        #[arg(long, help = "Don't include unidentified commits.")]
        exclude_unidentified: bool,
        #[arg(long, help = "We use the Github api to map commit sha to PRs.", default_value_t = GitProvider::Github)]
        provider: GitProvider,
        #[arg(long, help = "Owner of the repo. Needed for Github integration.")]
        owner: Option<String>,
        #[arg(long, help = "Repo name. Needed for Github integration.")]
        repo: Option<String>,
        #[arg(long, help = "Omit the PR link from the output.")]
        omit_pr_link: bool,
        #[arg(long, help = "Omit contributors' acknowledgements/mention.")]
        omit_thanks: bool,
    },
    /// Generate a new release
    Release {
        #[arg(
            short,
            long,
            help = "Path to the changelog file.",
            default_value = "CHANGELOG.md"
        )]
        file: Option<String>,
        #[arg(short, long, help = "Version number for the release")]
        version: String,
        #[arg(long, help = "Ommit the commit history between releases.")]
        omit_diff: bool,
    },
    /// Validate a changelog syntax
    Validate {
        #[arg(
            short,
            long,
            help = "Path to the changelog file.",
            default_value = "CHANGELOG.md"
        )]
        file: Option<String>,
    },
    /// Show a specific release on stdout
    Show {
        #[arg(
            short,
            long,
            help = "Path to the changelog file.",
            default_value = "CHANGELOG.md"
        )]
        file: Option<String>,
        #[arg(
            short,
            help = "0 being unreleased, 1 is the last release",
            default_value_t = 1
        )]
        n: u32,
    },
    /// Create a new changelog file with an accepted syntax
    New {
        #[arg(
            short,
            long,
            help = "Path to the changelog file.",
            default_value = "CHANGELOG.md"
        )]
        file: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // match &cli.command {
    //     Commands::Generate { output_file } => {}
    //     Commands::Release { version } => {}
    // }
}

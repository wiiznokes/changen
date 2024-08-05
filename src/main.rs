#![allow(unused_variables)]
#![allow(dead_code)]

use core::str;
use std::{fs::File, io::Read, path::PathBuf};

use changelog::parse_changelog;
use clap::{Parser, Subcommand, ValueHint};
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
            default_value = "CHANGELOG.md",
            value_hint = ValueHint::FilePath,
            short_alias = 'o',
            alias = "output",
        )]
        file: Option<String>,
        #[arg(long, help = "Path to the commit type to changelog section map.", value_hint = ValueHint::FilePath)]
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
            default_value = "CHANGELOG.md",
            value_hint = ValueHint::FilePath,
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
            default_value = "CHANGELOG.md",
            value_hint = ValueHint::FilePath,
        )]
        file: Option<String>,
        #[arg(long, help = "Show the Abstract Syntax Tree.")]
        ast: bool,
    },
    /// Show a specific release on stdout
    Show {
        #[arg(
            short,
            long,
            help = "Path to the changelog file.",
            default_value = "CHANGELOG.md",
            value_hint = ValueHint::FilePath,
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
            default_value = "CHANGELOG.md",
            value_hint = ValueHint::FilePath,
        )]
        file: Option<String>,
    },
}

fn get_changelog_path(path: Option<String>) -> PathBuf {
    match path {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("CHANGELOG.md"),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            file,
            map,
            parsing,
            exclude_unidentified,
            provider,
            owner,
            repo,
            omit_pr_link,
            omit_thanks,
        } => todo!(),
        Commands::Release {
            file,
            version,
            omit_diff,
        } => todo!(),
        Commands::Validate { file, ast } => {
            let path = get_changelog_path(file);
            let mut file = File::open(path)?;
            let mut input = String::new();
            file.read_to_string(&mut input)?;

            let changelog = parse_changelog(&input)?;

            if ast {
                dbg!(&changelog);
            }

            println!("Changelog parser with success!");
        }
        Commands::Show { file, n } => todo!(),
        Commands::New { file } => todo!(),
    }

    Ok(())
}

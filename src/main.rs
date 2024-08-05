#![allow(unused_variables)]
#![allow(dead_code)]

use core::str;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Ok};
use changelog::{de::parse_changelog, ser::serialize_changelog, ChangeLog};
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
        #[arg(long, alias = "fmt", help = "Format the changelog.")]
        format: bool,
        #[arg(long, hide = true, help = "Show the Abstract Syntax Tree.")]
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
        n: usize,
        #[arg(short, long, help = "Specific version.")]
        version: Option<String>,
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
        path: Option<String>,
        #[arg(short, long, help = "Override of existing file.")]
        force: bool,
    },
}

fn get_changelog_path(path: Option<String>) -> PathBuf {
    match path {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("CHANGELOG.md"),
    }
}

fn read_file(path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    Ok(input)
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
        Commands::Validate { file, ast, format } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;

            if ast {
                dbg!(&changelog);
            }

            if format {
                let output = serialize_changelog(&changelog);
                let mut file = File::options().truncate(true).write(true).open(&path)?;
                file.write_all(output.as_bytes())?;
            }

            println!("Changelog parser with success!");
        }
        Commands::Show { file, n, version } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;

            let release = if let Some(ref version) = version {
                changelog.releases.get(version)
            } else {
                changelog.releases.get_index(n).map(|(_, r)| r)
            };

            match release {
                Some(release) => {
                    let mut output = String::new();
                    changelog::ser::serialize_release(&mut output, release);
                    println!("{}", output);
                }
                None => {
                    bail!("No release found");
                }
            };
        }
        Commands::New { path, force } => {
            let path = get_changelog_path(path);

            if path.exists() && !force {
                bail!("Path already exist. Delete it or use the --force option");
            }

            let changelog = ChangeLog::new();

            let output = serialize_changelog(&changelog);

            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path)?;

            file.write_all(output.as_bytes())?;

            println!("Changelog successfully created!");
        }
    }

    Ok(())
}

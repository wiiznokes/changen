use core::str;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Ok};
use changelog::{
    de::parse_changelog,
    ser::{serialize_changelog, Options},
    ChangeLog, ReleaseSection,
};
use clap::{Parser, Subcommand, ValueHint};
use config::{CommitMessageParsing, Config, GitProvider};
use note_generator::get_release_note;

mod commit_parser;
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
        file: Option<PathBuf>,
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
        file: Option<PathBuf>,
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
        file: Option<PathBuf>,
        #[arg(long, alias = "fmt", help = "Format the changelog.")]
        format: bool,
        #[arg(long, help = "Path to the commit type to changelog section map.", value_hint = ValueHint::FilePath)]
        map: Option<PathBuf>,
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
        file: Option<PathBuf>,
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
        path: Option<PathBuf>,
        #[arg(short, long, help = "Override of existing file.")]
        force: bool,
    },
}

fn get_changelog_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or(PathBuf::from("CHANGELOG.md"))
}

fn read_file(path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut input = String::new();
    file.read_to_string(&mut input)?;
    Ok(input)
}

fn get_config(path: Option<PathBuf>) -> anyhow::Result<Config> {
    match path {
        Some(path) => {
            let mut file = File::open(&path)?;

            let mut content = Vec::new();

            file.read_to_end(&mut content)?;

            let map = serde_json::de::from_slice(&content)?;
            Ok(map)
        }
        None => Ok(Config::default()),
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
        } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let mut changelog = parse_changelog(&input)?;

            let (_, unreleased) = changelog.releases.get_index_mut(0).expect("no release");

            let config = get_config(map)?;

            let Some((section, release_note)) = get_release_note(
                &parsing,
                exclude_unidentified,
                &provider,
                &owner,
                &repo,
                omit_pr_link,
                omit_thanks,
                &config.map,
            )?
            else {
                return Ok(());
            };

            let section = if let Some(section) = unreleased.note_sections.get_mut(&section) {
                section
            } else {
                let release_section = ReleaseSection {
                    title: section.clone(),
                    notes: vec![],
                };

                unreleased
                    .note_sections
                    .insert(section.clone(), release_section);
                unreleased.note_sections.get_mut(&section).unwrap()
            };

            section.notes.push(release_note);

            let output = serialize_changelog(&changelog, &config.into_changelog_ser_options());
            let mut file = File::options().truncate(true).write(true).open(&path)?;
            file.write_all(output.as_bytes())?;
        }
        #[allow(unused_variables)]
        Commands::Release {
            file,
            version,
            omit_diff,
        } => todo!(),
        Commands::Validate {
            file,
            ast,
            format,
            map,
        } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;

            if ast {
                dbg!(&changelog);
            }

            if format {
                let options = get_config(map)?.into_changelog_ser_options();

                let output = serialize_changelog(&changelog, &options);
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
                    changelog::ser::serialize_release(&mut output, release, &Options::default());
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

            let output = serialize_changelog(&changelog, &Options::default());

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

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{collections::HashSet, fmt::Display};

use clap::{arg, Args, Parser, Subcommand, ValueHint};

use changelog::ser::{ChangeLogSerOption, ChangeLogSerOptionRelease};
use clap::ValueEnum;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::git_provider::GitProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMessageToSection(pub IndexMap<String, HashSet<String>>);

impl Default for MapMessageToSection {
    fn default() -> Self {
        let map = include_str!("../res/map_commit_type_to_section.json");
        serde_json::de::from_str(map).unwrap()
    }
}

impl MapMessageToSection {
    pub fn into_changelog_ser_options(self) -> ChangeLogSerOption {
        ChangeLogSerOption {
            release_option: ChangeLogSerOptionRelease {
                section_order: self.0.into_iter().map(|(section, _)| section).collect(),
                ..Default::default()
            },
        }
    }

    pub fn map_section(&self, section: &str) -> Option<String> {
        let section_normalized = section.to_lowercase();

        for (section, needles) in &self.0 {
            for needle in needles {
                let needle_normalized = needle.to_lowercase();

                if section_normalized == needle_normalized {
                    return Some(section.to_owned());
                }
            }
        }

        None
    }

    /// Best effort recognition
    pub fn try_find_section(&self, (message, desc): (&str, &str)) -> Option<String> {
        let message_normalized = message.to_lowercase();
        let desc_normalized = desc.to_lowercase();

        for (section, needles) in &self.0 {
            for needle in needles {
                let needle_normalized = needle.to_lowercase();

                if message_normalized.contains(&needle_normalized) {
                    return Some(section.to_owned());
                }
                if desc_normalized.contains(&needle_normalized) {
                    return Some(section.to_owned());
                }
            }
        }

        None
    }

    pub fn try_new<P: AsRef<Path>>(path: Option<P>) -> anyhow::Result<MapMessageToSection> {
        match path {
            Some(path) => {
                let mut file = File::open(&path)?;

                let mut content = Vec::new();

                file.read_to_end(&mut content)?;

                let map = serde_json::de::from_slice(&content)?;
                Ok(map)
            }
            None => Ok(MapMessageToSection::default()),
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommitMessageParsing {
    #[default]
    Smart,
    Strict,
}

// todo: use derive_more::Display when this issue is resolved
// https://github.com/JelteF/derive_more/issues/216
impl Display for CommitMessageParsing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitMessageParsing::Smart => write!(f, "smart"),
            CommitMessageParsing::Strict => write!(f, "strict"),
        }
    }
}

#[derive(Parser)]
#[command(name = "changelog", about = "Changelog generator", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(alias = "gen")]
    Generate(Generate),
    Release(Release),
    Validate(Validate),
    Show(Show),
    New(New),
}

/// Generate release notes. By default, generate from the last release in the changelog to HEAD.
#[derive(Args)]
pub struct Generate {
    #[arg(
        short,
        long,
        help = "Path to the changelog file.",
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
        short_alias = 'o',
        alias = "output",
    )]
    pub file: Option<PathBuf>,
    #[arg(long, help = "Path to the commit type to changelog section map.", value_hint = ValueHint::FilePath)]
    pub map: Option<PathBuf>,
    #[arg(long, help = "Parsing of the commit message.", default_value_t)]
    pub parsing: CommitMessageParsing,
    #[arg(long, help = "Don't include unidentified commits.")]
    pub exclude_unidentified: bool,
    #[arg(
        long,
        help = "Don't include commits which are not attached to a pull request."
    )]
    pub exclude_not_pr: bool,
    #[arg(
        long,
        help = "We use the Github api to map commit sha to PRs.",
        default_value_t
    )]
    pub provider: GitProvider,
    #[arg(
        long,
        help = "Needed for fetching PRs. Example: 'wiiznokes/changelog-generator'. Already defined for you in Github Actions."
    )]
    pub repo: Option<String>,
    #[arg(long, help = "Omit the PR link from the output.")]
    pub omit_pr_link: bool,
    #[arg(long, help = "Omit contributors' acknowledgements/mention.")]
    pub omit_thanks: bool,
    #[arg(long, help = "Print the result on the standard output.")]
    pub stdout: bool,
    #[arg(
        long,
        help = "Generate only this commit, or tag.",
        conflicts_with_all = ["milestone", "since", "until"],
    )]
    pub specific: Option<String>,
    #[arg(
        long,
        help = "Include all commits of this milestone",
        conflicts_with_all = ["since", "until"],
    )]
    pub milestone: Option<String>,
    #[arg(long, help = "Include all commits in \"since..until\".")]
    pub since: Option<String>,
    #[arg(
        long,
        help = "Include all commits in \"since..until\".",
        requires = "since"
    )]
    pub until: Option<String>,
}

/// Generate a new release. By default, use the last tag present in the repo, sorted using the [semver](https://semver.org/) format.
#[derive(Args)]
pub struct Release {
    #[arg(
        short,
        long,
        help = "Path to the changelog file.",
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub file: Option<PathBuf>,
    #[arg(
        short,
        long,
        help = "Version number for the release. If omitted, use the last tag present in the repo.",
        num_args(0..=1),
        default_missing_value=None
    )]
    pub version: Option<String>,
    #[arg(long, help = "Previous version number. Used for the diff.")]
    pub previous_version: Option<String>,
    #[arg(
        long,
        help = "We use the Github link to produce the tags diff",
        default_value_t
    )]
    pub provider: GitProvider,
    #[arg(
        long,
        help = "Needed for the tags diff PRs. Example: 'wiiznokes/changelog-generator'. Already defined for you in Github Actions."
    )]
    pub repo: Option<String>,
    #[arg(long, help = "Omit the commit history between releases.")]
    pub omit_diff: bool,
    #[arg(
        long,
        help = "Override the last release if exist, by replacing all the existing release notes."
    )]
    pub force: bool,
    #[arg(long, help = "Print the result on the standard output.")]
    pub stdout: bool,
}

/// Validate a changelog syntax
#[derive(Args)]
pub struct Validate {
    #[arg(
        short,
        long,
        help = "Path to the changelog file.",
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub file: Option<PathBuf>,
    #[arg(long, alias = "fmt", help = "Format the changelog.")]
    pub format: bool,
    #[arg(long, help = "Path to the commit type to changelog section map.", value_hint = ValueHint::FilePath)]
    pub map: Option<PathBuf>,
    #[arg(long, help = "Show the Abstract Syntax Tree.")]
    pub ast: bool,
    #[arg(long, help = "Print the result on the standard output.")]
    pub stdout: bool,
}

/// Show a specific release on stdout
#[derive(Args)]
pub struct Show {
    #[arg(
        short,
        long,
        help = "Path to the changelog file.",
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub file: Option<PathBuf>,
    #[arg(
        short,
        help = "0 being unreleased, 1 is the last release",
        default_value_t = 1,
        conflicts_with = "version"
    )]
    pub n: usize,
    #[arg(short, long, help = "Specific version.")]
    pub version: Option<String>,
}
/// Create a new changelog file with an accepted syntax
#[derive(Args)]
pub struct New {
    #[arg(
        short,
        long,
        help = "Path to the changelog file.",
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub path: Option<PathBuf>,
    #[arg(short, long, help = "Override of existing file.")]
    pub force: bool,
}

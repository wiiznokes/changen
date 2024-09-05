use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{collections::HashSet, fmt::Display};

use changelog::fmt::SortOptions;
use clap::{arg, Args, Parser, Subcommand, ValueHint};

use changelog::ser::{Options, OptionsRelease};
use changelog::Version;
use clap::ValueEnum;
use indexmap::IndexMap;
use regex::Regex;
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
    pub fn to_fmt_options(self) -> changelog::fmt::Options {
        changelog::fmt::Options {
            sort_options: SortOptions {
                section_order: self.0.into_iter().map(|(section, _)| section).collect(),
                ..Default::default()
            },
        }
    }
    pub fn into_changelog_ser_options(self) -> Options {
        Options {
            release_option: OptionsRelease {
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

#[derive(ValueEnum, Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum MergeDevVersions {
    /// Yes if the version is stable, no otherwise
    #[default]
    Auto,
    No,
    Yes,
}

impl Display for MergeDevVersions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeDevVersions::Auto => write!(f, "auto"),
            MergeDevVersions::No => write!(f, "no"),
            MergeDevVersions::Yes => write!(f, "yes"),
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(version, about = "Changelog generator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    New(New),
    Validate(Validate),
    #[command(alias = "gen")]
    Generate(Generate),
    Release(Release),
    Show(Show),
    #[command(aliases = ["delete", "rm"])]
    Remove(Remove),
}

/// Generate release notes. By default, generate from the last release in the changelog to HEAD.
#[derive(Debug, Clone, Args)]
pub struct Generate {
    /// Path to the changelog file.
    #[arg(
        short,
        long,
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
        short_alias = 'o',
        alias = "output",
    )]
    pub file: Option<PathBuf>,
    /// Path to the commit type to changelog section map.
    #[arg(long, value_hint = ValueHint::FilePath)]
    pub map: Option<PathBuf>,
    /// Parsing of the commit message.
    #[arg(long, default_value_t)]
    pub parsing: CommitMessageParsing,
    /// Don't include unidentified commits.
    #[arg(long)]
    pub exclude_unidentified: bool,
    /// Don't include commits which are not attached to a pull request.
    #[arg(long)]
    pub exclude_not_pr: bool,
    /// We use the Github api to map commit sha to PRs.
    #[arg(long, default_value_t)]
    pub provider: GitProvider,
    /// Needed for fetching PRs. Example: 'wiiznokes/changen'. Already defined for you in Github Actions.
    #[arg(long)]
    pub repo: Option<String>,
    /// Omit the PR link from the output.
    #[arg(long)]
    pub omit_pr_link: bool,
    /// Omit contributors' acknowledgements/mention.
    #[arg(long)]
    pub omit_thanks: bool,
    /// Print the result on the standard output.
    #[arg(long)]
    pub stdout: bool,
    /// Generate only this commit, or tag.
    #[arg(
        long,
        conflicts_with_all = ["milestone", "since", "until"],
    )]
    pub specific: Option<String>,
    /// Include all commits of this milestone.
    #[arg(
        long,
        conflicts_with_all = ["since", "until"],
    )]
    pub milestone: Option<String>,
    /// Include all commits in \"since..until\".
    #[arg(long)]
    pub since: Option<String>,
    /// Include all commits in \"since..until\".
    #[arg(long, requires = "since")]
    pub until: Option<String>,
}

/// Generate a new release. By default, use the last tag present in the repo.
#[derive(Debug, Clone, Args)]
pub struct Release {
    /// Path to the changelog file.
    #[arg(
        short,
        long,
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub file: Option<PathBuf>,
    /// Version number for the release. If omitted, use the last tag present in the repo.
    #[arg(
        short,
        long,
        num_args(0..=1),
        default_missing_value=None
    )]
    pub version: Option<Version>,
    /// Previous version number. Used for the diff.
    #[arg(long)]
    pub previous_version: Option<Version>,
    /// We use the Github link to produce the tags diff.
    #[arg(long, default_value_t)]
    pub provider: GitProvider,
    /// Needed for the tags diff PRs. Example: 'wiiznokes/changen'. Already defined for you in Github Actions.
    #[arg(long)]
    pub repo: Option<String>,
    /// Omit the commit history between releases.
    #[arg(long)]
    pub omit_diff: bool,
    /// Override the release with the same version if it exist, by replacing all the existing release notes.
    #[arg(long)]
    pub force: bool,
    /// Add this text as a header of the release. If a header already exist, it will be inserted before the existing one.
    #[arg(long)]
    pub header: Option<String>,
    /// Merge older dev version into this new release
    #[arg(long, default_value_t)]
    pub merge_dev_versions: MergeDevVersions,
    /// Print the result on the standard output.
    #[arg(long)]
    pub stdout: bool,
}

/// Validate a changelog syntax
#[derive(Debug, Clone, Args)]
pub struct Validate {
    /// Path to the changelog file.
    #[arg(
        short,
        long,
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub file: Option<PathBuf>,
    /// Format the changelog.
    #[arg(long, alias = "fmt")]
    pub format: bool,
    /// Path to the commit type to changelog section map.
    #[arg(long, value_hint = ValueHint::FilePath)]
    pub map: Option<PathBuf>,
    /// Show the Abstract Syntax Tree.
    #[arg(long)]
    pub ast: bool,
    /// Print the result on the standard output.
    #[arg(long)]
    pub stdout: bool,
}

/// Show a releases on stdout. By default, show the last release.
#[derive(Debug, Clone, Args)]
pub struct Show {
    /// Path to the changelog file.
    #[arg(
        short,
        long,
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub file: Option<PathBuf>,
    /// -1 being unreleased, 0 the last release, ...
    #[arg(
        short,
        default_value_t = 0,
        conflicts_with = "version",
        allow_hyphen_values = true
    )]
    pub n: i32,
    /// Show a specific version. Also accept regex. Example: 1.0.0-*
    #[arg(
        short,
        long,
        num_args(0..=1),
        default_missing_value=None
    )]
    pub version: Option<Regex>,
}
/// Create a new changelog file with an accepted syntax
#[derive(Debug, Clone, Args)]
pub struct New {
    /// Path to the changelog file.
    #[arg(
        short,
        long,
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub path: Option<PathBuf>,
    /// Override of existing file.
    #[arg(short, long)]
    pub force: bool,
}

/// Remove a release
#[derive(Debug, Clone, Args)]
pub struct Remove {
    /// Path to the changelog file.
    #[arg(
        short,
        long,
        default_value = "CHANGELOG.md",
        value_hint = ValueHint::FilePath,
    )]
    pub file: Option<PathBuf>,
    /// Print the result on the standard output.
    #[arg(long)]
    pub stdout: bool,

    #[clap(flatten)]
    pub remove_id: RemoveSelection,
}

// fixme: move this to an enum https://github.com/clap-rs/clap/issues/2621
#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
pub struct RemoveSelection {
    /// -1 being unreleased, 0 the last release, ...
    #[arg(short, conflicts_with = "version", allow_hyphen_values = true)]
    pub n: Option<i32>,
    /// Remove a specific version. Also accept regex. Example: 1.0.0-*
    #[arg(short, long)]
    pub version: Option<Regex>,
}

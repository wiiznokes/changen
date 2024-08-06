use core::str;
use std::{
    fs::{File, OpenOptions},
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::bail;
use changelog::{
    de::parse_changelog,
    ser::{serialize_changelog, serialize_release, Options},
    Release, ReleaseSection, ReleaseTitle,
};
use clap::{Parser, Subcommand, ValueHint};
use config::{CommitMessageParsing, Config};
use git_helpers_function::{tags_list, try_get_repo};
use git_provider::{DiffTags, GitProvider};
use indexmap::IndexMap;
use note_generator::get_release_note;

#[macro_use]
extern crate log;

mod commit_parser;
mod config;
mod git_helpers_function;
mod git_provider;
mod note_generator;

#[cfg(test)]
mod test;

const UNRELEASED: &str = "Unreleased";

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
        #[arg(
            long,
            help = "Needed for fetching PRs. Example: 'wiiznokes/changelog-generator'. Already defined for you in Github Actions."
        )]
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
        #[arg(
            short,
            long,
            help = "Version number for the release. If ommited, use the last tag using \"git\" (ommiting the 'v' perfix)."
        )]
        version: Option<String>,
        #[arg(long, help = "We use the Github link to produce the tags diff", default_value_t = GitProvider::Github)]
        provider: GitProvider,
        #[arg(
            long,
            help = "Needed for the tags diff PRs. Example: 'wiiznokes/changelog-generator'. Already defined for you in Github Actions."
        )]
        repo: Option<String>,
        #[arg(long, help = "Omit the commit history between releases.")]
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
    let mut buf = String::new();

    if !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut buf)?;
    } else {
        let mut file = File::open(path)?;
        file.read_to_string(&mut buf)?;
    }

    Ok(buf)
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
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            file,
            map,
            parsing,
            exclude_unidentified,
            provider,
            repo,
            omit_pr_link,
            omit_thanks,
        } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let mut changelog = parse_changelog(&input)?;

            debug!("path: {}", path.display());
            debug!("input: {}", input);
            debug!("changelog: {:?}", changelog);

            let (_, unreleased) = changelog.releases.get_index_mut(0).expect("no release");

            let config = get_config(map)?;

            let Some((section, release_note)) = get_release_note(
                path.to_string_lossy().to_string(),
                &parsing,
                exclude_unidentified,
                &provider,
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

            if !io::stdout().is_terminal() {
                print!("{output}")
            } else {
                let mut file = File::options().truncate(true).write(true).open(&path)?;
                file.write_all(output.as_bytes())?;
            }
        }
        #[allow(unused_variables)]
        Commands::Release {
            file,
            version,
            provider,
            repo,
            omit_diff,
        } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let mut changelog = parse_changelog(&input)?;

            debug!("changelog: {:?}", changelog);

            let version = match version {
                Some(version) => {
                    if version.starts_with('v') {
                        bail!("Error: You shound't include the v prefix in the version.")
                    }
                    version
                }
                None => {
                    if let Some(tag) = tags_list().pop_back() {
                        match tag.strip_prefix('v') {
                            Some(version) => version.to_owned(),
                            None => tag,
                        }
                    } else {
                        bail!("No version provided. Can't fall back to last tag because there is none.");
                    }
                }
            };

            if changelog.releases.get(&version).is_some() {
                bail!("Version {} already exist", version);
            };

            let Some(mut prev_unreleased) = changelog.releases.shift_remove(UNRELEASED) else {
                bail!("No Unreleased section found.")
            };

            prev_unreleased.title.version = version.clone();

            if !omit_diff {
                let link = if let Some(repo) = try_get_repo(repo) {
                    let mut tags = tags_list();

                    match tags.pop_back() {
                        Some(current) => {
                            let prev = tags.pop_back();

                            let diff_tags = DiffTags { prev, current };

                            match provider.diff_link(&repo, &diff_tags) {
                                Ok(link) => Some(link),
                                Err(e) => {
                                    eprintln!("{e}");
                                    None
                                }
                            }
                        }
                        None => {
                            eprintln!("No tags defined. Can't produce the diff");
                            None
                        }
                    }
                } else {
                    None
                };

                if let Some(link) = link {
                    let line = format!("Full Changelog: {link}\n");

                    match &mut prev_unreleased.footer {
                        Some(footer) => {
                            footer.push_str("\n\n");
                            footer.push_str(&line);
                        }
                        None => {
                            prev_unreleased.footer = Some(line);
                        }
                    }
                    let footer = prev_unreleased.footer.clone().unwrap_or_default();
                }
            }

            changelog.releases.insert(version, prev_unreleased);

            let new_unreleased = Release {
                title: ReleaseTitle {
                    version: UNRELEASED.into(),
                    title: None,
                },
                header: None,
                note_sections: IndexMap::new(),
                footer: None,
            };

            changelog.releases.insert(UNRELEASED.into(), new_unreleased);

            let output = serialize_changelog(&changelog, &Options::default());

            if !io::stdout().is_terminal() {
                print!("{output}")
            } else {
                let mut file = File::options().truncate(true).write(true).open(&path)?;
                file.write_all(output.as_bytes())?;
            }
        }
        Commands::Validate {
            file,
            ast,
            format,
            map,
        } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;

            debug!("changelog: {:?}", changelog);

            if ast {
                dbg!(&changelog);
            }

            if format {
                let options = get_config(map)?.into_changelog_ser_options();

                let output = serialize_changelog(&changelog, &options);

                if !io::stdout().is_terminal() {
                    print!("{output}")
                } else {
                    let mut file = File::options().truncate(true).write(true).open(&path)?;
                    file.write_all(output.as_bytes())?;
                }
            }

            eprintln!("Changelog parser with success!");
        }
        Commands::Show { file, n, version } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;

            debug!("changelog: {:?}", changelog);

            let release = if let Some(ref version) = version {
                changelog.releases.get(version)
            } else {
                changelog.releases.get_index(n).map(|(_, r)| r)
            };

            match release {
                Some(release) => {
                    let mut output = String::new();
                    serialize_release(&mut output, release, &Options::default());
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

            let changelog = include_str!("../res/CHANGELOG_DEFAULT.md");

            let mut file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path)?;

            file.write_all(changelog.as_bytes())?;

            println!("Changelog successfully created!");
        }
    }

    Ok(())
}

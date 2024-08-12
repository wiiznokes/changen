use core::str;
use std::{
    fs::{File, OpenOptions},
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::bail;
use changelog::{
    de::parse_changelog,
    ser::{
        serialize_changelog, serialize_release, serialize_release_section_note,
        ChangeLogSerOptionRelease,
    },
    ReleaseSection,
};
use clap::{Parser, Subcommand, ValueHint};
use config::{CommitMessageParsing, Config};
use git_provider::GitProvider;
use release_note_generation::{get_release_note, GenerateReleaseNoteOptions};

#[macro_use]
extern crate log;

mod commit_parser;
mod config;
mod git_helpers_function;
mod git_provider;
mod release_generation;
mod release_note_generation;

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
        #[arg(long, help = "Parsing of the commit message.", default_value_t)]
        parsing: CommitMessageParsing,
        #[arg(long, help = "Don't include unidentified commits.")]
        exclude_unidentified: bool,
        #[arg(
            long,
            help = "Don't include commits which are not attached to a pull request."
        )]
        exclude_not_pr: bool,
        #[arg(
            long,
            help = "We use the Github api to map commit sha to PRs.",
            default_value_t
        )]
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
        #[arg(long, help = "Print the result on the standard output.")]
        stdout: bool,
        #[arg(
            long,
            help = "Include all commits of this milestone",
            conflicts_with = "tags"
        )]
        milestone: Option<String>,
        #[arg(
            long,
            help = "Include all commits between this tags. Ex: \"v1.0.1..HEAD\".",
            conflicts_with = "milestone"
        )]
        tags: Option<String>,
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
            help = "Version number for the release. If omitted, use the last tag using \"git\" (omitting the 'v' prefix)."
        )]
        version: Option<String>,
        #[arg(
            long,
            help = "We use the Github link to produce the tags diff",
            default_value_t
        )]
        provider: GitProvider,
        #[arg(
            long,
            help = "Needed for the tags diff PRs. Example: 'wiiznokes/changelog-generator'. Already defined for you in Github Actions."
        )]
        repo: Option<String>,
        #[arg(long, help = "Omit the commit history between releases.")]
        omit_diff: bool,
        #[arg(long, help = "Print the result on the standard output.")]
        stdout: bool,
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
        #[arg(long, help = "Print the result on the standard output.")]
        stdout: bool,
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

    let mut from_stdin = !io::stdin().is_terminal();

    if from_stdin {
        io::stdin().read_to_string(&mut buf)?;

        if buf.is_empty() {
            info!("Read stdin because is was not a terminal, but it is empty. Fallback to file.");
            from_stdin = false;
        }
    }

    if !from_stdin {
        let mut file = File::open(path)?;
        file.read_to_string(&mut buf)?;
    }

    Ok(buf)
}

fn write_output(output: &str, path: &Path, stdout: bool) -> anyhow::Result<()> {
    // !io::stdout().is_terminal()
    // won't work on Github action because stdout is piped somehow.
    if stdout {
        print!("{output}")
    } else {
        let mut file = File::options().truncate(true).write(true).open(path)?;
        file.write_all(output.as_bytes())?;
    }

    Ok(())
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
            exclude_not_pr,
            provider,
            repo,
            omit_pr_link,
            omit_thanks,
            stdout,
            milestone,
            tags,
        } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let mut changelog = parse_changelog(&input)?;

            debug!("is terminal: {}", io::stdin().is_terminal());
            debug!("is terminal stdout: {}", io::stdout().is_terminal());
            debug!("path: {}", path.display());
            debug!("input: {}", input);
            debug!("changelog: {:?}", changelog);

            let (_, unreleased) = changelog.releases.get_index_mut(0).expect("no release");

            let config = get_config(map)?;

            let Some((section_title, release_note)) =
                get_release_note(GenerateReleaseNoteOptions {
                    changelog_path: path.to_string_lossy().to_string(),
                    parsing,
                    exclude_unidentified,
                    exclude_not_pr,
                    provider,
                    repo,
                    omit_pr_link,
                    omit_thanks,
                    map: &config.map,
                })?
            else {
                return Ok(());
            };

            let section = if let Some(section) = unreleased.note_sections.get_mut(&section_title) {
                section
            } else {
                let release_section = ReleaseSection {
                    title: section_title.clone(),
                    notes: vec![],
                };

                unreleased
                    .note_sections
                    .insert(section_title.clone(), release_section);
                unreleased.note_sections.get_mut(&section_title).unwrap()
            };

            section.notes.push(release_note.clone());

            let output = serialize_changelog(&changelog, &config.into_changelog_ser_options());

            write_output(&output, &path, stdout)?;

            let mut added = String::new();
            serialize_release_section_note(&mut added, &release_note);
            eprintln!("Release note:\n{added}succefully added in the {section_title} section.",)
        }
        #[allow(unused_variables)]
        Commands::Release {
            file,
            version,
            provider,
            repo,
            omit_diff,
            stdout,
        } => {
            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;

            let (version, output) =
                release_generation::release(changelog, version, provider, repo, omit_diff)?;

            write_output(&output, &path, stdout)?;

            eprintln!("New release {} succefully created.", version);
        }
        Commands::Validate {
            file,
            ast,
            format,
            map,
            stdout,
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

                write_output(&output, &path, stdout)?;
            }

            eprintln!("Changelog parsed with success!");
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
                    debug!("show release: {:?}", release);

                    let mut output = String::new();
                    serialize_release(
                        &mut output,
                        release,
                        &ChangeLogSerOptionRelease {
                            section_order: vec![],
                            serialise_title: false,
                        },
                    );
                    print!("{}", output);
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

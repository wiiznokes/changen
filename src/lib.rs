use core::str;
use std::{
    fs::{File, OpenOptions},
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::bail;
use changelog::{
    de::parse_changelog,
    ser::{serialize_changelog, serialize_release, ChangeLogSerOptionRelease},
};
use config::{Cli, Commands, MapMessageToSection, New, Show, Validate};
use git_helpers_function::try_get_repo;
use release_note_generation::gen_release_notes;

#[macro_use]
extern crate log;

mod commit_parser;
pub mod config;
mod git_helpers_function;
mod git_provider;
mod release_generation;
mod release_note_generation;
mod utils;

#[cfg(test)]
mod test;

const UNRELEASED: &str = "Unreleased";

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

pub fn run(cli: Cli) -> anyhow::Result<()> {
    debug!("is terminal: {}", io::stdin().is_terminal());
    debug!("is terminal stdout: {}", io::stdout().is_terminal());

    match cli.command {
        Commands::Generate(mut options) => {
            let path = get_changelog_path(options.file.clone());
            let input = read_file(&path)?;
            let mut changelog = parse_changelog(&input)?;

            options.repo = try_get_repo(options.repo);

            debug!("path: {}", path.display());
            debug!("input: {}", input);
            debug!("changelog: {:?}", changelog);

            let map = MapMessageToSection::try_new(options.map.as_ref())?;

            let changelog_cloned = changelog.clone();

            let (_, unreleased) = changelog.releases.get_index_mut(0).expect("no release");

            gen_release_notes(
                &changelog_cloned,
                unreleased,
                path.to_string_lossy().to_string(),
                &map,
                &options,
            )?;

            let output = serialize_changelog(&changelog, &map.into_changelog_ser_options());

            write_output(&output, &path, options.stdout)?;
        }

        Commands::Release(mut options) => {
            let path = get_changelog_path(options.file.clone());
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;
            options.repo = try_get_repo(options.repo);

            let (version, output) = release_generation::release(changelog, &options)?;

            write_output(&output, &path, options.stdout)?;

            eprintln!("New release {} succefully created.", version);
        }

        Commands::Validate(options) => {
            let Validate {
                file,
                format,
                map,
                ast,
                stdout,
            } = options;

            let path = get_changelog_path(file);
            let input = read_file(&path)?;
            let changelog = parse_changelog(&input)?;

            debug!("changelog: {:?}", changelog);

            if ast {
                dbg!(&changelog);
            }

            if format {
                let ser_options = MapMessageToSection::try_new(map)?.into_changelog_ser_options();

                let output = serialize_changelog(&changelog, &ser_options);

                write_output(&output, &path, stdout)?;
            }

            eprintln!("Changelog parsed with success!");
        }

        Commands::Show(options) => {
            let Show { file, n, version } = options;

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
                            serialize_title: false,
                        },
                    );
                    print!("{}", output);
                }
                None => {
                    bail!("No release found");
                }
            };
        }

        Commands::New(options) => {
            let New { path, force } = options;

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

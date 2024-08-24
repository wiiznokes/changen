use pretty_assertions::assert_eq;
use std::{fs::OpenOptions, io::Write};

use changelog::{
    de::parse_changelog,
    ser::{serialize_changelog, Options},
};

use crate::{
    config::{Cli, Generate, MapMessageToSection},
    repository::FsTest,
    run_generic,
};

#[test]
fn validate_default_changelog() {
    let input = include_str!("../res/CHANGELOG_DEFAULT.md");

    let changelog = parse_changelog(input).unwrap();

    let output = serialize_changelog(&changelog, &Options::default());

    assert_eq!(input, output);
}

#[test]
fn validate_default_map() {
    let map = include_str!("../res/map_commit_type_to_section.json");
    serde_json::de::from_str::<MapMessageToSection>(map).unwrap();
}

#[test]
fn format_default_changelog() {
    let input = include_str!("../res/CHANGELOG_DEFAULT.md");

    let changelog = parse_changelog(input).unwrap();

    let output = serialize_changelog(&changelog, &Options::default());

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("./res/CHANGELOG_DEFAULT.md")
        .unwrap();

    file.write_all(output.as_bytes()).unwrap();
}

#[test]
fn test_repo() {
    let cli = Cli {
        command: crate::config::Commands::Generate(Generate {
            file: todo!(),
            map: todo!(),
            parsing: todo!(),
            exclude_unidentified: todo!(),
            exclude_not_pr: todo!(),
            provider: todo!(),
            repo: todo!(),
            omit_pr_link: todo!(),
            omit_thanks: todo!(),
            stdout: todo!(),
            specific: todo!(),
            milestone: todo!(),
            since: todo!(),
            until: todo!(),
        }),
    };

    run_generic::<FsTest>(cli).unwrap();
}

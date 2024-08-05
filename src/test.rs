use std::{fs::OpenOptions, io::Write};

use changelog::{
    de::parse_changelog,
    ser::{serialize_changelog, Options},
};

use crate::config::Config;

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
    serde_json::de::from_str::<Config>(map).unwrap();
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

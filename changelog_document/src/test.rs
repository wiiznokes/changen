use std::{fs::File, io::Read, sync::LazyLock};

use pretty_assertions::assert_eq;

use crate::*;
use de::parse_changelog;
use ser::OptionsRelease;

#[test]
fn test_file() {
    let mut file = File::open("../tests/changelogs/CHANGELOG3.md").unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let changelog = parse_changelog(&input).unwrap();
    dbg!(&changelog);
}

#[test]
fn perf() {
    let mut file = File::open("../tests/changelogs/CHANGELOG2.md").unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let now = std::time::Instant::now();
    let _ = parse_changelog(&input).unwrap();
    println!("{:?}", now.elapsed());
}

pub static CHANGELOG1: LazyLock<ChangeLog> = LazyLock::new(|| ChangeLog {
    header: Some("## Changelog\n\nHello, this is a changelog that follow semver!".into()),
    releases: {
        let mut releases = IndexMap::new();

        let version = String::from("Unreleased");
        releases.insert(
            version.clone(),
            Release {
                title: ReleaseTitle {
                    version,
                    title: Some("i'm am the title of the night".into()),
                    release_link: None,
                },
                header: Some("header".into()),
                note_sections: {
                    let mut notes = IndexMap::new();

                    let section = String::from("Fixed");
                    notes.insert(
                        section.clone(),
                        ReleaseSection {
                            title: section,
                            notes: vec![
                                ReleaseSectionNote {
                                    scope: Some("data".into()),
                                    message: "the program".into(),
                                    context: vec![
                                        "- fix la base".into(),
                                        "49-3 hihi".into(),
                                        "lol".into(),
                                    ],
                                },
                                ReleaseSectionNote {
                                    scope: Some("ui".into()),
                                    message: "the widget".into(),
                                    context: vec![],
                                },
                                ReleaseSectionNote {
                                    scope: None,
                                    message: "lol".into(),
                                    context: vec![],
                                },
                                ReleaseSectionNote {
                                    scope: Some("ui".into()),
                                    message: "the widget".into(),
                                    context: vec![],
                                },
                                ReleaseSectionNote {
                                    scope: Some("data".into()),
                                    message: "the widget".into(),
                                    context: vec![],
                                },
                            ],
                        },
                    );
                    notes
                },
                footer: None,
            },
        );

        let version = String::from("0.1.0");
        releases.insert(
            version.clone(),
            Release {
                title: ReleaseTitle {
                    version,
                    title: None,
                    release_link: Some(
                        "https://github.com/wiiznokes/fan-control/releases/tag/v2024.7.30".into(),
                    ),
                },
                header: None,
                note_sections: IndexMap::new(),
                footer: None,
            },
        );
        releases
    },
    footer_links: FooterLinks {
        links: vec![
            FooterLink {
                text: "0.6.8".into(),
                link: "https://github.com/taiki-e/parse-changelog/compare/v0.6.7...v0.6.8".into(),
            },
            FooterLink {
                text: "0.6.7".into(),
                link: "https://github.com/taiki-e/parse-changelog/compare/v0.6.6...v0.6.7".into(),
            },
        ],
    },
});

#[test]
fn release_title() {
    let input = "## [2024.7] - 2024-07-24\n";

    let f_input = input.chars().collect::<Vec<_>>();

    let parser = de::release();

    let res = parser.parse(&f_input).unwrap();

    let mut s = String::new();

    ser::serialize_release(&mut s, &res, &OptionsRelease::default());

    assert_eq!(input, s);
}

use std::{
    fs::{read_dir, File},
    io::Read,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use fmt::SortOptions;
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
        let mut releases = BTreeMap::new();

        let version = Version::new(0, 1, 0);
        releases.insert(
            version.clone(),
            Release {
                title: ReleaseTitle {
                    version: version.to_string(),
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

        let version = Version::new(0, 1, 1);
        releases.insert(
            version.clone(),
            Release {
                title: ReleaseTitle {
                    version: version.to_string(),
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
    unreleased: None,
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

#[test]
fn last_version() {
    assert_eq!(CHANGELOG1.last_version().unwrap(), Version::new(0, 1, 1));
}

fn default_sort_order() -> Vec<String> {
    vec![
        "Security".into(),
        "Added".into(),
        "Changed".into(),
        "Removed".into(),
        "Fixed".into(),
        "Deprecated".into(),
        "Documentation".into(),
    ]
}

#[test]
fn end_to_end_file() {
    let path = "./tests/scope.init";

    end_to_end_init(&PathBuf::from(path));
}

#[test]
fn end_to_end() {
    for e in read_dir("./tests").unwrap() {
        let path = e.unwrap().path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        if filename.ends_with(".init") {
            end_to_end_init(&path);
        }

        if filename.ends_with(".err") {
            end_to_end_err(&path);
        }
    }
}

fn end_to_end_init(path: &Path) {
    let filename = path.file_name().unwrap().to_str().unwrap();

    println!("testing {} ...", filename);

    let mut content = String::new();

    File::open(path)
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();

    let mut changelog = parse_changelog(&content).unwrap();

    changelog.sanitize(&fmt::Options {
        sort_options: SortOptions {
            section_order: default_sort_order(),
            sort_scope: !filename.contains("nosort"),
        },
    });

    let res = ser::serialize_changelog(&changelog, &ser::Options::default());

    let filename = filename.replace(".init", ".expect");

    let mut expected = String::new();

    let path = path.parent().unwrap().join(&filename);

    File::open(path)
        .unwrap()
        .read_to_string(&mut expected)
        .unwrap();

    println!("{}", res);
    assert_eq!(res, expected);
}

fn end_to_end_err(path: &Path) {
    let mut content = String::new();

    File::open(path)
        .unwrap()
        .read_to_string(&mut content)
        .unwrap();

    parse_changelog(&content).unwrap_err();
}

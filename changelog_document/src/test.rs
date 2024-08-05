use std::{fs::File, io::Read, sync::LazyLock};

use parser::parse_changelog;

use crate::*;

#[test]
fn changelog2() {
    let mut file = File::open("../tests/changelogs/CHANGELOG2.md").unwrap();

    let mut input = String::new();

    file.read_to_string(&mut input).unwrap();

    let now = std::time::Instant::now();

    let _res = parse_changelog(&input).unwrap();

    println!("{:?}", now.elapsed())

    // dbg!(&res);
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
                },
                header: Some("header".into()),
                note_sections: {
                    let mut notes = HashMap::new();

                    let section = String::from("Fixed");
                    notes.insert(
                        section.clone(),
                        ReleaseSection {
                            title: section,
                            notes: vec![
                                ReleaseSectionNote {
                                    component: Some("data".into()),
                                    message: "the program".into(),
                                },
                                ReleaseSectionNote {
                                    component: Some("ui".into()),
                                    message: "the widget".into(),
                                },
                            ],
                        },
                    );
                    notes
                },
                footer: None,
            },
        );

        let version = String::from("Unreleased");
        releases.insert(
            version.clone(),
            Release {
                title: ReleaseTitle {
                    version,
                    title: None,
                },
                header: None,
                note_sections: HashMap::new(),
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

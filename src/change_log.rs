use std::sync::LazyLock;

use indexmap::IndexMap;
use regex::Regex;

pub fn parse_change_log(changelog: &str) -> Changelog<'_> {
    let (header, changelog) = changelog
        .find("## [Unreleased]")
        .map(|pos| (changelog[0..pos].trim(), &changelog[pos..]))
        .expect("no '## [Unreleased]' found");

    let mut offset = 0;

    let mut foot_links = IndexMap::new();

    for line in changelog.lines().rev() {
        if FOOTER_REGEX.is_match(line) {
            let title = {
                let start = memchr::memchr(b'[', line.as_bytes()).unwrap();
                let end = memchr::memchr(b']', line.as_bytes()).unwrap();

                line[start + 1..end].trim()
            };

            let content = {
                let colon = memchr::memchr(b':', line.as_bytes()).unwrap();

                line[colon + 1..].trim()
            };

            foot_links.insert(title, content);

            offset += line.len();
        } else {
            break;
        }
    }

    let mut changelog = changelog[..changelog.len() - offset].trim();

    let mut releases = IndexMap::new();

    // dbg!(&changelog);

    while let Some((release, offset)) = Release::new(changelog) {
        releases.insert(release.version, release);

        changelog = &changelog[offset..];
    }

    let res = Changelog {
        header,
        releases,
        foot_links,
    };

    // println!("{:?}", res);

    dbg!(&res);

    res
}

static FOOTER_REGEX: LazyLock<Regex> = LazyLock::new(|| regex::Regex::new(r"\[.*\]:").unwrap());

static RELEASE_TITLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| regex::Regex::new(r"## \[.*\]").unwrap());

static RELEASE_SECTION_TITLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| regex::Regex::new(r"### .*").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changelog<'a> {
    pub header: &'a str,
    pub releases: IndexMap<&'a str, Release<'a>>,
    pub foot_links: IndexMap<&'a str, &'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release<'a> {
    pub version: &'a str,
    pub title: &'a str,
    pub notes: IndexMap<&'a str, &'a str>,
}

impl<'a> Release<'a> {
    fn new(text: &'a str) -> Option<(Self, usize)> {
        let lines = text.lines();

        #[derive(Clone, Debug, PartialEq, Eq)]
        enum State<'a> {
            Init,
            Title,
            Section {
                title: &'a str,
                start: usize,
                end: usize,
            },
        }

        let mut state = State::Init;

        let mut version = None;

        let mut title = None;

        let mut notes = IndexMap::new();

        let mut pos = 0;

        for line in lines {
            if RELEASE_TITLE_REGEX.is_match(line) {
                if state != State::Init {
                    break;
                } else {
                    version = {
                        let start = memchr::memchr(b'[', line.as_bytes()).unwrap();
                        let end = memchr::memchr(b']', line.as_bytes()).unwrap();

                        Some(line[start + 1..end].trim())
                    };
                    title = Some(line);

                    state = State::Title;
                }
            } else if RELEASE_SECTION_TITLE_REGEX.is_match(line) {
                if state == State::Init {
                    panic!("release section {} found before a release title", line);
                }

                if let State::Section { title, start, end } = state {
                    notes.insert(title, &text[start..end]);
                }

                let title = { line["### ".len()..].trim() };
                state = State::Section {
                    title,
                    start: pos,
                    end: pos + line.len(),
                }
            } else if let State::Section {
                title: _,
                start: _,
                end,
            } = &mut state
            {
                *end += line.len();
            } else {
                // panic!("not in a section({:?}): {}", state, line);
                // todo: find out what to do in this situation
            }

            pos += line.len() + 1;
        }

        if state == State::Init {
            return None;
        }

        let release = Release {
            version: version.unwrap(),
            title: title.unwrap(),
            notes,
        };

        Some((release, pos))
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Read};

    use crate::change_log::{FOOTER_REGEX, RELEASE_SECTION_TITLE_REGEX, RELEASE_TITLE_REGEX};

    use super::parse_change_log;

    #[test]
    fn test_changelog1() {
        let mut file = File::open("tests/changelogs/CHANGELOG1.md").unwrap();

        let mut changelog = String::new();

        file.read_to_string(&mut changelog).unwrap();

        parse_change_log(&changelog);
    }

    #[test]
    fn test_changelog2() {
        let mut file = File::open("tests/changelogs/CHANGELOG2.md").unwrap();

        let mut changelog = String::new();

        file.read_to_string(&mut changelog).unwrap();

        parse_change_log(&changelog);
    }

    #[test]
    fn test_regex() {
        assert!(FOOTER_REGEX.is_match("[hello]:"));
        assert!(!FOOTER_REGEX.is_match("[hello]"));
        assert!(!FOOTER_REGEX.is_match("[hello:"));
        assert!(!FOOTER_REGEX.is_match("hello]:"));
    }

    #[test]
    fn test_regex_release() {
        assert!(RELEASE_TITLE_REGEX.is_match("## [2024.7.30]"));
        assert!(!RELEASE_TITLE_REGEX.is_match("# [2024.7.30]"));
        assert!(RELEASE_TITLE_REGEX.is_match("## [Unreleased]"));
        assert!(RELEASE_TITLE_REGEX.is_match("## [2024.7] - 2024-07-24"));
        assert!(!RELEASE_TITLE_REGEX.is_match("##[2024.7] - 2024-07-24"));
        assert!(!RELEASE_TITLE_REGEX.is_match("# [2024.7] - 2024-07-24"));
    }

    #[test]
    fn test_regex_release_section() {
        assert!(RELEASE_SECTION_TITLE_REGEX.is_match("### Added"));
        assert!(!RELEASE_SECTION_TITLE_REGEX.is_match("## Added"));
        assert!(!RELEASE_SECTION_TITLE_REGEX.is_match("###Added"));
        assert!(RELEASE_SECTION_TITLE_REGEX.is_match("### [Added]"));
    }
}

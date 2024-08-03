use std::{collections::HashMap, fs::File, io::Read};

use pom::parser::*;
use utils::into_string;

fn main() {
    let mut file = File::open("tests/changelogs/CHANGELOG4.md").unwrap();

    let mut input = String::new();

    file.read_to_string(&mut input).unwrap();


    let input = input.chars().collect::<Vec<_>>();

    let res = changelog();

    let res = res.parse(&input).unwrap();

    dbg!(&res);

}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseTitle {
    pub version: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseSection {
    pub title: String,
    // todo: hashmap multi using component
    pub notes: Vec<ReleaseSectionNote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseSectionNote {
    pub component: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Release {
    pub title: ReleaseTitle,
    pub header: Option<String>,
    pub notes: HashMap<String, ReleaseSection>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterLink {
    pub text: String,
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterLinks {
    pub links: Vec<FooterLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChangeLog {
    pub header: Option<String>,
    pub releases: HashMap<String, Release>,
    pub footer_links: FooterLinks,
}

fn changelog<'a>() -> Parser<'a, char, ChangeLog> {
    let header = (!call(release) * any()).repeat(0..).convert(|header| {
        let header = into_string(header);

        if header.is_empty() {
            Ok::<_, ()>(None)
        } else {
            Ok(Some(header))
        }
    });

    let parser = header + release().repeat(0..);

    parser.convert(|(header, releases_vec)| {
        let mut releases = HashMap::new();

        for release in releases_vec.into_iter() {
            releases.insert(release.title.version.clone(), release);
        }

        let res = ChangeLog {
            header,
            releases,
            footer_links: FooterLinks { links: Vec::new() },
        };

        Ok::<ChangeLog, ()>(res)
    })
}

fn release_title<'a>() -> Parser<'a, char, ReleaseTitle> {
    let version = sym('#').repeat(2) * sym(' ') * sym('[') * none_of("\n]").repeat(1..) - sym(']');

    let title = sym(' ') * sym('-') * sym(' ') * none_of("\n]").repeat(1..);

    let parser = version + title.opt();

    parser.convert(|(version, title)| {
        let res = ReleaseTitle {
            version: into_string(version),
            title: title.map(into_string),
        };

        Ok::<ReleaseTitle, ()>(res)
    })
}

fn release_section_note<'a>() -> Parser<'a, char, ReleaseSectionNote> {
    let component = none_of(":\n").repeat(1..) - sym(':');

    let parser = sym('-') * sym(' ') * component.opt() + none_of("\n").repeat(1..) - sym('\n');

    parser.convert(|(component, note)| {
        let res = ReleaseSectionNote {
            component: component.map(into_string),
            note: into_string(note),
        };

        Ok::<ReleaseSectionNote, ()>(res)
    })
}

fn release_section<'a>() -> Parser<'a, char, ReleaseSection> {
    let title = sym('#').repeat(3) * sym(' ') * none_of("\n").repeat(1..) - sym('\n');

    let parser = title - sym('\n') + release_section_note().repeat(1..);

    parser.convert(|(title, notes)| {
        let res = ReleaseSection {
            title: into_string(title),
            notes,
        };

        Ok::<ReleaseSection, ()>(res)
    })
}

fn release<'a>() -> Parser<'a, char, Release> {
    let header = (!call(release_section) * any())
        .repeat(0..)
        .convert(|header| {
            let header = into_string(header);

            if header.is_empty() {
                Ok::<_, ()>(None)
            } else {
                Ok(Some(header))
            }
        });

    // todo: add footer_links here
    let footer = ((!call(release) + !call(release_section)) * any()).repeat(0..).convert(|footer| {
        let footer = into_string(footer);

        if footer.is_empty() {
            Ok::<_, ()>(None)
        } else {
            Ok(Some(footer))
        }
    });

    let parser = release_title() + header + release_section().repeat(0..) + footer;

    parser.convert(|(((title, header), sections), footer)| {
        let mut notes = HashMap::new();

        for section in sections.into_iter() {
            notes.insert(section.title.clone(), section);
        }

        let res = Release {
            title,
            header,
            notes,
            footer,
        };

        Ok::<Release, ()>(res)
    })
}

mod utils {
    use pom::parser::*;

    pub fn into_string(v: Vec<char>) -> String {
        let str = v.into_iter().collect::<String>();
        let str = str.trim();
        str.to_owned()
    }

    pub fn space<'a>() -> Parser<'a, char, ()> {
        one_of(" \t\r\n").repeat(0..).discard()
    }
}

#[test]
fn t() {
    let input = r#"
hello
la miff

## [2024.7] - 2024-07-24

"#;

    let input = input.chars();

    let input = input.collect::<Vec<_>>();

    let res = changelog();

    let res = res.parse(&input).unwrap();

    dbg!(&res);
}



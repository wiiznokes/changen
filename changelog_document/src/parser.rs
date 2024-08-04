use super::*;
use pom::parser::*;
use utils::*;

pub fn changelog<'a>() -> Parser<'a, char, ChangeLog> {
    let header = (!call(release) * any()).repeat(0..).convert(|header| {
        let header = into_string(header);

        if header.is_empty() {
            Ok::<_, ()>(None)
        } else {
            Ok(Some(header))
        }
    });

    let parser = header + release().repeat(0..) + footer_links();

    parser.convert(|((header, releases_vec), footer_links)| {
        let mut releases = IndexMap::new();

        for release in releases_vec.into_iter() {
            releases.insert(release.title.version.clone(), release);
        }

        let res = ChangeLog {
            header,
            releases,
            footer_links,
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
    // todo: maybe drop this ?
    let component = none_of(" \t\r`:\n").repeat(1..) - sym(':');

    let parser =
        spaceline() * sym('-') * sym(' ') * component.opt() + none_of("\n").repeat(1..) - sym('\n');

    parser.convert(|(component, note)| {
        let res = ReleaseSectionNote {
            component: component.map(into_string),
            note: into_string(note),
        };

        Ok::<ReleaseSectionNote, ()>(res)
    })
}

fn release_section<'a>() -> Parser<'a, char, ReleaseSection> {
    let title = space() * sym('#').repeat(3) * sym(' ') * none_of("\n").repeat(1..) - sym('\n');

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
    let header = ((!call(release_title) + !call(release_section) + !call(footer_links)) * any())
        .repeat(0..)
        .convert(|header| {
            let header = into_string(header);

            if header.is_empty() {
                Ok::<_, ()>(None)
            } else {
                Ok(Some(header))
            }
        });

    let footer = ((!call(release_title) + !call(release_section) + !call(footer_links)) * any())
        .repeat(0..)
        .convert(|footer| {
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

fn footer_link<'a>() -> Parser<'a, char, FooterLink> {
    let parser = sym('[') * none_of("\n]").repeat(1..) - sym(']') * sym(':') * sym(' ')
        + none_of("\n").repeat(1..)
        - sym('\n');

    parser.convert(|(text, link)| {
        let res = FooterLink {
            text: into_string(text),
            link: into_string(link),
        };

        Ok::<FooterLink, ()>(res)
    })
}

fn footer_links<'a>() -> Parser<'a, char, FooterLinks> {
    let parser = space() * footer_link().repeat(0..) - space() - end();

    parser.convert(|links| {
        let res = FooterLinks { links };

        Ok::<FooterLinks, ()>(res)
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
    pub fn spaceline<'a>() -> Parser<'a, char, ()> {
        one_of(" \n").repeat(0..).discard()
    }
}

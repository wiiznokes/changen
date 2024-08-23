use crate::*;

// todo: use io::Write

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub release_option: OptionsRelease,
}

#[derive(Debug, Clone)]
pub struct OptionsRelease {
    pub serialize_title: bool,
}

impl Default for OptionsRelease {
    fn default() -> Self {
        Self {
            serialize_title: true,
        }
    }
}

pub fn serialize_changelog(changelog: &ChangeLog, options: &Options) -> String {
    let mut s = String::new();

    let mut should_new_line = false;

    if let Some(header) = &changelog.header {
        s.push_str(header);
        s.push('\n');

        should_new_line = true;
    }

    if let Some(unreleased) = &changelog.unreleased {
        if should_new_line {
            s.push('\n');
        }
        should_new_line = true;

        serialize_release(&mut s, unreleased, &options.release_option);
    }

    for release in changelog.releases.values().rev() {
        if should_new_line {
            s.push('\n');
        }
        should_new_line = true;
        serialize_release(&mut s, release, &options.release_option);
    }

    if !changelog.footer_links.links.is_empty() {
        s.push('\n');
    }

    for footer_link in &changelog.footer_links.links {
        s.push_str(&format!("[{}]: {}\n", footer_link.text, footer_link.link));
    }

    s
}

// todo: handle footer links
pub fn serialize_release(s: &mut String, release: &Release, options: &OptionsRelease) {
    let mut should_new_line = false;

    if options.serialize_title {
        let mut full_title = format!("## [{}]", release.title.version);

        if let Some(release_link) = &release.title.release_link {
            full_title.push_str(&format!("({})", release_link));
        }

        if let Some(title) = &release.title.title {
            full_title.push_str(&format!(" - {}", title));
        }

        full_title.push('\n');

        s.push_str(&full_title);

        should_new_line = true;
    }

    if let Some(header) = &release.header {
        if should_new_line {
            s.push('\n');
        }
        s.push_str(&format!("{}\n", header));
        should_new_line = true;
    }

    for (_, section) in &release.note_sections {
        if !section.notes.is_empty() {
            if should_new_line {
                s.push('\n');
            }
            should_new_line = true;

            s.push_str(&format!("### {}\n\n", section.title));

            for note in &section.notes {
                serialize_release_section_note(s, note);
            }
        }
    }

    if let Some(footer) = &release.footer {
        if should_new_line {
            s.push('\n');
        }
        s.push_str(&format!("{}\n", footer));
    }
}

pub fn serialize_release_section_note(s: &mut String, note: &ReleaseSectionNote) {
    let note_title = if let Some(scope) = &note.scope {
        format!("- {}: {}\n", scope, note.message)
    } else {
        format!("- {}\n", note.message)
    };

    s.push_str(&note_title);

    for context in &note.context {
        s.push_str(&format!("  {}\n", context));
    }
}

#[cfg(test)]
mod test {

    use crate::test::CHANGELOG1;

    use super::*;

    #[test]
    fn test() {
        let output = serialize_changelog(&CHANGELOG1, &Options::default());

        println!("{}", output);
    }

    #[test]
    fn test2() {
        let release_note = ReleaseSectionNote {
            scope: Some("data".into()),
            message: "the program".into(),
            context: vec!["- fix la base".into(), "49-3 hihi".into()],
        };

        let mut output = String::new();

        serialize_release_section_note(&mut output, &release_note);

        println!("{:?}", output);
    }
}

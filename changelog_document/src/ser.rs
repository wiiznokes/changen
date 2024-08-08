use crate::*;

// todo: use io::Write

#[derive(Debug, Clone, Default)]
pub struct ChangeLogSerOption {
    pub release_option: ChangeLogSerOptionRelease,
}

#[derive(Debug, Clone)]
pub struct ChangeLogSerOptionRelease {
    pub section_order: Vec<String>,
    pub serialise_title: bool,
}

impl Default for ChangeLogSerOptionRelease {
    fn default() -> Self {
        Self {
            section_order: Default::default(),
            serialise_title: true,
        }
    }
}

pub fn serialize_changelog(changelog: &ChangeLog, options: &ChangeLogSerOption) -> String {
    let mut s = String::new();

    let mut should_new_line = false;

    if let Some(header) = &changelog.header {
        s.push_str(header);
        s.push('\n');

        should_new_line = true;
    }

    for release in changelog.releases.values() {
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
pub fn serialize_release(s: &mut String, release: &Release, options: &ChangeLogSerOptionRelease) {
    let mut should_new_line = false;

    if options.serialise_title {
        let mut full_title = format!("## [{}]", release.title.version);

        if let Some(release_link) = &release.title.release_link {
            full_title.push_str(&format!("({})", release_link));
        }

        if let Some(title) = &release.title.title {
            full_title.push_str(&format!("- {}", title));
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

    let note_section_sorted = {
        let mut sorted = Vec::new();

        let mut section_cloned = release.note_sections.clone();

        for section in &options.section_order {
            if let Some(section) = section_cloned.shift_remove(section) {
                sorted.push(section);
            }
        }

        sorted.extend(section_cloned.into_values());
        sorted
    };

    for sections in &note_section_sorted {
        if !sections.notes.is_empty() {
            if should_new_line {
                s.push('\n');
            }
            should_new_line = true;

            s.push_str(&format!("### {}\n\n", sections.title));

            for note in &sections.notes {
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
        let output = serialize_changelog(&CHANGELOG1, &ChangeLogSerOption::default());

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

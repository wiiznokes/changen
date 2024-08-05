use crate::{ChangeLog, Release};

// todo: use io::Write

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub section_order: Vec<String>,
}

pub fn serialize_changelog(changelog: &ChangeLog, options: &Options) -> String {
    let mut s = String::new();

    if let Some(header) = &changelog.header {
        s.push_str(header);
        s.push('\n');
    }

    for release in changelog.releases.values() {
        serialize_release(&mut s, release, options);
    }

    if !changelog.footer_links.links.is_empty() {
        s.push('\n');
    }

    for footer_link in &changelog.footer_links.links {
        s.push_str(&format!("[{}]: {}\n", footer_link.text, footer_link.link));
    }

    s.push('\n');

    s
}

// todo: handle footer links
pub fn serialize_release(s: &mut String, release: &Release, options: &Options) {
    let title = if let Some(title) = &release.title.title {
        format!("\n## [{}] - {}\n", release.title.version, title)
    } else {
        format!("\n## [{}]\n", release.title.version)
    };

    s.push_str(&title);

    if let Some(header) = &release.header {
        s.push_str(&format!("\n{}\n", header));
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
        s.push_str(&format!("\n### {}\n\n", sections.title));

        for note in &sections.notes {
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
    }

    if let Some(footer) = &release.footer {
        s.push_str(&format!("\n{}\n", footer));
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
}

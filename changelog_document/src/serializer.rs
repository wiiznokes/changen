use crate::{ChangeLog, Release};

pub fn serialize_changelog(changelog: &ChangeLog) -> String {
    let mut s = String::new();

    if let Some(header) = &changelog.header {
        s.push_str(header);
        s.push('\n');
    }

    for release in changelog.releases.values() {
        ser_release(&mut s, release);
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

fn ser_release(s: &mut String, release: &Release) {
    let title = if let Some(title) = &release.title.title {
        format!("\n## [{}] - {}\n", release.title.version, title)
    } else {
        format!("\n## [{}]\n", release.title.version)
    };

    s.push_str(&title);

    if let Some(header) = &release.header {
        s.push_str(&format!("\n{}\n", header));
    }

    for sections in release.note_sections.values() {
        s.push_str(&format!("\n### {}\n\n", sections.title));

        for note in &sections.notes {
            let note = if let Some(component) = &note.component {
                format!("- {}: {}\n", component, note.message)
            } else {
                format!("- {}\n", note.message)
            };

            s.push_str(&note);
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
        let output = serialize_changelog(&CHANGELOG1);

        println!("{}", output);
    }
}

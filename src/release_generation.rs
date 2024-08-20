use anyhow::bail;
use changelog::{
    ser::{serialize_changelog, ChangeLogSerOption},
    ChangeLog, Release, ReleaseTitle,
};
use indexmap::IndexMap;

use crate::{
    git_provider::{DiffTags, GitProvider},
    UNRELEASED,
};

pub fn release(
    mut changelog: ChangeLog,
    version: Option<String>,
    provider: GitProvider,
    repo: Option<String>,
    omit_diff: bool,
) -> anyhow::Result<(String, String)> {
    fn get_prev(changelog: &ChangeLog) -> Option<String> {
        let mut keys = changelog.releases.keys();
        if let Some(e) = keys.next() {
            if e != UNRELEASED {
                return Some(e.to_owned());
            }
        }
        keys.next().cloned()
    }

    let diff_tags = DiffTags::new(version, get_prev(&changelog))?;

    if changelog.releases.get(&diff_tags.new).is_some() {
        bail!(
            "Version {} already exist. Create a new tag or use the --version option.",
            diff_tags.new
        );
    };

    let empty_unreleased = Release {
        title: ReleaseTitle {
            version: UNRELEASED.into(),
            title: None,
            release_link: None,
        },
        header: None,
        note_sections: IndexMap::new(),
        footer: None,
    };

    let (pos, Some(mut prev_unreleased)) = changelog
        .releases
        .insert_full(UNRELEASED.into(), empty_unreleased)
    else {
        bail!("No Unreleased section found.")
    };

    debug_assert!(pos == 0);

    prev_unreleased.title.version = diff_tags.new.clone();

    if let Some(repo) = &repo {
        match provider.release_link(repo, &diff_tags.new) {
            Ok(link) => {
                prev_unreleased.title.release_link = Some(link);
            }
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }

    if !omit_diff {
        let link = if let Some(repo) = &repo {
            match provider.diff_link(repo, &diff_tags) {
                Ok(link) => Some(link),
                Err(e) => {
                    eprintln!("{e}");
                    None
                }
            }
        } else {
            None
        };

        if let Some(link) = link {
            let line = format!("Full Changelog: {link}");

            match &mut prev_unreleased.footer {
                Some(footer) => {
                    footer.push_str("\n\n");
                    footer.push_str(&line);
                }
                None => {
                    prev_unreleased.footer = Some(line);
                }
            }
        }
    }

    changelog
        .releases
        .shift_insert(1, diff_tags.new.clone(), prev_unreleased);

    debug!("release: serialize changelog: {:?}", changelog);

    let output = serialize_changelog(&changelog, &ChangeLogSerOption::default());

    Ok((diff_tags.new, output))
}

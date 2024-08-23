use anyhow::bail;
use changelog::{ser::serialize_changelog, ChangeLog, Release, ReleaseTitle};
use indexmap::IndexMap;

use crate::{git_provider::DiffTags, utils::get_last_tag, UNRELEASED};

pub fn release(
    mut changelog: ChangeLog,
    options: &crate::config::Release,
) -> anyhow::Result<(String, String)> {
    let crate::config::Release {
        file: _,
        version,
        previous_version,
        provider,
        repo,
        omit_diff,
        stdout: _,
        force,
    } = options;

    let previous_version = previous_version
        .clone()
        .or_else(|| get_last_tag(&changelog));

    let diff_tags = DiffTags::new(version.clone(), previous_version)?;

    if changelog.releases.get(&diff_tags.new).is_some() {
        if *force {
            changelog.releases.shift_remove(&diff_tags.new);
            eprintln!("The release {} will be overwritten", diff_tags.new)
        } else {
            bail!(
                "Version {} already exist. Create a new tag or use the --version option. You can also use the --force option to override the existing release.",
                diff_tags.new
            );
        }
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

    changelog.sanitize(&changelog::fmt::Options::default());

    let output = serialize_changelog(&changelog, &changelog::ser::Options::default());

    Ok((diff_tags.new, output))
}

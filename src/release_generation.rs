use anyhow::bail;
use changelog::{
    ser::{serialize_changelog, ChangeLogSerOption},
    ChangeLog, Release, ReleaseTitle,
};
use indexmap::IndexMap;

use crate::{
    git_helpers_function::{tags_list, try_get_repo},
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
    let version = match version {
        Some(version) => {
            if version.starts_with('v') {
                bail!("Error: You shouldn't include the v prefix in the version.")
            }
            version
        }
        None => {
            let Some(tag) = tags_list()?.pop_back() else {
                bail!("No version provided. Can't fall back to last tag because there is none.");
            };
            eprintln!("No version provided. Using the existing last tag: {}", tag);
            tag
        }
    };

    if changelog.releases.get(&version).is_some() {
        bail!(
            "Version {} already exist. Create a new tag or use the --version option.",
            version
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

    prev_unreleased.title.version = version.clone();

    try_get_repo(&repo).inspect(|repo| {
        let mut tags = tags_list().unwrap();

        match tags.pop_back() {
            Some(tag) => match provider.release_link(repo, &tag) {
                Ok(link) => {
                    prev_unreleased.title.release_link = Some(link);
                }
                Err(e) => {
                    eprintln!("{e}");
                }
            },
            None => {
                eprintln!("No tags defined. Can't produce the release link.");
            }
        }
    });

    if !omit_diff {
        let link = if let Some(repo) = try_get_repo(&repo) {
            let mut tags = tags_list()?;

            match tags.pop_back() {
                Some(current) => {
                    let prev = tags.pop_back();

                    let diff_tags = DiffTags { prev, current };

                    match provider.diff_link(&repo, &diff_tags) {
                        Ok(link) => Some(link),
                        Err(e) => {
                            eprintln!("{e}");
                            None
                        }
                    }
                }
                None => {
                    eprintln!("No tags defined. Can't produce the diff");
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
        .shift_insert(1, version.clone(), prev_unreleased);

    debug!("release: serialize changelog: {:?}", changelog);

    let output = serialize_changelog(&changelog, &ChangeLogSerOption::default());

    Ok((version, output))
}

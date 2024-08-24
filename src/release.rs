use anyhow::bail;
use changelog::{ser::serialize_changelog, utils::DEFAULT_UNRELEASED, ChangeLog};

use crate::{
    config::MergeDevVersions,
    git_provider::DiffTags,
    repository::{try_detect_new_version, Repository},
};

pub fn release<R: Repository>(
    r: &R,
    mut changelog: ChangeLog,
    options: &crate::config::Release,
) -> anyhow::Result<(String, String)> {
    let crate::config::Release {
        file: _,
        version,
        previous_version,
        provider,
        repo,
        header,
        merge_dev_versions,
        omit_diff,
        stdout: _,
        force,
    } = options;

    let new_version = try_detect_new_version(r, version.clone())?;

    if changelog.releases.contains_key(&new_version) {
        if *force {
            changelog.releases.remove(&new_version);
            eprintln!("The release {} will be overwritten", new_version)
        } else {
            bail!(
                "Version {} already exist. Create a new tag or use the --version option. You can also use the --force option to override the existing release.",
                new_version
            );
        }
    };

    let mut prev_unreleased = changelog
        .unreleased
        .replace(DEFAULT_UNRELEASED.clone())
        .unwrap_or(DEFAULT_UNRELEASED.clone());

    prev_unreleased.title.version = new_version.to_string();

    if let Some(header) = header {
        match &prev_unreleased.header {
            Some(prev_header) => {
                prev_unreleased.header = Some(format!("{}\n{}", header, prev_header))
            }
            None => prev_unreleased.header = Some(header.clone()),
        }
    }

    if let Some(repo) = &repo {
        match provider.release_link(repo, &new_version.to_string()) {
            Ok(link) => {
                prev_unreleased.title.release_link = Some(link);
            }
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }

    match merge_dev_versions {
        MergeDevVersions::Yes | MergeDevVersions::Auto if new_version.pre.is_empty() => {
            let dev_releases = changelog
                .releases
                .extract_if(|k, _| {
                    k.major == new_version.major
                        && k.minor == new_version.minor
                        && k.patch == new_version.patch
                })
                .collect::<Vec<_>>();

            for (_, dev_release) in dev_releases {
                prev_unreleased
                    .insert_release_notes(dev_release.note_sections.into_iter().map(|(_, sec)| sec))
            }
        }
        _ => {}
    }

    let previous_version = previous_version
        .clone()
        .or_else(|| changelog.last_version());

    let diff_tags = DiffTags::new(new_version, previous_version)?;

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
        .insert(diff_tags.new.clone(), prev_unreleased);

    debug!("release: serialize changelog: {:?}", changelog);

    changelog.sanitize(&changelog::fmt::Options::default());

    let output = serialize_changelog(&changelog, &changelog::ser::Options::default());

    Ok((diff_tags.new.to_string(), output))
}

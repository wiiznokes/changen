use crate::{
    commit_parser::{parse_commit, FormattedCommit},
    config::Generate,
    git_helpers_function::{commits_between_tags, RawCommit},
    git_provider::RelatedPr,
    utils::get_last_tag,
};
use anyhow::{bail, Result};
use changelog::{
    ser::serialize_release_section_note, ChangeLog, Release, ReleaseSection, ReleaseSectionNote,
};

use crate::config::{CommitMessageParsing, MapMessageToSection};

pub fn gen_release_notes(
    changelog: &ChangeLog,
    unreleased: &mut Release,
    changelog_path: String,
    map: &MapMessageToSection,
    options: &Generate,
) -> Result<()> {
    if let Some(specific) = &options.specific {
        return handle_specific(unreleased, changelog_path, map, options, specific);
    }

    if let Some(milestone) = &options.milestone {
        return handle_milestone(unreleased, changelog_path, map, options, milestone);
    }

    handle_period(changelog, unreleased, changelog_path, map, options)
}

#[derive(Debug, Clone)]
pub struct Period {
    pub since: Option<String>,
    pub until: Option<String>,
}

fn handle_milestone(
    unreleased: &mut Release,
    changelog_path: String,
    map: &MapMessageToSection,
    options: &Generate,
    milestone: &str,
) -> Result<()> {
    for pr in options
        .provider
        .milestone_prs(&options.repo.clone().unwrap(), milestone)?
    {
        let raw_commit = RawCommit {
            title: pr.title.clone().unwrap_or_default(),
            body: pr.body.clone().unwrap_or_default(),
            sha: "".into(),
            list_files: vec![],
            author: pr.author.clone().unwrap_or_default(),
        };

        match get_release_note(&raw_commit, Some(&pr), &changelog_path, map, options) {
            Ok((section_title, release_note)) => {
                insert_release_note(unreleased, section_title, release_note);
            }
            Err(e) => eprintln!("commit {}: {e}", raw_commit.short_commit()),
        }
    }

    Ok(())
}

fn handle_specific(
    unreleased: &mut Release,
    changelog_path: String,
    map: &MapMessageToSection,
    options: &Generate,
    specific: &str,
) -> Result<()> {
    let raw_commit = RawCommit::from_sha(specific);

    let related_pr = match &options.repo {
        Some(repo) => match options.provider.related_pr(repo, &raw_commit.sha) {
            Ok(related_pr) => Some(related_pr),
            Err(e) => {
                eprintln!("error while requesting pr link: {}", e);
                None
            }
        },
        None => None,
    };

    match get_release_note(
        &raw_commit,
        related_pr.as_ref(),
        &changelog_path,
        map,
        options,
    ) {
        Ok((section_title, release_note)) => {
            let mut added = String::new();
            serialize_release_section_note(&mut added, &release_note);

            insert_release_note(unreleased, section_title.clone(), release_note);

            eprintln!("Release note:\n{added}successfully added in the {section_title} section.")
        }
        Err(e) => eprintln!("commit {}: {e}", raw_commit.short_commit()),
    }

    Ok(())
}

fn handle_period(
    changelog: &ChangeLog,
    unreleased: &mut Release,
    changelog_path: String,
    map: &MapMessageToSection,
    options: &Generate,
) -> Result<()> {
    let since = options.since.clone().or_else(|| get_last_tag(changelog));

    let period = Period {
        since,
        until: options.until.clone(),
    };

    info!("generate period: {:?}", period);

    let commits = commits_between_tags(&period);

    let mut last_prs = match &options.repo {
        Some(repo) => match options.provider.last_prs(repo, commits.len()) {
            Ok(last_prs) => Some(last_prs),
            Err(e) => {
                eprintln!("error while requesting pr link: {}", e);
                None
            }
        },
        None => None,
    };

    for sha in commits {
        let raw_commit = RawCommit::from_sha(&sha);

        let related_pr = match last_prs {
            Some(ref mut last_prs) => last_prs.remove(&sha),
            None => None,
        };

        // fallback to derive from commit
        let related_pr = match related_pr {
            Some(related_pr) => Some(related_pr),
            None => match &options.repo {
                Some(repo) => options.provider.offline_related_pr(repo, &raw_commit),
                None => None,
            },
        };

        match get_release_note(
            &raw_commit,
            related_pr.as_ref(),
            &changelog_path,
            map,
            options,
        ) {
            Ok((section_title, release_note)) => {
                insert_release_note(unreleased, section_title, release_note);
            }
            Err(e) => eprintln!("commit {}: {e}", raw_commit.short_commit()),
        }
    }

    Ok(())
}

fn get_release_note(
    raw_commit: &RawCommit,
    related_pr: Option<&RelatedPr>,
    changelog_path: &str,
    map: &MapMessageToSection,
    options: &Generate,
) -> Result<(String, ReleaseSectionNote)> {
    if let Response::Yes { reason } = commit_should_be_ignored(raw_commit, changelog_path) {
        bail!("Ignoring commit. {reason}");
    }

    let mut commit = match parse_commit(&raw_commit.title) {
        Ok(mut commit) => {
            let section = match map.map_section(&commit.section) {
                Some(section) => section,
                None => {
                    if options.parsing == CommitMessageParsing::Strict {
                        bail!(
                            "no corresponding commit type was found for {}",
                            commit.section
                        );
                    }

                    if let Some(section) =
                        map.try_find_section((&raw_commit.title, &raw_commit.body))
                    {
                        section
                    } else {
                        if options.exclude_unidentified {
                            bail!(
                                "No corresponding commit type was found for {}",
                                commit.section
                            );
                        }
                        "Unidentified".into()
                    }
                }
            };

            commit.section = section;
            commit
        }
        Err(e) => {
            if options.parsing == CommitMessageParsing::Strict {
                bail!(
                    "Commit {}: invalid syntax: {}",
                    raw_commit.short_commit(),
                    e
                );
            }

            let section = if let Some(section) =
                map.try_find_section((&raw_commit.title, &raw_commit.body))
            {
                section
            } else {
                if options.exclude_unidentified {
                    bail!("Not identified.");
                }
                "Unidentified".into()
            };

            FormattedCommit {
                section,
                scope: None,
                message: raw_commit.title.clone(),
            }
        }
    };

    if let Some(related_pr) = &related_pr {
        if !related_pr.is_pr && options.exclude_not_pr {
            bail!("No upstream pr was found");
        }

        if !options.omit_pr_link {
            commit
                .message
                .push_str(&format!(" in [{}]({})", related_pr.pr_id, related_pr.url));
        }

        if !options.omit_thanks {
            if let (Some(author), Some(author_link)) = (&related_pr.author, &related_pr.author_link)
            {
                commit
                    .message
                    .push_str(&format!(" by [@{author}]({author_link})"));
            }
        }
    } else if options.exclude_not_pr {
        bail!("no upstream pr was found");
    };

    Ok((
        commit.section,
        ReleaseSectionNote {
            scope: commit.scope,
            message: commit.message,
            context: vec![],
        },
    ))
}

fn insert_release_note(
    unreleased: &mut Release,
    section_title: String,
    release_note: ReleaseSectionNote,
) {
    let section = if let Some(section) = unreleased.note_sections.get_mut(&section_title) {
        section
    } else {
        let release_section = ReleaseSection {
            title: section_title.clone(),
            notes: vec![],
        };

        unreleased
            .note_sections
            .insert(section_title.clone(), release_section);
        unreleased.note_sections.get_mut(&section_title).unwrap()
    };

    section.notes.push(release_note);
}

#[derive(Debug, Clone)]
enum Response {
    Yes { reason: String },
    No,
}

impl Response {
    #[allow(dead_code)]
    fn bool(&self) -> bool {
        match self {
            Response::Yes { .. } => true,
            Response::No => false,
        }
    }
}

fn commit_should_be_ignored(raw: &RawCommit, changelog_path: &str) -> Response {
    debug!("{:?}", raw);
    debug!("{:?}", changelog_path);

    // if raw.list_files.iter().any(|path| path == changelog_path) {
    //     return Response::Yes {
    //         reason: "The changelog was modified in this commit.".into(),
    //     };
    // }

    let names = ["changelog", "log", "chglog", "notes"];

    let match_pat = |pat: &str| raw.title.contains(pat) || raw.body.contains(pat);

    for n in names {
        let patterns = [
            format!("(skip {n})"),
            format!("(ignore {n})"),
            format!("!{n}"),
        ];

        for pattern in &patterns {
            if match_pat(pattern) {
                return Response::Yes {
                    reason: format!(
                        "\"{pattern}\" was matched in the commit title or description."
                    ),
                };
            }
        }
    }

    Response::No
}

#[cfg(test)]
mod test {
    use crate::{
        git_helpers_function::RawCommit, release_note_generation::commit_should_be_ignored,
    };

    #[test]
    fn ignore_commit() {
        let mut raw = RawCommit {
            title: "fix: something !log".into(),
            body: "".into(),
            sha: "".into(),
            list_files: vec![],
            author: "".into(),
        };

        assert!(commit_should_be_ignored(&raw, "").bool());

        raw.title = "fix: something log".into();

        assert!(!commit_should_be_ignored(&raw, "").bool());
    }
}

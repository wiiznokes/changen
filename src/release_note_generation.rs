use crate::{
    commit_parser::{parse_commit, Commit},
    git_helpers_function::{commits_between_tags, try_get_repo, RawCommit},
    git_provider::{GitProvider, RelatedPr},
};
use anyhow::{bail, Result};
use changelog::{ser::serialize_release_section_note, Release, ReleaseSection, ReleaseSectionNote};

use crate::config::{CommitMessageParsing, MapMessageToSection};

#[derive(Debug, Clone)]
pub struct GenerateReleaseNoteOptions<'a> {
    pub changelog_path: String,
    pub parsing: CommitMessageParsing,
    pub exclude_unidentified: bool,
    pub exclude_not_pr: bool,
    pub provider: GitProvider,
    pub repo: Option<String>,
    pub omit_pr_link: bool,
    pub omit_thanks: bool,
    pub map: &'a MapMessageToSection,
}

pub fn gen_release_notes(
    unreleased: &mut Release,
    milestone: Option<String>,
    tags: Option<String>,
    options: GenerateReleaseNoteOptions,
) -> Result<()> {
    if let Some(milestone) = milestone {
        let repo = try_get_repo(options.repo.to_owned()).unwrap();

        for pr in options.provider.milestone_prs(&repo, &milestone)? {
            let raw_commit = RawCommit {
                message: pr.message,
                desc: pr.body,
                sha: "".into(),
                list_files: vec![],
            };

            match get_release_note(raw_commit, Some(pr.inner), options.clone()) {
                Ok(Some((section_title, release_note))) => {
                    insert_release_note(unreleased, section_title, release_note);
                }
                Ok(None) => {}
                Err(e) => error!("{e}"),
            }
        }

        return Ok(());
    }

    if let Some(tags) = tags {
        for sha in commits_between_tags(&tags) {
            let raw_commit = RawCommit::from_sha(&sha);

            match get_release_note(raw_commit, None, options.clone()) {
                Ok(Some((section_title, release_note))) => {
                    insert_release_note(unreleased, section_title, release_note);
                }
                Ok(None) => {}
                Err(e) => error!("{e}"),
            }
        }

        return Ok(());
    }

    if let Some((section_title, release_note)) =
        get_release_note(RawCommit::last_from_fs(), None, options)?
    {
        let mut added = String::new();
        serialize_release_section_note(&mut added, &release_note);

        insert_release_note(unreleased, section_title.clone(), release_note);

        eprintln!("Release note:\n{added}successfully added in the {section_title} section.",)
    }

    Ok(())
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

fn get_release_note(
    raw_commit: RawCommit,
    related_pr_arg: Option<RelatedPr>,
    options: GenerateReleaseNoteOptions,
) -> Result<Option<(String, ReleaseSectionNote)>> {
    let GenerateReleaseNoteOptions {
        changelog_path,
        parsing,
        exclude_unidentified,
        exclude_not_pr,
        provider,
        repo,
        omit_pr_link,
        omit_thanks,
        map,
    } = options;

    if let Response::Yes { reason } = commit_should_be_ignored(&raw_commit, &changelog_path) {
        eprintln!("Ignoring this commit. {reason}");
        return Ok(None);
    }

    let mut commit = match parse_commit(&raw_commit.message) {
        Ok(mut commit) => {
            let section = match map.map_section(&commit.section) {
                Some(section) => section,
                None => {
                    if parsing == CommitMessageParsing::Strict {
                        bail!("No commit type found for this: {}", commit.section);
                    }

                    if let Some(section) =
                        map.try_find_section((&raw_commit.message, &raw_commit.desc))
                    {
                        section
                    } else {
                        if exclude_unidentified {
                            bail!("Unidentified commit type");
                        }
                        "Unidentified".into()
                    }
                }
            };

            commit.section = section;
            commit
        }
        Err(e) => {
            if parsing == CommitMessageParsing::Strict {
                bail!("invalid commit syntax: {}", e);
            }

            let section = if let Some(section) =
                map.try_find_section((&raw_commit.message, &raw_commit.desc))
            {
                section
            } else {
                if exclude_unidentified {
                    bail!("Unidentified commit type");
                }
                "Unidentified".into()
            };

            Commit {
                section,
                scope: None,
                message: raw_commit.message,
            }
        }
    };

    let related_pr = if omit_pr_link && omit_thanks {
        None
    } else if related_pr_arg.is_some() {
        related_pr_arg
    } else {
        match try_get_repo(repo.to_owned()) {
            Some(repo) => match provider.related_pr(&repo, &raw_commit.sha) {
                Ok(related_pr) => Some(related_pr),
                Err(e) => {
                    eprintln!("error while requesting pr link: {}", e);
                    None
                }
            },
            None => None,
        }
    };

    if let Some(related_pr) = &related_pr {
        if !related_pr.is_pr && exclude_not_pr {
            bail!("The commit {} was not attached to pr.", raw_commit.sha);
        }

        if !omit_pr_link {
            commit
                .message
                .push_str(&format!(" in [{}]({})", related_pr.pr_id, related_pr.url));
        }

        if !omit_thanks {
            commit.message.push_str(&format!(
                " by [@{}]({})",
                related_pr.author, related_pr.author_link
            ));
        }
    } else if exclude_not_pr {
        bail!("Error: No upstream commit or pr was found.");
    };

    Ok(Some((
        commit.section,
        ReleaseSectionNote {
            scope: commit.scope,
            message: commit.message,
            context: vec![],
        },
    )))
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

    if raw.list_files.iter().any(|path| path == changelog_path) {
        return Response::Yes {
            reason: "The changelog was modified in this commit.".into(),
        };
    }

    let names = ["changelog", "log", "chglog", "notes"];

    let match_pat = |pat: &str| raw.message.contains(pat) || raw.desc.contains(pat);

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
                        "The pattern \"{pattern}\" was matched in the commit message or description."
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
            message: "fix: something !log".into(),
            desc: "".into(),
            sha: "".into(),
            list_files: vec![],
        };

        assert!(commit_should_be_ignored(&raw, "").bool());

        raw.message = "fix: something log".into();

        assert!(!commit_should_be_ignored(&raw, "").bool());
    }
}

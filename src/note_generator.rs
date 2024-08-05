use std::{io::Read, process::Command};

use crate::commit_parser::{parse_commit, Commit};
use anyhow::{anyhow, bail, Result};
use changelog::ReleaseSectionNote;
use log::debug;
use reqwest::{blocking::Client, header::USER_AGENT};
use serde_json::Value;

use crate::config::{CommitMessageParsing, GitProvider, MapMessageToSection};

#[derive(Clone, Debug)]
struct RawCommit {
    message: String,
    desc: String,
    sha: String,
    list_files: Vec<String>,
}

impl RawCommit {
    fn new() -> Self {
        let sha = last_commit_sha();

        Self {
            message: last_commit_message(),
            desc: last_commit_description(),
            list_files: commit_files_list(&sha),
            sha,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn get_release_note(
    changelog_path: String,
    parsing: &CommitMessageParsing,
    exclude_unidentified: bool,
    provider: &GitProvider,
    owner: &Option<String>,
    repo: &Option<String>,
    omit_pr_link: bool,
    omit_thanks: bool,
    map: &MapMessageToSection,
) -> Result<Option<(String, ReleaseSectionNote)>> {
    let raw_commit = RawCommit::new();

    if commit_should_be_ignored(&raw_commit, &changelog_path) {
        println!("Ignoring this commit.");
        return Ok(None);
    }

    let mut commit = match parse_commit(&raw_commit.message) {
        Ok(mut commit) => {
            let section = match map.map_section(&commit.section) {
                Some(section) => section,
                None => {
                    if *parsing == CommitMessageParsing::Strict {
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
            if *parsing == CommitMessageParsing::Strict {
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
    } else {
        match provider {
            GitProvider::Github => {
                // todo: use GITHUB_REPOSITORY env variable

                if let (Some(owner), Some(repo)) = (owner, repo) {
                    match request_related_pr(owner, repo, &raw_commit.sha) {
                        Ok(related_pr) => Some(related_pr),
                        Err(e) => {
                            eprintln!("error while requesting pr link: {}", e);
                            None
                        }
                    }
                } else {
                    eprintln!("Can't get information on the PR without the owner and repo name.");
                    eprintln!("This api is used to get information: https://api.github.com/repos/{{owner}}/{{repo}}/commits/{{sha}}/pulls");
                    None
                }
            }
            GitProvider::Other => None,
        }
    };

    if let Some(related_pr) = &related_pr {
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

fn commit_should_be_ignored(raw: &RawCommit, changelog_path: &str) -> bool {
    debug!("{:?}", raw);
    debug!("{:?}", changelog_path);

    if raw.list_files.iter().any(|path| path == changelog_path) {
        return true;
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
                return true;
            }
        }
    }

    false
}

fn last_commit_message() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn last_commit_description() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%b"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn last_commit_sha() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn commit_files_list(sha: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", sha])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .map(ToString::to_string)
        .collect()
}

#[derive(Debug, Clone)]
struct RelatedPr {
    pub url: String,
    pub pr_id: String,
    pub author: String,
    pub author_link: String,
}

fn request_related_pr(owner: &str, repo: &str, sha: &str) -> anyhow::Result<RelatedPr> {
    let api = format!("https://api.github.com/repos/{owner}/{repo}/commits/{sha}/pulls");

    let client = Client::new();

    let mut response = client
        .get(&api)
        .header(USER_AGENT, "my-github-client")
        .send()?;

    if response.status().is_success() {
        let mut body = String::new();
        response.read_to_string(&mut body)?;

        let json = serde_json::from_str::<Value>(&body)?;

        let obj = json.get(0).ok_or(anyhow!("no index 0"))?;

        let url = obj
            .get("html_url")
            .ok_or(anyhow!("no html_url found"))?
            .as_str()
            .unwrap()
            .to_string();

        let pr_id = obj
            .get("number")
            .ok_or(anyhow!("no number found"))?
            .as_u64()
            .unwrap();

        let pr_id = format!("#{}", pr_id);

        let author = obj
            .get("user")
            .ok_or(anyhow!("no user found"))?
            .get("login")
            .ok_or(anyhow!("no login found"))?
            .as_str()
            .unwrap()
            .to_string();

        let author_link = format!("https://github.com/{}", author);

        Ok(RelatedPr {
            url,
            author,
            pr_id,
            author_link,
        })
    } else {
        bail!(format!("GitHub API returned status: {}", response.status()))
    }
}

#[cfg(test)]
mod test {

    use crate::note_generator::{last_commit_description, last_commit_sha, RawCommit};

    use super::{commit_should_be_ignored, last_commit_message, request_related_pr};

    #[test]
    fn test() {
        let res = last_commit_message();

        dbg!(&res);

        let res = last_commit_description();

        dbg!(&res);

        let res = last_commit_sha();

        dbg!(&res);
    }

    #[test]
    fn pr() {
        let res = request_related_pr("wiiznokes", "fan-control", "74c8a3c").unwrap();

        dbg!(&res);
    }

    #[test]
    fn ignore_commit() {
        let mut raw = RawCommit {
            message: "fix: something !log".into(),
            desc: "".into(),
            sha: "".into(),
            list_files: vec![],
        };

        assert!(commit_should_be_ignored(&raw, ""));

        raw.message = "fix: something log".into();

        assert!(!commit_should_be_ignored(&raw, ""));
    }
}

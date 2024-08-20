use std::{collections::VecDeque, process::Command};

use anyhow::bail;
use semver::Version;

use crate::git_provider::DiffTags;

#[derive(Clone, Debug)]
pub struct RawCommit {
    pub author: String,
    pub title: String,
    pub body: String,
    pub sha: String,
    pub list_files: Vec<String>,
}

impl RawCommit {
    pub fn last_from_fs() -> Self {
        let sha = last_commit_sha();
        Self::from_sha(&sha)
    }

    pub fn from_sha(sha: &str) -> Self {
        Self {
            author: commit_author(sha),
            title: commit_title(sha),
            body: commit_body(sha),
            list_files: commit_files(sha),
            sha: sha.into(),
        }
    }

    pub fn short_commit(&self) -> &str {
        &self.sha[0..7]
    }
}

pub fn last_commit_sha() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr))
    }

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit_author(sha: &str) -> String {
    let output = Command::new("git")
        .args(["show", "-s", "--pretty=%an", sha])
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr))
    }

    String::from_utf8(output.stdout)
        .expect("Failed to parse UTF-8")
        .trim()
        .into()
}

pub fn commit_title(sha: &str) -> String {
    let output = Command::new("git")
        .args(["show", "-s", "--pretty=%s", sha])
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr))
    }

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit_body(sha: &str) -> String {
    let output = Command::new("git")
        .args(["show", "-s", "--pretty=%b", sha])
        .output()
        .expect("Failed to execute git command");
    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr))
    }

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit_files(sha: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", sha])
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr))
    }

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .map(ToString::to_string)
        .collect()
}

pub fn commits_between_tags(tags: &DiffTags) -> Vec<String> {
    let prev = tags.prev.as_deref().unwrap_or("HEAD");
    let tags = format!("{}..{}", prev, tags.new);

    let output = Command::new("git")
        .args(["log", "--oneline", &tags, "--format=format:%H"])
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        panic!(
            "commits_between_tags error: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .rev()
        .map(ToString::to_string)
        .collect()
}

pub fn tags_list() -> anyhow::Result<VecDeque<Version>> {
    let output = Command::new("git")
        .arg("tag")
        .output()
        .expect("Failed to execute git command");

    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr))
    }

    let mut tags = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .map(Version::parse)
        .collect::<Result<Vec<Version>, _>>()?;

    tags.sort();

    let tags = tags.into();

    debug!("tags: {:?}", tags);

    Ok(tags)
}

pub fn try_get_repo(repo: Option<String>) -> Option<String> {
    let repo = match repo {
        Some(repo) => Some(repo),
        None => std::env::var("GITHUB_REPOSITORY").ok(),
    };

    if repo.is_none() {
        eprintln!("couldn't get the repo name. Example: \"wiiznokes/changelog-generator\".");
    }

    repo
}

impl DiffTags {
    pub fn new(new: Option<String>, prev: Option<String>) -> anyhow::Result<Self> {
        let new = match new {
            Some(new) => Version::parse(&new)?,
            None => match tags_list()?.pop_back() {
                Some(v) => v,
                None => {
                    bail!("No version provided. Can't fall back to last tag because there is none.")
                }
            },
        };

        if let Some(prev) = &prev {
            let prev = Version::parse(prev)?;

            if prev > new {
                bail!(
                    "The new version {} is inferior to the previous version {}",
                    new.to_string(),
                    prev.to_string()
                )
            }
        }

        Ok(DiffTags {
            prev,
            new: new.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "github problem with fetching tags ?"]
    fn test() {
        let raw = RawCommit::last_from_fs();

        dbg!(&raw);

        let res = tags_list();

        dbg!(&res);

        let res = commits_between_tags(&DiffTags {
            prev: None,
            new: "0.1.7".into(),
        });

        dbg!(&res);
    }
}

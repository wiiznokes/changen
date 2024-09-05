use std::{collections::VecDeque, process::Command, str::FromStr};

use anyhow::bail;
use changelog::Version;

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
    pub fn last_from_fs<R: Repository>(r: &R) -> Self {
        let sha = r.last_commit_sha();
        Self::from_sha(r, &sha)
    }

    pub fn from_sha<R: Repository>(r: &R, sha: &str) -> Self {
        Self {
            author: r.commit_author(sha),
            title: r.commit_title(sha),
            body: r.commit_body(sha),
            list_files: r.commit_files(sha),
            sha: sha.into(),
        }
    }

    pub fn short_commit(&self) -> &str {
        &self.sha[0..7]
    }
}

/// Period is use to retrieve a list of commits during two refs (tag, sha).
/// fields are optional because it can represent the commit 0 and the HEAD.
#[derive(Debug, Clone)]
pub struct Period {
    pub since: Option<String>,
    pub until: Option<String>,
}

pub trait Repository {
    fn last_commit_sha(&self) -> String;

    fn commit_author(&self, sha: &str) -> String;

    fn commit_title(&self, sha: &str) -> String;

    fn commit_body(&self, sha: &str) -> String;

    fn commit_files(&self, sha: &str) -> Vec<String>;

    fn commits_between_tags(&self, tags: &Period) -> Vec<String>;

    /// Most recent at the end
    fn tags_list(&self) -> anyhow::Result<VecDeque<Version>>;
}

/// Represent the real implementation of the Repository trait
#[derive(Default)]
pub struct Fs;

impl Repository for Fs {
    fn last_commit_sha(&self) -> String {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("Failed to execute git command");

        if !output.status.success() {
            panic!("{}", String::from_utf8_lossy(&output.stderr))
        }

        String::from_utf8(output.stdout).unwrap().trim().into()
    }

    fn commit_author(&self, sha: &str) -> String {
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

    fn commit_title(&self, sha: &str) -> String {
        let output = Command::new("git")
            .args(["show", "-s", "--pretty=%s", sha])
            .output()
            .expect("Failed to execute git command");

        if !output.status.success() {
            panic!("{}", String::from_utf8_lossy(&output.stderr))
        }

        String::from_utf8(output.stdout).unwrap().trim().into()
    }

    fn commit_body(&self, sha: &str) -> String {
        let output = Command::new("git")
            .args(["show", "-s", "--pretty=%b", sha])
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            panic!("{}", String::from_utf8_lossy(&output.stderr))
        }

        String::from_utf8(output.stdout).unwrap().trim().into()
    }

    fn commit_files(&self, sha: &str) -> Vec<String> {
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

    fn commits_between_tags(&self, tags: &Period) -> Vec<String> {
        let until = tags.until.as_deref().unwrap_or("HEAD");

        let period = match &tags.since {
            Some(since) => format!("{}..{}", since, until),
            None => until.to_string(),
        };

        let output = Command::new("git")
            .args(["log", "--oneline", &period, "--format=format:%H"])
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

    fn tags_list(&self) -> anyhow::Result<VecDeque<Version>> {
        let output = Command::new("git")
            .arg("tag")
            .output()
            .expect("Failed to execute git command");

        if !output.status.success() {
            panic!("{}", String::from_utf8_lossy(&output.stderr))
        }

        let mut tags = Vec::new();

        for tag in String::from_utf8(output.stdout)?.trim().lines() {
            match Version::from_str(tag) {
                Ok(v) => tags.push(v),
                Err(e) => {
                    eprintln!("incorrect semver tag {tag}: {e}");
                }
            }
        }

        tags.sort();

        let tags = tags.into();

        debug!("tags: {:?}", tags);

        Ok(tags)
    }
}

pub fn try_detect_new_version<R: Repository>(
    r: &R,
    new: Option<Version>,
) -> anyhow::Result<Version> {
    match new {
        Some(new) => Ok(new),
        None => match r.tags_list()?.pop_back() {
            Some(v) => Ok(v),
            None => {
                bail!("No version provided. Can't fall back to last tag because there is none.")
            }
        },
    }
}

impl DiffTags {
    pub fn new(new: Version, prev: Option<Version>) -> anyhow::Result<Self> {
        let prev = if let Some(prev) = prev {
            if prev > new {
                bail!(
                    "The new version {} is inferior to the previous version {}",
                    new.to_string(),
                    prev.to_string()
                )
            }
            Some(prev)
        } else {
            None
        };

        Ok(DiffTags { prev, new })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore = "github problem with fetching tags ?"]
    fn test() {
        let r = Fs;

        let raw = RawCommit::last_from_fs(&r);

        dbg!(&raw);

        let res = r.tags_list();

        dbg!(&res);

        let res = r.commits_between_tags(&Period {
            since: Some("0.1.5".into()),
            until: None,
        });

        dbg!(&res);
    }
}

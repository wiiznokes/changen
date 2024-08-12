use std::{collections::VecDeque, process::Command};

use cached::proc_macro::cached;

#[derive(Clone, Debug)]
pub struct RawCommit {
    pub message: String,
    pub desc: String,
    pub sha: String,
    pub list_files: Vec<String>,
}

impl RawCommit {
    pub fn last_from_fs() -> Self {
        let sha = last_commit_sha();

        Self {
            message: commit_message(&sha),
            desc: commit_description(&sha),
            list_files: commit_files(&sha),
            sha,
        }
    }

    pub fn from_sha(sha: &str) -> Self {
        Self {
            message: commit_message(sha),
            desc: commit_description(sha),
            list_files: commit_files(sha),
            sha: sha.into(),
        }
    }
}

pub fn last_commit_sha() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit_message(sha: &str) -> String {
    let output = Command::new("git")
        .args(["show", "-s", "--pretty=%s", sha])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit_description(sha: &str) -> String {
    let output = Command::new("git")
        .args(["show", "-s", "--pretty=%b", sha])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit_files(sha: &str) -> Vec<String> {
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

pub fn commits_between_tags(tags: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--oneline", tags, "--format=format:%H"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .rev()
        .map(ToString::to_string)
        .collect()
}

#[cached]
pub fn tags_list() -> VecDeque<String> {
    let output = Command::new("git")
        .arg("tag")
        .output()
        .expect("Failed to execute git command");

    let tags = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
        .map(ToString::to_string)
        .collect();

    debug!("tags: {:?}", tags);

    tags
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let raw = RawCommit::last_from_fs();

        dbg!(&raw);

        let res = tags_list();

        dbg!(&res);
    }
}

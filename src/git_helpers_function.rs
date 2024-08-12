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
            message: last_commit_message(),
            desc: last_commit_description(),
            list_files: commit_files_list(&sha),
            sha,
        }
    }

    pub fn from_sha(sha: &str) -> Self {
        Self {
            message: last_commit_message(),
            desc: last_commit_description(),
            list_files: commit_files_list(&sha),
            sha: sha.into(),
        }
    }
}

pub fn commit_message(sha: &str) -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn last_commit_message() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn last_commit_description() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%b"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn last_commit_sha() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

pub fn commit_files_list(sha: &str) -> Vec<String> {
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

pub fn commit_between_tags_list(tags: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["log", "--oneline", tags, "--format=format:%H"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .lines()
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
        eprintln!("Couln't get the repo name. Example: \"wizznokes/changelog-generator\".");
    }

    repo
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let res = last_commit_message();

        dbg!(&res);

        let res = last_commit_description();

        dbg!(&res);

        let res = last_commit_sha();

        dbg!(&res);

        let res = tags_list();

        dbg!(&res);
    }
}

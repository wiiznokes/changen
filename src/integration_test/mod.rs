use std::collections::VecDeque;

use semver::Version;

use crate::repository::{Period, RawCommit, Repository};

mod test1;

struct Tag {
    pub name: String,
    pub sha: String,
}

struct FsTest {
    pub commits: Vec<RawCommit>,
    pub tags: Vec<Tag>,
}

impl Repository for FsTest {
    fn last_commit_sha(&self) -> String {
        self.commits.last().unwrap().sha.clone()
    }

    fn commit_author(&self, sha: &str) -> String {
        self.commits
            .iter()
            .find(|e| e.sha == sha)
            .unwrap()
            .author
            .clone()
    }

    fn commit_title(&self, sha: &str) -> String {
        self.commits
            .iter()
            .find(|e| e.sha == sha)
            .unwrap()
            .title
            .clone()
    }

    fn commit_body(&self, sha: &str) -> String {
        self.commits
            .iter()
            .find(|e| e.sha == sha)
            .unwrap()
            .body
            .clone()
    }

    fn commit_files(&self, sha: &str) -> Vec<String> {
        self.commits
            .iter()
            .find(|e| e.sha == sha)
            .unwrap()
            .list_files
            .clone()
    }

    fn commits_between_tags(&self, tags: &Period) -> Vec<String> {
        let mut res = Vec::new();

        let start = tags
            .since
            .as_deref()
            .map(
                |repo_ref| match self.tags.iter().find(|e| e.name == repo_ref) {
                    Some(tag) => self.commits.iter().position(|e| e.sha == tag.sha).unwrap(),
                    None => self.commits.iter().position(|e| e.sha == repo_ref).unwrap(),
                },
            )
            .unwrap_or(0);

        let end = tags
            .until
            .as_deref()
            .map(
                |repo_ref| match self.tags.iter().find(|e| e.name == repo_ref) {
                    Some(tag) => self.commits.iter().position(|e| e.sha == tag.sha).unwrap(),
                    None => self.commits.iter().position(|e| e.sha == repo_ref).unwrap(),
                },
            )
            .unwrap_or(self.commits.len());

        for e in &self.commits[start..end] {
            res.push(e.sha.clone());
        }
        res
    }

    fn tags_list(&self) -> anyhow::Result<VecDeque<Version>> {
        Ok(self
            .tags
            .iter()
            .filter_map(|e| Version::parse(&e.name).ok())
            .collect())
    }
}

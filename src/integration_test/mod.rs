use std::{collections::VecDeque, fs::File, io::Read, path::Path, str::FromStr, sync::LazyLock};

use changelog::{de::parse_changelog, ChangeLog, Version};

use crate::{
    config::{CommitMessageParsing, Generate},
    git_provider::GitProvider,
    repository::{Period, RawCommit, Repository},
};

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

        for e in &self.commits[start..=end] {
            res.push(e.sha.clone());
        }
        res
    }

    fn tags_list(&self) -> anyhow::Result<VecDeque<Version>> {
        Ok(self
            .tags
            .iter()
            .filter_map(|e| Version::from_str(&e.name).ok())
            .collect())
    }
}

static DEFAULT_GENERATE: LazyLock<Generate> = LazyLock::new(|| Generate {
    file: None,
    map: None,
    parsing: CommitMessageParsing::Smart,
    exclude_unidentified: true,
    exclude_not_pr: false,
    provider: GitProvider::None,
    repo: None,
    omit_pr_link: false,
    omit_thanks: false,
    stdout: false,
    specific: None,
    milestone: None,
    since: None,
    until: None,
});

fn raw_commit(title: &str, sha: &str) -> RawCommit {
    RawCommit {
        author: "wiiznokes".to_owned(),
        title: title.to_owned(),
        body: "".to_owned(),
        sha: sha.to_owned(),
        list_files: vec![],
    }
}

fn tag(name: &str, sha: &str) -> Tag {
    Tag {
        name: name.to_owned(),
        sha: sha.to_owned(),
    }
}

fn read_file<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    let mut buf = String::new();

    let mut file = File::open(path)?;
    file.read_to_string(&mut buf)?;

    Ok(buf)
}

fn read_changelog<P: AsRef<Path>>(path: P) -> anyhow::Result<ChangeLog> {
    let buf = read_file(path)?;

    let changelog = parse_changelog(&buf)?;

    Ok(changelog)
}

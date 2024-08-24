use std::{collections::HashMap, fmt::Display};

use anyhow::bail;
use semver::Version;

use crate::repository::RawCommit;

mod github;

#[derive(clap::ValueEnum, Debug, Clone, Default, PartialEq, Eq)]
pub enum GitProvider {
    #[default]
    Github,
    None,
}
// todo: use derive_more::Display when this issue is resolved
// https://github.com/JelteF/derive_more/issues/216
impl Display for GitProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitProvider::Github => write!(f, "github"),
            GitProvider::None => write!(f, "none "),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelatedPr {
    pub url: String,
    pub pr_id: String,
    pub author: Option<String>,
    pub author_link: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub merge_commit: Option<String>,
    pub is_pr: bool,
}

/// Represent two or one tag to produce a diff link.
#[derive(Debug, Clone)]
pub struct DiffTags {
    pub prev: Option<Version>,
    pub new: Version,
}

impl GitProvider {
    pub fn related_pr(&self, repo: &str, sha: &str) -> anyhow::Result<RelatedPr> {
        match self {
            GitProvider::Github => github::request_related_pr(repo, sha),
            GitProvider::None => bail!("No git provider was selected"),
        }
    }

    pub fn diff_link(&self, repo: &str, diff_tags: &DiffTags) -> anyhow::Result<String> {
        match self {
            GitProvider::Github => github::diff_link(repo, diff_tags),
            GitProvider::None => bail!("No git provider was selected"),
        }
    }

    pub fn release_link(&self, repo: &str, tag: &str) -> anyhow::Result<String> {
        match self {
            GitProvider::Github => github::release_link(repo, tag),
            GitProvider::None => bail!("No git provider was selected"),
        }
    }

    pub fn milestone_prs(&self, repo: &str, milestone: &str) -> anyhow::Result<Vec<RelatedPr>> {
        match self {
            GitProvider::Github => github::milestone_prs(repo, milestone),
            GitProvider::None => bail!("No git provider was selected"),
        }
    }

    pub fn last_prs(&self, repo: &str, n: usize) -> anyhow::Result<HashMap<String, RelatedPr>> {
        let prs = match self {
            GitProvider::Github => github::last_prs(repo, n),
            GitProvider::None => bail!("No git provider was selected"),
        }?;

        let mut hashmap = HashMap::new();

        for pr in prs {
            hashmap.insert(pr.merge_commit.clone().unwrap(), pr);
        }

        Ok(hashmap)
    }

    /// Fallback function
    pub fn offline_related_pr(&self, repo: &str, raw_commit: &RawCommit) -> Option<RelatedPr> {
        match self {
            GitProvider::Github => github::offline_related_pr(repo, raw_commit),
            GitProvider::None => None,
        }
    }
}

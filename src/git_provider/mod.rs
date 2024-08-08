use std::fmt::Display;

use anyhow::bail;

mod github;

#[derive(clap::ValueEnum, Debug, Clone, Default, PartialEq, Eq)]
pub enum GitProvider {
    #[default]
    Github,
    Other,
}

impl Display for GitProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitProvider::Github => write!(f, "github"),
            GitProvider::Other => write!(f, "other "),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RelatedPr {
    pub url: String,
    pub pr_id: String,
    pub author: String,
    pub author_link: String,
}

#[derive(Debug, Clone)]
pub struct DiffTags {
    pub prev: Option<String>,
    pub current: String,
}

impl GitProvider {
    pub fn related_pr(&self, repo: &str, sha: &str) -> anyhow::Result<RelatedPr> {
        match self {
            GitProvider::Github => github::request_related_pr(repo, sha),
            GitProvider::Other => bail!("No git provider was selected"),
        }
    }

    pub fn diff_link(&self, repo: &str, diff_tags: &DiffTags) -> anyhow::Result<String> {
        match self {
            GitProvider::Github => github::diff_link(repo, diff_tags),
            GitProvider::Other => bail!("No git provider was selected"),
        }
    }

    pub fn release_link(&self, repo: &str, tag: &str) -> anyhow::Result<String> {
        match self {
            GitProvider::Github => github::release_link(repo, tag),
            GitProvider::Other => bail!("No git provider was selected"),
        }
    }
}

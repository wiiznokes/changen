use std::env;

use anyhow::{anyhow, bail};
use reqwest::{
    blocking::{Client, RequestBuilder},
    header::USER_AGENT,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::utils::{self, TextInterpolate};

use super::*;

trait ClientExt {
    fn bearer_auth_env(self, name: &str) -> Self;
}

impl ClientExt for RequestBuilder {
    fn bearer_auth_env(self, name: &str) -> Self {
        if let Ok(token) = env::var(name) {
            self.bearer_auth(token)
        } else {
            self
        }
    }
}

fn request_github(api: &str) -> anyhow::Result<Value> {
    let client = Client::new();

    let response = client
        .get(api)
        .header(USER_AGENT, "my-github-client")
        .bearer_auth_env("GITHUB_TOKEN")
        .send()?;

    if response.status().is_success() {
        let obj = response.json()?;
        Ok(obj)
    } else {
        bail!(format!(
            "GitHub API returned status for {}: {}",
            api,
            response.status()
        ))
    }
}

fn request_github_graphql(query: &str) -> anyhow::Result<Value> {
    let client = Client::new();

    let request_body = json!({
        "query": query,
    });

    let response = client
        .post("https://api.github.com/graphql")
        .header(USER_AGENT, "my-github-client")
        .bearer_auth_env("GITHUB_TOKEN")
        .json(&request_body)
        .send()?;

    if response.status().is_success() {
        let obj = response.json()?;
        Ok(obj)
    } else {
        bail!(format!(
            "GitHub API graphql returned status {}",
            response.status()
        ))
    }
}

pub fn request_related_pr(repo: &str, sha: &str) -> anyhow::Result<Option<RelatedPr>> {
    let json = request_github(&format!(
        "https://api.github.com/repos/{repo}/commits/{sha}/pulls"
    ))?;

    match json.get(0) {
        Some(obj) => {
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

            let message = obj
                .get("title")
                .ok_or(anyhow!("no title found"))?
                .to_string();
            let body = obj
                .get("body")
                .ok_or(anyhow!("no title found"))?
                .to_string();

            Ok(Some(RelatedPr {
                url,
                author,
                pr_id,
                author_link,
                message,
                body,
                merge_commit: Some(sha.into()),
            }))
        }
        None => Ok(None),
    }
}

pub fn diff_link(repo: &str, diff_tags: &DiffTags) -> anyhow::Result<String> {
    let base = format!("https://github.com/{repo}");

    let link = match &diff_tags.prev {
        Some(prev) => {
            format!("{base}/compare/{prev}...{}", diff_tags.current)
        }
        None => {
            format!("{base}/commits/{}", diff_tags.current)
        }
    };

    Ok(link)
}

pub fn release_link(repo: &str, tag: &str) -> anyhow::Result<String> {
    Ok(format!("https://github.com/{repo}/releases/tag/{tag}"))
}

pub fn milestone_prs(repo: &str, milestone: &str) -> anyhow::Result<Vec<RelatedPr>> {
    let json = request_github(&format!(
        "https://api.github.com/search/issues?q=repo:{repo}+is:pr+is:merged+milestone:{milestone}"
    ))?;

    let array = json
        .get("items")
        .expect("no items")
        .as_array()
        .expect("not an array");

    let mut res = Vec::new();

    for obj in array {
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

        let message = obj
            .get("title")
            .ok_or(anyhow!("no title found"))?
            .to_string();
        let body = obj
            .get("body")
            .ok_or(anyhow!("no title found"))?
            .to_string();

        res.push(RelatedPr {
            url,
            pr_id,
            author,
            author_link,
            message,
            body,
            merge_commit: None,
        });
    }

    Ok(res)
}

pub fn last_prs(repo: &str, n: usize) -> anyhow::Result<Vec<RelatedPr>> {
    let query = include_str!("./last_prs.graphql");

    let mut interpolate = TextInterpolate::new(query.into(), "###", "");

    let repo = utils::Repo::try_from(repo)?;

    interpolate.interpolate("name", &repo.name);
    interpolate.interpolate("owner", &repo.owner);
    interpolate.interpolate("first", &n.to_string());

    let value = request_github_graphql(&interpolate.text())?;

    #[derive(Debug, Deserialize)]
    struct Response {
        data: Data,
    }

    #[derive(Debug, Deserialize)]
    struct Data {
        repository: Repository,
    }

    #[derive(Debug, Deserialize)]
    struct Repository {
        #[serde(rename = "pullRequests")]
        pull_requests: PullRequests,
    }

    #[derive(Debug, Deserialize)]
    struct PullRequests {
        nodes: Vec<PullRequest>,
    }

    #[derive(Debug, Deserialize)]
    struct PullRequest {
        author: Author,
        body: String,
        #[serde(rename = "mergeCommit")]
        merge_commit: MergeCommit,
        number: u32,
        title: String,
        url: String,
    }

    #[derive(Debug, Deserialize)]
    struct Author {
        login: String,
    }

    #[derive(Debug, Deserialize)]
    struct MergeCommit {
        oid: String,
    }

    let response = serde_json::value::from_value::<Response>(value)?;

    let res = response
        .data
        .repository
        .pull_requests
        .nodes
        .into_iter()
        .map(|e| RelatedPr {
            url: e.url,
            pr_id: format!("#{}", e.number),
            author_link: format!("https://github.com/{}", e.author.login),
            author: e.author.login,
            message: e.title,
            body: e.body,
            merge_commit: Some(e.merge_commit.oid),
        })
        .collect();

    Ok(res)
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn pr() {
        let res = request_related_pr("wiiznokes/fan-control", "74c8a3c").unwrap();

        dbg!(&res);

        let res = request_related_pr("wiiznokes/changelog-generator", "84d7fa4").unwrap();

        dbg!(&res);
    }

    #[test]
    fn link() {
        let res = diff_link(
            "wiiznokes/fan-control",
            &DiffTags {
                prev: None,
                current: "v0.1.0".into(),
            },
        )
        .unwrap();

        assert_eq!(
            res,
            "https://github.com/wiiznokes/fan-control/commits/v0.1.0".to_owned()
        );

        let res = diff_link(
            "wiiznokes/fan-control",
            &DiffTags {
                prev: Some("v2024.7".into()),
                current: "v2024.7.30".into(),
            },
        )
        .unwrap();

        assert_eq!(
            res,
            "https://github.com/wiiznokes/fan-control/compare/v2024.7...v2024.7.30".to_owned()
        );
    }

    #[test]
    fn milestone() {
        let res = milestone_prs("iced-rs/iced", "0.13").unwrap();

        dbg!(&res);
    }

    #[test]
    fn lasts() {
        let res = last_prs("iced-rs/iced", 3).unwrap();

        dbg!(&res);
    }
}

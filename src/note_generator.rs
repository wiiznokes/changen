use std::{io::Read, process::Command};

use anyhow::{anyhow, bail, Ok};
use reqwest::{blocking::Client, header::USER_AGENT};
use serde_json::Value;

fn last_commit_title() -> String {
    let output = Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .output()
        .expect("Failed to execute git command");

    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn last_commit_message() -> String {
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

#[derive(Debug, Clone)]
struct RelatedPr {
    pub url: String,
    pub author: String,
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

        let author = obj
            .get("user")
            .ok_or(anyhow!("no user found"))?
            .get("login")
            .ok_or(anyhow!("no login found"))?
            .as_str()
            .unwrap()
            .to_string();

        Ok(RelatedPr { url, author })
    } else {
        bail!(format!("GitHub API returned status: {}", response.status()))
    }
}

#[cfg(test)]
mod test {

    use crate::note_generator::{last_commit_message, last_commit_sha};

    use super::{last_commit_title, request_related_pr};

    #[test]
    fn test() {
        let res = last_commit_title();

        dbg!(&res);

        let res = last_commit_message();

        dbg!(&res);

        let res = last_commit_sha();

        dbg!(&res);
    }

    #[test]
    fn pr() {
        let res = request_related_pr("wiiznokes", "fan-control", "74c8a3c").unwrap();

        dbg!(&res);
    }
}

use std::io::Read;

use anyhow::{anyhow, bail, Ok};
use reqwest::{blocking::Client, header::USER_AGENT};
use serde_json::Value;

use super::*;

fn request_github(api: &str) -> anyhow::Result<Value> {
    let client = Client::new();

    let mut response = client
        .get(api)
        .header(USER_AGENT, "my-github-client")
        .send()?;

    if response.status().is_success() {
        let mut body = String::new();
        response.read_to_string(&mut body)?;

        let json = serde_json::from_str::<Value>(&body)?;

        Ok(json)
    } else {
        bail!(format!(
            "GitHub API returned status for {}: {}",
            api,
            response.status()
        ))
    }
}

pub fn request_related_pr(repo: &str, sha: &str) -> anyhow::Result<RelatedPr> {
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

            Ok(RelatedPr {
                url,
                author,
                pr_id,
                author_link,
                is_pr: true,
            })
        }
        None => {
            let json = request_github(&format!(
                "https://api.github.com/repos/{repo}/commits/{sha}"
            ))?;

            let url = json
                .get("html_url")
                .ok_or(anyhow!("no html_url found"))?
                .as_str()
                .unwrap()
                .to_string();

            let author = json
                .get("author")
                .ok_or(anyhow!("no user found"))?
                .get("login")
                .ok_or(anyhow!("no login found"))?
                .as_str()
                .unwrap()
                .to_string();

            let author_link = format!("https://github.com/{}", author);

            Ok(RelatedPr {
                url,
                author,
                pr_id: "commit".into(),
                author_link,
                is_pr: false,
            })
        }
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

#[cfg(test)]
mod test {
    use super::{diff_link, request_related_pr, DiffTags};

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
}

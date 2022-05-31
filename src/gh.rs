use anyhow::{anyhow, Result};
use regex::{Captures, Regex};
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::env;
use tracing::info;

#[derive(Deserialize, Debug, Default)]
pub struct GithubRelease {
    pub(crate) tag_name: String,
    pub(crate) published_at: String,
    pub(crate) body: String,
    pub(crate) html_url: String,
}

pub fn get_repo_details(repo: &str) -> Result<Captures, String> {
    let m = match Regex::new(r"https://github.com/([\S]+)/([\S]+)") {
        Ok(value) => {
            match value.is_match(repo) {
                true => {
                    let m = value.captures(repo).unwrap();
                    //println!("{:?}",&m);
                    Ok(m)
                }
                false => Err(format!("No valid repo found for {}", repo)),
            }
        }
        Err(_) => Err(format!("Failed to parse the repo {}", repo)),
    };
    m
}

#[allow(dead_code)]
pub(crate) struct Issue<'a> {
    pub(crate) headers: HeaderMap,
    pub(crate) client: Client,
    pub(crate) release_url: &'a str,
    pub(crate) release_body: &'a str,
    pub(crate) repo_name: &'a str,
    pub(crate) release_version: &'a str,
    pub(crate) labels: &'a Option<String>,
}

impl<'a> Issue<'a> {
    pub fn new(
        headers: HeaderMap,
        c: Client,
        release_url: &'a str,
        release_body: &'a str,
        repo_name: &'a str,
        release_version: &'a str,
        labels: &'a Option<String>,
    ) -> Self {
        Issue {
            headers,
            client: c,
            release_url,
            release_body,
            repo_name,
            release_version,
            labels,
        }
    }

    pub async fn create_issue(&self) -> Result<()> {
        let mut map = Map::new();
        let he = &self.headers.to_owned();
        let github_org: &str =
            &env::var("GITHUB_REPOSITORY").expect("Missing input parameter: repo");
        let current_repo: Vec<&str> = github_org.split('/').collect();
        map.insert(
            "title".to_string(),
            Value::String(format!(
                "New version of {} {} available",
                &self.repo_name, &self.release_version
            )),
        );
        map.insert(
            "body".to_string(),
            Value::String(format!(
                " Upstream new release {} available at {} \n\n **Release Details:** \n {}",
                &self.repo_name, &self.release_url, &self.release_body
            )),
        );

        if let Some(labels) = &self.labels {
            let m: Vec<Value> = labels
                .split(',')
                .map(|s| Value::String(s.to_string()))
                .collect();
            map.insert("labels".to_string(), Value::Array(m));
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/issues",
            current_repo[0], current_repo[1]
        );

        let issue_response = self
            .client
            .post(url)
            .headers(he.to_owned())
            .json(&serde_json::json!(&map))
            .send()
            .await?;
        if issue_response.status() == 201 {
            info!(
                "Successfully issue created with details for repo: {}",
                &self.repo_name
            );
            Ok(())
        } else {
            Err(anyhow!(format!(
                "Failed to create GH issue for {} ",
                &self.repo_name
            )))
        }
    }
}

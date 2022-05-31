use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use feed_rs::{model::Entry, parser};
use reqwest::{header::HeaderMap, header::HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{error, info};

use crate::{get_repo_details, GithubRelease, Issue};

#[derive(Deserialize, Serialize, Debug)]
pub enum InputType {
    Github,
    Rss,
}

impl FromStr for InputType {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Github" => Ok(InputType::Github),
            "Rss" => Ok(InputType::Rss),
            _ => Err(()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Input {
    pub token: String,
    pub input_type: InputType,
    pub repo: String,
    pub days: i64,
    pub labels: Option<String>,
}

impl Input {
    pub fn new(ty: String, token: String, repo: String, days: i64, labels: Option<String>) -> Self {
        let input_type = InputType::from_str(&ty).unwrap();
        Input {
            input_type,
            token,
            repo,
            days,
            labels,
        }
    }

    pub fn header(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let token = HeaderValue::from_str(&self.token).unwrap();
        headers.insert("Authorization", token);
        headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());
        headers
    }

    pub fn check_new_release(&self, published_date: String) -> bool {
        let dt = Utc::now();
        let x = dt - Duration::days(self.days);
        let datetime = published_date.parse::<DateTime<Utc>>();
        if let Ok(pub_date) = datetime {
            if pub_date > x {
                return true;
            }
        }
        false
    }

    pub async fn gh(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let mut headers = self.header();
        for repo in self.repo.split(',') {
            let validate_repo = get_repo_details(repo);
            if let Err(e) = get_repo_details(repo) {
                error!("Failed to parse input repo url : {}", e);
                return Ok(());
            }
            let final_org = validate_repo.unwrap();
            let (parsed_org, parsed_repo) = (&final_org[1], &final_org[2]);
            headers.insert("User-Agent", repo.parse().unwrap());
            let url = format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                &parsed_org, &parsed_repo
            );
            info!("Checking for the latest release in repo : {}", &parsed_repo);
            let resp = client.get(&url).headers(headers.to_owned()).send().await?;
            match resp.status() {
                StatusCode::OK => {
                    info!(
                        "Checking if there any latest release in past {} days in repo: {}",
                        &self.days, &parsed_repo
                    );
                    let rbody: GithubRelease = resp.json::<GithubRelease>().await?;
                    if Self::check_new_release(self, rbody.published_at) {
                        let issue = Issue::new(
                            headers.clone(),
                            client.clone(),
                            &rbody.html_url,
                            &rbody.body,
                            parsed_repo,
                            &rbody.tag_name,
                            &self.labels,
                        );
                        issue.create_issue().await?;
                        info!("New release found and github issue created");
                    } else {
                        info!(
                            "No new release found in past {} days in repo: {}",
                            &self.days, &parsed_repo
                        );
                    }
                }
                StatusCode::PAYLOAD_TOO_LARGE => {
                    info!("Request payload is too large!");
                }
                s => info!("Received response status: {:?}", s),
            };
        }
        Ok(())
    }

    pub async fn rss(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let mut headers = self.header();
        for url in self.repo.split(',') {
            headers.insert("User-Agent", url.parse().unwrap());
            let xml = client.get(url).send().await?.text().await?;
            let f = parser::parse(xml.as_bytes()).unwrap();
            let title = if let Some(t) = f.title {
                t.content
            } else {
                "".to_string()
            };
            // get the latest feed date
            let updated_date = if let Some(u) = f.updated {
                u
            } else {
                return Ok(());
            };
            // get the details based on the date
            if Self::check_new_release(self, updated_date.to_string()) {
                info!("New release found :{}", title);
                let release_data = &f
                    .entries
                    .iter()
                    .filter(|f| f.updated.unwrap() == updated_date)
                    .cloned()
                    .collect::<Vec<Entry>>()[0];
                let body = html2md::parse_html(
                    release_data
                        .content
                        .to_owned()
                        .unwrap()
                        .body
                        .unwrap()
                        .as_str(),
                );
                let issue = Issue::new(
                    headers.clone(),
                    client.clone(),
                    &release_data.links[0].href,
                    &body,
                    &title,
                    "",
                    &self.labels,
                );
                issue.create_issue().await?
            } else {
                info!(
                    "No new release found in past {} days for: {}",
                    &self.days, title
                );
            }
        }
        Ok(())
    }
}

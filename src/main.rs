use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use regex::{Captures, Regex};
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde::Deserialize;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use std::{env};

#[derive(Deserialize, Debug)]
struct Release {
    name: String,
    tag_name: String,
    published_at: String,
    body: String,
    html_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    process().await?;
    Ok(())
}

async fn process() -> Result<(), reqwest::Error> {
    let repo: &str = &env::var("INPUT_REPO").expect("Missing input parameter: repo");
    let days: i64 = env::var("INPUT_DAYS")
        .unwrap_or_else(|_| 1.to_string())
        .parse::<i64>()
        .unwrap();
    let token = &format!(
        "token {}",
        env::var("INPUT_GITHUB_TOKEN").expect("Missing input parameter: github_token")
    );

    // convert comma seperate repos to vector
    let repo_list: Vec<&str> = repo.split(',').collect();
    let repo_list_string: Vec<String> = repo_list.iter().map(|s| s.to_string()).collect();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", token.parse().unwrap());
    headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());

    let tasks: Vec<_> = repo_list_string
        .into_iter()
        .map(|repo| {
            let mut headers_clone = headers.to_owned();
            let arc_client = Arc::new(reqwest::Client::new());
            let client = arc_client.clone();
            tokio::spawn(async move {
                let m = match get_repo_details(&repo) {
                    Ok(cap) => Some(cap),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                };

                //TODO check if None and return back
                let final_org = m.unwrap();
                let (parsed_org, parsed_repo) = (&final_org[1], &final_org[2]);

                headers_clone.insert("User-Agent", repo.parse().unwrap());
                let url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    &parsed_org, &parsed_repo
                );
                let res = client
                    .get(&url)
                    .headers(headers_clone.to_owned())
                    .send()
                    .await;
                let response = res.unwrap();
                println!("val {}", response.status());
                if response.status() == 200 {
                    let body = response.json::<Release>().await;
                    let get_response = body.unwrap();
                    if get_response.check_new_release(days) {
                        let issue_response = get_response.create_issue(client, headers_clone).await;
                        let output = match issue_response {
                            Ok(val) => {
                                println!("val {}", val.status());
                                Some(val)
                            }
                            Err(_) => None,
                        };
                        if let None = output {
                            return "Failed to create issue".to_string();
                        }
                        return get_response.body;
                        // TODO check the status code and accordingly fill the details and return the struct
                    }
                }
                return "No new releases found".to_string();
            })
        })
        .collect();
    let mut items: Vec<String> = vec![];
    for task in tasks {
        items.push(task.await.unwrap());
    }
    for item in &items {
        println!("hi {}", *item);
    }
    Ok(())
}

impl Release {
    fn check_new_release(&self, days: i64) -> bool {
        let dt = Utc::now();
        let x = dt - Duration::days(days);
        let datetime = self.published_at.parse::<DateTime<Utc>>();
        if let Ok(pub_date) = datetime {
            if pub_date > x {
                return true;
            }
        }
        false
    }

    async fn create_issue(
        &self,
        client: Arc<Client>,
        headers: HeaderMap,
    ) -> Result<Response, reqwest::Error> {
        let mut map = HashMap::new();
        let github_org: &str =
            &env::var("GITHUB_REPOSITORY").expect("Missing input parameter: repo");
        let current_repo: Vec<&str> = github_org.split('/').collect();
        map.insert(
            "title",
            format!("New version of {} {} available", &self.name, &self.tag_name),
        );
        map.insert(
            "body",
            format!(
                " Upstream new release {} available at {}",
                &self.name, &self.html_url
            ),
        );
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues",
            current_repo[0], current_repo[1]
        );
        client.post(url).headers(headers).json(&map).send().await
    }
}

fn get_repo_details(repo: &str) -> Result<Captures, String> {
    let m = match Regex::new(r"https://github.com/([\S]+)/([\S]+)") {
        Ok(value) => {
            match value.is_match(&repo) {
                true => {
                    let m = value.captures(&repo).unwrap();
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

#[tokio::test]
async fn test_main() {
    let start = Instant::now();
    env::set_var(
        "INPUT_REPO",
        "https://github.com/jetstack/cert-manager,https://github.com/jetstack/google-cas-issuer,https://github.com/cert-manager/istio-csr,https://github.com/kubernetes-sigs/secrets-store-csi-driver",
    );
    env::set_var("INPUT_DAYS", "2");
    env::set_var("GITHUB_REPOSITORY", "maheshrayas/sample");
    env::set_var("INPUT_GITHUB_TOKEN", "");
    if let Err(_) = process().await {
        panic!("Failed",);
    }
    println!("Success");
    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
}

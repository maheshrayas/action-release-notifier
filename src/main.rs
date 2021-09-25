use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use regex::{Captures, Regex};
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Instant;

#[derive(Deserialize, Debug, Default)]
struct Release {
    tag_name: String,
    published_at: String,
    body: String,
    html_url: String,
}

#[derive(Deserialize, Debug)]
struct Output {
    repo_name: String,
    release: Option<Release>,
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

    let repo_list_string = repo.split(',').into_iter().map(|s| s.to_string());
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", token.parse().unwrap());
    headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());

    let tasks: Vec<_> = repo_list_string
        .into_iter()
        .map(|repo| {
            let mut headers_clone = headers.to_owned();
            let client = Arc::new(reqwest::Client::new());
            tokio::spawn(async move {
                let m = match get_repo_details(&repo) {
                    Ok(cap) => Some(cap),
                    Err(e) => {
                        println!("{}", e);
                        None
                    }
                };
                if m.is_none() {
                    return Output {
                        repo_name: repo,
                        release: None,
                    };
                }
                let final_org = m.unwrap();
                let (parsed_org, parsed_repo) = (&final_org[1], &final_org[2]);

                headers_clone.insert("User-Agent", repo.parse().unwrap());
                let url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    &parsed_org, &parsed_repo
                );
                let resp = client
                    .get(&url)
                    .headers(headers_clone.to_owned())
                    .send()
                    .await;
                match resp {
                    Ok(res) => {
                        if res.status() == 200 {
                            let body = res.json::<Release>().await;
                            let get_response = body.unwrap();
                            if get_response.check_new_release(days) {
                                let issue_response = get_response
                                    .create_issue(client, headers_clone, parsed_repo)
                                    .await;
                                let _ = match issue_response {
                                    Ok(val) => {
                                        if val.status() == 201 {
                                            return Output {
                                                repo_name: parsed_repo.to_string(),
                                                release: Some(get_response),
                                            };
                                        } else {
                                            println!(
                                                "Failed to create GH issue for {} status code {:?}",
                                                parsed_repo,
                                                val.error_for_status()
                                            );
                                        }
                                    }
                                    Err(err) => {
                                        println!("Failed to create GH issue for {}", err);
                                    }
                                };
                            } else {
                                println!(
                                    "No latest release found for {:?} in past {} day(s)",
                                    parsed_repo, days,
                                );
                            }
                        } else {
                            println!(
                                "Failed to get latest release info {} status code {:?}",
                                parsed_repo,
                                &res.error_for_status()
                            );
                        }
                    }
                    Err(_) => todo!(),
                }
                Output {
                    repo_name: parsed_repo.to_string(),
                    release: None,
                }
            })
        })
        .collect();
    let mut items: Vec<Output> = vec![];
    for task in tasks {
        items.push(task.await.unwrap());
    }
    for item in &items {
        if let Some(val) = &item.release {
            println!("GH Issue created for {}", &item.repo_name);
            println!("{}", &val.body);
        }
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
        repo_name: &str,
    ) -> Result<Response, reqwest::Error> {
        let mut map = HashMap::new();
        let github_org: &str =
            &env::var("GITHUB_REPOSITORY").expect("Missing input parameter: repo");
        let current_repo: Vec<&str> = github_org.split('/').collect();
        map.insert(
            "title",
            format!("New version of {} {} available", &repo_name, &self.tag_name),
        );
        map.insert(
            "body",
            format!(
                " Upstream new release {} available at {}",
                &repo_name, &self.html_url
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

#[tokio::test]
async fn test_main() {
    let start = Instant::now();
    env::set_var("INPUT_REPO", "https://github.com/jetstack/cert-manager,https://github.com/jetstack/google-cas-issuer,https://github.com/cert-manager/istio-csr,https://github.com/kubernetes-sigs/secrets-store-csi-driver");
    env::set_var("INPUT_DAYS", "7");
    env::set_var("GITHUB_REPOSITORY", "maheshrayas/sample");
    env::set_var(
        "INPUT_GITHUB_TOKEN",
        "",
    );
    if let Err(_) = process().await {
        panic!("Failed",);
    }
    println!("Success");
    let duration = start.elapsed();
    println!("Time taken for execution is: {:?}", duration);
}

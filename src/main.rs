use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::{env, process::exit};
use std::sync::Arc;

#[derive(Deserialize, Debug)]
struct Release {
    name: String,
    tag_name: String,
    published_at: String,
    body: String,
    html_url: String,
}
#[derive(Deserialize)]
struct Issue {
    html_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    process().await?;
    Ok(())
}

async fn process() -> Result<(), reqwest::Error> {
    let repo: &str = &env::var("INPUT_REPO").expect("Missing input parameter: repo");
    let github_org: &str = &env::var("GITHUB_REPOSITORY").expect("Missing input parameter: repo");
    let current_repo: Vec<&str> = github_org.split('/').collect();
    let current_repo_list_string: Vec<String> = current_repo.iter().map(|s|s.to_string()).collect();
    let curr_org = current_repo_list_string[0].to_owned();
    let curr_repo = current_repo_list_string[1].to_owned();
    println!("curr_org curr_repo body {} {:?}", curr_org, curr_repo);

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
    let repo_list_string: Vec<String> = repo_list.iter().map(|s|s.to_string()).collect();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", token.parse().unwrap());
    headers.insert("Accept", "application/vnd.github.v3+json".parse().unwrap());

    for repo in repo_list_string {
        let mut headers_clone = headers.to_owned();
        let curr_org_clone = curr_org.clone();
        let curr_repo_clone = curr_repo.clone();
        let arc_client =  Arc::new(reqwest::Client::new());
        let client =  arc_client.clone();

        tokio::spawn(async move {
            println!("repo {}",repo);
      
        // strip org and repo from url
        let m = match Regex::new(r"https://github.com/([\S]+)/([\S]+)") {
            Ok(value) => {
                match value.is_match(&repo) {
                    true => {
                        let m = value.captures(&repo).unwrap();
                        //println!("{:?}",&m);
                        Some(m)
                    }
                    false => None,
                }
            }
            Err(_) => {
                println!("Failed to parse repo");
                exit(1)
            }
        };
        if m.is_none() {
            panic!("Entered repo is not in valid http(s), ssh, git formtat")
        };
        let final_org = m.unwrap();
        let (parsed_org, parsed_repo) = (&final_org[1], &final_org[2]);

        headers_clone.insert("User-Agent", repo.parse().unwrap());
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            &parsed_org, &parsed_repo
        );
        let res = client.get(&url).headers(headers_clone.to_owned()).send().await.unwrap();
        if res.status() == 200 {
            let body = res.json::<Release>().await.unwrap();
            if body.check_new_release(days) {

                let mut map = HashMap::new();
                map.insert("title",  format!("New version of {} {} available",body.name, body.tag_name));
                map.insert("body",  format!(" Upstream new release {} available at {}",body.name, body.html_url));
                let url = format!(
                    "https://api.github.com/repos/{}/{}/issues",
                    curr_org_clone, curr_repo_clone
                );
                let res = client
                    .post(url)
                    .headers(headers_clone)
                    .json(&map)
                    .send()
                    .await.unwrap();
                if res.status() == 201 {
                    let issue = res.json::<Issue>().await.unwrap();
                    println!("Created Issue at {}", issue.html_url);
                }
            }
        } else {
            println!("Status code {}", res.status());
        }
    });
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
}

#[tokio::test]
async fn test_main() {
    env::set_var(
        "INPUT_REPO",
        "https://github.com/maheshrayas/action-pr-comment-delete, https://github.com/maheshrayas/gcpPowerCycle",
    );
    env::set_var("INPUT_DAYS", "70");
    env::set_var("GITHUB_REPOSITORY", "maheshrayas/sample");
    env::set_var("INPUT_GITHUB_TOKEN", "");
    if let Err(_) = process().await {
        panic!("Failed",);
    }
    println!("Success");
}

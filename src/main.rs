use futures::future;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::{env};

#[derive(Deserialize, Debug)]
struct Release {
    name: String,
    tag_name: String
}
#[derive(Deserialize, Debug)]
struct User {
    login: String,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // let repo = env::var("INPUT_REPO").expect("Missing input parameter: repo");
    // let org = env::var("INPUT_ORG").expect("Missing input parameter: org");
    // let days = env::var("INPUT_ISSUE").unwrap_or_default();
    let token = format!(
        "token {}",
        env::var("INPUT_GITHUB_TOKEN").expect("Missing input parameter: github_token")
    );

    let github_org = env::var("GITHUB_REPOSITORY").expect("Missing input parameter: repo");
    println!("GITHUB_ORG {}",github_org);

    // let token = "";
    // let repos = "action-pr-comment-delete";
    // let org = "maheshrayas";
    // //let days = env::var("INPUT_ISSUE").unwrap_or_default();
    // let client = reqwest::Client::new();
    // let mut headers = HeaderMap::new();
    // headers.insert("Authorization", token.parse().unwrap());
    // headers.insert(
    //     "Accept",
    //     "application/vnd.github.v3+jso".parse().unwrap(),
    // );
    // headers.insert("User-Agent", repos.parse().unwrap());

    // let url = format!(
    //     "https://api.github.com/repos/{}/{}/releases/latest",
    //     &org, &repos
    // );
    // let res = client.get(url).headers(headers.to_owned()).send().await?;
    // let body: Option<Release> = match res.json::<Release>().await {
    //     Ok(body) => {
    //         Some(body)
    //     },
    //     Err(_) => {
    //         None
    //     },
    // };
    // let response = future::join_all(
    //     body.iter()

    //         .map(|pr_com| {
    //             println!("Deleting comment id {}", pr_com.id);
    //             client
    //                 .delete(&pr_com.url)
    //                 .headers(headers.to_owned())
    //                 .send()
    //         }),
    // )
    // .await;

    //println!("Rsponse {:?}",body.unwrap());

    // for b in response {
    //     match b {
    //         Ok(b) => println!("Got {} response", b.status()),
    //         Err(e) => eprintln!("Got an error: {}", e),
    //     }
    // }

    Ok(())


    // get the repo name
    // frequency of release


    // Query the repo and get the latest release
    // check if the release is done in the x no of days
    // if any release is found, create a github issue or slack notify
}
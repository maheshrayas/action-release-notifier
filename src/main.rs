use anyhow::Result;
use release_notifier::{Input, InputType};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_line_number(false)
        .with_target(false)
        .with_file(false)
        .with_max_level(tracing::Level::INFO)
        .init();
    process().await?;
    Ok(())
}

async fn process() -> Result<()> {
    let repo: String = env::var("INPUT_REPO").expect("Missing input parameter: repo");
    let days: i64 = match env::var("INPUT_DAYS") {
        Ok(days) => days.parse::<i64>().unwrap(),
        Err(_) => 1,
    };
    let token = format!(
        "token {}",
        env::var("INPUT_GITHUB_TOKEN").expect("Missing input parameter: github_token")
    );
    let input_type = env::var("INPUT_TYPE").expect("Missing input parameter: type");

    let labels: Option<String> = match env::var("INPUT_LABELS") {
        Ok(labels) => Some(labels),
        Err(_) => None,
    };

    // intialize the struct
    let input = Input::new(input_type, token, repo, days, labels);

    let m = match input.input_type {
        InputType::Github => input.gh().await?,
        InputType::Rss => input.rss().await?,
    };
    Ok(m)
}

#[tokio::test]
async fn test_gh() {
    use std::time::Instant;
    let start = Instant::now();
    let gh_token = &env::var("TOKEN").unwrap();
    env::set_var("INPUT_REPO", "https://github.com/maheshrayas/action-release-notifier,https://github.com/maheshrayas/action-pr-comment-delete,https://github.com/kubernetes/kubernetes");
    // env::set_var("INPUT_DAYS", "4");
    env::set_var("GITHUB_REPOSITORY", "maheshrayas/action-release-notifier");
    env::set_var("INPUT_TYPE", "Github");
    env::set_var("INPUT_LABELS", "release,google");
    env::set_var("INPUT_GITHUB_TOKEN", gh_token);
    if let Err(_) = process().await {
        panic!("Failed",);
    }
    let duration = start.elapsed();
    println!("Time taken for execution is: {:?}", duration);
}

#[tokio::test]
async fn test_rss() {
    use std::time::Instant;
    let start = Instant::now();
    let gh_token = &env::var("TOKEN").unwrap();
    env::set_var(
        "INPUT_REPO",
        "https://cloud.google.com/feeds/anthosconfig-release-notes.xml",
    );
    // env::set_var("INPUT_DAYS", "9");
    env::set_var("GITHUB_REPOSITORY", "maheshrayas/action-release-notifier");
    env::set_var("INPUT_TYPE", "Rss");
    env::set_var("INPUT_DAYS", "7");
    env::set_var("INPUT_GITHUB_TOKEN", gh_token);
    if let Err(_) = process().await {
        panic!("Failed",);
    }
    let duration = start.elapsed();
    println!("Time taken for execution is: {:?}", duration);
}

use std::{env, fs::File};

use conda_dep_check::{discover_environment_file, Manifest, Snapshot};
use reqwest::{
    header::{HeaderValue, USER_AGENT},
    Error,
};

fn main() -> Result<(), Error> {
    let env_file = discover_environment_file().expect("No env files were discovered.");
    let manifest = Manifest::new(env_file)
        .expect("The env file could not be read.")
        .parse_env_file();
    let snapshot = Snapshot::new(manifest);
    let token = env::var("GITHUB_TOKEN").expect("Could no find a token.");
    let request_url = format!(
        "https://api.github.com/repos/{repo}/dependency-graph/snapshots",
        repo = env::var("GITHUB_REPOSITORY").expect("Could not find repo info.")
    );
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(request_url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header(USER_AGENT, HeaderValue::from_static("reqwest"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Content-Type", "application/vnd.github+json")
        .json(&snapshot)
        .send()?;

    println!("{:?}", res);

    let file = File::create("test2.json").expect("Could not write file");
    let _ = serde_json::to_writer(file, &snapshot);

    Ok(())
}

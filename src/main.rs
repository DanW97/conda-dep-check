use conda_dep_check::{discover_environment_file, Manifest};
use reqwest::Error;
use serde_json::to_value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let env_file = discover_environment_file().expect("No env files were discovered.");
    let manifest = Manifest::new(env_file)
        .expect("The env file could not be read.")
        .parse_env_file();
    // println!("{:?}", to_value(manifest));

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.github.com/repos/DanW97/conda-dep-check/dependency-graph/snapshots")
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("Authorization", "Bearer TOKEN")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&manifest)
        .send()
        .await?;
    println!("{:?}", res);

    Ok(())
}

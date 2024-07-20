use conda_dep_check::*;
use serde_json::to_value;
fn main() {
    let env_file = discover_environment_file().expect("No env files were discovered.");
    let manifest = Manifest::new(env_file)
        .expect("The env file could not be read.")
        .parse_env_file();
    println!("{:?}", to_value(manifest));
}

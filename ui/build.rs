use std::{env::var, process::Command};

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();

    let api_endpoint = var("SANDBOX_API_ENDPOINT").unwrap_or("https://sandbox.nikitavbv.com".to_owned());

    println!("cargo:rustc-env=GIT_HASH={} cargo:rustc-env=API_ENDPOINT={}", git_hash, api_endpoint);
}
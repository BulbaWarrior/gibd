#![warn(
    clippy::unwrap_used,
    clippy::expect_used
)]
use reqwest::header::{self, HeaderMap};
use std::process;

use grafana_backup::{run, Config};

fn gen_headers(conf: &Config) -> HeaderMap {
    let mut headers = header::HeaderMap::new();
    let auth = format!("Bearer {}", conf.grafana_token);
    headers.append(header::AUTHORIZATION, auth.parse().unwrap());
    headers
}

#[tokio::main]
async fn main() {
    let config: Config = envy::from_env().unwrap_or_else(|e| {
        println!("configuration error: {e}");
        process::exit(1);
    });

    let headers = gen_headers(&config);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap_or_else(|e| {
            println!("http client initialization error: {e}");
            process::exit(1);
        });

    if let Err(e) = run(&config, &client).await {
        println!("backup error: {e}");
    }
}

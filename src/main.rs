#![warn(clippy::unwrap_used, clippy::expect_used)]
use clap::Parser;
use color_eyre::Result;
use reqwest::header::{self, HeaderMap};

use grafana_backup::{config, run};

fn gen_headers(
    conf: &config::Env,
) -> core::result::Result<HeaderMap, reqwest::header::InvalidHeaderValue> {
    let mut headers = header::HeaderMap::new();
    let auth = format!("Bearer {}", conf.grafana_token);
    headers.append("Authorization", auth.parse()?);
    Ok(headers)
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = config::Args::parse();
    let env: config::Env = envy::from_env()?;

    let headers = gen_headers(&env)?;
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    run(&args, &client).await?;
    Ok(())
}

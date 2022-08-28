#![warn(clippy::unwrap_used, clippy::expect_used)]
use color_eyre::Result;
use reqwest::header::{self, HeaderMap};

use grafana_backup::{run, Config};

fn gen_headers(
    conf: &Config,
) -> core::result::Result<HeaderMap, reqwest::header::InvalidHeaderValue> {
    let mut headers = header::HeaderMap::new();
    let auth = format!("Bearer {}", conf.grafana_token);
    headers.append(header::AUTHORIZATION, auth.parse()?);
    Ok(headers)
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config: Config = envy::from_env()?;

    let headers = gen_headers(&config)?;
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    run(&config, &client).await?;
    Ok(())
}

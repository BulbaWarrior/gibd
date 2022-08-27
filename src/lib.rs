#![warn(
    clippy::unwrap_used,
    clippy::expect_used
)]
use color_eyre::Result;
use serde::Deserialize;
use std::{
    fs,
    path::{self, Path},
};
mod requests;
use crate::requests::{get_dashboard, search_dashboards};

mod dashboard;
pub use crate::dashboard::*;

fn prep_fs(data_dir: &Path, folders_dir: &Path) -> Result<()>{
    for dir in [data_dir, folders_dir] {
        if !dir.exists() {
            fs::create_dir(dir)?;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct Config {
    pub grafana_token: String,
    pub grafana_host: String,
}

pub async fn run(config: &Config, client: &reqwest::Client) -> Result<()> {
    let summaries = search_dashboards(client, &config.grafana_host).await?;
    let mut dashboards = Vec::new();

    let data_dir = path::Path::new("data");
    let folders_dir = path::Path::new("folders");
    prep_fs(data_dir, folders_dir)?;
    for summary in &summaries {
        let json = get_dashboard(&summary.uid, client, &config.grafana_host).await?;
        let dashboard = Dashboard::build(summary, json)?;
        dashboard.store(data_dir)?.link(folders_dir)?;
        dashboards.push(dashboard);
    }
    Ok(())
}

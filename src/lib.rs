#![warn(clippy::unwrap_used, clippy::expect_used)]
use color_eyre::{Report, Result};
use std::{
    fs,
    path::{self, Path},
    sync::Arc,
};

mod requests;
use crate::requests::{get_dashboard, search_dashboards};

mod dashboard;
pub use crate::dashboard::*;

pub mod config;

fn prep_fs(data_dir: &Path, folders_dir: &Path) -> Result<()> {
    for dir in [data_dir, folders_dir] {
        if !dir.exists() {
            fs::create_dir(dir)?;
        }
    }
    Ok(())
}
pub async fn run(config: config::Args, client: reqwest::Client) -> Result<()> {
    let config = Arc::new(config);
    let client = Arc::new(client);
    let summaries = search_dashboards(&client, &config.grafana_url).await?;
    let data_dir = path::Path::new("data");
    let folders_dir = path::Path::new("folders");
    prep_fs(data_dir, folders_dir)?;
    let mut handles: Vec<_> = Vec::new();

    for summary in summaries {
        let config = Arc::clone(&config);
        let client = Arc::clone(&client);
        let handle = tokio::spawn(async move {
            let json = get_dashboard(&summary.uid, &client, &config.grafana_url).await?;
            let dashboard = Dashboard::build(&summary, json)?;
            dashboard.store(data_dir)?.link(folders_dir)?;
            Ok::<(), Report>(())
        });
        handles.push(handle);
    }

    let mut results: Vec<Result<()>> = Vec::new();
    for handle in handles {
        results.push(handle.await?);
    }
    results.into_iter().collect::<Result<Vec<()>>>()?;

    Ok(())
}

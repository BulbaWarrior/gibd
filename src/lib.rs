#![warn(clippy::unwrap_used, clippy::expect_used)]
use color_eyre::{Report, Result};
use futures::{stream::FuturesUnordered, StreamExt, TryStreamExt};
use std::{
    fs,
    iter::FromIterator,
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
    let data_dir = path::Path::new("data");
    let folders_dir = path::Path::new("folders");
    prep_fs(data_dir, folders_dir)?;

    let summaries = search_dashboards(&client, &config.grafana_url).await?;
    let results_futures = summaries.into_iter().map(|summary| {
        let config = Arc::clone(&config);
        let client = Arc::clone(&client);
        async move {
            let json = get_dashboard(&summary.uid, &client, &config.grafana_url).await?;
            let dashboard = Dashboard::build(&summary, json)?;
            dashboard.store(data_dir).await?.link(folders_dir).await?;
            Ok::<(), Report>(())
        }
    });
    let results: Vec<_> = FuturesUnordered::from_iter(results_futures).collect().await;
    let result: Result<Vec<()>> = results.into_iter().collect();
    result?;

    Ok(())
}

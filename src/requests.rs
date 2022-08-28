use std::{error::Error, fmt::Display};

use crate::DashSummary;
use color_eyre::Result;
use serde::Deserialize;
use serde_json::Value as JSON;

#[derive(Debug, Deserialize)]
struct GrafanaError {
    message: String,
}
impl Display for GrafanaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "message: {}", self.message)
    }
}
impl Error for GrafanaError {}

async fn query_grafana(client: &reqwest::Client, endpoint: &str) -> Result<String> {
    let resp = client.get(endpoint).send().await?;
    let status = resp.status();
    let data: String = resp.text().await?;
    if !status.is_success() {
        let err: GrafanaError = serde_json::from_str(&data)?;
        Err(err)?
    }

    Ok(data)
}

pub(crate) async fn get_dashboard(
    uid: &str,
    client: &reqwest::Client,
    grafana_host: &str,
) -> Result<JSON> {
    let endpoint = &format!("{}/api/dashboards/uid/{}", grafana_host, uid);
    let data: String = query_grafana(client, endpoint).await?;
    let res = serde_json::from_str(&data)?;
    Ok(res)
}

pub(crate) async fn search_dashboards(
    client: &reqwest::Client,
    grafana_host: &str,
) -> Result<Vec<DashSummary>> {
    let endpoint = &format!("{}/api/search?type=dash-db", grafana_host);
    let data: String = query_grafana(client, endpoint).await?;
    let res = serde_json::from_str(&data)?;
    Ok(res)
}

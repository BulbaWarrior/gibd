use crate::DashSummary;
use serde_json::Value as JSON;
use color_eyre::{Result, eyre::Context};

pub(crate) async fn get_dashboard(
    uid: &str,
    client: &reqwest::Client,
    grafana_host: &str,
) -> Result<JSON> {
    let endpoint = format!("http://{}/api/dashboards/uid/{}", grafana_host, uid);
    let data: String = client.get(endpoint).send().await?.text().await?;
    serde_json::from_str(&data).wrap_err("Failed to deserialize to JSON")
}

pub(crate) async fn search_dashboards(
    client: &reqwest::Client,
    grafana_host: &str,
) -> Result<Vec<DashSummary>> {
    let endpoint = format!("http://{}/api/search?type=dash-db", grafana_host);
    let data: String = client
        .get(endpoint)
        .send()
        .await?
        .text()
        .await?;
    serde_json::from_str(&data).wrap_err("Failed to deserialize JSON")
}

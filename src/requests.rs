use crate::DashSummary;
use serde_json::Value as JSON;

pub(crate) async fn get_dashboard(uid: &str, client: &reqwest::Client, grafana_host: &str) -> JSON {
    let endpoint = format!("http://{}/api/dashboards/uid/{}", grafana_host, uid);
    let data: String = client
        .get(endpoint)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    serde_json::from_str(&data).unwrap()
}

pub(crate) async fn search_dashboards(
    client: &reqwest::Client,
    grafana_host: &str,
) -> Vec<DashSummary> {
    let endpoint = format!("http://{}/api/search?type=dash-db", grafana_host);
    let data: String = client
        .get(endpoint)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    serde_json::from_str(&data).unwrap()
}

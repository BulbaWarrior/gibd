use reqwest::header::{self, HeaderMap};
use serde::Deserialize;
use serde_json::Value as JSON;
use std::{
    fs, io,
    path::{self, Path, PathBuf},
};

fn gen_headers(conf: &Config) -> HeaderMap {
    let mut headers = header::HeaderMap::new();
    let mut auth = format!("Bearer {}", conf.grafana_token);
    headers.append(header::AUTHORIZATION, auth.parse().unwrap());
    headers
}
async fn get_dashboard(uid: &str, client: &reqwest::Client, conf: &Config) -> JSON {
    let endpoint = format!("http://{}/api/dashboards/uid/{}", conf.grafana_host, uid);
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

async fn search_dashboards(client: &reqwest::Client, conf: &Config) -> Vec<DashSummary> {
    let endpoint = format!("http://{}/api/search?type=dash-db", conf.grafana_host);
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

fn prep_fs(data_dir: &Path, folders_dir: &Path) {
    for dir in [data_dir, folders_dir] {
        if !dir.exists() {
            fs::create_dir(dir).unwrap();
        }
    }
}
#[tokio::main]
async fn main() {
    let config: Config = envy::from_env().unwrap();
    let headers = gen_headers(&config);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();
    let summaries = search_dashboards(&client, &config).await;
    let mut dashboards = Vec::new();

    let data_dir = path::Path::new("data");
    let folders_dir = path::Path::new("folders");
    prep_fs(data_dir, folders_dir);
    for summary in &summaries {
        let json = get_dashboard(&summary.uid, &client, &config).await;
        let dashboard = Dashboard::new(summary, json);
        dashboard.store(data_dir, folders_dir).await;
        dashboards.push(dashboard);
    }
}

#[derive(Deserialize)]
struct Config {
    grafana_token: String,
    grafana_host: String,
}

struct Dashboard<'a> {
    summary: &'a DashSummary,
    json: JSON,
}
// struct DashName<'a>(&'a str);
impl<'a> Dashboard<'a> {
    async fn store(&self, data_dir: &Path, folders_dir: &Path) {
        let data_file = data_dir.join(self.summary.uid.to_string());
        let data = serde_json::to_string_pretty(&self.json).unwrap();
        fs::write(data_file.clone(), data).expect("could not write dashboard data to file");
        let folder_name = match &self.summary.folder_title {
            Some(name) => name.replace("/", "_"),
            None => "General".into(),
        };

        let dashboard_folder = folders_dir.join(folder_name);
        if !dashboard_folder.exists() {
            fs::create_dir(&dashboard_folder).unwrap();
        }
        let dashboard_link = dashboard_folder.join(
            self.json["title"]
                .to_string()
                .replace("/", "_")
                .trim_matches('"'),
        );
        let mut link_target = PathBuf::from("../../");
        link_target.push(data_file);

        if let Err(err) = fs::remove_file(&dashboard_link) {
            match err.kind() {
                io::ErrorKind::NotFound => {}
                _ => Err(err).unwrap(),
            }
        }

        std::os::unix::fs::symlink(&link_target, &dashboard_link).expect(&format!(
            "could not create symlink for dashboard \"{}\" -> \"{}\"",
            dashboard_link.to_str().unwrap(),
            link_target.to_str().unwrap()
        ));
    }

    fn new(summary: &'a DashSummary, json: JSON) -> Dashboard {
        if json["dashboard"] == JSON::Null && json["title"] == JSON::Null {
            panic!("wrong dashboard json format");
        }
        match json["dashboard"] {
            JSON::Null => Dashboard { summary, json },
            _ => Dashboard {
                summary,
                json: json["dashboard"].clone(),
            },
        }
    }
}

#[derive(Deserialize, Debug)]
struct DashSummary {
    uid: String,
    #[serde(rename(deserialize = "folderTitle"))]
    folder_title: Option<String>,
}

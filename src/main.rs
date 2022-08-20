use reqwest::header::{self, HeaderMap};
use serde::Deserialize;
use serde_json::Value as JSON;
use std::{
    error::Error,
    fs, io,
    path::{self, Path, PathBuf},
};

fn gen_headers(conf: &Config) -> HeaderMap {
    let mut headers = header::HeaderMap::new();
    let auth = format!("Bearer {}", conf.grafana_token);
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
async fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = envy::from_env()?;
    let headers = gen_headers(&config);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let summaries = search_dashboards(&client, &config).await;
    let mut dashboards = Vec::new();

    let data_dir = path::Path::new("data");
    let folders_dir = path::Path::new("folders");
    prep_fs(data_dir, folders_dir);
    for summary in &summaries {
        let json = get_dashboard(&summary.uid, &client, &config).await;
        let dashboard = Dashboard::new(summary, json);
        dashboard.store(data_dir).await?.link(folders_dir).await?;
        dashboards.push(dashboard);
    }
    Ok(())
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
struct StoredDashboard<'a>(PathBuf, &'a Dashboard<'a>);

impl<'a> Dashboard<'a> {
    async fn store(&'a self, data_dir: &'a Path) -> io::Result<StoredDashboard<'a>> {
        let data_file = data_dir.join(self.summary.uid.to_string());
        let data = serde_json::to_string_pretty(&self.json)
            .expect("unexpected serialization error, this should serialize a decerialized JSON");
        fs::write(data_file.clone(), data)?;
        Ok(StoredDashboard(data_file, self))
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

impl<'a> StoredDashboard<'a> {
    async fn link(&self, folders_dir: &Path) -> io::Result<()> {
        let StoredDashboard(data_file, dashboard) = self;
        let folder_name = match &dashboard.summary.folder_title {
            Some(name) => name.replace("/", "_"),
            None => "General".into(),
        };

        let dashboard_folder = folders_dir.join(folder_name);
        if !dashboard_folder.exists() {
            fs::create_dir(&dashboard_folder)?;
        }
        let dashboard_link = dashboard_folder.join(
            dashboard.json["title"]
                .to_string()
                .replace("/", "_")
                .trim_matches('"'),
        );
        let mut link_target = PathBuf::from("../../");
        link_target.push(data_file);

        if let Err(err) = fs::remove_file(&dashboard_link) {
            match err.kind() {
                io::ErrorKind::NotFound => {}
                _ => return Err(err),
            }
        }

        std::os::unix::fs::symlink(&link_target, &dashboard_link)?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct DashSummary {
    uid: String,
    #[serde(rename(deserialize = "folderTitle"))]
    folder_title: Option<String>,
}

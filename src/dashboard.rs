use color_eyre::Result;
use serde::Deserialize;
use serde_json::Value as JSON;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Deserialize, Debug)]
pub struct DashSummary {
    pub uid: String,
    #[serde(rename(deserialize = "folderTitle"))]
    pub folder_title: Option<String>,
}

pub struct Dashboard<'a> {
    summary: &'a DashSummary,
    json: JSON,
}

pub struct StoredDashboard<'a>(PathBuf, &'a Dashboard<'a>);

impl<'a> Dashboard<'a> {
    pub fn store(&'a self, data_dir: &'a Path) -> Result<StoredDashboard<'a>> {
        let data_file = data_dir.join(&self.summary.uid);
        let data = serde_json::to_string_pretty(&self.json)?;

        fs::write(data_file.clone(), data)?;
        Ok(StoredDashboard(data_file, self))
    }

    pub fn build(summary: &'a DashSummary, json: JSON) -> color_eyre::Result<Dashboard> {
        if json["dashboard"] == JSON::Null && json["title"] == JSON::Null {
            return Err(color_eyre::eyre::eyre!("Unfamiliar dashboard format"));
        }
        let dashboard = match json["dashboard"] {
            JSON::Null => Dashboard { summary, json },
            _ => Dashboard {
                summary,
                json: json["dashboard"].clone(),
            },
        };
        Ok(dashboard)
    }
}

impl<'a> StoredDashboard<'a> {
    pub fn link(&self, folders_dir: &Path) -> io::Result<()> {
        let StoredDashboard(data_file, dashboard) = self;
        let folder_name = match &dashboard.summary.folder_title {
            Some(name) => name.replace('/', "_"),
            None => "General".into(),
        };

        let dashboard_folder = folders_dir.join(folder_name);
        if !dashboard_folder.exists() {
            fs::create_dir(&dashboard_folder)?;
        }
        let dashboard_link = dashboard_folder.join(
            dashboard.json["title"]
                .to_string()
                .replace('/', "_")
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

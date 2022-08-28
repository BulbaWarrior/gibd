
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Env {
    pub grafana_token: String,
    pub grafana_host: String,
}

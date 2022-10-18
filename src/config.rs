use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Env {
    pub grafana_token: String,
}

#[derive(Parser)]
pub struct Args {
    #[clap(value_parser)]
    /// Example: http://localhost:3000.
    /// Org is detrmined by api token, so http://localhost:3000?orgId=123 is invalid
    pub grafana_url: String,
}

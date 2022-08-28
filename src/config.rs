use serde::Deserialize;
use clap::Parser;

#[derive(Deserialize)]
pub struct Env {
    pub grafana_token: String,
}

#[derive(Parser)]
pub struct Args {
    #[clap(value_parser)]
    /// Example: http://localhost:3000
    pub grafana_url: String,
}

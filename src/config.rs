use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use matrix_sdk::ruma::{OwnedRoomId, OwnedServerName};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RepoConfig {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct CrawlConfig {
    pub servers: HashSet<String>,

    pub seeds: Vec<String>,

    #[serde(with = "humantime_serde")]
    pub interval: Duration,
}

#[derive(Debug, Deserialize)]
pub struct NotifyConfig {
    #[serde(rename = "cool-down", with = "humantime_serde")]
    pub cool_down: Duration,

    pub server: OwnedServerName,

    pub token: String,

    pub room: OwnedRoomId,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub repo: RepoConfig,

    pub crawl: CrawlConfig,

    pub notify: NotifyConfig,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let config = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(&config)?;
        return Ok(config);
    }
}

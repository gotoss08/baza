use anyhow::{Context, anyhow};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

use crate::Result;

pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub launcher_path: Option<PathBuf>,
    pub ibcmd_path: Option<PathBuf>,

    pub default_platform_arch: Option<PlatformArch>,
    pub default_platform_version: Option<String>,

    pub default_ib_username: Option<String>,
    pub default_ib_password: Option<String>,

    #[serde(default)]
    pub servers: Vec<Server>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum PlatformArch {
    X86,
    X86_64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    pub kind: ServerKind,
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub password: String,
}

impl Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.port {
            Some(port) => write!(f, "{}:{}", self.host, port),
            None => write!(f, "{}", self.host),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ServerKind {
    Postgres,
    Mssql,
}

/// Returns the path to the config file, creating parent directories if needed.
pub fn path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "baza")
        .ok_or_else(|| anyhow!("cannot determine config directory"))?;
    let dir = dirs.config_dir().to_path_buf();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("creating config directory {}", dir.display()))?;
    Ok(dir.join("config.toml"))
}

/// Loads the config from disk. Creates an empty config if the file does not exist.
pub fn load() -> Result<Config> {
    let path = path()?;
    if !path.exists() {
        let default = Config::default();
        save(&default)?;
        return Ok(default);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("reading config from {}", path.display()))?;
    let cfg: Config =
        toml::from_str(&raw).with_context(|| format!("parsing config at {}", path.display()))?;
    Ok(cfg)
}

/// Persists the config back to disk.
pub fn save(cfg: &Config) -> Result<()> {
    let path = path()?;
    let raw = toml::to_string_pretty(cfg).context("serializing config")?;
    std::fs::write(&path, raw).with_context(|| format!("writing config to {}", path.display()))
}

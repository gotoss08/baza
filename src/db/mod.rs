use anyhow::{Context, anyhow};

use crate::Result;
use crate::config::{Server, ServerKind};

pub mod mssql;
pub mod postgres;

/// Checks whether an infobase with the given name exists on the server.
pub async fn base_exists(server: &Server, base_name: &str) -> Result<bool> {
    match server.kind {
        ServerKind::Postgres => postgres::base_exists(server, base_name).await,
        ServerKind::Mssql => mssql::base_exists(server, base_name).await,
    }
}

/// Returns the size of the infobase on disk in bytes.
pub async fn base_size(server: &Server, base_name: &str) -> Result<Option<u64>> {
    match server.kind {
        ServerKind::Postgres => postgres::base_size(server, base_name).await,
        ServerKind::Mssql => mssql::base_size(server, base_name).await,
    }
}

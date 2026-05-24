use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use super::*;

const DEFAULT_PORT: u16 = 1433;

async fn connect(server: &Server) -> Result<Client<tokio_util::compat::Compat<TcpStream>>> {
    let mut config = Config::new();
    config.host(&server.host);
    config.port(server.port.unwrap_or(DEFAULT_PORT));
    config.authentication(AuthMethod::sql_server(&server.user, &server.password));
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr())
        .await
        .with_context(|| format!("connecting to {}", server))?;
    tcp.set_nodelay(true)?;

    Client::connect(config, tcp.compat_write())
        .await
        .with_context(|| format!("tiberius handshake with {}", server))
}

pub async fn base_exists(server: &Server, base_name: &str) -> Result<bool> {
    let mut client = connect(server).await?;

    let row = client
        .query(
            "SELECT 1 FROM sys.databases WHERE name = @P1",
            &[&base_name],
        )
        .await
        .with_context(|| format!("checking existence of '{}' on {}", base_name, server))?
        .into_row()
        .await
        .with_context(|| format!("reading result for '{}' on {}", base_name, server))?;

    Ok(row.is_some())
}

pub async fn base_size(server: &Server, base_name: &str) -> Result<Option<u64>> {
    let mut client = connect(server).await?;

    let row = client
        .query(
            "SELECT SUM(CAST(mf.size AS BIGINT)) * 8192
             FROM sys.databases d
             JOIN sys.master_files mf ON mf.database_id = d.database_id
             WHERE d.name = @P1
             GROUP BY d.database_id",
            &[&base_name],
        )
        .await
        .with_context(|| format!("querying size of '{}' on {}", base_name, server))?
        .into_row()
        .await
        .with_context(|| format!("reading size result for '{}' on {}", base_name, server))?;

    if row.is_none() {
        return Ok(None);
    }

    let size: i64 = row
        .unwrap()
        .get(0)
        .ok_or_else(|| anyhow!("NULL size for '{}' on {}", base_name, server))?;

    Ok(Some(size as u64))
}

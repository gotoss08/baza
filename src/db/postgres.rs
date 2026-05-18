use super::*;

const DEFAULT_PORT: u16 = 5432;

async fn connect(server: &Server) -> Result<tokio_postgres::Client> {
    let (client, conn) = tokio_postgres::Config::new()
        .host(&server.host)
        .port(server.port.unwrap_or(DEFAULT_PORT))
        .user(&server.user)
        .password(&server.password)
        .dbname("postgres")
        .connect(tokio_postgres::NoTls)
        .await
        .with_context(|| format!("connecting to {}", server))?;

    tokio::spawn(conn);
    Ok(client)
}

pub async fn base_exists(server: &Server, base_name: &str) -> Result<bool> {
    let client = connect(server).await?;
    let row = client
        .query_opt(
            "SELECT 1 FROM pg_database WHERE lower(datname) = lower($1)",
            &[&base_name],
        )
        .await
        .with_context(|| format!("checking existence of '{}' on {}", base_name, server))?;
    Ok(row.is_some())
}

pub async fn base_size(server: &Server, base_name: &str) -> Result<Option<u64>> {
    let client = connect(server).await?;
    let row = client
        .query_opt(
            "SELECT pg_database_size(datname) FROM pg_database WHERE lower(datname) = lower($1)",
            &[&base_name],
        )
        .await
        .with_context(|| format!("querying size of '{}' on {}", base_name, server))?;

    if row.is_none() {
        return Ok(None);
    }

    let size: i64 = row.unwrap().get(0);
    Ok(Some(size as u64))
}

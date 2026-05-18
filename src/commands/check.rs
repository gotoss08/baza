use anyhow::bail;
use tokio::task::JoinSet;

use crate::Result;
use crate::cli::CheckArgs;
use crate::config::Config;
use crate::db;
use crate::utils;

pub async fn execute(args: CheckArgs, cfg: Config) -> Result<()> {
    if cfg.servers.is_empty() {
        bail!("No servers configured.");
    }

    let servers: Vec<_> = match &args.server {
        Some(host) => {
            let filtered: Vec<_> = cfg
                .servers
                .into_iter()
                .filter(|s| s.host == *host)
                .collect();
            if filtered.is_empty() {
                bail!("Server not found: '{host}'");
            }
            filtered
        }
        None => cfg.servers,
    };

    let mut set = JoinSet::new();
    for (idx, server) in servers.into_iter().enumerate() {
        let name = args.name.clone();
        let size_flag = args.size;
        let bytes_flag = args.bytes;
        set.spawn(async move {
            let line = if size_flag {
                match db::base_size(&server, &name).await {
                    Ok(Some(size)) => {
                        let size = if bytes_flag {
                            format!("{size} bytes")
                        } else {
                            utils::human_size(size)
                        };
                        Some(format!("{}: {}", server, size))
                    }
                    Ok(None) => None,
                    Err(e) => Some(format!("error: {}", e)),
                }
            } else {
                match db::base_exists(&server, &name).await {
                    Ok(exists) if exists => Some(format!("{}", server)),
                    Ok(_) => None,
                    Err(e) => Some(format!("error: {}", e)),
                }
            };
            if !args.sync {
                if let Some(line) = &line {
                    println!("{line}");
                }
            }
            Ok::<_, anyhow::Error>((idx, line))
        });
    }

    if args.sync {
        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            results.push(res??);
        }
        results.sort_by_key(|(idx, _)| *idx);

        for (_, line) in results {
            if let Some(line) = line {
                println!("{line}");
            }
        }
    } else {
        while let Some(res) = set.join_next().await {
            res??; // first ? for JoinError, second for anyhow::Error
        }
    }

    Ok(())
}

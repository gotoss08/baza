use anyhow::Context;

use crate::Result;
use crate::cli::CleanArgs;
use crate::config::Config;
use crate::onec::{cache, ibases};
use crate::utils;

pub async fn execute(args: CleanArgs, _cfg: Config) -> Result<()> {
    let cache = if args.all {
        cache::scan().context("scanning all cache directories")?
    } else {
        let ibases =
            ibases::Ibases::new().context("loading ibases.v8i to get known infobase IDs")?;
        let known = ibases.registered_ids();
        cache::orphaned(&known).context("scanning orphaned cache directories")?
    };

    if cache.is_empty() {
        println!("no cache folders found");
        return Ok(());
    }

    let mut total_freed: u64 = 0;
    let mut failed = 0usize;

    for entry in &cache {
        let size = dir_size(&entry.path).unwrap_or(0);
        total_freed += size;

        let prefix = if args.dry_run {
            "would remove"
        } else {
            "removing"
        };
        println!(
            "{prefix}: {} ({})",
            entry.path.display(),
            utils::human_size(size)
        );

        if !args.dry_run {
            if let Err(e) = std::fs::remove_dir_all(&entry.path) {
                eprintln!("  failed: {e}");
                failed += 1;
            }
        }
    }

    let verb = if args.dry_run { "would free" } else { "freed" };
    println!(
        "\n{} cache folders, {verb} {}",
        cache.len(),
        utils::human_size(total_freed)
    );
    if failed > 0 {
        println!("{failed} cache folder(s) could not be removed");
    }

    Ok(())
}

fn dir_size(path: &std::path::Path) -> Result<u64> {
    let mut total: u64 = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        total += if meta.is_dir() {
            dir_size(&entry.path()).unwrap_or(0)
        } else {
            meta.len()
        };
    }
    Ok(total)
}

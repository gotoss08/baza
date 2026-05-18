use anyhow::{Context, anyhow, bail};

use crate::Result;
use crate::cli::RunArgs;
use crate::config::Config;
use crate::input;
use crate::onec::{connection, launcher};

pub async fn execute(args: RunArgs, cfg: Config) -> Result<()> {
    let target = resolve_target(&args)?;
    launcher::launch(&args, &cfg, &target).context("launching 1C")
}

fn resolve_target(args: &RunArgs) -> Result<connection::Target> {
    if args.stdin {
        let s = input::from_stdin()?;
        return connection::parse(&s);
    }

    match (&args.target, &args.base) {
        (Some(t), Some(b)) => Ok(connection::Target::Server {
            srvr: t.clone(),
            refname: b.clone(),
        }),
        (Some(t), None) => connection::parse(t),
        (None, None) => {
            let s =
                input::from_clipboard().context("no target specified and clipboard is empty")?;
            if s.trim().is_empty() {
                bail!("no target specified and clipboard is empty");
            }
            connection::parse(&s)
        }
        (None, Some(_)) => Err(anyhow!("base name given without server name")),
    }
}

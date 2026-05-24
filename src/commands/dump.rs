use anyhow::Context;

use crate::Result;
use crate::cli::DumpArgs;
use crate::config::{Config, Credentials};
use crate::input;
use crate::onec::ibcmd::{self, DumpOptions};

pub async fn execute(args: DumpArgs, cfg: Config) -> Result<()> {
    let server = cfg
        .servers
        .iter()
        .find(|s| s.host == args.server)
        .with_context(|| format!("find server '{}' in config's server list", args.server))?;

    let opts = DumpOptions {
        arch: args.arch,
        platform_version: args.platform_version.clone(),
        ib_creds: ib_credentials(&args)?,
        out: args.out,
        verbose: args.verbose,
        disable_telegram_notifications: args.disable_telegram_notifications,
    };

    ibcmd::dump(&cfg, &server, &args.name, opts).context("dumping infobase")?;

    Ok(())
}

fn ib_credentials(args: &DumpArgs) -> Result<Credentials> {
    if args.ib_username.is_none() && args.ib_password.is_none() {
        return Ok(Credentials {
            username: Some(
                input::from_stdin_with_prompt("Infobase username: ")
                    .context("prompt for IB username")?,
            ),
            password: Some(
                input::from_stdin_with_prompt_no_echo("Infobase password: ")
                    .context("prompt for IB password")?,
            ),
        });
    }

    Ok(Credentials {
        username: args.ib_username.clone(),
        password: args.ib_password.clone(),
    })
}

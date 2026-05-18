use crate::Result;
use crate::cli::ConfigArgs;
use crate::config::Config;

pub async fn execute(_args: ConfigArgs, _cfg: Config) -> Result<()> {
    if _args.open {
        open_config_folder()?;
    }
    Ok(())
}

fn open_config_folder() -> Result<()> {
    let config_dir = crate::config::path()?
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&config_dir)
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&config_dir)
            .spawn()?;
    }
    Ok(())
}

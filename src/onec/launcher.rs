use anyhow::{Context, anyhow, bail, ensure};
use std::path::PathBuf;
use std::process::Command;

use crate::Result;
use crate::cli::RunArgs;
use crate::config::Config;
use crate::input;
use crate::onec::connection::Target;

const LAUNCHER_SUBPATH: &str = r"1cv8\common\1cestart.exe";

/// Launches 1cestart.exe to open the given infobase.
pub fn launch(args: &RunArgs, cfg: &Config, target: &Target) -> Result<()> {
    // TODO: factor out args from this module to keep it focused on just launching
    let mut cmd = Command::new(path(cfg).context("resolving 1cestart.exe path")?);
    apply_mode(&mut cmd, args);
    apply_target(&mut cmd, target);
    apply_auth(&mut cmd, args);
    apply_args(&mut cmd, args);

    let status = cmd.status().context("spawning 1cestart.exe")?;
    ensure!(
        status.success(),
        "1cestart.exe exited with code {}",
        status.code().unwrap_or(-1)
    );
    Ok(())
}

/// Returns the path to 1cestart.exe, first checking the environment variable `BAZA_LAUNCHER_PATH`,
/// then the `launcher_path` field in the config file, and finally the standard 1C installation directories.
pub fn path(cfg: &Config) -> Result<PathBuf> {
    let candidates: [Option<(&str, PathBuf)>; 2] = [
        std::env::var_os("BAZA_LAUNCHER_PATH").map(|p| ("BAZA_LAUNCHER_PATH", PathBuf::from(p))),
        cfg.launcher_path
            .clone()
            .map(|p| ("config.launcher_path", p)),
    ];

    for (source, path) in candidates.into_iter().flatten() {
        if path.is_file() {
            return Ok(path);
        }
        bail!("{source} points to a non-existent file: {}", path.display());
    }

    launcher_default_path()
}

/// Returns the path to 1cestart.exe, searching in the standard 1C installation directories.
fn launcher_default_path() -> Result<PathBuf> {
    let bases = [
        std::env::var_os("ProgramFiles"),
        std::env::var_os("ProgramFiles(x86)"),
    ];

    let mut tried = Vec::new();
    for base in bases.into_iter().flatten() {
        let candidate = PathBuf::from(base).join(LAUNCHER_SUBPATH);
        if candidate.is_file() {
            return Ok(candidate);
        }
        tried.push(candidate);
    }

    let tried_str = tried
        .iter()
        .map(|p| format!("  - {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Err(anyhow!("1cestart.exe not found. Checked:\n{tried_str}"))
}

/// Applies the mode to the command.
fn apply_mode(cmd: &mut Command, args: &RunArgs) {
    if args.designer {
        cmd.arg("DESIGNER");
    } else {
        cmd.arg("ENTERPRISE");
    }
}

/// Applies the target infobase to the command.
fn apply_target(cmd: &mut Command, target: &Target) {
    match target {
        Target::File(path) => {
            cmd.arg("/F").arg(path);
        }
        Target::Server { srvr, refname } => {
            cmd.arg("/S").arg(format!("{srvr}/{refname}"));
        }
        Target::Web(url) => {
            cmd.arg("/WS").arg(url);
        }
    }
}

/// Applies the authentication credentials to the command.
fn apply_auth(cmd: &mut Command, args: &RunArgs) {
    if args.auth {
        let username = input::from_stdin_with_prompt("Username: ").ok();
        if let Some(username) = username {
            cmd.arg("/N").arg(username);
        }
        let password = input::from_stdin_with_prompt_no_echo("Password: ").ok();
        if let Some(password) = password {
            cmd.arg("/P").arg(password);
        }
    } else {
        if let Some(username) = &args.username {
            cmd.arg("/N").arg(username);
        }
        if let Some(password) = &args.password {
            cmd.arg("/P").arg(password);
        }
    }
}

/// Applies other misc arguments to the command.
fn apply_args(cmd: &mut Command, args: &RunArgs) {
    if args.x86 {
        cmd.arg("/AppArch").arg("x86");
    }
    if args.x86_64 {
        cmd.arg("/AppArch").arg("x86_64");
    }
}

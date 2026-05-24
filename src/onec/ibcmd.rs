use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail, ensure};
use chrono::Local;

use crate::Result;
use crate::config::{Config, Credentials, PlatformArch, Server, ServerKind};

const IBCMD_REL: &str = r"bin\ibcmd.exe";
const PLATFORM_DIR: &str = "1cv8";

pub struct DumpOptions {
    pub arch: Option<PlatformArch>,
    pub platform_version: Option<String>,
    pub ib_creds: Credentials,
    pub verbose: bool,
    pub out: Option<PathBuf>,
}

/// Entry point for the `dump` subcommand.
///
/// Resolves `ibcmd.exe`, then (TODO) invokes it against the target infobase.
pub fn dump(cfg: &Config, server: &Server, name: &str, opts: DumpOptions) -> Result<()> {
    let ibcmd_path = resolve_ibcmd_path(cfg, opts.arch, opts.platform_version.as_deref())
        .context("resolving ibcmd.exe path")?;

    let mut cmd = Command::new(ibcmd_path);

    cmd.raw_arg("infobase");

    cmd.raw_arg(format!(
        "--dbms={}",
        match server.kind {
            ServerKind::Postgres => "PostgreSQL",
            ServerKind::Mssql => "MSSQLServer",
        }
    ));

    if server.port.is_some() {
        cmd.raw_arg(format!(
            "--database-server={}{}",
            server.host,
            match server.kind {
                ServerKind::Postgres => format!(" port={}", server.port.unwrap()),
                ServerKind::Mssql => format!(":{}", server.port.unwrap()),
            }
        ));
    } else {
        cmd.raw_arg(format!("--database-server=\"{}\"", server.host));
    }

    cmd.raw_arg(format!("--database-name=\"{}\"", name));
    cmd.raw_arg(format!("--database-user=\"{}\"", server.user));
    cmd.raw_arg(format!("--database-password=\"{}\"", server.password));

    cmd.raw_arg("dump");

    if opts.ib_creds.username.is_some() {
        cmd.raw_arg(format!(
            "--user=\"{}\"",
            opts.ib_creds.username.as_ref().unwrap()
        ));
    }

    if opts.ib_creds.password.is_some() {
        cmd.raw_arg(format!(
            "--password=\"{}\"",
            opts.ib_creds.password.as_ref().unwrap()
        ));
    }

    let out_path = if opts.out.is_some() {
        opts.out.unwrap().to_string_lossy().to_string()
    } else {
        let now = Local::now();
        format!("./{}_{}.dt", name, now.format("%d%m%Y"))
    };
    cmd.arg(out_path);

    if opts.verbose {
        println!("Executing command: {cmd:?}");
    }

    let status = cmd.status().context("spawning ibcmd.exe")?;
    ensure!(
        status.success(),
        "ibcmd.exe exited with code {}",
        status.code().unwrap_or(-1)
    );
    Ok(())
}

/// Resolves the absolute path to `ibcmd.exe`.
///
/// Sources are tried in priority order; the first one that yields a result wins:
///
/// 1. **CLI args** (`arch` and/or `version`) — most explicit. Partial input is
///    allowed: if only `arch` is given, the highest installed version under that
///    arch is chosen; if only `version` is given, both arches are searched for
///    that exact version. A malformed `version` is rejected up front.
/// 2. **`BAZA_IBCMD_PATH`** environment variable — must point to an existing file.
/// 3. **`cfg.ibcmd_path`** from the config file — must point to an existing file.
/// 4. **Auto-discovery** — scans `%ProgramFiles[(x86)]%\1cv8\<version>\bin\ibcmd.exe`
///    and returns the highest installed version. `cfg.default_platform_arch` and
///    `cfg.default_platform_version` are used as hints when set.
///
/// Sources 2 and 3 are treated as hard overrides: if set but invalid, the call
/// fails rather than silently falling through.
pub fn resolve_ibcmd_path(
    cfg: &Config,
    arch: Option<PlatformArch>,
    version: Option<&str>,
) -> Result<PathBuf> {
    // 1. CLI args (either or both)
    if arch.is_some() || version.is_some() {
        if let Some(v) = version {
            ensure!(
                is_version_format(v),
                "invalid --platform-version {v:?} (expected N.N.N.N)"
            );
        }
        let archs: Vec<PlatformArch> = match arch {
            Some(a) => vec![a],
            None => vec![PlatformArch::X86_64, PlatformArch::X86],
        };
        return discover_highest_ibcmd(&archs, version);
    }

    // 2. Env var
    if let Some(raw) = std::env::var_os("BAZA_IBCMD_PATH") {
        let path = PathBuf::from(raw);
        ensure!(
            path.is_file(),
            "BAZA_IBCMD_PATH points to a non-existent file: {}",
            path.display()
        );
        return Ok(path);
    }

    // 3. Explicit config path
    if let Some(path) = &cfg.ibcmd_path {
        ensure!(
            path.is_file(),
            "config.ibcmd_path points to a non-existent file: {}",
            path.display()
        );
        return Ok(path.clone());
    }

    // 4. Auto-discovery using config defaults as hints
    let archs: Vec<PlatformArch> = match cfg.default_platform_arch {
        Some(a) => vec![a],
        None => vec![PlatformArch::X86_64, PlatformArch::X86],
    };
    let pinned = cfg
        .default_platform_version
        .as_deref()
        .filter(|v| is_version_format(v));
    discover_highest_ibcmd(&archs, pinned)
}

/// Walks the `1cv8` directory for each requested arch and returns the path to
/// `ibcmd.exe` for the chosen platform version.
///
/// - If `pinned_version` is `Some`, only that exact version directory is checked.
///   Archs are tried in the order given; the first match wins.
/// - If `pinned_version` is `None`, every subdirectory whose name parses as a
///   four-part numeric version (e.g. `8.3.19.1467`) is considered a candidate,
///   and the highest version with an existing `ibcmd.exe` is returned.
///
/// Missing base directories are tolerated (treated as "this arch isn't installed").
/// Returns an error listing every probed path if nothing usable is found.
fn discover_highest_ibcmd(archs: &[PlatformArch], pinned_version: Option<&str>) -> Result<PathBuf> {
    let mut tried: Vec<PathBuf> = Vec::new();
    let mut found: Vec<([u32; 4], PathBuf)> = Vec::new();

    for &arch in archs {
        let base = platform_root(arch)?;

        if let Some(version) = pinned_version {
            let candidate = ibcmd_in(&base, version);
            if candidate.is_file() {
                return Ok(candidate);
            }
            tried.push(candidate);
            continue;
        }

        let entries = match fs::read_dir(&base) {
            Ok(entries) => entries,
            Err(_) => continue, // base dir doesn't exist for this arch
        };

        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(parsed) = parse_version(name) else {
                continue;
            };

            let candidate = ibcmd_in(&base, name);
            if candidate.is_file() {
                found.push((parsed, candidate));
            } else {
                tried.push(candidate);
            }
        }
    }

    if let Some((_, path)) = found.into_iter().max_by_key(|(v, _)| *v) {
        return Ok(path);
    }

    if tried.is_empty() {
        bail!("ibcmd.exe not found: no 1C installations discovered");
    }

    let tried_str = tried
        .iter()
        .map(|p| format!("  - {}", p.display()))
        .collect::<Vec<_>>()
        .join("\n");

    bail!("ibcmd.exe not found. Checked:\n{tried_str}")
}

/// Returns the `1cv8` install root for the given arch — i.e.
/// `%ProgramFiles%\1cv8` for x86_64 or `%ProgramFiles(x86)%\1cv8` for x86.
///
/// Fails if the underlying environment variable is not set (which on Windows
/// would be highly unusual).
fn platform_root(arch: PlatformArch) -> Result<PathBuf> {
    let env_var = match arch {
        PlatformArch::X86 => "ProgramFiles(x86)",
        PlatformArch::X86_64 => "ProgramFiles",
    };
    let value = std::env::var_os(env_var)
        .with_context(|| format!("environment variable {env_var:?} is not set"))?;
    Ok(PathBuf::from(value).join(PLATFORM_DIR))
}

/// Builds the canonical path to `ibcmd.exe` for a specific version under a base
/// directory, i.e. `<base>\<version>\bin\ibcmd.exe`. Does not check that the
/// file actually exists.
fn ibcmd_in(base: &Path, version: &str) -> PathBuf {
    base.join(version).join(IBCMD_REL)
}

/// Parses a 1C platform version string (`"8.3.19.1467"`) into its four numeric
/// components.
///
/// Returns `None` if the string does not have exactly four dot-separated
/// non-empty unsigned integer segments. The returned array is suitable for use
/// as a sort key — `[u32; 4]` compares lexicographically, which matches version
/// ordering.
fn parse_version(s: &str) -> Option<[u32; 4]> {
    let mut parts = s.split('.');
    let v = [
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ];
    if parts.next().is_some() {
        return None;
    }
    Some(v)
}

/// Convenience predicate over [`parse_version`] — `true` iff `s` is a
/// well-formed four-part numeric version string.
fn is_version_format(s: &str) -> bool {
    parse_version(s).is_some()
}

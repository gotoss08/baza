use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::config::PlatformArch;

#[derive(Parser, Debug)]
#[command(name = "baza", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
#[command(flatten_help = true)]
pub enum Command {
    /// Run the 1C executable with the specified connection target and infobase reference.
    ///
    /// User can set launcher path via 'BAZA_LAUNCHER_PATH' environment variable or by specifying 'launcher_path' in config file. If neither is set, the default launcher path is used.
    Run(RunArgs),

    /// Check the existence and size of an infobase on a DB server.
    Check(CheckArgs),

    /// Dump the infobase to a .dt file. If output path is not specified, the file is saved to the current directory. For DB authentication uses credentials from config file.
    ///
    /// User can set ibcmd.exe path via 'BAZA_IBCMD_PATH' environment variable or by specifying 'ibcmd_path' in config file.
    /// If neither is set, the automatic platform search will be performed. Prioritizing platform version from config file (field 'default_platform_version') if available, otherwise the latest available version will be used.
    Dump(DumpArgs),

    /// Clean up orphaned cache folders (not associated with registered infobases in ibases.v8i file).
    Clean(CleanArgs),

    /// Configure baza settings.
    Config(ConfigArgs),
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Connection target: file path, web URL, 1C server name, or full connection string.
    /// If omitted, the connection string is read from the clipboard.
    pub target: Option<String>,

    /// When `target` is a 1C server name, this is the infobase reference on that server.
    pub base: Option<String>,

    /// Read the connection string from stdin instead of arguments or clipboard.
    #[arg(long, conflicts_with_all = ["target", "base"])]
    pub stdin: bool,

    /// Username to use for infobase authentication.
    #[arg(short, long)]
    pub username: Option<String>,

    /// Password to use for infobase authentication.
    #[arg(short, long)]
    pub password: Option<String>,

    /// Interactive prompt for infobase authentication credentials (password hidden) from the user inside the terminal.
    #[arg(long)]
    pub auth: bool,

    /// Open the infobase in the 1C Designer.
    #[arg(short, long)]
    pub designer: bool,

    /// Use the x86 version of the 1C executable. If not specified, the default architecture from the config (field 'default_platform_arch') or the automatically determined architecture will be used.
    #[arg(long, conflicts_with = "x86_64")]
    pub x86: bool,

    /// Use the x86_64 version of the 1C executable. If not specified, the default architecture from the config (field 'default_platform_arch') or the automatically determined architecture will be used.
    #[arg(long, conflicts_with = "x86")]
    pub x86_64: bool,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Infobase name as it appears on the DB server.
    pub name: String,

    /// Restrict the check to a specific DB server from the config.
    /// If omitted, all configured servers are checked.
    #[arg(short, long)]
    pub server: Option<String>,

    /// Report the size of the infobase instead of just existence.
    #[arg(long)]
    pub size: bool,

    /// Report the size in bytes instead of human-readable format.
    #[arg(long, requires = "size")]
    pub bytes: bool,

    /// Check the servers synchronously (preserves base order from config).
    #[arg(long)]
    pub sync: bool,
}

#[derive(Args, Debug, Clone)]
pub struct DumpArgs {
    /// Database server name.
    pub server: String,

    /// Database base name as it appears on the DB server.
    pub name: String,

    /// Username to use for infobase authentication. Will be prompted if not specified.
    #[arg(short = 'u', long, visible_alias = "username")]
    pub ib_username: Option<String>,

    /// Password to use for infobase authentication. Will be prompted if not specified.
    #[arg(
        short = 'p',
        long,
        visible_alias = "password",
        requires = "ib_username"
    )]
    pub ib_password: Option<String>,

    /// Platform architecture. If not specified, the default architecture from the config (field 'default_platform_arch') or the automatically determined architecture will be used.
    #[arg(short, long, value_enum)]
    pub arch: Option<PlatformArch>,

    /// Platform version in default format (e.g. x.x.x.x). If not specified, the default version from the config (field 'default_platform_version') or the latest available version will be used.
    #[arg(short = 'v', long)]
    pub platform_version: Option<String>,

    /// Output .dt file path.
    #[arg(short, long)]
    pub out: Option<PathBuf>,

    /// Show verbose output
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Args, Debug)]
pub struct CleanArgs {
    /// Show what would be removed without deleting anything.
    #[arg(long)]
    pub dry_run: bool,

    /// Remove all cache folders, not just orphaned.
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// Open the config folder in the default file explorer.
    #[arg(short, long)]
    pub open: bool,
}

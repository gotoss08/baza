# baza

A command-line tool for managing **1C:Enterprise** infobases on Windows. Written in Rust.

`baza` wraps the most common day-to-day tasks — opening a base, checking whether it exists on a DB server, dumping it to a `.dt` file, and cleaning up stale cache folders — into a single ergonomic CLI.

---

## Features

- **`run`** — Open any infobase (file, web publication, or 1C server) in Enterprise or Designer mode. Reads the connection string from CLI args, clipboard, or stdin.
- **`check`** — Query one or all configured DB servers (PostgreSQL / MS SQL) to verify an infobase exists and, optionally, report its size.
- **`dump`** — Export an infobase to a `.dt` archive via `ibcmd.exe`. Auto-discovers the highest installed platform version.
- **`clean`** — Remove orphaned 1C cache folders whose UUIDs no longer appear in `ibases.v8i`.
- **`config`** — Open the `baza` config directory in the system file explorer.

---

## Installation

**Prerequisites:** Rust toolchain (`cargo`).

```sh
cargo install --path .
```

The binary is placed in `~/.cargo/bin/baza` (or the Cargo bin directory on your `PATH`).

---

## Configuration

On first run `baza` creates an empty config file. To open the config directory:

```sh
baza config --open
```

**Location:**

| OS      | Path                                             |
|---------|--------------------------------------------------|
| Windows | `%APPDATA%\Roaming\baza\config\config.toml`      |
| macOS   | `~/Library/Application Support/baza/config.toml` |
| Linux   | `~/.config/baza/config.toml`                     |

### config.toml reference

```toml
# Path to 1cestart.exe (optional — auto-discovered from %ProgramFiles% if omitted)
launcher_path = "C:\\Program Files\\1cv8\\common\\1cestart.exe"

# Path to ibcmd.exe (optional — auto-discovered from %ProgramFiles% if omitted)
ibcmd_path = "C:\\Program Files\\1cv8\\8.3.25.1517\\bin\\ibcmd.exe"

# Default platform architecture for run/dump: "x86" or "x86_64"
default_platform_arch = "x86_64"

# Default platform version used when searching for ibcmd.exe (format: N.N.N.N)
default_platform_version = "8.3.25.1517"

# Default infobase credentials used by `run`
default_ib_username = "admin"
default_ib_password = "secret"

# One entry per database server
[[servers]]
kind     = "postgres"   # "postgres" or "mssql"
host     = "pg-server"
port     = 5432         # optional; defaults: postgres=5432, mssql=1433
user     = "sa"
password = "db-secret"

[[servers]]
kind     = "mssql"
host     = "sql-server"
user     = "sa"
password = "db-secret"
```

### Environment variables

| Variable            | Purpose                                       |
|---------------------|-----------------------------------------------|
| `BAZA_LAUNCHER_PATH`| Override path to `1cestart.exe`              |
| `BAZA_IBCMD_PATH`   | Override path to `ibcmd.exe`                 |

Environment variables take priority over `config.toml`. If set, the path must point to an existing file — `baza` will not silently fall back.

---

## Commands

### `baza run`

Open a 1C infobase.

```
baza run [TARGET [BASE]] [OPTIONS]
```

**Target forms:**

| Form | Example |
|------|---------|
| File path | `baza run "C:\bases\mybase"` |
| Web URL | `baza run "http://srv/base"` |
| Full connection string | `baza run 'File="C:\base"'` |
| Server + infobase name | `baza run myserver mybase` |
| Clipboard (no args) | `baza run` |
| Stdin | `baza run --stdin` |

**Options:**

| Flag | Description |
|------|-------------|
| `--stdin` | Read connection string from stdin instead of clipboard |
| `-u`, `--username` | Infobase username |
| `-p`, `--password` | Infobase password |
| `--auth` | Prompt for credentials interactively (password hidden) |
| `-d`, `--designer` | Open in Designer mode (default: Enterprise) |
| `--x86` | Force 32-bit platform |
| `--x86-64` | Force 64-bit platform |

**Examples:**

```sh
# Open a file base
baza run "C:\bases\accounting"

# Open a server base in Designer
baza run myserver accounting --designer

# Open the base whose connection string is in the clipboard
baza run

# Pipe a connection string from another tool
echo 'Srvr="srv";Ref="base"' | baza run --stdin

# Prompt for credentials interactively
baza run --stdin --auth
```

---

### `baza check`

Check whether an infobase exists on configured DB servers.

```
baza check <NAME> [OPTIONS]
```

**Options:**

| Flag | Description |
|------|-------------|
| `-s`, `--server <HOST>` | Restrict check to one server |
| `--size` | Report infobase size instead of just existence |
| `--bytes` | Report size in raw bytes (requires `--size`) |
| `--sync` | Run checks sequentially, preserving server order from config |

**Examples:**

```sh
# Check all servers for a base named "accounting"
baza check accounting

# Check only one server
baza check accounting -s pg-server

# Show size in human-readable format
baza check accounting --size

# Show size in bytes, one server only
baza check accounting -s sql-server --size --bytes
```

Output prints `host:port` for every server where the base was found. With `--size`, output is `host:port: 1.2 GB`.

---

### `baza dump`

Export an infobase to a `.dt` file using `ibcmd.exe`.

```
baza dump <SERVER> <NAME> [OPTIONS]
```

The `SERVER` argument must match the `host` field of a server entry in `config.toml`.

If `--ib-username` / `--ib-password` are not provided, `baza` prompts for them interactively (password input is hidden).

**Options:**

| Flag | Description |
|------|-------------|
| `-u`, `--ib-username` | Infobase username |
| `-p`, `--ib-password` | Infobase password (requires `--ib-username`) |
| `-a`, `--arch` | Platform architecture (`x86` / `x86_64`) |
| `-v`, `--platform-version` | Exact platform version (`N.N.N.N`) |
| `-o`, `--out` | Output `.dt` file path |
| `--verbose` | Print the resolved `ibcmd.exe` path before running |

When `--out` is omitted, the dump is saved to the current directory as `<NAME>_<DDMMYYYY>.dt`.

**`ibcmd.exe` resolution order:**

1. `--arch` / `--platform-version` CLI flags → scans `%ProgramFiles%\1cv8\` for matching version
2. `BAZA_IBCMD_PATH` environment variable
3. `ibcmd_path` in `config.toml`
4. Auto-discovery: highest installed version under `%ProgramFiles%\1cv8\` (x86_64 preferred)

**Examples:**

```sh
# Dump with interactive credential prompt
baza dump pg-server accounting

# Dump with credentials, specific output path
baza dump pg-server accounting -u admin -p secret -o C:\backups\accounting.dt

# Use a specific platform version
baza dump pg-server accounting -v 8.3.25.1517 --verbose
```

---

### `baza clean`

Remove orphaned 1C cache folders — UUID-named directories under `%LOCALAPPDATA%\1C\1cv8` and `%APPDATA%\1C\1cv8` whose IDs are not registered in `ibases.v8i`.

```
baza clean [OPTIONS]
```

**Options:**

| Flag | Description |
|------|-------------|
| `--dry-run` | Print what would be deleted without removing anything |
| `--all` | Remove all UUID cache folders, not just orphaned ones |

**Examples:**

```sh
# Preview orphaned cache folders
baza clean --dry-run

# Remove orphaned cache folders
baza clean

# Remove all 1C cache folders
baza clean --all
```

---

### `baza config`

Manage `baza` configuration.

```
baza config [OPTIONS]
```

**Options:**

| Flag | Description |
|------|-------------|
| `-o`, `--open` | Open the config directory in the system file explorer |

---

## Project structure

```
src/
├── main.rs            # Entry point, command dispatch
├── lib.rs             # Crate root, public module declarations
├── cli.rs             # Clap CLI definitions (Cli, Command, *Args structs)
├── config.rs          # Config file loading/saving (TOML), Server/PlatformArch types
├── input.rs           # Clipboard, stdin, and interactive credential readers
├── utils.rs           # Human-readable byte formatting
├── commands/
│   ├── run.rs         # `run` command — resolve target, launch 1cestart.exe
│   ├── check.rs       # `check` command — parallel DB existence/size queries
│   ├── dump.rs        # `dump` command — resolve credentials, invoke ibcmd
│   ├── clean.rs       # `clean` command — scan and remove cache folders
│   └── config.rs      # `config` command — open config folder
├── db/
│   ├── mod.rs         # Unified base_exists / base_size dispatch
│   ├── postgres.rs    # tokio-postgres queries
│   └── mssql.rs       # tiberius (MS SQL) queries
└── onec/
    ├── connection.rs  # 1C connection string parser (File / Web / Server targets)
    ├── launcher.rs    # 1cestart.exe discovery and invocation
    ├── ibcmd.rs       # ibcmd.exe discovery and dump invocation
    ├── ibases.rs      # ibases.v8i reader/writer
    └── cache.rs       # 1C cache folder scanner
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `tokio` | Async runtime |
| `tokio-postgres` | PostgreSQL client |
| `tiberius` + `tokio-util` | MS SQL Server client |
| `serde` + `toml` | Config file serialization |
| `directories` | Platform-appropriate config directory |
| `arboard` | Clipboard access |
| `url` | URL parsing for web publications |
| `uuid` | UUID generation for `ibases.v8i` entries |
| `rpassword` | Hidden password prompt |
| `chrono` | Timestamp in dump file names |
| `anyhow` | Error handling |

---

## License

[MIT](LICENSE.md)

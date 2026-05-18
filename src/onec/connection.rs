use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use anyhow::{anyhow, bail};

use crate::Result;

/// All known shapes of a 1C connection target.
#[derive(Debug, Clone)]
pub enum Target {
    /// File-based infobase, e.g. File="C:\base\"
    File(PathBuf),
    /// Web publication, e.g. ws="http://host/base"
    Web(String),
    /// 1C cluster server reference, e.g. Srvr="srv";Ref="base"
    Server { srvr: String, refname: String },
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Target::File(path) => write!(f, "File={}", path.display()),
            Target::Web(url) => write!(f, "Web={}", url),
            Target::Server { srvr, refname } => write!(f, "Srvr={};Ref={}", srvr, refname),
        }
    }
}

/// Parses a 1C connection string of the form `Key="value";Key="value";`.
///
/// Returns a classified `Target` based on which keys are present.
pub fn parse(input: &str) -> Result<Target> {
    let fields = parse_fields(input)?;

    if let Some(path) = fields.get("file") {
        return Ok(Target::File(PathBuf::from(path)));
    }
    if let (Some(srvr), Some(refname)) = (fields.get("srvr"), fields.get("ref")) {
        return Ok(Target::Server {
            srvr: srvr.clone(),
            refname: refname.clone(),
        });
    }
    if let Some(url) = fields.get("ws") {
        return Ok(Target::Web(url.clone()));
    }

    bail!("connection string contains no recognized target (File, Srvr+Ref, or ws)")
}

/// Parses the raw `Key="value";` format into a flat map.
/// Values are taken verbatim — no escape processing.
/// Keys are case-sensitive (1C convention).
fn parse_fields(input: &str) -> Result<HashMap<String, String>> {
    let mut fields = HashMap::new();
    let mut rest = input.trim();

    while !rest.is_empty() {
        // Skip leading semicolons and whitespace between pairs
        rest = rest.trim_start_matches(|c: char| c == ';' || c.is_whitespace());
        if rest.is_empty() {
            break;
        }

        // Read key up to '='
        let eq_idx = rest
            .find('=')
            .ok_or_else(|| anyhow!("expected '=' near: {}", snippet(rest)))?;
        let key = rest[..eq_idx].trim().to_string();
        if key.is_empty() {
            bail!("empty key near: {}", snippet(rest));
        }
        rest = &rest[eq_idx + 1..];

        // Expect opening quote
        let after_eq = rest.trim_start();
        if !after_eq.starts_with('"') {
            bail!("expected '\"' after '=' for key '{key}'");
        }
        rest = &after_eq[1..];

        // Find closing quote — first one wins, no escapes
        let close_idx = rest
            .find('"')
            .ok_or_else(|| anyhow!("unterminated value for key '{key}'"))?;
        let value = rest[..close_idx].to_string();
        rest = &rest[close_idx + 1..];

        fields.insert(key.to_ascii_lowercase(), value);
    }

    Ok(fields)
}

/// Returns the first 30 chars of `s` for error messages.
fn snippet(s: &str) -> String {
    s.chars().take(30).collect::<String>() + if s.chars().count() > 30 { "…" } else { "" }
}

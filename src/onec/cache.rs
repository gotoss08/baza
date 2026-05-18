use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Context;

use crate::Result;

/// A 1C cache folder found on disk.
#[derive(Debug)]
pub struct CacheEntry {
    /// Full path to the cache folder.
    pub path: PathBuf,
    /// UUID extracted from the folder name (lowercase).
    pub id: String,
}

/// Returns the two standard 1C cache directories.
pub fn cache_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local).join("1C").join("1cv8"));
    }
    if let Some(roaming) = std::env::var_os("APPDATA") {
        dirs.push(PathBuf::from(roaming).join("1C").join("1cv8"));
    }
    dirs
}

/// Scans the cache directories and returns folders with UUID-shaped names.
pub fn scan() -> Result<Vec<CacheEntry>> {
    let mut entries = Vec::new();

    for dir in cache_dirs() {
        if !dir.is_dir() {
            continue;
        }
        let read = std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))?;

        for entry in read {
            let entry = entry.with_context(|| format!("listing {}", dir.display()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if looks_like_uuid(name) {
                entries.push(CacheEntry {
                    id: name.to_ascii_lowercase(),
                    path,
                });
            }
        }
    }

    Ok(entries)
}

/// Returns cache entries whose IDs are NOT present in the `known` set.
pub fn orphaned(known: &HashSet<String>) -> Result<Vec<CacheEntry>> {
    Ok(scan()?
        .into_iter()
        .filter(|e| !known.contains(&e.id))
        .collect())
}

/// Strict UUID check: 8-4-4-4-12 hex chars separated by dashes.
fn looks_like_uuid(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (i, &b) in bytes.iter().enumerate() {
        match i {
            8 | 13 | 18 | 23 => {
                if b != b'-' {
                    return false;
                }
            }
            _ => {
                if !b.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_valid_uuid() {
        assert!(looks_like_uuid("7571f760-527e-4eb4-9188-0d3ca6634b05"));
        assert!(looks_like_uuid("b703bef2-4068-49f4-a7c3-06729c863e41"));
    }

    #[test]
    fn rejects_non_uuid() {
        assert!(!looks_like_uuid(""));
        assert!(!looks_like_uuid("Templates"));
        assert!(!looks_like_uuid("7571f760_527e_4eb4_9188_0d3ca6634b05"));
        assert!(!looks_like_uuid("7571f760-527e-4eb4-9188-0d3ca6634b0")); // 35 chars
        assert!(!looks_like_uuid("7571f760-527e-4eb4-9188-0d3ca6634b056")); // 37 chars
        assert!(!looks_like_uuid("xxxxxxxx-527e-4eb4-9188-0d3ca6634b05")); // non-hex
    }
}

use std::collections::HashSet;
use std::fmt::Write;
use std::path::PathBuf;

use anyhow::{Context, anyhow};
use uuid::Uuid;

use crate::Result;

/// Represents the ibases file content and provides methods to read, modify, and save it.
pub struct Ibases {
    /// The path to the ibases file.
    path: PathBuf,

    /// The ibases file content.
    content: String,

    /// The original length of the ibases file content.
    original_len: usize,
}

impl Ibases {
    /// Reads the ibases content from the default path.
    pub fn new() -> Result<Self> {
        let path = path()?;
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let original_len = content.len();
        Ok(Self {
            path,
            content,
            original_len,
        })
    }

    /// Returns all infobase IDs currently registered in ibases.v8i.
    /// IDs are lowercased for case-insensitive comparison.
    pub fn registered_ids(&self) -> HashSet<String> {
        extract_ids(&self.content)
    }

    /// Returns a unique infobase ID that is not already registered in ibases.v8i.
    pub fn unique_id(&self) -> Uuid {
        loop {
            let id = Uuid::new_v4();
            if !self.registered_ids().contains(&id.to_string()) {
                return id;
            }
        }
    }

    /// Returns whether the given name is present in the ibases content.
    pub fn contains_name(&self, name: &str) -> bool {
        let needle = format!("[{}]", name);
        self.content
            .lines()
            .any(|line| line.trim().starts_with(&needle))
    }

    /// Registers a new folder with the given name.
    pub fn register_folder(&mut self, name: &str) -> Result<()> {
        if !self.content.ends_with("\r\n") {
            self.content.push('\r');
            self.content.push('\n');
        }
        let id = self.unique_id();
        write!(
            self.content,
            "[{name}]\r\n\
            ID={id}\r\n\
            OrderInList=-1\r\n\
            Folder=/\r\n\
            External=0\r\n"
        )
        .context("writing ibases content")?;
        Ok(())
    }

    /// Registers a new base with the given name and connection string to the 'baza' folder.
    pub fn register_base(&mut self, name: &str, connect: &str) -> Result<()> {
        if !self.content.ends_with("\r\n") {
            self.content.push('\r');
            self.content.push('\n');
        }
        let id = self.unique_id();
        write!(
            self.content,
            "[{name}]\r\n\
            Connect={connect}\r\n\
            ID={id}\r\n\
            OrderInList=-1\r\n\
            Folder=/baza\r\n\
            External=0\r\n"
        )
        .context("writing ibases content")?;
        Ok(())
    }

    /// Saves the current content back to ibases.v8i.
    pub fn save(&mut self) -> Result<()> {
        std::fs::write(&self.path, &self.content)
            .with_context(|| format!("writing {}", self.path.display()))
            .inspect_err(|_| self.content.truncate(self.original_len))?;
        self.original_len = self.content.len();
        Ok(())
    }
}

/// Returns the path to ibases.v8i in the user's AppData.
pub fn path() -> Result<PathBuf> {
    let appdata = std::env::var_os("APPDATA")
        .ok_or_else(|| anyhow!("APPDATA environment variable is not set"))?;
    Ok(PathBuf::from(appdata)
        .join("1C")
        .join("1CEStart")
        .join("ibases.v8i"))
}

/// Extracts all infobase IDs from the given content.
fn extract_ids(content: &str) -> HashSet<String> {
    content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("ID="))
        .map(|id| id.trim().to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_ids_from_typical_file() {
        let content = "\
[test base]
Connect=File=\"c:\\foo\"
ID=b703bef2-4068-49f4-a7c3-06729c863e41
OrderInList=1

[another base]
Connect=File=\"c:\\bar\"
ID=7571f760-527e-4eb4-9188-0d3ca6634b05
OrderInList=2
";
        let ids = extract_ids(content);
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("b703bef2-4068-49f4-a7c3-06729c863e41"));
        assert!(ids.contains("7571f760-527e-4eb4-9188-0d3ca6634b05"));
    }

    #[test]
    fn case_normalises_ids() {
        let content = "ID=AAAA1111-BBBB-CCCC-DDDD-EEEEEEEEEEEE";
        let ids = extract_ids(content);
        assert!(ids.contains("aaaa1111-bbbb-cccc-dddd-eeeeeeeeeeee"));
    }
}

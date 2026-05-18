use anyhow::Context;
use std::io::{self, BufRead, IsTerminal, Read, Write};

use crate::Result;

/// Reads connection string from the system clipboard.
pub fn from_clipboard() -> Result<String> {
    let mut cb = arboard::Clipboard::new().context("opening system clipboard")?;
    cb.get_text().context("reading clipboard contents")
}

/// Reads connection string from stdin until EOF.
pub fn from_stdin() -> Result<String> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        // Interactive: read a single line until Enter
        io::stderr().flush().ok();
        let mut line = String::new();
        stdin
            .lock()
            .read_line(&mut line)
            .context("reading line from stdin")?;
        Ok(line.trim_end_matches(['\r', '\n']).to_string())
    } else {
        // Piped: read everything until EOF
        let mut buf = String::new();
        stdin
            .lock()
            .read_to_string(&mut buf)
            .context("reading piped stdin")?;
        Ok(buf.trim().to_string())
    }
}

/// Reads connection string from stdin until EOF.
pub fn from_stdin_with_prompt(prompt: &str) -> Result<String> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        eprint!("{prompt}");
        io::stderr().flush().ok();
    }
    from_stdin()
}

pub fn from_stdin_with_prompt_no_echo(prompt: &str) -> Result<String> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        eprint!("{prompt}");
        io::stderr().flush().ok();
    }
    let secure = rpassword::read_password().context("reading password from stdin")?;
    Ok(secure)
}

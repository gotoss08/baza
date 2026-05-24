//! baza — 1C infobase management CLI.

pub mod cli;
pub mod commands;
pub mod config;
pub mod db;
pub mod input;
pub mod onec;
pub mod telegram;
pub mod utils;

/// Project-wide Result alias.
pub type Result<T> = anyhow::Result<T>;

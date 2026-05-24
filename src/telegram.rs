use anyhow::{Context, ensure};
use reqwest::Client;
use serde::Serialize;

use crate::Result;
use crate::config::Config;

#[derive(Serialize)]
struct SendMessage<'a> {
    chat_id: i64,
    text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    parse_mode: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_notification: Option<bool>,
}

pub async fn send_message(cfg: &Config, text: &str) -> Result<()> {
    send_message_ex(cfg, text, false, true).await
}

pub async fn send_message_ex(cfg: &Config, text: &str, silent: bool, markdown: bool) -> Result<()> {
    ensure!(
        cfg.telegram_notifications_enabled,
        "Telegram notifications are disabled in config"
    );
    ensure!(
        cfg.telegram_bot_token.is_some(),
        "Telegram bot token is not set in config"
    );
    ensure!(
        cfg.telegram_chat_id.is_some(),
        "Telegram chat ID is not set in config"
    );

    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        cfg.telegram_bot_token.as_ref().unwrap()
    );

    let client = Client::new();

    let payload = SendMessage {
        chat_id: cfg.telegram_chat_id.unwrap(),
        text: text,
        parse_mode: match markdown {
            true => Some("MarkdownV2"),
            false => None,
        },
        disable_notification: match silent {
            true => Some(true),
            false => None,
        },
    };

    let res = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("sending telegram API POST sendMessage")?;

    if !res.status().is_success() {
        anyhow::bail!("Telegram API returned error status: {}", res.status());
    }

    Ok(())
}

pub fn escape_markdown(text: &str) -> String {
    let special_chars = [
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];
    text.chars()
        .flat_map(|c| {
            if special_chars.contains(&c) {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}

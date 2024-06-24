use crate::settings::Settings;
use crate::data::Data;
use anyhow::Result;
use teloxide::prelude::*;
use teloxide::types::Message;
use tracing::info;

pub async fn start(settings: &Settings, data: Data) -> Result<()> {
    let token = &settings.tg_token;
    info!("Starting echo bot...");

    let bot = Bot::new(token);

    teloxide::repl(bot, |message: Message, bot: Bot| async move {
        if let Some(text) = message.text() {
            bot.send_message(message.chat.id, text).await?;
        }
        respond(())
    })
    .await;

    Ok(())
}


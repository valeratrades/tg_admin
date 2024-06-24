use std::sync::Arc;

use crate::data::{Data, Level};
use crate::settings::Settings;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use teloxide::prelude::*;
use teloxide::types::Message;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

#[tracing::instrument]
pub async fn start(settings: Arc<Settings>, data: Data) -> Result<()> {
	let token = &settings.tg_token;

	let bot = Bot::new(token);
	teloxide::repl(bot, move |message: Message, bot: Bot| {
		let data = data.clone();
		let level = Level::default();
		let settings = Arc::clone(&settings);

		async move {
			if let Some(admin_list) = &settings.admin_list {
				if !admin_list.contains(&message.chat.id.0) {
					bot.send_message(message.chat.id, "Access denied.").await?;
					return respond(());
				}
			}

			if let Some(text) = message.text() {
				if text == "/admin" {
					let markup = render_markup(&data, &level);
					bot.send_message(message.chat.id, "Admin Menu").reply_markup(markup).await?;
				}
			}
			respond(())
		}
	})
	.await;

	Ok(())
}

fn render_markup(data: &Data, level: &Level) -> InlineKeyboardMarkup {
	let items = level_representation(&data.at(level).unwrap());
	create_markup(items, level)
}

//TODO!: should send callbacks with full path to the new subarea of the data
fn create_markup(items: Vec<MarkdownItem>, level: &Level) -> InlineKeyboardMarkup {
	let mut keyboard = vec![];

	for item in items {
		let callback_data = level.join(&item.key).into_string();
		let button = InlineKeyboardButton::callback(item.full_text, callback_data);
		keyboard.push(vec![button]);
	}

	InlineKeyboardMarkup::new(keyboard)
}

#[derive(Clone, Debug, Default, derive_new::new, Serialize, Deserialize)]
struct MarkdownItem {
	pub key: String,
	pub full_text: String,
}

fn level_representation(current_level: &Value) -> Vec<MarkdownItem> {
	let mut result = Vec::new();

	if let Value::Object(map) = current_level {
		for (key, val) in map {
			let f = match val {
				Value::Object(_) => format!("{} {}", "{}".to_string(), key),
				Value::Array(_) => format!("{} {}", "[]".to_string(), key),
				_ => format!("{}: {}", key, val.to_string()),
			};
			let markdown_item = MarkdownItem::new(key.to_string(), f);
			result.push(markdown_item);
		}
	}

	result
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn test_level_representation() {
		let json_value = json!({
			"name": "Alice",
			"age": 25,
			"address": {
				"street": "456 Another St",
				"city": "Elsewhere"
			},
			"emails": ["alice@example.com", "a@example.com"]
		});

		let formatted_output = level_representation(&json_value);

		insta::assert_json_snapshot!(
			formatted_output,
			@r###"
  [
    {
      "key": "address",
      "full_text": "{} address"
    },
    {
      "key": "age",
      "full_text": "age: 25"
    },
    {
      "key": "emails",
      "full_text": "[] emails"
    },
    {
      "key": "name",
      "full_text": "name: \"Alice\""
    }
  ]
  "###
		);
	}
}

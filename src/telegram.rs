use crate::settings::Settings;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::data::Data;
use anyhow::Result;
use serde_json::Value;
use teloxide::prelude::*;
use teloxide::types::Message;

#[tracing::instrument]
pub async fn start(settings: &Settings, data: Data) -> Result<()> {
	let token = &settings.tg_token;

	let bot = Bot::new(token);
	teloxide::repl(bot, move |message: Message, bot: Bot| {
		let data = data.clone();

		async move {
			if let Some(text) = message.text() {
				if text == "/admin" {
					let markup = render_markup(data.as_ref());
					bot.send_message(message.chat.id, "Admin Menu")
						.reply_markup(markup)
					.await?;
				}
			}
			respond(())
		}
	})
	.await;

	Ok(())
}

fn render_markup(current_level: &Value) -> InlineKeyboardMarkup {
	let items = level_representation(current_level);
	create_markup(items)
}

//TODO!: should send callbacks with full path to the new subarea of the data
fn create_markup(items: Vec<String>) -> InlineKeyboardMarkup {
    let mut keyboard = vec![];

    for item in items {
        let parts: Vec<&str> = item.split(": ").collect();
        if parts.len() == 2 {
            let key = parts[0].to_string();
            let button_text = key.clone();
            let callback_data = key;

            let button = InlineKeyboardButton::callback(button_text, callback_data);
            keyboard.push(vec![button]);
        }
    }

    InlineKeyboardMarkup::new(keyboard)
}

fn level_representation(current_level: &Value) -> Vec<String> {
	let mut result = Vec::new();

	if let Value::Object(map) = current_level {
		for (key, val) in map {
			let formatted_value = match val {
				Value::Object(_) => "{}".to_string(),
				Value::Array(_) => "[]".to_string(),
				_ => val.to_string(),
			};

			let formatted_pair = format!("{}: {}", key, formatted_value);
			result.push(formatted_pair);
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
    "address: {}",
    "age: 25",
    "emails: []",
    "name: \"Alice\""
  ]
  "###
		);
	}
}

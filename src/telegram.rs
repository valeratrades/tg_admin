use crate::data::{Data, Level};
use crate::settings::Settings;
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};

#[tracing::instrument]
pub async fn start(settings: Arc<Settings>, data: Data) -> Result<()> {
	let token = &settings.tg_token;

	let bot = teloxide::Bot::new(token);

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
	fn escape_markdown(s: &str) -> String {
		//s.replace('_', r"\_")
		//	.replace('*', r"\*")
		//	.replace('[', r"\[")
		//	.replace(']', r"\]")
		//	.replace('(', r"\(")
		//	.replace(')', r"\)")
		//	.replace('~', r"\~")
		//	.replace('`', r"\`")
		//	.replace('>', r"\>")
		//	.replace('#', r"\#")
		//	.replace('+', r"\+")
		//	.replace('-', r"\-")
		//	.replace('=', r"\=")
		//	.replace('|', r"\|")
		//	.replace('.', r"\.")
		//	.replace('!', r"\!")
		// For some reason can't render it, even with .parse_mode(ParseMode::MarkdownV2) on the bot or send_message
		s.to_string()
	}

	let mut keyboard = Vec::new();
	let current_level = &data.at(level).unwrap();
	if let Value::Object(map) = current_level {
		if !level.is_top() {
			let mut l = level.clone();
			l.pop();
			let callback_data = l.to_string();
			let button = InlineKeyboardButton::callback("..", callback_data);
			keyboard.push(vec![button]);
		}
		for (key, val) in map {
			let f = match val {
				Value::Object(_) => format!("`{}` {}", escape_markdown("{}"), escape_markdown(key)),
				Value::Array(_) => format!("`{}` {}", escape_markdown("[]"), escape_markdown(key)),
				_ => format!("{}: `{}`", escape_markdown(key), escape_markdown(&val.to_string())),
			};

			let callback_data = level.join(key).into_string();
			let button = InlineKeyboardButton::callback(f, callback_data);
			keyboard.push(vec![button]);
		}
	}
	InlineKeyboardMarkup::new(keyboard)
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn gen_data() -> (Data, Level) {
		let json_value = json!({
			"name": "Alice",
			"age": 25,
			"address": {
				"street": "456 Another St",
				"city": "Elsewhere"
			},
			"emails": ["alice@example.com", "a@example.com"]
		});
		(Data::mock(json_value), Level::default())
	}

	#[test]
	fn test_top_level_representation() {
		let (data, level) = gen_data();
		let r = render_markup(&data, &level);

		insta::assert_json_snapshot!(
			r,
			@r###"
  {
    "inline_keyboard": [
      [
        {
          "text": "`{}` address",
          "callback_data": "address"
        }
      ],
      [
        {
          "text": "age: `25`",
          "callback_data": "age"
        }
      ],
      [
        {
          "text": "`[]` emails",
          "callback_data": "emails"
        }
      ],
      [
        {
          "text": "name: `\"Alice\"`",
          "callback_data": "name"
        }
      ]
    ]
  }
  "###
		);
	}

	#[test]
	fn test_nested_level_representation() {
		let (data, mut level) = gen_data();
		level.push("address");
		let r = render_markup(&data, &level);
		insta::assert_json_snapshot!(
			r,
			@r###"
  {
    "inline_keyboard": [
      [
        {
          "text": "..",
          "callback_data": "address"
        }
      ],
      [
        {
          "text": "city: `\"Elsewhere\"`",
          "callback_data": "address::city"
        }
      ],
      [
        {
          "text": "street: `\"456 Another St\"`",
          "callback_data": "address::street"
        }
      ]
    ]
  }
  "###
		);
	}
}

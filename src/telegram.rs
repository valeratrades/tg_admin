use crate::data::{Data, ValuePath};
use crate::settings::Settings;
use anyhow::Result;
//use dptree::HandlerResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex, RwLock};
use teloxide::dispatching::dialogue::{self, GetChatId, InMemStorage};
use teloxide::dispatching::UpdateHandler;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message, MessageId};
use teloxide::utils::command::{self, BotCommands};
use tracing::info;

type MyDialogue = Dialogue<ChatState, InMemStorage<ChatState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>; //dbg

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
enum ChatState {
	/// Most actions are prohibited from this state. Other states can be reached only through authorization from here.
	#[default]
	Unauthorized,
	/// Dummy state, only here to not have to spawn Navigation with random value on authorization.
	Authorized,
	Navigation { message_id: i32 },
	Input(ValueInput),
}
#[derive(Clone, Debug, derive_new::new, Serialize, Deserialize, PartialEq, Eq)]
struct ValueInput {
	input_type: InputValueType,
	value_path: ValuePath,
}
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum InputValueType {
	UpdateAt,
	AddTo,
	RemoveFrom,
}
#[derive(BotCommands, Clone, Debug)]
#[command(description = "Commands:", rename_rule = "lowercase")]
enum Command {
	#[command(description = "Display all commands")]
	Help,
	#[command(description = "Open admin panel at the top")]
	Admin,
}

#[tracing::instrument]
pub async fn run(settings: Arc<Settings>, data: Arc<RwLock<Data>>) -> Result<()> {
	let token = &settings.tg_token;
	let bot = Bot::new(token);
	info!("Starting telegram bot...");
	Dispatcher::builder(bot, schema())
		.dependencies(dptree::deps![data, settings, InMemStorage::<ChatState>::new()])
		.error_handler(LoggingErrorHandler::with_custom_text("An error has occurred in the dispatcher"))
		.enable_ctrlc_handler()
		.build()
		.dispatch()
		.await;
	Ok(())
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
	use dptree::case;

	let command_handler = teloxide::filter_command::<Command, _>()
		.branch(case![Command::Help].endpoint(help_handler))
		.branch(case![Command::Admin].endpoint(admin_handler));

	let message_handler = Update::filter_message()
		.branch(command_handler)
		.branch(case![ChatState::Input(value_input)].endpoint(value_input_handler))
		.branch(dptree::endpoint(invalid_state_handler));

	let callback_query_handler = Update::filter_callback_query().endpoint(callback_query_handler);

	let auth_handler = dptree::filter_map_async(|dialogue: MyDialogue, settings: Arc<Settings>, update: Update| async move {
		match dialogue.get().await {
			Ok(Some(ChatState::Unauthorized)) => {
				if let Some(admin_list) = &settings.admin_list {
					let user_id = update.user().unwrap().id.0;
					if !admin_list.contains(&user_id) {
						return None;  // Not authorized
					}
				}
				dialogue.update(ChatState::Authorized).await.ok()?;
				Some(())  // Authorized
			},
			Ok(Some(_)) => Some(()),  // Already authorized
			_ => None,  // Error or no state, treat as unauthorized
		}
	});

	dialogue::enter::<Update, InMemStorage<ChatState>, ChatState, _>()
		.chain(auth_handler)
		.branch(message_handler)
		.branch(callback_query_handler)
}

async fn admin_handler(bot: Bot, msg: Message, dialogue: MyDialogue, data: Arc<RwLock<Data>>) -> HandlerResult {
	let value_path = ValuePath::default();
	let (header, markup) = {
		let data = data.read().unwrap();
		render_header_and_markup(&data, &value_path)
	
	};
	let sent_message = bot.send_message(msg.chat.id, &header)
		.reply_markup(markup)
	.await?;
	dialogue.update(ChatState::Navigation { message_id: sent_message.id.0 }).await?;
	Ok(())
}

async fn value_input_handler(bot: Bot, dialogue: MyDialogue, msg: Message, value_input: ValueInput, data: Arc<RwLock<Data>>) -> HandlerResult {
	// Should resend the message from which we went to the input state after we update the value and report the change.
	// Since it's new message, update the Navigation { message_id } accordingly.
	unimplemented!()
}

async fn invalid_state_handler(bot: Bot, msg: Message) -> HandlerResult {
	bot.send_message(msg.chat.id, "Unable to handle the message. Type /help to see available commands.")
		.await?;
	Ok(())
}
async fn help_handler(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
	bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
	Ok(())
}

async fn callback_query_handler(bot: Bot, dialogue: MyDialogue, q: CallbackQuery, data: Arc<RwLock<Data>>) -> HandlerResult {
	bot.answer_callback_query(q.id.clone()).await?; // normally this is done after, but I like how it stops for a moment before the action is performed. Otherwise looks cut.
	if let Some(j) = q.data {
		let action: CallbackAction = serde_json::from_str(&j).unwrap();
		match action {
			CallbackAction::Go(value_path) => {
				continue_navigation(bot.clone(), dialogue, data, value_path).await?;
			},
			CallbackAction::UpdateAt(value_path) => {
				dialogue.update(ChatState::Input(ValueInput::new(InputValueType::UpdateAt, value_path))).await?; //dbg
			},
			CallbackAction::AddTo(value_path) => {
				unimplemented!()
			},
			CallbackAction::RemoveFrom(value_path) => {
				unimplemented!()
			},
		}
	}
	Ok(())
}

async fn continue_navigation(bot: Bot, dialogue: MyDialogue, data: Arc<RwLock<Data>>, value_path: ValuePath) -> HandlerResult {
	let (header, markup) = {
		let data = data.read().unwrap();
		render_header_and_markup(&data, &value_path)
	};

	let state = dialogue.get().await.unwrap().unwrap();
	let message_id = match state {
		ChatState::Navigation { message_id } => message_id,
		_ => unreachable!(),
	};

	match bot.edit_message_text(dialogue.chat_id(), MessageId(message_id), &header)
		.reply_markup(markup.clone())
	.await
	{
		Ok(_) => Ok(()),
		//TODO!: assert that the err is about message being too old, as it's the only recoverable one.
		Err(err) => {
			dbg!(err);
			let sent_message = bot.send_message(dialogue.chat_id(), &header)
				.reply_markup(markup)
			.await?;
			dialogue.update(ChatState::Navigation { message_id: sent_message.id.0 }).await?;
			Ok(())
		}
	}
}

#[derive(Clone, Debug, Default, derive_new::new, Serialize, Deserialize, PartialEq, Eq)]
struct More {
	pub value_path: ValuePath,
	pub start_at: usize,
}
#[derive(Clone, Debug, derive_new::new, Serialize, Deserialize, PartialEq, Eq)]
enum CallbackAction {
	Go(ValuePath),
	UpdateAt(ValuePath),
	AddTo(ValuePath),
	RemoveFrom(ValuePath),
}

fn render_header_and_markup(data: &Data, value_path: &ValuePath) -> (String, InlineKeyboardMarkup) {
	let mut keyboard = Vec::new();
	let current_value_at_path = &data.at(value_path).unwrap();
	let mut header = "Admin Menu".to_owned();

	// Add parent navigation button if not at top level
	if !value_path.is_top() {
		let callback_data = value_path.parent().to_string();
		let button = InlineKeyboardButton::callback("..", callback_data);
		keyboard.push(vec![button]);
	}

	match current_value_at_path {
		Value::Object(map) => {
			for (key, val) in map {
				let (display_text, callback_data) = match val {
					Value::Object(_) => {
						(format!("{} {}", "{}", key), CallbackAction::Go(value_path.join(key)))
					},
					Value::Array(arr) => {
						(format!("[{}] {}", arr.len(), key), CallbackAction::Go(value_path.join(key)))
					},
					_ => {
						(format!("{}: {}", key, &val.to_string()), CallbackAction::UpdateAt(value_path.join(key)))
					},
				};

				let button = InlineKeyboardButton::callback(display_text, serde_json::to_string(&callback_data).unwrap());
				keyboard.push(vec![button]);
			}
		},
		Value::Array(arr) => {
			header = format!("[{}] {}", arr.len(), value_path.basename());

			let start = arr.len().saturating_sub(25);
			let mut array_str = "\n```json\n".to_owned();
			for a in arr.iter().skip(start) {
				array_str.push_str(&format!("{a}\n"));
			}
			array_str.push_str("```");
			header += &array_str;

			let bottom_row = vec![
				InlineKeyboardButton::callback("Add", value_path.join("add").into_string()),
				InlineKeyboardButton::callback("Remove", value_path.join("remove").into_string()),
			];
			//TODO!: make doubled horizontally `<-` and `->` buttons that modify starting position of the count
			keyboard.push(bottom_row);
		},
		_ => {
			unreachable!();
		}
	}

	(header, InlineKeyboardMarkup::new(keyboard))
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	fn gen_data() -> (Data, ValuePath) {
		let json_value = json!({
			"name": "Alice",
			"age": 25,
			"address": {
				"street": "456 Another St",
				"city": "Elsewhere"
			},
			"emails": ["alice@example.com", "a@example.com"]
		});
		(Data::mock(json_value), ValuePath::default())
	}

	#[test]
	fn test_top_value_path_representation() {
		let (data, value_path) = gen_data();
		let (_h, r) = render_header_and_markup(&data, &value_path);

		insta::assert_json_snapshot!(
			r,
			@r###"
  {
    "inline_keyboard": [
      [
        {
          "text": "{} address",
          "callback_data": "{\"Go\":\"::address\"}"
        }
      ],
      [
        {
          "text": "age: 25",
          "callback_data": "{\"UpdateAt\":\"::age\"}"
        }
      ],
      [
        {
          "text": "[2] emails",
          "callback_data": "{\"Go\":\"::emails\"}"
        }
      ],
      [
        {
          "text": "name: \"Alice\"",
          "callback_data": "{\"UpdateAt\":\"::name\"}"
        }
      ]
    ]
  }
  "###
		);
	}

	#[test]
	fn test_nested_value_path_representation() {
		let (data, mut value_path) = gen_data();
		value_path.push("address");
		let (_h, r) = render_header_and_markup(&data, &value_path);
		insta::assert_json_snapshot!(
			r,
			@r###"
  {
    "inline_keyboard": [
      [
        {
          "text": "..",
          "callback_data": "::"
        }
      ],
      [
        {
          "text": "city: \"Elsewhere\"",
          "callback_data": "{\"UpdateAt\":\"::address::city\"}"
        }
      ],
      [
        {
          "text": "street: \"456 Another St\"",
          "callback_data": "{\"UpdateAt\":\"::address::street\"}"
        }
      ]
    ]
  }
  "###
		);
	}
	#[test]
	fn test_array_value_path_representation() {
		let (data, mut value_path) = gen_data();
		value_path.push("emails");
		let (h, r) = render_header_and_markup(&data, &value_path);

		insta::assert_snapshot!(
			h,
			"Admin Menu",
		);

		insta::assert_json_snapshot!(
			r,
			@r###"
  {
    "inline_keyboard": [
      [
        {
          "text": "..",
          "callback_data": "::"
        }
      ],
      [
        {
          "text": "[0] \"alice@example.com\"",
          "callback_data": "::emails::0"
        }
      ],
      [
        {
          "text": "[1] \"a@example.com\"",
          "callback_data": "::emails::1"
        }
      ],
      [
        {
          "text": "..",
          "callback_data": "::"
        },
        {
          "text": "Add",
          "callback_data": "::emails::add"
        },
        {
          "text": "Remove",
          "callback_data": "::emails::remove"
        }
      ]
    ]
  }
  "###
		);
	}
}

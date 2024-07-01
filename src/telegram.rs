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
use teloxide::types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message};
use teloxide::utils::command::{self, BotCommands};
use tracing::info;

type MyDialogue = Dialogue<ChatState, InMemStorage<ChatState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>; //dbg

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
enum ChatState {
	/// Most actions are prohibited from this state. Other states can be reached only through authorization from here.
	#[default]
	Unauthorized,
	Navigation,
	Input(ValueInput),
}
#[derive(Clone, Debug, derive_new::new, Serialize, Deserialize, PartialEq, Eq)]
struct ValueInput {
	input_type: InputType,
	value_path: ValuePath,
	new_value: Value,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum InputType {
	UpdateValue,
	AddToValue,
	RemoveFromValue,
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

	dialogue::enter::<Update, InMemStorage<ChatState>, ChatState, _>()
		.branch(
			case![ChatState::Unauthorized]
				.endpoint(|bot, dialogue, update, settings, deps| async move { authorize_handler(bot, dialogue, update, settings, deps).await }),
		)
		.branch(message_handler)
		.branch(callback_query_handler)
}

async fn admin_handler(bot: Bot, dialogue: MyDialogue, msg: Message, data: Arc<RwLock<Data>>) -> HandlerResult {
	let value_path = ValuePath::default();
	let markup = {
		let data = data.read().unwrap();
		let m = render_markup(&data, &value_path);
		m
	};
	bot.send_message(msg.chat.id, "Admin Menu").reply_markup(markup).await?;
	Ok(())
}

async fn value_input_handler(bot: Bot, dialogue: MyDialogue, msg: Message, value_input: ValueInput, data: Arc<RwLock<Data>>) -> HandlerResult {
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

async fn callback_query_handler(bot: Bot, dialogue: MyDialogue, query: CallbackQuery) -> HandlerResult {
	unimplemented!()
}

async fn authorize_handler(bot: Bot, dialogue: MyDialogue, update: Update, settings: Arc<Settings>, deps: DependencyMap) -> HandlerResult {
	if let Some(admin_list) = &settings.admin_list {
		let user_id = update.user().unwrap().id.0;

		if !admin_list.contains(&user_id) {
			bot.send_message(update.chat_id().unwrap(), "Access denied.").await?;
			return Ok(());
		}
	}

	dialogue.update(ChatState::Navigation).await?;
	// Re-process the message as if nothing happened
	tokio::spawn(async move {
		schema().dispatch(deps).await;
	});

	Ok(())
}

fn render_markup(data: &Data, value_path: &ValuePath) -> InlineKeyboardMarkup {
	let mut keyboard = Vec::new();
	let current_value_path = &data.at(value_path).unwrap();
	if let Value::Object(map) = current_value_path {
		if !value_path.is_top() {
			let mut l = value_path.clone();
			l.pop();
			let callback_data = l.to_string();
			let button = InlineKeyboardButton::callback("..", callback_data);
			keyboard.push(vec![button]);
		}
		for (key, val) in map {
			let f = match val {
				Value::Object(_) => format!("`{}` {}", "{}", key),
				Value::Array(_) => format!("`{}` {}", "[]", key),
				_ => format!("{}: `{}`", key, &val.to_string()),
			};

			let callback_data = value_path.join(key).into_string();
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
		let r = render_markup(&data, &value_path);

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
	fn test_nested_value_path_representation() {
		let (data, mut value_path) = gen_data();
		value_path.push("address");
		let r = render_markup(&data, &value_path);
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

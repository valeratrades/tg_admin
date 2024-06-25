use std::sync::{Arc, RwLock};

use clap::{Args, Parser, Subcommand};
use settings::Settings;
use v_utils::io::ExpandedPath;
pub mod data;
pub mod settings;
pub mod telegram;

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	#[arg(long, default_value = "~/.config/tg_admin.toml")]
	config: ExpandedPath,
	#[command(subcommand)]
	command: Commands,
}
#[derive(Subcommand, Debug)]
pub enum Commands {
	/// Start the server allowing to change the config at the specified path using telegram.
	///Ex
	///```sh
	///tg_config start -t "${THE_BOT_TOKEN}" ./config/config.json
	///```
	Manage(ManageArgs),
}
impl Default for Commands {
	fn default() -> Self {
		Self::Manage(ManageArgs::default())
	}
}

#[derive(Args, Debug, Default)]
pub struct ManageArgs {
	/// Path to the target file
	path: ExpandedPath,
	/// Override token in config
	#[arg(short, long)]
	tg_token: Option<String>,
	/// Users with admin rights
	#[arg(short, long)]
	admin_list: Option<Vec<u64>>,
}

#[tokio::main]
async fn main() {
	v_utils::utils::init_subscriber();
	let cli = Cli::parse();
	let app_config = match Settings::new_with_cli(&cli) {
		Ok(config) => config,
		Err(e) => {
			eprintln!("Error: Failed to initialize settings with CLI arguments. Details: {}", e);
			std::process::exit(1);
		}
	};

	match &cli.command {
		Commands::Manage(args) => {
			let target_data = match data::Data::load(args.path.as_ref()) {
				Ok(data) => data,
				Err(e) => {
					eprintln!("Error: Failed to load data from the target file. Details: {}", e);
					std::process::exit(1);
				}
			};
			telegram::run(Arc::new(app_config), Arc::new(RwLock::new(target_data)))
				.await
				.unwrap_or_else(|e| {
					eprintln!("Error: Failed to start the telegram bot. Details: {}", e);
					std::process::exit(1);
				})
		}
	}
}

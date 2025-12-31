#![allow(clippy::len_zero)] // wait, so are the ones in Cargo.toml not enough?
#![allow(clippy::get_first)]
#![allow(clippy::comparison_to_empty)]
#![feature(trait_alias)]
#![feature(type_changing_struct_update)]
use clap::{Args, Parser, Subcommand};
use config::{LiveSettings, SettingsFlags};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use v_utils::io::ExpandedPath;
pub mod config;
pub mod data;
pub mod telegram;
pub mod utils;

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	command: Commands,
}
#[derive(Subcommand, Debug)]
pub enum Commands {
	/// Start the server allowing to change the config at the specified path using telegram.
	///Ex
	///```
	///tg_admin watch -t "${THE_BOT_TOKEN}" ./config/config.json
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
	#[clap(flatten)]
	settings_flags: SettingsFlags,
}

#[tokio::main]
async fn main() {
	v_utils::utils::init_subscriber(v_utils::utils::LogDestination::default());
	let cli = Cli::parse();

	match &cli.command {
		Commands::Manage(args) => {
			let app_config = match LiveSettings::new(args.settings_flags.clone(), Duration::from_secs(5)) {
				Ok(config) => config,
				Err(e) => {
					eprintln!("Error: Failed to initialize settings. Details: {}", e);
					std::process::exit(1);
				}
			};
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

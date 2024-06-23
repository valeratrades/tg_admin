use clap::{Args, Parser, Subcommand};
use settings::Settings;
use v_utils::io::ExpandedPath;
pub mod settings;

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	#[arg(long, default_value = "~/.config/tg_config.toml")]
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
	Start(StartArgs),
}
impl Default for Commands {
	fn default() -> Self {
		Self::Start(StartArgs::default())
	}
}

#[derive(Args, Debug, Default)]
pub struct StartArgs {
	/// Path to the target file
	path: ExpandedPath,
	/// Override token in config
	#[arg(short, long)]
	tg_token: Option<String>,
}

fn main() {
	let cli = Cli::parse();
	let app_config = match Settings::new_with_cli(&cli) {
		Ok(config) => config,
		Err(e) => {
			eprintln!("Error: Failed to initialize settings with CLI arguments. Details: {}", e);
			std::process::exit(1);
		}
	};

	match &cli.command {
		Commands::Start(args) => {
			unimplemented!();
		}
	}
}

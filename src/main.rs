use clap::{Args, Parser, Subcommand};
use v_utils::io::ExpandedPath;

#[derive(Parser, Debug, Default, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	#[arg(long, default_value = "~/.config/tg_config.toml")]
	config: ExpandedPath,
	#[command(subcommand)]
	command: Commands,
}
#[derive(Subcommand)]
pub enum Commands {
	/// Start the server allowing to change the config at the specified path using telegram.
	///Ex
	///```sh
	///tg_config start -t "${THE_BOT_TOKEN}" ./config/config.json
	///```
	Start(StartArgs),
}

#[derive(Args)]
pub struct StartArgs {
	/// Path to the target file
	path: ExpandedPath,
	/// Override token in config
	#[arg(short, long)]
	token: Option<String>,
}

fn main() {
	let cli = Cli::parse();
	match cli.command {
		Commands::Start(args) => {
			let hello_target = match (args.world, args.rust) {
				(true, false) => "World",
				(false, true) => "Rust",
				(true, true) => panic!("Cannot hello two things"),
				(false, false) => panic!("Specify what to hello"),
			};

			let message = format!("Hello, {hello_target}{}", &args.after_hello_message.join(""));
			println!("{message}");
		}
	}
}

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use v_utils::{io::ExpandedPath, macros::MyConfigPrimitives};

#[derive(Clone, Debug, Default, MyConfigPrimitives)]
pub struct AppConfig {
	pub tg_token: String,
	pub target_path: ExpandedPath,
}

impl AppConfig {
	/// Uses config at the provided position and considers the cli arguments. Cli overrides the config.
	pub fn new_with_cli(path: ExpandedPath, cli: crate::Cli) -> Result<Self> {
		let builder = config::Config::builder().add_source(config::File::with_name(&path.to_string()));

		let settings: config::Config = builder.build()?;
		let settings: Self = settings.try_deserialize()?;

		let mut default_cli = crate::Cli::default();

		// for each point:
		// init default
		// if in conf, override
		// if provided in cli, override

		match cli.command {
			crate::Commands::Start(args) => {
				if args.token.is_some() {
					settings.tg_token = args.token.unwrap();
				}
			}
		}
		Ok(settings)
	}
}

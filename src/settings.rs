use anyhow::Result;
use config::Config;
use v_utils::macros::MyConfigPrimitives;

#[derive(Clone, Debug, Default, MyConfigPrimitives)]
pub struct Settings {
	pub tg_token: String,
}

impl Settings {
	/// Uses config at the provided position and considers the cli arguments. Cli overrides the config.
	pub fn new_with_cli(cli: &crate::Cli) -> Result<Self> {
		let crate::Commands::Start(cmd_args) = &cli.command;

		let s = Config::builder()
			.add_source(config::File::with_name(&cli.config.to_string()).required(false))
			.set_override_option("tg_token", cmd_args.tg_token.clone())?
			.build()?;

		let settings: Self = s.try_deserialize()?;

		Ok(settings)
	}
}

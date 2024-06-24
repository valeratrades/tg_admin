use crate::settings::Settings;
use anyhow::Result;
use crate::data::Data;

pub fn start(settings: &Settings, data: Data) -> Result<()> {
	let token = &settings.tg_token;

	Ok(())
}

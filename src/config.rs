use serde::Serialize;
use v_utils::macros::{LiveSettings, MyConfigPrimitives, Settings};

#[derive(Clone, Debug, Default, LiveSettings, MyConfigPrimitives, Serialize, Settings)]
pub struct Settings {
	#[serde(default)]
	pub tg_token: String,
	#[serde(default)]
	pub admin_list: Option<Vec<u64>>,
}

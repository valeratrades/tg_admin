use serde::Serialize;
use v_utils::macros::{MyConfigPrimitives, Settings};

#[derive(Clone, Debug, Default, MyConfigPrimitives, Serialize, Settings)]
pub struct Settings {
	#[serde(default)]
	pub tg_token: String,
	#[serde(default)]
	pub admin_list: Option<Vec<u64>>,
}

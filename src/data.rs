use crate::utils::get_json_type;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::io::{Read, Write};
use std::{
	fs::File,
	io::{BufReader, BufWriter},
	path::{Path, PathBuf},
};
use toml::Value as TomlValue;

#[derive(Clone, Debug, Default, derive_new::new)]
pub struct Data {
	inner: JsonValue,
	path: PathBuf,
}

impl Data {
	/// Load data from a file
	pub fn load(path: &Path) -> Result<Self> {
		let file = File::open(path)?;
		let mut reader = BufReader::new(file);
		let extension = path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or("");

		let data: JsonValue = match extension {
			"json" | "json5" => {
				let mut content = String::new();
				reader.read_to_string(&mut content)?;
				let json5_value: JsonValue = json5::from_str(&content).context("Failed to read JSON file")?;
				json5_value
			}
			"yaml" | "yml" => {
				let yaml_value: YamlValue = serde_yaml::from_reader(reader).context("Failed to read YAML file")?;
				serde_json::to_value(yaml_value).context("Failed to convert YAML to JSON")?
			}
			"toml" => {
				let mut content = String::new();
				reader.into_inner().read_to_string(&mut content)?;
				let toml_value: TomlValue = toml::from_str(&content).context("Failed to read TOML file")?;
				serde_json::to_value(toml_value).context("Failed to convert TOML to JSON")?
			}
			_ => return Err(anyhow::anyhow!("Unsupported file format")),
		};

		Ok(Self::new(data, path.to_path_buf()))
	}

	/// Write data to the source file
	pub fn write(&self) -> Result<()> {
		let file = File::create(&self.path)?;
		let mut writer = BufWriter::new(file);
		let extension = self.path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or("");

		match extension {
			"json" | "json5" => serde_json::to_writer(writer, &self.inner).context("Failed to write JSON file")?,
			"yaml" | "yml" => {
				let yaml_value: YamlValue = serde_json::from_value(self.inner.clone()).context("Failed to convert JSON to YAML")?;
				serde_yaml::to_writer(writer, &yaml_value).context("Failed to write YAML file")?
			}
			"toml" => {
				let toml_value: TomlValue = serde_json::from_value(self.inner.clone()).context("Failed to convert JSON to TOML")?;
				writer.write_all(toml::to_string(&toml_value).context("Failed to write TOML file")?.as_bytes())?;
			}
			_ => return Err(anyhow::anyhow!("Unsupported file format")),
		}

		Ok(())
	}

	/// Load the file without needing to provide the path again
	pub fn reload(&mut self) -> Result<()> {
		self.inner = Self::load(&self.path)?.inner;
		Ok(())
	}

	pub fn at(&self, level: &ValuePath) -> Option<JsonValue> {
		let mut current = &self.inner;
		for part in level.to_vec() {
			current = current.get(&part)?;
		}
		Some(current.clone())
	}

	pub fn update(&mut self, new_value: JsonValue) {
		self.inner = new_value;
	}

	pub fn update_at<UA>(&mut self, level: &ValuePath, new_value: JsonValue, into_action: UA) -> Result<(), String>
	where
		UA: Into<UpdateAction>,
	{
		let mut current = &mut self.inner;
		let path = level.to_vec();
		let action = into_action.into();

		for (i, part) in path.iter().enumerate() {
			if i == path.len() - 1 {
				// We're at the last element, so we apply the update action
				let obj = current.as_object_mut().unwrap();
				match action {
					UpdateAction::Set => {
						obj.insert(part.clone(), new_value);
					}
					UpdateAction::AddTo => {
						let existing = obj.get_mut(part).unwrap();
						if let JsonValue::Array(existing_arr) = existing {
							if !existing_arr.is_empty() && get_json_type(&existing_arr[0]) != get_json_type(&new_value) {
								return Err(format!(
									"Type mismatch: Expected {}, got {}",
									get_json_type(&existing_arr[0]),
									get_json_type(&new_value)
								));
							}
							existing_arr.push(new_value);
						} else {
							panic!("Target is not an array");
						}
					}
					UpdateAction::RemoveFrom => {
						dbg!(&part);
						let existing = obj.get_mut(part).unwrap();
						if let JsonValue::Array(existing_arr) = existing {
							if existing_arr.is_empty() {
								return Err("Cannot remove from an empty array".to_string());
							}
							if get_json_type(&existing_arr[0]) != get_json_type(&new_value) {
								return Err(format!(
									"Type mismatch: Expected {}, got {}",
									get_json_type(&existing_arr[0]),
									get_json_type(&new_value)
								));
							}
							let initial_len = existing_arr.len();
							existing_arr.retain(|item| item != &new_value);
							if existing_arr.len() == initial_len {
								return Err("Value not found in the array".to_string());
							}
						} else {
							panic!("Target is not an array");
						}
					}
				}
				return Ok(());
			} else {
				// We're not at the last element, so we navigate to the next level
				current = current.get_mut(part).unwrap();
			}
		}
		panic!("Empty path");
	}

	#[doc(hidden)]
	pub fn mock(value: JsonValue) -> Self {
		Self::new(value, PathBuf::new())
	}
}
impl AsRef<JsonValue> for Data {
	fn as_ref(&self) -> &JsonValue {
		&self.inner
	}
}
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum UpdateAction {
	Set,
	AddTo,
	RemoveFrom,
}
impl From<crate::telegram::InputValueType> for UpdateAction {
	fn from(action: crate::telegram::InputValueType) -> Self {
		match action {
			crate::telegram::InputValueType::UpdateAt => Self::Set,
			crate::telegram::InputValueType::AddTo => Self::AddTo,
			crate::telegram::InputValueType::RemoveFrom => Self::RemoveFrom,
		}
	}
}

/// Callback data must never be empty, so 0 level is "/" and not ""
#[derive(Clone, Debug, derive_new::new, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValuePath(String);
impl ValuePath {
	pub fn push(&mut self, part: &str) {
		if !self.is_top() {
			self.0.push('/');
		}
		self.0.push_str(part);
	}

	pub fn parent(&self) -> Self {
		let mut new_v = self.0.clone();
		let pos = new_v.rfind("/").unwrap_or(0);
		new_v.truncate(pos);
		match new_v.is_empty() {
			true => Self::default(),
			false => Self(new_v),
		}
	}

	pub fn basename(&self) -> String {
		let pos = self.0.rfind("/").unwrap_or(0);
		let offset = Self::default().0.len();
		self.0[(pos + offset)..].to_string()
	}

	pub fn join(&self, part: &str) -> Self {
		let mut new_level = self.clone();
		new_level.push(part);
		new_level
	}

	pub fn is_top(&self) -> bool {
		assert!(!self.0.is_empty());
		self.0 == "/"
	}

	fn to_vec(&self) -> Vec<String> {
		self.0.split("/").map(String::from).filter(|v| v != "").collect()
	}

	pub fn into_string(self) -> String {
		self.0
	}
}
impl Default for ValuePath {
	fn default() -> Self {
		Self("/".to_string())
	}
}
impl From<Vec<String>> for ValuePath {
	fn from(parts: Vec<String>) -> Self {
		let s = "/".to_owned() + &parts.join("/");
		Self(s)
	}
}
impl From<&str> for ValuePath {
	fn from(s: &str) -> Self {
		Self(s.to_string())
	}
}
impl From<String> for ValuePath {
	fn from(s: String) -> Self {
		Self(s)
	}
}
impl From<ValuePath> for Vec<String> {
	fn from(level: ValuePath) -> Self {
		level.to_vec()
	}
}
impl std::fmt::Display for ValuePath {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;
	use std::fs::{create_dir_all, write};
	use tempfile::tempdir;

	fn generate_test_data(format: &str) -> (PathBuf, String) {
		let dir = tempdir().unwrap();
		let path = dir.path().join(format!("test_data.{}", format));

		let json_value = json!({
			"key": "value",
			"number": 42,
			"array_of_numbers": [1, 2, 3],
			"array_of_strings": ["a", "b", "c"]
		});

		let content = match format {
			"json" => serde_json::to_string_pretty(&json_value).unwrap(),
			"yaml" | "yml" => serde_yaml::to_string(&json_value).unwrap(),
			"toml" => {
				let toml_value: toml::Value = serde_json::from_value(json_value).unwrap();
				toml::to_string(&toml_value).unwrap()
			}
			_ => panic!("Unsupported format"),
		};

		(path.to_path_buf(), content)
	}

	#[test]
	fn test_data_operations() -> Result<()> {
		let formats = vec!["json", "yaml", "yml", "toml"];
		for format in formats {
			let (path, content) = generate_test_data(format);

			let dir = path.parent().unwrap();
			create_dir_all(dir)?;

			write(&path, content)?;

			let mut data = Data::load(&path)?;
			assert_eq!(data.as_ref()["key"], "value");
			assert_eq!(data.as_ref()["number"], 42);

			let mut new_inner = data.as_ref().clone();
			if let Some(obj) = new_inner.as_object_mut() {
				obj.insert("key".to_string(), JsonValue::String("new_value".to_string()));
			}
			data.update(new_inner);
			data.write()?;

			let data = Data::load(&path)?;
			assert_eq!(data.as_ref()["key"], "new_value", "(Format: {})", format);
			assert_eq!(data.as_ref()["number"], 42, "(Format: {})", format);
		}
		Ok(())
	}

	#[test]
	fn test_data_update_at() {
		let formats = vec!["json", "yaml", "yml", "toml"];
		for format in formats {
			let (path, content) = generate_test_data(format);

			let dir = path.parent().unwrap();
			create_dir_all(dir).unwrap();

			write(&path, content).unwrap();

			let mut data = Data::load(&path).unwrap();

			// Test UpdateAction::Set
			assert!(data
				.update_at(&ValuePath::from("key"), JsonValue::String("updated_value".to_string()), UpdateAction::Set)
				.is_ok());
			assert_eq!(data.as_ref()["key"], "updated_value", "(Format: {})", format);

			// Test UpdateAction::AddTo for numbers
			let numbers_path = ValuePath::from("array_of_numbers");
			assert!(data.update_at(&numbers_path, JsonValue::Number(4.into()), UpdateAction::AddTo).is_ok());
			assert_eq!(
				data.as_ref()["array_of_numbers"],
				JsonValue::Array(vec![1, 2, 3, 4].into_iter().map(|n| JsonValue::Number(n.into())).collect()),
				"(Format: {})",
				format
			);

			// Test UpdateAction::AddTo for strings
			let strings_path = ValuePath::from("array_of_strings");
			assert!(data.update_at(&strings_path, JsonValue::String("d".to_string()), UpdateAction::AddTo).is_ok());
			assert_eq!(
				data.as_ref()["array_of_strings"],
				JsonValue::Array(vec!["a", "b", "c", "d"].into_iter().map(|s| JsonValue::String(s.to_string())).collect()),
				"(Format: {})",
				format
			);

			// Test UpdateAction::RemoveFrom for numbers
			assert!(data.update_at(&numbers_path, JsonValue::Number(2.into()), UpdateAction::RemoveFrom).is_ok());
			assert_eq!(
				data.as_ref()["array_of_numbers"],
				JsonValue::Array(vec![1, 3, 4].into_iter().map(|n| JsonValue::Number(n.into())).collect()),
				"(Format: {})",
				format
			);

			// Test error cases
			assert!(data
				.update_at(&numbers_path, JsonValue::String("not a number".to_string()), UpdateAction::AddTo)
				.is_err());
			assert!(data.update_at(&numbers_path, JsonValue::Number(5.into()), UpdateAction::RemoveFrom).is_err());
			data.write().unwrap();

			// Verify persistence
			let reloaded_data = Data::load(&path).unwrap();
			assert_eq!(reloaded_data.as_ref()["key"], "updated_value", "(Format: {})", format);
			assert_eq!(
				reloaded_data.as_ref()["array_of_numbers"],
				JsonValue::Array(vec![1, 3, 4].into_iter().map(|n| JsonValue::Number(n.into())).collect()),
				"(Format: {})",
				format
			);
			assert_eq!(
				reloaded_data.as_ref()["array_of_strings"],
				JsonValue::Array(vec!["a", "b", "c", "d"].into_iter().map(|s| JsonValue::String(s.to_string())).collect()),
				"(Format: {})",
				format
			);
		}
	}

	#[test]
	fn test_value_path() {
		let mut level = ValuePath::default();
		let path = ["key1", "key2", "key3"];

		assert!(level.0 != "");
		assert!(level.basename() == "");
		assert!(!level.to_vec().contains(&"".to_string()));

		for part in &path {
			level.push(part);
			assert!(level.parent().0 != "");
			assert!(!level.to_vec().contains(&"".to_string()));
			assert!(level.basename() == *part);
		}
		assert!(level.to_vec() == path.to_vec());
	}
}

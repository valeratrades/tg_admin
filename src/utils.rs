use serde_json::Value;

pub fn get_json_type(value: &Value) -> &str {
	match value {
		Value::Null => "Null",
		Value::Bool(_) => "Boolean",
		Value::Number(_) => "Number",
		Value::String(_) => "String",
		Value::Array(_) => "Array",
		Value::Object(_) => "Object",
	}
}

pub fn value_preview(key: &String, value: &Value) -> String {
	match value {
		Value::Object(_) => format!("{{}} {}", key),
		Value::Array(arr) => format!("[{}] {}", arr.len(), key),
		_ => format!("{}: {}", key, &value.to_string()),
	}
}

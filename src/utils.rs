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

//! Printing command results as text or JSON.

use serde_json::Value;

/// Print `value`: one JSON line under `--json`, otherwise one `key: value`
/// line per field, each prefixed with `indent`.
pub fn print(value: &Value, json: bool, indent: &str) {
	if json {
		println!("{value}");
		return;
	}
	if let Value::Object(fields) = value {
		for (key, field) in fields {
			println!("{indent}{key}: {}", text(field));
		}
	}
}

/// A field as a person reads it: strings without quotes, `null` as `none`.
fn text(value: &Value) -> String {
	match value {
		Value::String(text) => text.clone(),
		Value::Null => "none".to_owned(),
		other => other.to_string(),
	}
}

#[cfg(test)]
mod tests {
	use serde_json::json;

	use super::*;

	#[test]
	fn text_drops_quotes_and_names_missing_values() {
		assert_eq!(text(&json!("97610000000000000")), "97610000000000000");
		assert_eq!(text(&json!(null)), "none");
		assert_eq!(text(&json!(42)), "42");
		assert_eq!(text(&json!([1, 2])), "[1,2]");
	}
}

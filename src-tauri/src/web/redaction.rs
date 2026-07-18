//! Web response redaction and masked-secret merge helpers.

use serde_json::Value;

const MASK_MARKER: &str = "***";

fn is_sensitive_key(key: &str) -> bool {
    let compact = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    compact.ends_with("apikey")
        || compact.ends_with("token")
        || compact.contains("password")
        || compact.starts_with("secret")
}

fn mask_secret(secret: &str) -> String {
    let characters = secret.chars().collect::<Vec<_>>();
    if characters.len() <= 8 {
        return MASK_MARKER.to_string();
    }
    format!(
        "{}{}{}",
        characters[..4].iter().collect::<String>(),
        MASK_MARKER,
        characters[characters.len() - 4..]
            .iter()
            .collect::<String>()
    )
}

fn is_mask_placeholder(value: &str) -> bool {
    value.trim().is_empty() || value.contains(MASK_MARKER)
}

pub fn redact_sensitive_values(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if is_sensitive_key(key) {
                    if let Some(secret) = child.as_str() {
                        if !secret.is_empty() {
                            *child = Value::String(mask_secret(secret));
                        }
                    }
                } else {
                    redact_sensitive_values(child);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_sensitive_values(item);
            }
        }
        _ => {}
    }
}

pub fn preserve_masked_sensitive_values(incoming: &mut Value, existing: &Value) {
    let (Some(incoming_object), Some(existing_object)) =
        (incoming.as_object_mut(), existing.as_object())
    else {
        return;
    };

    for (key, incoming_child) in incoming_object {
        let Some(existing_child) = existing_object.get(key) else {
            continue;
        };
        if is_sensitive_key(key) {
            if incoming_child.as_str().is_some_and(is_mask_placeholder) {
                *incoming_child = existing_child.clone();
            }
        } else {
            preserve_masked_sensitive_values(incoming_child, existing_child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn redacts_nested_provider_secrets() {
        let mut value = json!({
            "env": {
                "OPENAI_API_KEY": "sk-1234567890",
                "ANTHROPIC_AUTH_TOKEN": "token-abcdefghij",
                "baseUrl": "https://example.com"
            },
            "password": "short"
        });
        redact_sensitive_values(&mut value);
        assert_eq!(value["env"]["OPENAI_API_KEY"], "sk-1***7890");
        assert_eq!(value["env"]["ANTHROPIC_AUTH_TOKEN"], "toke***ghij");
        assert_eq!(value["password"], "***");
        assert_eq!(value["env"]["baseUrl"], "https://example.com");
    }

    #[test]
    fn preserves_empty_and_masked_secrets_on_update() {
        let existing = json!({"auth": {"OPENAI_API_KEY": "real-secret"}, "apiKey": "other-secret"});
        let mut incoming = json!({"auth": {"OPENAI_API_KEY": "real***cret"}, "apiKey": ""});
        preserve_masked_sensitive_values(&mut incoming, &existing);
        assert_eq!(incoming, existing);
    }
}

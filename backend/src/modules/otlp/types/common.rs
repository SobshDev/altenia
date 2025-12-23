//! Common OTLP types shared across logs, metrics, and traces.

use serde::{Deserialize, Serialize};

/// Key-value pair for attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValue {
    pub key: String,
    pub value: AnyValue,
}

/// Any value type - matches OTLP AnyValue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bool_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub int_value: Option<String>, // OTLP uses string for int64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub array_value: Option<ArrayValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kvlist_value: Option<KeyValueList>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_value: Option<String>, // base64 encoded
}

impl AnyValue {
    pub fn to_json_value(&self) -> serde_json::Value {
        if let Some(ref s) = self.string_value {
            serde_json::Value::String(s.clone())
        } else if let Some(b) = self.bool_value {
            serde_json::Value::Bool(b)
        } else if let Some(ref i) = self.int_value {
            i.parse::<i64>()
                .map(|n| serde_json::Value::Number(n.into()))
                .unwrap_or(serde_json::Value::String(i.clone()))
        } else if let Some(d) = self.double_value {
            serde_json::Number::from_f64(d)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        } else if let Some(ref arr) = self.array_value {
            serde_json::Value::Array(arr.values.iter().map(|v| v.to_json_value()).collect())
        } else if let Some(ref kv) = self.kvlist_value {
            let mut map = serde_json::Map::new();
            for pair in &kv.values {
                map.insert(pair.key.clone(), pair.value.to_json_value());
            }
            serde_json::Value::Object(map)
        } else if let Some(ref b) = self.bytes_value {
            serde_json::Value::String(b.clone())
        } else {
            serde_json::Value::Null
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayValue {
    #[serde(default)]
    pub values: Vec<AnyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueList {
    #[serde(default)]
    pub values: Vec<KeyValue>,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub dropped_attributes_count: u32,
}

impl Resource {
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for attr in &self.attributes {
            map.insert(attr.key.clone(), attr.value.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    pub fn get_service_name(&self) -> Option<String> {
        self.attributes
            .iter()
            .find(|kv| kv.key == "service.name")
            .and_then(|kv| kv.value.string_value.clone())
    }

    pub fn get_service_version(&self) -> Option<String> {
        self.attributes
            .iter()
            .find(|kv| kv.key == "service.version")
            .and_then(|kv| kv.value.string_value.clone())
    }
}

/// Instrumentation scope (formerly InstrumentationLibrary)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentationScope {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub attributes: Vec<KeyValue>,
    #[serde(default)]
    pub dropped_attributes_count: u32,
}

/// Convert attributes to JSON object
pub fn attributes_to_json(attrs: &[KeyValue]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for attr in attrs {
        map.insert(attr.key.clone(), attr.value.to_json_value());
    }
    serde_json::Value::Object(map)
}

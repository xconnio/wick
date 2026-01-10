use serde::Serialize;
use serde_json::Value as SerdeValue;
use xconn::sync::Value as WampValue;

#[derive(Debug)]
pub enum ParsedArg {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
}

pub fn parse_arg(input: &str) -> ParsedArg {
    // Check for quoted strings to enforce string type
    if ((input.starts_with('\'') && input.ends_with('\''))
        || (input.starts_with('"') && input.ends_with('"')))
        && input.len() >= 2
    {
        return ParsedArg::String(input[1..input.len() - 1].to_string());
    }

    if let Ok(i) = input.parse::<i64>() {
        return ParsedArg::Integer(i);
    }

    if let Ok(f) = input.parse::<f64>() {
        return ParsedArg::Float(f);
    }

    if let Ok(b) = input.parse::<bool>() {
        return ParsedArg::Boolean(b);
    }

    ParsedArg::String(input.to_string())
}

#[derive(Serialize)]
pub struct CommandOutput {
    pub args: Vec<SerdeValue>,
    pub kwargs: std::collections::HashMap<String, SerdeValue>,
}

pub fn wamp_value_to_serde(v: &WampValue) -> SerdeValue {
    match v {
        WampValue::Int(i) => SerdeValue::Number((*i).into()),
        WampValue::Str(s) => SerdeValue::String(s.clone()),
        WampValue::Bool(b) => SerdeValue::Bool(*b),
        WampValue::Float(f) => serde_json::json!(f),
        WampValue::List(l) => SerdeValue::Array(l.iter().map(wamp_value_to_serde).collect()),
        WampValue::Dict(d) => SerdeValue::Object(
            d.iter()
                .map(|(k, v)| (k.clone(), wamp_value_to_serde(v)))
                .collect(),
        ),
        WampValue::Bytes(_) => SerdeValue::String("<binary>".to_string()),
        _ => SerdeValue::Null,
    }
}

pub fn wamp_async_value_to_serde(v: &xconn::async_::Value) -> SerdeValue {
    match v {
        xconn::async_::Value::Int(i) => SerdeValue::Number((*i).into()),
        xconn::async_::Value::Str(s) => SerdeValue::String(s.clone()),
        xconn::async_::Value::Bool(b) => SerdeValue::Bool(*b),
        xconn::async_::Value::Float(f) => serde_json::json!(f),
        xconn::async_::Value::List(l) => {
            SerdeValue::Array(l.iter().map(wamp_async_value_to_serde).collect())
        }
        xconn::async_::Value::Dict(d) => SerdeValue::Object(
            d.iter()
                .map(|(k, v)| (k.clone(), wamp_async_value_to_serde(v)))
                .collect(),
        ),
        xconn::async_::Value::Bytes(_) => SerdeValue::String("<binary>".to_string()),
        _ => SerdeValue::Null,
    }
}

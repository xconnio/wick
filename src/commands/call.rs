use crate::config::{CallConfig, ConnectionConfig};
use crate::utils::{CommandOutput, ParsedArg, parse_arg, wamp_value_to_serde};
use std::sync::Arc;
use tokio::sync::Semaphore;
use xconn::sync::CallRequest;

/// Parses a "key=value" string and returns the key and parsed value.
fn parse_key_value(input: &str) -> Option<(String, ParsedArg)> {
    let parts: Vec<&str> = input.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parse_arg(parts[1])))
    } else {
        None
    }
}

/// Builds a CallRequest from the CallConfig.
fn build_call_request(config: &CallConfig) -> CallRequest {
    let mut request = CallRequest::new(&config.procedure);

    // Add positional arguments
    for arg in &config.args {
        request = match parse_arg(arg) {
            ParsedArg::Integer(v) => request.arg(v),
            ParsedArg::Float(v) => request.arg(v),
            ParsedArg::Boolean(v) => request.arg(v),
            ParsedArg::String(v) => request.arg(v),
        };
    }

    // Add keyword arguments
    for kwarg in &config.kwargs {
        if let Some((key, value)) = parse_key_value(kwarg) {
            request = match value {
                ParsedArg::Integer(v) => request.kwarg(&key, v),
                ParsedArg::Float(v) => request.kwarg(&key, v),
                ParsedArg::Boolean(v) => request.kwarg(&key, v),
                ParsedArg::String(v) => request.kwarg(&key, v),
            };
        }
    }

    // Add options
    for opt in &config.options {
        if let Some((key, value)) = parse_key_value(opt) {
            request = match value {
                ParsedArg::Integer(v) => request.option(&key, v),
                ParsedArg::Float(v) => request.option(&key, v),
                ParsedArg::Boolean(v) => request.option(&key, v),
                ParsedArg::String(v) => request.option(&key, v),
            };
        }
    }

    request
}

/// Executes calls for a single session: connects, runs repeated calls, and disconnects.
async fn run_session(
    conn_config: Arc<ConnectionConfig>,
    call_config: Arc<CallConfig>,
    session_id: u32,
) {
    let session = match conn_config.connect().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Session {} Connection Error: {}", session_id, e);
            return;
        }
    };

    for iteration in 1..=call_config.repeat {
        let request = build_call_request(&call_config);

        match session.call(request).await {
            Ok(result) => {
                let output = CommandOutput {
                    args: result
                        .args
                        .as_ref()
                        .map(|a: &Vec<xconn::sync::Value>| {
                            a.iter().map(wamp_value_to_serde).collect()
                        })
                        .unwrap_or_default(),
                    kwargs: result
                        .kwargs
                        .as_ref()
                        .map(
                            |kw: &std::collections::HashMap<String, xconn::sync::Value>| {
                                kw.iter()
                                    .map(|(k, v)| (k.clone(), wamp_value_to_serde(v)))
                                    .collect()
                            },
                        )
                        .unwrap_or_default(),
                };
                match serde_json::to_string_pretty(&output) {
                    Ok(json) => println!("{}", json),
                    Err(e) => eprintln!(
                        "Session {} Iteration {} Error serializing result: {}",
                        session_id, iteration, e
                    ),
                }
            }
            Err(e) => eprintln!(
                "Session {} Iteration {} Call Error: {}",
                session_id, iteration, e
            ),
        }
    }

    if let Err(e) = session.leave().await {
        eprintln!("Session {} Error leaving: {}", session_id, e);
    }
}

pub async fn handle(
    conn_config: ConnectionConfig,
    call_config: CallConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(call_config.concurrency));
    let conn_config = Arc::new(conn_config);
    let call_config = Arc::new(call_config);

    let mut handles = Vec::with_capacity(call_config.parallel as usize);

    for session_id in 1..=call_config.parallel {
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let conn_config = conn_config.clone();
        let call_config = call_config.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            run_session(conn_config, call_config, session_id).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

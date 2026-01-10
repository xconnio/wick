use crate::config::{CallConfig, ConnectionConfig};
use crate::utils::{CommandOutput, ParsedArg, parse_arg, wamp_value_to_serde};
use std::sync::Arc;
use tokio::sync::Semaphore;
use xconn::sync::CallRequest;

/// Builds a CallRequest from the procedure name and parsed arguments.
fn build_call_request(procedure: &str, args: &[String]) -> CallRequest {
    let mut request = CallRequest::new(procedure);
    for arg in args {
        request = match parse_arg(arg) {
            ParsedArg::Integer(v) => request.arg(v),
            ParsedArg::Float(v) => request.arg(v),
            ParsedArg::Boolean(v) => request.arg(v),
            ParsedArg::String(v) => request.arg(v),
        };
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

    for iteration in 0..call_config.repeat {
        let request = build_call_request(&call_config.procedure, &call_config.args);

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

    for session_id in 0..call_config.parallel {
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

use crate::config::{ConnectionConfig, PublishConfig};
use crate::utils::{ParsedArg, parse_arg};
use std::sync::Arc;
use tokio::sync::Semaphore;
use xconn::sync::PublishRequest;

/// Parses a "key=value" string and returns the key and parsed value.
fn parse_key_value(input: &str) -> Option<(String, ParsedArg)> {
    let parts: Vec<&str> = input.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parse_arg(parts[1])))
    } else {
        None
    }
}

/// Builds a PublishRequest from the PublishConfig.
fn build_publish_request(config: &PublishConfig) -> PublishRequest {
    let mut request = PublishRequest::new(&config.topic);

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

    // Add acknowledge option if requested
    if config.acknowledge {
        request = request.option("acknowledge", true);
    }

    request
}

/// Executes publishes for a single session: connects, runs repeated publishes, and disconnects.
async fn run_session(
    conn_config: Arc<ConnectionConfig>,
    publish_config: Arc<PublishConfig>,
    session_id: u32,
) {
    let session = match conn_config.connect().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Session {} Connection Error: {}", session_id, e);
            return;
        }
    };

    for iteration in 1..=publish_config.repeat {
        let request = build_publish_request(&publish_config);

        match session.publish(request).await {
            Ok(_) => {}
            Err(e) => eprintln!(
                "Session {} Iteration {} Publish Error: {}",
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
    publish_config: PublishConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(publish_config.concurrency));
    let conn_config = Arc::new(conn_config);
    let publish_config = Arc::new(publish_config);

    let mut handles = Vec::with_capacity(publish_config.parallel as usize);

    for session_id in 1..=publish_config.parallel {
        let permit = semaphore.clone().acquire_owned().await.unwrap();

        let conn_config = conn_config.clone();
        let publish_config = publish_config.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            run_session(conn_config, publish_config, session_id).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

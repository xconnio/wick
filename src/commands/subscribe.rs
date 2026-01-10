use crate::config::{ConnectionConfig, SubscribeConfig};
use crate::utils::{CommandOutput, wamp_async_value_to_serde};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Semaphore;
use xconn::async_::{Event, SubscribeRequest};

/// Builds a SubscribeRequest from the SubscribeConfig.
fn build_subscribe_request(config: &SubscribeConfig) -> SubscribeRequest {
    // Note: SubscribeRequest doesn't support options via builder pattern
    // Options would need to be added at the xconn-rust library level
    SubscribeRequest::new(&config.topic, event_handler)
}

async fn event_handler(event: Event) {
    let output = CommandOutput {
        args: event.args.iter().map(wamp_async_value_to_serde).collect(),
        kwargs: event
            .kwargs
            .iter()
            .map(|(k, v): (_, _)| (k.clone(), wamp_async_value_to_serde(v)))
            .collect(),
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing event: {}", e),
    }
}

/// Runs a single subscribe session: connects, subscribes, and waits.
async fn run_session(
    conn_config: Arc<ConnectionConfig>,
    subscribe_config: Arc<SubscribeConfig>,
    session_id: u32,
    shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let session = match conn_config.connect().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Session {} Connection Error: {}", session_id, e);
            return;
        }
    };

    let request = build_subscribe_request(&subscribe_config);

    match session.subscribe(request).await {
        Ok(_) => {
            if subscribe_config.parallel > 1 {
                println!(
                    "Session {}: Subscribed to topic '{}'",
                    session_id, subscribe_config.topic
                );
            } else {
                println!("Subscribed to topic '{}'", subscribe_config.topic);
            }
        }
        Err(e) => {
            eprintln!("Session {} Subscribe Error: {}", session_id, e);
            return;
        }
    }

    // Wait for shutdown signal
    let mut shutdown = shutdown;
    let _ = shutdown.changed().await;

    if let Err(e) = session.leave().await {
        eprintln!("Session {} Error leaving: {}", session_id, e);
    }
}

pub async fn handle(
    conn_config: ConnectionConfig,
    subscribe_config: SubscribeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let semaphore = Arc::new(Semaphore::new(subscribe_config.concurrency));
    let conn_config = Arc::new(conn_config);
    let subscribe_config = Arc::new(subscribe_config);

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let mut handles = Vec::with_capacity(subscribe_config.parallel as usize);

    for session_id in 1..=subscribe_config.parallel {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let conn_config = conn_config.clone();
        let subscribe_config = subscribe_config.clone();
        let shutdown_rx = shutdown_rx.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            run_session(conn_config, subscribe_config, session_id, shutdown_rx).await;
        });

        handles.push(handle);
    }

    println!("Press Ctrl+C to exit");
    signal::ctrl_c().await?;
    println!("Exiting...");

    // Signal all sessions to shutdown
    let _ = shutdown_tx.send(true);

    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

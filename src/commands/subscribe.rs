use crate::colored_eprintln;
use crate::colored_println;
use crate::config::{ConnectionConfig, SubscribeConfig};
use crate::utils::{CommandOutput, format_connect_error, wamp_async_value_to_serde};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
    disconnect_tx: tokio::sync::mpsc::Sender<()>,
    ctrl_c_printed: Arc<AtomicBool>,
) {
    let session = match conn_config.connect().await {
        Ok(s) => s,
        Err(e) => {
            colored_eprintln!(
                "{}",
                format_connect_error(session_id, subscribe_config.parallel, e.as_ref())
            );
            return;
        }
    };

    let request = build_subscribe_request(&subscribe_config);

    match session.subscribe(request).await {
        Ok(resp) => {
            if let Some(err) = resp.error {
                colored_eprintln!("{}", err.uri);
                let _ = session.leave().await;
                return;
            }

            if subscribe_config.parallel > 1 {
                colored_println!(
                    "Session {}: Subscribed to topic '{}'",
                    session_id,
                    subscribe_config.topic
                );
            } else {
                colored_println!("Subscribed to topic '{}'", subscribe_config.topic);
            }

            // Print "Press Ctrl+C to exit" only once across all sessions
            if !ctrl_c_printed.swap(true, Ordering::Relaxed) {
                colored_println!("Press Ctrl+C to exit");
            }
        }
        Err(e) => {
            colored_eprintln!("Session {} Subscribe Error: {}", session_id, e);
            let _ = session.leave().await;
            return;
        }
    }

    // Wait for either shutdown signal or connection loss
    let mut shutdown = shutdown;
    let disconnected = tokio::select! {
        _ = shutdown.changed() => false,
        _ = session.wait_disconnect() => true,
    };

    if disconnected {
        let _ = disconnect_tx.send(()).await;
    } else if let Err(e) = session.leave().await {
        colored_eprintln!("Session {} Error leaving: {}", session_id, e);
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

    // Create disconnect notification channel
    let (disconnect_tx, mut disconnect_rx) = tokio::sync::mpsc::channel::<()>(1);

    let ctrl_c_printed = Arc::new(AtomicBool::new(false));

    let mut handles = Vec::with_capacity(subscribe_config.parallel as usize);

    for session_id in 1..=subscribe_config.parallel {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let conn_config = conn_config.clone();
        let subscribe_config = subscribe_config.clone();
        let shutdown_rx = shutdown_rx.clone();
        let disconnect_tx = disconnect_tx.clone();
        let ctrl_c_printed = ctrl_c_printed.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;
            run_session(
                conn_config,
                subscribe_config,
                session_id,
                shutdown_rx,
                disconnect_tx,
                ctrl_c_printed,
            )
            .await;
        });

        handles.push(handle);
    }

    // Spawn a task to track when all sessions finish
    let mut join_handle = tokio::spawn(async move {
        for handle in handles {
            let _ = handle.await;
        }
    });

    tokio::select! {
        _ = signal::ctrl_c() => {
            colored_println!("Exiting...");
        }
        _ = disconnect_rx.recv() => {
            colored_eprintln!("Lost connection to router");
        }
        _ = &mut join_handle => {
            // All sessions ended (e.g., all failed to connect)
            // Error messages already printed in run_session
        }
    }

    // Signal remaining sessions to shutdown
    let _ = shutdown_tx.send(true);
    drop(disconnect_tx);

    if !join_handle.is_finished() {
        let _ = join_handle.await;
    }

    Ok(())
}

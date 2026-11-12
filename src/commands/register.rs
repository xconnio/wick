use crate::colored_eprintln;
use crate::colored_println;
use crate::config::ConnectionConfig;
use crate::utils::{CommandOutput, format_connect_error, wamp_async_value_to_serde};
use tokio::signal;
use xconn::async_::{Invocation, Result as RpcResult};

async fn registration_handler(inv: Invocation) -> RpcResult {
    let output = CommandOutput {
        args: inv.args.iter().map(wamp_async_value_to_serde).collect(),
        kwargs: inv
            .kwargs
            .iter()
            .map(|(k, v): (_, _)| (k.clone(), wamp_async_value_to_serde(v)))
            .collect(),
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("Error serializing invocation: {}", e),
    }

    RpcResult::new(inv.args, inv.kwargs)
}

pub async fn handle(
    conn_config: ConnectionConfig,
    procedure: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let session = match conn_config.connect().await {
        Ok(s) => s,
        Err(e) => {
            colored_eprintln!("{}", format_connect_error(1, 1, e.as_ref()));
            return Ok(());
        }
    };

    match session
        .register(procedure, registration_handler)
        .execute()
        .await
    {
        Ok(resp) => {
            if let Some(err) = resp.error {
                colored_eprintln!("{}", err.uri);
                return Ok(());
            }
            colored_println!("Registered procedure '{}'", procedure);
        }
        Err(e) => {
            colored_eprintln!("Error registering procedure: {}", e);
            return Ok(());
        }
    }

    colored_println!("Press Ctrl+C to exit");
    tokio::select! {
        _ = signal::ctrl_c() => {
            colored_println!("Exiting...");
        }
        _ = session.wait_disconnect() => {
            colored_eprintln!("Lost connection to router");
        }
    }

    let _ = session.leave().await;

    Ok(())
}

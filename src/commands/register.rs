use crate::utils::{CommandOutput, wamp_async_value_to_serde};
use tokio::signal;
use xconn::async_::session::Session;
use xconn::async_::{Invocation, RegisterRequest, Yield};

async fn registration_handler(inv: Invocation) -> Yield {
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

    Yield::new(inv.args, inv.kwargs)
}

pub async fn handle(session: &Session, procedure: &str) -> Result<(), Box<dyn std::error::Error>> {
    let register_request = RegisterRequest::new(procedure, registration_handler);

    match session.register(register_request).await {
        Ok(reg) => println!("Registered procedure {}: {:?}", procedure, reg),
        Err(e) => println!("Error registering procedure: {}", e),
    }

    println!("Press Ctrl+C to exit");
    signal::ctrl_c().await?;
    println!("Exiting...");

    Ok(())
}

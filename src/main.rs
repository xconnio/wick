mod cli;
mod commands;
mod config;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use config::{CallConfig, ConnectionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("Connecting to {} in realm {}", cli.url, cli.realm);

    let conn_config = ConnectionConfig::from(&cli);

    match cli.command {
        Commands::Call {
            procedure,
            args,
            repeat,
            parallel,
            concurrency,
        } => {
            let call_config = CallConfig {
                procedure,
                args,
                repeat,
                parallel,
                concurrency,
            };
            commands::call::handle(conn_config, call_config).await?;
        }
        Commands::Register { procedure } => {
            let session = conn_config.connect().await?;
            commands::register::handle(&session, &procedure).await?;
            session.leave().await?;
        }
        Commands::Subscribe => {
            let session = conn_config.connect().await?;
            commands::subscribe::handle(&session).await?;
            session.leave().await?;
        }
        Commands::Publish => {
            let session = conn_config.connect().await?;
            commands::publish::handle(&session).await?;
            session.leave().await?;
        }
    }

    Ok(())
}

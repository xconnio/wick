mod cli;
mod commands;
mod config;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use config::{CallConfig, ConnectionConfig, PublishConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Commands::Keygen { output_file } = cli.command {
        return commands::keygen::handle(output_file);
    }

    let conn_config = ConnectionConfig::from(&cli);

    match cli.command {
        Commands::Call {
            procedure,
            args,
            kwargs,
            options,
            repeat,
            parallel,
            concurrency,
        } => {
            let call_config = CallConfig {
                procedure,
                args,
                kwargs,
                options,
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
        Commands::Subscribe {
            topic,
            parallel,
            concurrency,
        } => {
            let subscribe_config = config::SubscribeConfig {
                topic,
                parallel,
                concurrency,
            };
            commands::subscribe::handle(conn_config, subscribe_config).await?;
        }
        Commands::Publish {
            topic,
            args,
            kwargs,
            options,
            repeat,
            parallel,
            concurrency,
            acknowledge,
        } => {
            let publish_config = PublishConfig {
                topic,
                args,
                kwargs,
                options,
                repeat,
                parallel,
                concurrency,
                acknowledge,
            };
            commands::publish::handle(conn_config, publish_config).await?;
        }
        Commands::Keygen { .. } => unreachable!(), // Handled above
    }

    Ok(())
}

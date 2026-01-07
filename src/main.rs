use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wcli")]
#[command(about = "WAMP Command Line Interface", long_about = None)]
struct Cli {
    /// The URL of the router to connect to
    #[arg(long, default_value = "ws://localhost:8080/ws", global = true)]
    url: String,

    /// The realm to join
    #[arg(long, default_value = "realm1", global = true)]
    realm: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Call a procedure
    Call,
    /// Register a procedure
    Register,
    /// Subscribe to a topic
    Subscribe,
    /// Publish to a topic
    Publish,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("Connecting to {} in realm {}", cli.url, cli.realm);

    match cli.command {
        Commands::Call => {
            println!("Subcommand 'call' executed");
        }
        Commands::Register => {
            println!("Subcommand 'register' executed");
        }
        Commands::Subscribe => {
            println!("Subcommand 'subscribe' executed");
        }
        Commands::Publish => {
            println!("Subcommand 'publish' executed");
        }
    }
}

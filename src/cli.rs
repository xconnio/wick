use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wcli")]
#[command(about = "WAMP Command Line Interface", long_about = None)]
pub struct Cli {
    /// The URL of the router to connect to
    #[arg(long, default_value = "ws://localhost:8080/ws", global = true)]
    pub url: String,

    /// The realm to join
    #[arg(long, default_value = "realm1", global = true)]
    pub realm: String,

    /// Authentication ID
    #[arg(long, global = true)]
    pub authid: Option<String>,

    /// Authentication role
    #[arg(long, global = true)]
    pub authrole: Option<String>,

    /// Secret for ticket/wampcra authentication
    #[arg(long, global = true)]
    pub secret: Option<String>,

    /// Path to private key file for cryptosign
    #[arg(long, global = true)]
    pub private_key: Option<String>,

    /// Ticket for ticket authentication
    #[arg(long, global = true)]
    pub ticket: Option<String>,

    /// Serializer to use (json, msgpack, cbor)
    #[arg(long, default_value = "json", global = true)]
    pub serializer: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Call a procedure
    Call {
        /// Procedure to call
        procedure: String,

        /// Positional arguments for the call
        /// To enforce value is always a string, send value in quotes e.g. "'1'" or '"true"'
        #[arg()]
        args: Vec<String>,

        /// Keyword argument for the call. To enforce value is always a string, send value in quotes e.g."'1'" or '"true"'. (May be provided multiple times)
        #[arg(short = 'k', long = "kwarg", value_name = "KEY=VALUE")]
        kwargs: Vec<String>,

        /// WAMP call option (May be provided multiple times)
        #[arg(short = 'o', long = "option", value_name = "KEY=VALUE")]
        options: Vec<String>,

        /// Number of times to repeat the call per session
        #[arg(long, default_value_t = 1)]
        repeat: u32,

        /// Number of parallel sessions to create
        #[arg(long, default_value_t = 1)]
        parallel: u32,

        /// Maximum number of concurrent sessions
        #[arg(long, default_value_t = 1)]
        concurrency: usize,
    },
    /// Register a procedure
    Register {
        /// Procedure to register
        procedure: String,
    },
    /// Subscribe to a topic
    Subscribe,
    /// Publish to a topic
    Publish,
    /// Generate a WAMP cryptosign ed25519 keypair
    Keygen {
        /// Write keypair to file. Uses 'key' and 'key.pub' by default, or specify a custom name
        #[arg(short = 'O', long = "output-file", value_name = "NAME", num_args = 0..=1, default_missing_value = "key")]
        output_file: Option<String>,
    },
}

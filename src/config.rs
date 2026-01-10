use crate::cli::Cli;
use std::collections::HashMap;
use wampproto::authenticators::anonymous::AnonymousAuthenticator;
use wampproto::authenticators::authenticator::ClientAuthenticator;
use wampproto::authenticators::cryptosign::CryptoSignAuthenticator;
use wampproto::authenticators::ticket::TicketAuthenticator;
use wampproto::authenticators::wampcra::WAMPCRAAuthenticator;
use wampproto::messages::types::Value;
use xconn::async_::client::Client;
use xconn::async_::session::Session;
use xconn::sync::{CBORSerializerSpec, JSONSerializerSpec, MsgPackSerializerSpec, SerializerSpec};

/// Global connection and authentication configuration.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub url: String,
    pub realm: String,
    pub authid: Option<String>,
    pub authrole: Option<String>,
    pub secret: Option<String>,
    pub private_key: Option<String>,
    pub ticket: Option<String>,
    pub serializer: String,
}

impl ConnectionConfig {
    /// Connects to the router using the configured serializer and authentication method.
    pub async fn connect(&self) -> Result<Session, Box<dyn std::error::Error>> {
        let serializer = self.create_serializer()?;
        let authenticator = self.create_authenticator()?;

        let client = Client::new(serializer, authenticator);
        client
            .connect(&self.url, &self.realm)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    /// Creates the appropriate serializer based on the --serializer option.
    fn create_serializer(&self) -> Result<Box<dyn SerializerSpec>, String> {
        match self.serializer.to_lowercase().as_str() {
            "json" => Ok(Box::new(JSONSerializerSpec)),
            "msgpack" => Ok(Box::new(MsgPackSerializerSpec)),
            "cbor" => Ok(Box::new(CBORSerializerSpec)),
            other => Err(format!(
                "Unknown serializer '{}'. Valid options: json, msgpack, cbor",
                other
            )),
        }
    }

    /// Creates the appropriate authenticator based on authentication options.
    /// Priority: --private-key > --secret > --ticket > anonymous
    fn create_authenticator(
        &self,
    ) -> Result<Box<dyn ClientAuthenticator>, Box<dyn std::error::Error>> {
        let authid = self.authid.as_deref().unwrap_or("");
        let extra = self.build_auth_extra();

        // Check for cryptosign (private key)
        if let Some(ref private_key) = self.private_key {
            let auth = CryptoSignAuthenticator::try_new(authid, private_key, extra)
                .map_err(|e| format!("Failed to create CryptoSign authenticator: {}", e))?;
            return Ok(Box::new(auth));
        }

        // Check for WAMP-CRA (secret)
        if let Some(ref secret) = self.secret {
            let auth = WAMPCRAAuthenticator::new(authid, secret, extra);
            return Ok(Box::new(auth));
        }

        // Check for ticket authentication
        if let Some(ref ticket) = self.ticket {
            let auth = TicketAuthenticator::new(authid, ticket, extra);
            return Ok(Box::new(auth));
        }

        // Default to anonymous
        Ok(Box::new(AnonymousAuthenticator::new(authid, extra)))
    }

    /// Builds the authentication extra HashMap, including authrole if specified.
    fn build_auth_extra(&self) -> HashMap<String, Value> {
        let mut extra = HashMap::new();
        if let Some(ref role) = self.authrole {
            extra.insert("authrole".to_string(), Value::Str(role.clone()));
        }
        extra
    }
}

impl From<&Cli> for ConnectionConfig {
    fn from(cli: &Cli) -> Self {
        Self {
            url: cli.url.clone(),
            realm: cli.realm.clone(),
            authid: cli.authid.clone(),
            authrole: cli.authrole.clone(),
            secret: cli.secret.clone(),
            private_key: cli.private_key.clone(),
            ticket: cli.ticket.clone(),
            serializer: cli.serializer.clone(),
        }
    }
}

/// Configuration specific to the Call command.
#[derive(Debug, Clone)]
pub struct CallConfig {
    pub procedure: String,
    pub args: Vec<String>,
    pub kwargs: Vec<String>,
    pub options: Vec<String>,
    pub repeat: u32,
    pub parallel: u32,
    pub concurrency: usize,
}

/// Configuration specific to the Publish command.
#[derive(Debug, Clone)]
pub struct PublishConfig {
    pub topic: String,
    pub args: Vec<String>,
    pub kwargs: Vec<String>,
    pub options: Vec<String>,
    pub repeat: u32,
    pub parallel: u32,
    pub concurrency: usize,
    pub acknowledge: bool,
}

/// Configuration specific to the Subscribe command.
#[derive(Debug, Clone)]
pub struct SubscribeConfig {
    pub topic: String,
    pub parallel: u32,
    pub concurrency: usize,
}

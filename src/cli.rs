use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "matrix-bridge-message")]
#[command(about = "Matrix-SMS/Message Bridge", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, env = "CONFIG_PATH", default_value = "config.yaml")]
    pub config: PathBuf,

    #[arg(short, long, env = "REGISTRATION_PATH")]
    pub registration: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Generate a registration file for the Matrix homeserver")]
    GenerateRegistration {
        #[arg(short, long, default_value = "message-registration.yaml")]
        output: PathBuf,

        #[arg(long, default_value = "message")]
        id: String,

        #[arg(long, default_value = "http://localhost:8008")]
        homeserver_url: String,

        #[arg(long, default_value = "example.org")]
        domain: String,
    },

    #[command(about = "Grant admin privileges to a Matrix user")]
    AdminMe {
        #[arg(short, long)]
        user: String,

        #[arg(short, long)]
        room: Option<String>,

        #[arg(short, long, default_value = "100")]
        power_level: i64,
    },

    #[command(about = "List all bridged portals")]
    ListPortals {
        #[arg(short, long, default_value = "100")]
        limit: i64,
    },

    #[command(about = "Unbridge a room")]
    Unbridge {
        #[arg(short, long, help = "Matrix room ID")]
        room: String,

        #[arg(short, long, help = "Also leave the Matrix room")]
        leave: bool,
    },

    #[command(about = "Validate the configuration file")]
    ValidateConfig,

    #[command(about = "Show bridge status")]
    Status,

    #[command(about = "Test SMS gateway connection")]
    TestGateway {
        #[arg(short, long, help = "Phone number to test")]
        to: String,

        #[arg(short, long, help = "Test message")]
        message: Option<String>,
    },
}

pub fn generate_registration(id: &str, homeserver_url: &str, domain: &str) -> String {
    let as_token = generate_token();
    let hs_token = generate_token();

    let registration = json!({
        "id": id,
        "url": homeserver_url,
        "as_token": as_token,
        "hs_token": hs_token,
        "sender_localpart": "_message_",
        "rate_limited": false,
        "protocols": ["message"],
        "namespaces": {
            "users": [{
                "exclusive": true,
                "regex": "@_message_.*:".to_string() + domain
            }],
            "aliases": [{
                "exclusive": true,
                "regex": "#_message_.*:".to_string() + domain
            }],
            "rooms": []
        }
    });

    serde_yaml::to_string(&registration).unwrap_or_default()
}

fn generate_token() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;

    iter::repeat(())
        .map(|()| thread_rng().sample(Alphanumeric))
        .map(char::from)
        .take(64)
        .collect()
}

pub fn parse_args() -> Cli {
    Cli::parse()
}

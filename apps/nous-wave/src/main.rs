use axum::{Json, Router, extract::State, routing::get};
use clap::{Parser, Subcommand};
use nous_core::{Error, Result};
use nous_memory_service::{LocalRuntime, RuntimeStatus};
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};
mod http;

#[derive(Parser)]
#[command(version, about = "Nous Wave cognitive substrate")]
struct Cli {
    #[arg(long, default_value = "config.toml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Serve,
    Status,
    Subject {
        #[command(subcommand)]
        command: SubjectCommand,
    },
}

#[derive(Subcommand)]
enum SubjectCommand {
    Create {
        #[arg(long)]
        input: PathBuf,
    },
    Show {
        subject: uuid::Uuid,
    },
    List,
    SeedHistory {
        subject: uuid::Uuid,
    },
    ReplaceSeed {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
}

#[derive(Deserialize)]
struct Config {
    server: Server,
    microsystems: MicroSystems,
    postgres: Postgres,
    object_store: Objects,
    retrieval: Retrieval,
}
#[derive(Deserialize)]
struct Server {
    bind: SocketAddr,
    remote_access: bool,
}
#[derive(Deserialize)]
struct MicroSystems {
    memory: bool,
}
#[derive(Deserialize)]
struct Postgres {
    url: String,
    max_connections: u32,
}
#[derive(Deserialize)]
struct Objects {
    backend: String,
    root: String,
}
#[derive(Deserialize)]
struct Retrieval {
    lancedb_uri: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    if let Err(error) = run().await {
        tracing::error!(error = %error, "runtime failed");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let text = tokio::fs::read_to_string(cli.config)
        .await
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let mut config: Config = toml::from_str(&text).map_err(|e| Error::Invalid(e.to_string()))?;
    if let Ok(url) = std::env::var("NOUS_WAVE_POSTGRES_URL") {
        config.postgres.url = url;
    }
    if config.server.remote_access || !config.server.bind.ip().is_loopback() {
        return Err(Error::Invalid("remote binding requires an implemented authentication and transport policy; current runtime supports loopback only".into()));
    }
    if config.object_store.backend != "fs" {
        return Err(Error::Invalid(
            "current object repository requires backend=fs".into(),
        ));
    }
    let runtime = LocalRuntime::open(
        &config.postgres.url,
        config.postgres.max_connections,
        &config.object_store.root,
        config
            .microsystems
            .memory
            .then_some(config.retrieval.lancedb_uri.as_str()),
    )
    .await?;
    match cli.command {
        Command::Subject { command } => {
            let output = match command {
                SubjectCommand::Create { input } => {
                    serde_json::to_value(runtime.create_subject(read_input(input).await?).await?)
                }
                SubjectCommand::Show { subject } => {
                    serde_json::to_value(runtime.subject(nous_core::SubjectId(subject)).await?)
                }
                SubjectCommand::List => serde_json::to_value(runtime.subjects(None, 100).await?),
                SubjectCommand::SeedHistory { subject } => {
                    serde_json::to_value(runtime.seed_history(nous_core::SubjectId(subject)).await?)
                }
                SubjectCommand::ReplaceSeed { subject, input } => serde_json::to_value(
                    runtime
                        .replace_seed(nous_core::SubjectId(subject), read_input(input).await?)
                        .await?,
                ),
            }
            .map_err(|e| Error::Infrastructure(e.to_string()))?;
            println!("{output}");
        }
        Command::Status => println!(
            "{}",
            serde_json::to_string(&runtime.status().await)
                .map_err(|e| Error::Infrastructure(e.to_string()))?
        ),
        Command::Serve => {
            let app = Router::new()
                .route(
                    "/health",
                    get(|| async { Json(serde_json::json!({"api_version": 1, "alive": true})) }),
                )
                .route("/v1/status", get(runtime_status))
                .merge(http::routes())
                .with_state(runtime.clone());
            let listener = tokio::net::TcpListener::bind(config.server.bind)
                .await
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
            tracing::info!(bind = %config.server.bind, "local runtime listening");
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
        }
    }
    runtime.store.close().await;
    Ok(())
}

async fn runtime_status(State(runtime): State<LocalRuntime>) -> Json<RuntimeStatus> {
    Json(runtime.status().await)
}

async fn read_input<T: serde::de::DeserializeOwned>(path: PathBuf) -> Result<T> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| Error::Invalid(e.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|e| Error::Invalid(e.to_string()))
}

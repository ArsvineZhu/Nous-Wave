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
    Material {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Recall {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Cycle {
        #[command(subcommand)]
        command: CycleCommand,
    },
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    Process {
        #[arg(long, default_value_t = 16)]
        limit: usize,
    },
    ProjectionRebuild {
        subject: uuid::Uuid,
    },
    Bundle {
        #[command(subcommand)]
        command: BundleCommand,
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

#[derive(Subcommand)]
enum CycleCommand {
    Begin {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Inspect {
        subject: uuid::Uuid,
        cycle: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Feedback {
        subject: uuid::Uuid,
        cycle: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Close {
        subject: uuid::Uuid,
        cycle: uuid::Uuid,
        #[arg(long)]
        outcome: String,
    },
}
#[derive(Subcommand)]
enum MemoryCommand {
    Show {
        subject: uuid::Uuid,
        object: uuid::Uuid,
    },
    History {
        subject: uuid::Uuid,
        object: uuid::Uuid,
    },
    Correct {
        subject: uuid::Uuid,
        object: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Suppress {
        subject: uuid::Uuid,
        object: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Consolidate {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Purge {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
}
#[derive(Subcommand)]
enum BundleCommand {
    Export {
        subject: uuid::Uuid,
        path: PathBuf,
    },
    Import {
        path: PathBuf,
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
    #[serde(default)]
    models: Models,
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
    #[serde(default = "default_max_upload_bytes")]
    max_upload_bytes: u64,
}
fn default_max_upload_bytes() -> u64 {
    8 * 1024 * 1024 * 1024
}
#[derive(Deserialize)]
struct Retrieval {
    lancedb_uri: String,
}
#[derive(Deserialize, Default)]
struct Models {
    #[serde(default)]
    embedding: Model,
    #[serde(default)]
    rerank: Model,
    #[serde(default)]
    generation: Model,
}
#[derive(Deserialize)]
struct Model {
    #[serde(default)]
    enabled: bool,
    #[serde(default = "default_model_mode")]
    mode: String,
    #[serde(default)]
    endpoint: String,
    #[serde(default)]
    model: String,
    #[serde(default = "default_model_revision")]
    revision: String,
    #[serde(default = "default_preprocessing")]
    preprocessing: String,
    #[serde(default = "default_model_timeout")]
    timeout_seconds: u64,
    #[serde(default = "default_model_response")]
    max_response_bytes: usize,
    #[serde(default)]
    cache_dir: String,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: default_model_mode(),
            endpoint: String::new(),
            model: String::new(),
            revision: default_model_revision(),
            preprocessing: default_preprocessing(),
            timeout_seconds: default_model_timeout(),
            max_response_bytes: default_model_response(),
            cache_dir: String::new(),
        }
    }
}
fn default_model_mode() -> String {
    "http".into()
}
fn default_model_revision() -> String {
    "unconfigured".into()
}
fn default_preprocessing() -> String {
    "identity".into()
}
fn default_model_timeout() -> u64 {
    30
}
fn default_model_response() -> usize {
    8 * 1024 * 1024
}
#[derive(serde::Deserialize)]
struct CliObjectRefs {
    object_refs: Vec<uuid::Uuid>,
}
#[derive(serde::Deserialize)]
struct CliSuppress {
    suppressed: bool,
    reason: String,
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
    let runtime = LocalRuntime::open_with_options(
        &config.postgres.url,
        config.postgres.max_connections,
        &config.object_store.root,
        config
            .microsystems
            .memory
            .then_some(config.retrieval.lancedb_uri.as_str()),
        config.microsystems.memory,
        config.object_store.max_upload_bytes,
    )
    .await?
    .with_models({
        let embedding = config.models.embedding;
        let local = if embedding.enabled && embedding.mode == "local" {
            let cache_dir = if embedding.cache_dir.is_empty() {
                PathBuf::from("models")
            } else {
                PathBuf::from(&embedding.cache_dir)
            };
            Some(nous_memory_service::models::LocalEmbeddingModel::load(cache_dir).await?)
        } else {
            None
        };
        let services = nous_memory_service::models::ModelServices::new(
            Some(model_endpoint(embedding)),
            Some(model_endpoint(config.models.rerank)),
            Some(model_endpoint(config.models.generation)),
        )?;
        local.map_or(services.clone(), |model| {
            services.with_local_embedding(model)
        })
    });
    match cli.command {
        Command::Material { subject, input } => println!("{}", serde_json::to_string(&runtime.ingest(nous_core::SubjectId(subject), read_input(input).await?).await?).map_err(|e|Error::Infrastructure(e.to_string()))?),
        Command::Recall { subject, input } => println!("{}", serde_json::to_string(&runtime.recall(nous_core::SubjectId(subject), read_input(input).await?).await?).map_err(|e|Error::Infrastructure(e.to_string()))?),
        Command::Cycle { command } => {
            let output=match command {
                CycleCommand::Begin {subject,input} => serde_json::to_value(serde_json::json!({"cycle_id":runtime.begin_cycle(nous_core::SubjectId(subject),read_input::<serde_json::Value>(input).await?).await?})),
                CycleCommand::Inspect {subject,cycle,input} => serde_json::to_value(runtime.inspect_cycle(nous_core::SubjectId(subject),cycle,&read_input::<CliObjectRefs>(input).await?.object_refs).await?),
                CycleCommand::Feedback {subject,cycle,input} => {runtime.feedback(nous_core::SubjectId(subject),cycle,read_input(input).await?).await?;Ok(serde_json::json!({"status":"accepted"}))},
                CycleCommand::Close {subject,cycle,outcome} => {runtime.close_cycle(nous_core::SubjectId(subject),cycle,&outcome).await?;Ok(serde_json::json!({"status":"closed"}))},
            }.map_err(|e:serde_json::Error|Error::Infrastructure(e.to_string()))?;
            println!("{output}");
        }
        Command::Memory { command } => {
            let output=match command {
                MemoryCommand::Show {subject,object} => serde_json::to_value(runtime.memory(nous_core::SubjectId(subject),object,None).await?),
                MemoryCommand::History {subject,object} => serde_json::to_value(runtime.memory_history(nous_core::SubjectId(subject),object).await?),
                MemoryCommand::Correct {subject,object,input} => serde_json::to_value(runtime.correct_memory(nous_core::SubjectId(subject),object,read_input(input).await?).await?),
                MemoryCommand::Suppress {subject,object,input} => {let input:CliSuppress=read_input(input).await?;runtime.suppress(nous_core::SubjectId(subject),object,input.suppressed,&input.reason).await?;Ok(serde_json::json!({"status":"updated"}))},
                MemoryCommand::Consolidate {subject,input} => serde_json::to_value(runtime.consolidate(nous_core::SubjectId(subject),read_input(input).await?).await?),
                MemoryCommand::Purge {subject,input} => serde_json::to_value(runtime.purge(nous_core::SubjectId(subject),read_input(input).await?).await?),
            }.map_err(|e:serde_json::Error|Error::Infrastructure(e.to_string()))?;
            println!("{output}");
        }
        Command::Process { limit } => println!("{}",serde_json::to_string(&runtime.process_pending(limit).await?).map_err(|e|Error::Infrastructure(e.to_string()))?),
        Command::ProjectionRebuild { subject } => println!("{}",serde_json::to_string(&serde_json::json!({"rebuilt_rows":runtime.rebuild_projection(nous_core::SubjectId(subject)).await?})).map_err(|e|Error::Infrastructure(e.to_string()))?),
        Command::Bundle { command } => {
            let output=match command {
                BundleCommand::Export {subject,path} => serde_json::to_value(runtime.export_bundle(nous_core::SubjectId(subject),path).await?),
                BundleCommand::Import {path,input} => serde_json::to_value(runtime.import_bundle(path,read_input(input).await?).await?),
            }.map_err(|e:serde_json::Error|Error::Infrastructure(e.to_string()))?;
            println!("{output}");
        }
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

fn model_endpoint(model: Model) -> nous_memory_service::models::ModelEndpoint {
    nous_memory_service::models::ModelEndpoint {
        enabled: model.enabled,
        endpoint: model.endpoint,
        model: model.model,
        revision: model.revision,
        preprocessing: model.preprocessing,
        timeout_seconds: model.timeout_seconds,
        max_response_bytes: model.max_response_bytes,
    }
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

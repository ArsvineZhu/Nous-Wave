use axum::{Json, Router, extract::State, routing::get};
use clap::{Parser, Subcommand};
use nous_cognitive_runtime::UseFeedback;
use nous_core::{DerivationId, Error, MemoryId, Result, SubjectId};
use nous_memory_domain::ExplicitMemoryInput;
use nous_subject_core::{CharacterSeedInput, CreateSubject};
use nous_wave::{NousRuntime, RuntimeOptions, RuntimeProviderConfig, RuntimeStatus};
use postgresql_embedded::{PostgreSQL, SettingsBuilder, VersionReq};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

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
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    Observe {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    FormMemory {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Query {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Use {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    ProjectionsRefresh {
        subject: uuid::Uuid,
    },
    Derivation {
        #[command(subcommand)]
        command: DerivationCommand,
    },
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    Resource {
        #[command(subcommand)]
        command: ResourceCommand,
    },
}

#[derive(Subcommand)]
enum SubjectCommand {
    Create {
        #[arg(long)]
        seed: PathBuf,
        #[arg(long)]
        metadata: Option<PathBuf>,
    },
    Show {
        subject: uuid::Uuid,
    },
}

#[derive(Subcommand)]
enum SessionCommand {
    Open {
        subject: uuid::Uuid,
    },
    Show {
        subject: uuid::Uuid,
        session: uuid::Uuid,
    },
    Close {
        subject: uuid::Uuid,
        session: uuid::Uuid,
    },
}

#[derive(Subcommand)]
enum MemoryCommand {
    Form {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Show {
        subject: uuid::Uuid,
        memory: uuid::Uuid,
    },
    History {
        subject: uuid::Uuid,
        memory: uuid::Uuid,
    },
    Suppress {
        subject: uuid::Uuid,
        memory: uuid::Uuid,
    },
    Restore {
        subject: uuid::Uuid,
        memory: uuid::Uuid,
    },
    Purge {
        subject: uuid::Uuid,
        memory: uuid::Uuid,
    },
}

#[derive(Subcommand)]
enum ResourceCommand {
    Put {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    List {
        subject: uuid::Uuid,
    },
    Delete {
        subject: uuid::Uuid,
        resource: String,
    },
}

#[derive(Subcommand)]
enum DerivationCommand {
    RunPending {
        #[arg(long, default_value_t = 16)]
        limit: usize,
        #[arg(long, default_value = "cli")]
        owner: String,
    },
    Retry {
        derivation: uuid::Uuid,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    server: ServerConfig,
    #[serde(default = "default_true")]
    memory_enabled: bool,
    database: DatabaseConfig,
    object_store: ObjectStoreConfig,
    #[serde(default)]
    retrieval: RetrievalConfig,
    #[serde(default)]
    providers: RuntimeProviderConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ServerConfig {
    bind: SocketAddr,
    #[serde(default)]
    remote_access: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DatabaseConfig {
    #[serde(default = "default_database_mode")]
    mode: String,
    #[serde(default)]
    url: String,
    #[serde(default = "default_max_connections")]
    max_connections: u32,
    #[serde(default = "default_database_name")]
    name: String,
    #[serde(default = "default_install_dir")]
    install_dir: String,
    #[serde(default = "default_data_dir")]
    data_dir: String,
}

fn default_database_mode() -> String {
    "managed".into()
}
fn default_max_connections() -> u32 {
    8
}
fn default_database_name() -> String {
    "nous_wave_20260908".into()
}
fn default_install_dir() -> String {
    "./data/postgres-install".into()
}
fn default_data_dir() -> String {
    "./data/postgres".into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectStoreConfig {
    #[serde(default = "default_backend")]
    backend: String,
    #[serde(default = "default_object_root")]
    root: String,
    #[serde(default = "default_upload_limit")]
    max_upload_bytes: u64,
}

fn default_backend() -> String {
    "fs".into()
}
fn default_object_root() -> String {
    "./data/objects".into()
}
fn default_upload_limit() -> u64 {
    8 * 1024 * 1024 * 1024
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetrievalConfig {
    #[serde(default = "default_resident_limit")]
    resident_limit: usize,
    #[serde(default = "default_serving_root")]
    root: String,
    #[serde(default = "default_true")]
    lexical_enabled: bool,
    #[serde(default = "default_true")]
    dense_enabled: bool,
    #[serde(default = "default_true")]
    topology_enabled: bool,
}
fn default_serving_root() -> String {
    "./data/serving".into()
}
fn default_true() -> bool {
    true
}
impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            resident_limit: 256,
            root: default_serving_root(),
            lexical_enabled: true,
            dense_enabled: true,
            topology_enabled: true,
        }
    }
}

fn default_resident_limit() -> usize {
    256
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
    let config_text = tokio::fs::read_to_string(&cli.config)
        .await
        .map_err(|error| Error::Invalid(error.to_string()))?;
    let mut config: Config =
        toml::from_str(&config_text).map_err(|error| Error::Invalid(error.to_string()))?;
    if let Ok(url) = std::env::var("NOUS_WAVE_POSTGRES_URL") {
        config.database.mode = "external".into();
        config.database.url = url;
    }
    if config.server.remote_access || !config.server.bind.ip().is_loopback() {
        return Err(Error::Invalid(
            "current runtime supports loopback binding only".into(),
        ));
    }
    if config.object_store.backend != "fs" {
        return Err(Error::Invalid("object_store.backend must be fs".into()));
    }
    let app_root = std::env::current_exe()
        .map_err(|error| Error::Infrastructure(error.to_string()))?
        .parent()
        .ok_or_else(|| Error::Infrastructure("executable has no parent".into()))?
        .to_path_buf();
    let (postgres_url, managed) = open_database(&app_root, &config.database).await?;
    let object_root = resolve_path(&app_root, &config.object_store.root);
    let runtime_result = NousRuntime::open(RuntimeOptions {
        postgres_url,
        max_connections: config.database.max_connections,
        object_root: object_root.to_string_lossy().into_owned(),
        max_upload_bytes: config.object_store.max_upload_bytes,
        resident_limit: config.retrieval.resident_limit,
        memory_enabled: config.memory_enabled,
        serving_options: nous_serving::ServingOptions {
            root: resolve_path(&app_root, &config.retrieval.root),
            lexical: config.retrieval.lexical_enabled,
            dense: config.retrieval.dense_enabled,
            topology: config.retrieval.topology_enabled,
            memory_enabled: config.memory_enabled,
        },
        embedding: None,
        providers: config.providers,
    })
    .await;
    let runtime = match runtime_result {
        Ok(runtime) => runtime,
        Err(error) => {
            stop_managed(managed).await?;
            return Err(error);
        }
    };
    let result = dispatch(cli.command, runtime.clone(), config.server.bind).await;
    runtime.store.close().await;
    stop_managed(managed).await?;
    result
}

// CLI dispatch is the stable host boundary for the modular runtime composition.
#[expect(
    clippy::too_many_lines,
    reason = "CLI dispatch is the single host boundary for the shipped commands"
)]
async fn dispatch(command: Command, runtime: NousRuntime, bind: SocketAddr) -> Result<()> {
    match command {
        Command::Serve => {
            let app = Router::new()
                .route(
                    "/health",
                    get(|| async { Json(serde_json::json!({"api_version":1,"alive":true})) }),
                )
                .route("/v1/status", get(status))
                .merge(http::routes())
                .with_state(runtime);
            let listener = tokio::net::TcpListener::bind(bind)
                .await
                .map_err(|error| Error::Infrastructure(error.to_string()))?;
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await
                .map_err(|error| Error::Infrastructure(error.to_string()))
        }
        Command::Status => print_json(runtime.status().await),
        Command::Subject { command } => match command {
            SubjectCommand::Create { seed, metadata } => {
                let metadata = match metadata {
                    Some(path) => read_json(path).await?,
                    None => serde_json::json!({}),
                };
                print_json(
                    runtime
                        .subjects
                        .create_subject(CreateSubject {
                            subject_id: None,
                            config: metadata,
                            character_seed: CharacterSeedInput {
                                text: tokio::fs::read_to_string(&seed)
                                    .await
                                    .map_err(|error| Error::Invalid(error.to_string()))?,
                                media_type: "text/plain; charset=utf-8".into(),
                                provenance: serde_json::json!({"source_path": seed}),
                            },
                        })
                        .await?,
                )
            }
            SubjectCommand::Show { subject } => {
                print_json(runtime.subjects.subject(SubjectId(subject)).await?)
            }
        },
        Command::Session { command } => match command {
            SessionCommand::Open { subject } => print_json(
                runtime
                    .cognition
                    .open_session(SubjectId(subject), serde_json::json!({}))
                    .await?,
            ),
            SessionCommand::Show { subject, session } => print_json(
                runtime
                    .cognition
                    .session(SubjectId(subject), nous_core::SessionId(session))
                    .await?,
            ),
            SessionCommand::Close { subject, session } => print_json(
                runtime
                    .cognition
                    .close_session(SubjectId(subject), nous_core::SessionId(session))
                    .await?,
            ),
        },
        Command::Observe { subject, input } => {
            let mut input: nous_material::ObservationInput = read_json(input).await?;
            input.subject = SubjectId(subject);
            print_json(runtime.observe(input).await?)
        }
        Command::FormMemory { subject, input } => {
            let mut input: ExplicitMemoryInput = read_json(input).await?;
            input.subject = SubjectId(subject);
            print_json(runtime.require_memory()?.form_memory(input).await?)
        }
        Command::Query { subject, input } => {
            let mut input: nous_core::CognitiveQuery = read_json(input).await?;
            input.subject = SubjectId(subject);
            print_json(runtime.query(input).await?)
        }
        Command::Use { subject, input } => {
            let mut input: UseFeedback = read_json(input).await?;
            input.subject = SubjectId(subject);
            runtime.cognition.use_feedback(input).await?;
            print_json(serde_json::json!({"status":"accepted"}))
        }
        Command::ProjectionsRefresh { subject } => {
            print_json(runtime.serving.refresh(SubjectId(subject)).await?)
        }
        Command::Derivation { command } => match command {
            DerivationCommand::RunPending { .. } => Err(Error::Unavailable(
                "document derivation provider is not configured for the CLI runtime".into(),
            )),
            DerivationCommand::Retry { derivation } => {
                runtime
                    .material
                    .retry_derivation(DerivationId(derivation))
                    .await?;
                print_json(serde_json::json!({"status":"queued"}))
            }
        },
        Command::Memory { command } => match command {
            MemoryCommand::Form { subject, input } => {
                let mut input: ExplicitMemoryInput = read_json(input).await?;
                input.subject = SubjectId(subject);
                print_json(runtime.require_memory()?.form_memory(input).await?)
            }
            MemoryCommand::Show { subject, memory } => print_json(
                runtime
                    .require_memory()?
                    .memory(SubjectId(subject), MemoryId(memory), None)
                    .await?,
            ),
            MemoryCommand::History { subject, memory } => print_json(
                runtime
                    .require_memory()?
                    .memory_history(SubjectId(subject), MemoryId(memory))
                    .await?,
            ),
            MemoryCommand::Suppress { subject, memory } => print_json(
                runtime
                    .require_memory()?
                    .suppress(SubjectId(subject), MemoryId(memory))
                    .await?,
            ),
            MemoryCommand::Restore { subject, memory } => print_json(
                runtime
                    .require_memory()?
                    .restore(SubjectId(subject), MemoryId(memory))
                    .await?,
            ),
            MemoryCommand::Purge { subject, memory } => {
                runtime
                    .require_memory()?
                    .purge_memory(SubjectId(subject), MemoryId(memory))
                    .await?;
                print_json(serde_json::json!({"status":"purged"}))
            }
        },
        Command::Resource { command } => match command {
            ResourceCommand::Put { subject, input } => print_json(
                runtime
                    .cognition
                    .upsert_resource(SubjectId(subject), read_json(input).await?)
                    .await?,
            ),
            ResourceCommand::List { subject } => {
                print_json(runtime.cognition.list_resources(SubjectId(subject)).await?)
            }
            ResourceCommand::Delete { subject, resource } => {
                runtime
                    .cognition
                    .delete_resource(SubjectId(subject), nous_core::ResourceRef::new(resource)?)
                    .await?;
                print_json(serde_json::json!({"status":"deleted"}))
            }
        },
    }
}

async fn status(State(runtime): State<NousRuntime>) -> Json<RuntimeStatus> {
    Json(runtime.status().await)
}

fn print_json<T: Serialize>(value: T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&value)
            .map_err(|error| Error::Infrastructure(error.to_string()))?
    );
    Ok(())
}

async fn read_json<T: DeserializeOwned>(path: PathBuf) -> Result<T> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|error| Error::Invalid(error.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|error| Error::Invalid(error.to_string()))
}

async fn open_database(
    root: &Path,
    config: &DatabaseConfig,
) -> Result<(String, Option<PostgreSQL>)> {
    match config.mode.as_str() {
        "external" => {
            if config.url.trim().is_empty() {
                return Err(Error::Invalid(
                    "external database mode requires database.url".into(),
                ));
            }
            Ok((config.url.clone(), None))
        }
        "managed" => {
            let install_dir = resolve_path(root, &config.install_dir);
            let data_dir = resolve_path(root, &config.data_dir);
            tokio::fs::create_dir_all(&install_dir)
                .await
                .map_err(|error| Error::Infrastructure(error.to_string()))?;
            if let Some(parent) = data_dir.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|error| Error::Infrastructure(error.to_string()))?;
            }
            let password_file = data_dir
                .parent()
                .unwrap_or(&data_dir)
                .join("postgres.pgpass");
            let settings = SettingsBuilder::new()
                .version(
                    VersionReq::parse("=18.6.0")
                        .map_err(|error| Error::Invalid(error.to_string()))?,
                )
                .host("127.0.0.1")
                .port(0)
                .username("postgres")
                .password("nous_wave")
                .installation_dir(install_dir)
                .data_dir(data_dir)
                .password_file(password_file)
                .temporary(false)
                .build();
            let mut postgres = PostgreSQL::new(settings);
            postgres.setup().await.map_err(|error| {
                Error::Infrastructure(format!("managed PostgreSQL setup: {error}"))
            })?;
            postgres.start().await.map_err(|error| {
                Error::Infrastructure(format!("managed PostgreSQL start: {error}"))
            })?;
            let database = if config.name.trim().is_empty() {
                "nous_wave_20260908"
            } else {
                &config.name
            };
            if !postgres
                .database_exists(database)
                .await
                .map_err(|error| Error::Infrastructure(error.to_string()))?
            {
                postgres
                    .create_database(database)
                    .await
                    .map_err(|error| Error::Infrastructure(error.to_string()))?;
            }
            Ok((postgres.settings().url(database), Some(postgres)))
        }
        other => Err(Error::Invalid(format!("unsupported database.mode {other}"))),
    }
}

async fn stop_managed(mut postgres: Option<PostgreSQL>) -> Result<()> {
    if let Some(postgres) = postgres.as_mut() {
        postgres
            .stop()
            .await
            .map_err(|error| Error::Infrastructure(error.to_string()))?;
    }
    Ok(())
}

fn resolve_path(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

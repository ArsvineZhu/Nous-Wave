use axum::{Json, Router, extract::State, routing::get};
use clap::{Parser, Subcommand};
use futures::StreamExt;
use nous_core::{Error, Result};
use nous_memory_service::{LocalRuntime, RuntimeStatus};
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
mod http;
#[cfg(test)]
#[path = "../../../crates/memory-service/tests/support/mod.rs"]
mod support;

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
        #[command(subcommand)]
        command: MaterialCommand,
    },
    Source {
        #[command(subcommand)]
        command: SourceCommand,
    },
    Artifact {
        #[command(subcommand)]
        command: ArtifactCommand,
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
enum MaterialCommand {
    Ingest {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    Upload {
        subject: uuid::Uuid,
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        metadata: PathBuf,
    },
    Status {
        subject: uuid::Uuid,
        source: uuid::Uuid,
    },
    Derive {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
    ToolObservation {
        subject: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
}
#[derive(Subcommand)]
enum SourceCommand {
    Show {
        subject: uuid::Uuid,
        source: uuid::Uuid,
    },
}
#[derive(Subcommand)]
enum ArtifactCommand {
    Show {
        subject: uuid::Uuid,
        artifact: uuid::Uuid,
    },
    Get {
        subject: uuid::Uuid,
        artifact: uuid::Uuid,
        #[arg(long)]
        output: PathBuf,
    },
    Lineage {
        subject: uuid::Uuid,
        artifact: uuid::Uuid,
    },
}

#[derive(Subcommand)]
enum SubjectCommand {
    Create {
        #[arg(
            long,
            required_unless_present = "seed_file",
            conflicts_with = "seed_file"
        )]
        input: Option<PathBuf>,
        #[arg(long)]
        seed_file: Option<PathBuf>,
        #[arg(long, default_value = "application/octet-stream")]
        media_type: String,
        #[arg(long, default_value = "host:cli")]
        authored_by: String,
        #[arg(long)]
        label: Option<String>,
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
    Recall {
        subject: uuid::Uuid,
        cycle: uuid::Uuid,
        #[arg(long)]
        input: PathBuf,
    },
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
    EpisodeMembership {
        subject: uuid::Uuid,
        object: uuid::Uuid,
    },
    Associations {
        subject: uuid::Uuid,
        object: uuid::Uuid,
    },
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
#[serde(deny_unknown_fields)]
struct Config {
    server: Server,
    microsystems: MicroSystems,
    postgres: Postgres,
    object_store: Objects,
    retrieval: Retrieval,
    #[serde(default)]
    models: Models,
    #[serde(default)]
    association: nous_memory_retrieval::association::ActivationConfig,
    #[serde(default)]
    accessibility: Accessibility,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Server {
    bind: SocketAddr,
    remote_access: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MicroSystems {
    memory: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Postgres {
    url: String,
    max_connections: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
struct Retrieval {
    lancedb_uri: String,
    #[serde(default = "default_effort")]
    default_effort: nous_memory_domain::recall::RecallEffort,
    #[serde(default)]
    budgets: std::collections::BTreeMap<
        nous_memory_domain::recall::RecallEffort,
        nous_memory_service::recall::RecallBudget,
    >,
}
fn default_effort() -> nous_memory_domain::recall::RecallEffort {
    nous_memory_domain::recall::RecallEffort::Normal
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Accessibility {
    use_decay: f64,
}
impl Default for Accessibility {
    fn default() -> Self {
        Self { use_decay: 0.5 }
    }
}
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Models {
    #[serde(default)]
    embedding: Model,
    #[serde(default)]
    rerank: Model,
    #[serde(default)]
    generation: Model,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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
    config.association.validate()?;
    for budget in config.retrieval.budgets.values() {
        budget.validate()?;
    }
    if !config.accessibility.use_decay.is_finite()
        || config.accessibility.use_decay <= 0.0
        || config.accessibility.use_decay > 1.0
    {
        return Err(Error::Invalid(
            "accessibility.use_decay must be in (0,1]".into(),
        ));
    }
    if !matches!(config.models.embedding.mode.as_str(), "local" | "http") {
        return Err(Error::Invalid(
            "embedding mode must be local or http".into(),
        ));
    }
    let mut runtime = LocalRuntime::open_with_options(
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
        let mut embedding = config.models.embedding;
        let local = if embedding.enabled && embedding.mode == "local" {
            let cache_dir = if embedding.cache_dir.is_empty() {
                PathBuf::from("models")
            } else {
                PathBuf::from(&embedding.cache_dir)
            };
            Some(
                nous_memory_service::models::LocalEmbeddingModel::load(cache_dir, &embedding.model)
                    .await?,
            )
        } else {
            None
        };
        if local.is_some() {
            embedding.enabled = false;
        }
        let services = nous_memory_service::models::ModelServices::new(
            Some(model_endpoint(embedding)),
            Some(model_endpoint(config.models.rerank)),
            Some(model_endpoint(config.models.generation)),
        )?;
        local.map_or(services.clone(), |model| {
            services.with_local_embedding(model)
        })
    });
    runtime.default_effort = config.retrieval.default_effort;
    runtime.recall_budgets = config.retrieval.budgets;
    runtime.association_config = config.association;
    runtime.use_decay = config.accessibility.use_decay;
    match cli.command {
        Command::Material { command } => {
            let output = match command {
                MaterialCommand::Ingest { subject, input } => serde_json::to_value(runtime.ingest(nous_core::SubjectId(subject), read_input(input).await?).await?),
                MaterialCommand::Status { subject, source } => serde_json::to_value(runtime.material_status(nous_core::SubjectId(subject), source).await?),
                MaterialCommand::Derive { subject, input } => serde_json::to_value(runtime.submit_derivation(nous_core::SubjectId(subject), read_input(input).await?).await?),
                MaterialCommand::ToolObservation { subject, input } => serde_json::to_value(runtime.record_tool_observation(nous_core::SubjectId(subject), read_input(input).await?).await?),
                MaterialCommand::Upload { subject, file, metadata } => {
                    let metadata_file = tokio::fs::File::open(metadata).await.map_err(|e| Error::Invalid(e.to_string()))?;
                    let mut metadata_bytes = Vec::new();
                    metadata_file.take(65537).read_to_end(&mut metadata_bytes).await.map_err(|e| Error::Invalid(e.to_string()))?;
                    if metadata_bytes.len() > 65536 { return Err(Error::Invalid("metadata exceeds 64 KiB".into())); }
                    let metadata = serde_json::from_slice(&metadata_bytes).map_err(|e| Error::Invalid(e.to_string()))?;
                    let file = tokio::fs::File::open(file).await.map_err(|e| Error::Invalid(e.to_string()))?;
                    let chunks = futures::stream::try_unfold(file, |mut file| async move {
                        let mut bytes = vec![0; 1024 * 1024];
                        let size = file.read(&mut bytes).await.map_err(|e| Error::Infrastructure(e.to_string()))?;
                        if size == 0 { return Ok(None); }
                        bytes.truncate(size);
                        Ok(Some((bytes, file)))
                    });
                    serde_json::to_value(runtime.ingest_stream(nous_core::SubjectId(subject), metadata, chunks).await?)
                }
            }.map_err(|e| Error::Infrastructure(e.to_string()))?;
            println!("{output}");
        }
        Command::Source { command: SourceCommand::Show { subject, source } } => println!("{}", runtime.source_show(nous_core::SubjectId(subject), source).await?),
        Command::Artifact { command } => {
            match command {
                ArtifactCommand::Show { subject, artifact } => println!("{}", serde_json::to_value(runtime.artifact(nous_core::SubjectId(subject), artifact).await?).map_err(|e| Error::Infrastructure(e.to_string()))?),
                ArtifactCommand::Lineage { subject, artifact } => println!("{}", runtime.artifact_lineage(nous_core::SubjectId(subject), artifact).await?),
                ArtifactCommand::Get { subject, artifact, output } => {
                    let (_, stream) = runtime.artifact_stream(nous_core::SubjectId(subject), artifact).await?;
                    let mut file = tokio::fs::File::create(output).await.map_err(|e| Error::Invalid(e.to_string()))?;
                    futures::pin_mut!(stream);
                    while let Some(chunk) = stream.next().await { file.write_all(&chunk?).await.map_err(|e| Error::Infrastructure(e.to_string()))?; }
                    file.flush().await.map_err(|e| Error::Infrastructure(e.to_string()))?;
                }
            }
        }
        Command::Recall { subject, input } => println!("{}", serde_json::to_string(&runtime.recall(nous_core::SubjectId(subject), runtime.recall_input(read_input(input).await?)?).await?).map_err(|e|Error::Infrastructure(e.to_string()))?),
        Command::Cycle { command } => {
            let output=match command {
                CycleCommand::Recall {subject,cycle,input} => { let mut intent = runtime.recall_input(read_input(input).await?)?; intent.cycle_id = Some(cycle); serde_json::to_value(runtime.recall(nous_core::SubjectId(subject),intent).await?) },
                CycleCommand::Begin {subject,input} => serde_json::to_value(serde_json::json!({"cycle_id":runtime.begin_cycle(nous_core::SubjectId(subject),read_input::<serde_json::Value>(input).await?).await?})),
                CycleCommand::Inspect {subject,cycle,input} => serde_json::to_value(runtime.inspect_cycle(nous_core::SubjectId(subject),cycle,&read_input::<CliObjectRefs>(input).await?.object_refs).await?),
                CycleCommand::Feedback {subject,cycle,input} => {runtime.feedback(nous_core::SubjectId(subject),cycle,read_input(input).await?).await?;Ok(serde_json::json!({"status":"accepted"}))},
                CycleCommand::Close {subject,cycle,outcome} => {runtime.close_cycle(nous_core::SubjectId(subject),cycle,&outcome).await?;Ok(serde_json::json!({"status":"closed"}))},
            }.map_err(|e:serde_json::Error|Error::Infrastructure(e.to_string()))?;
            println!("{output}");
        }
        Command::Memory { command } => {
            let output=match command {
                MemoryCommand::EpisodeMembership {subject,object} => serde_json::to_value(runtime.episode_membership(nous_core::SubjectId(subject),object).await?),
                MemoryCommand::Associations {subject,object} => serde_json::to_value(runtime.association_evidence(nous_core::SubjectId(subject),object).await?),
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
                SubjectCommand::Create { input, seed_file, media_type, authored_by, label } => {
                    let input = if let Some(input) = input { read_input(input).await? } else {
                        let file = tokio::fs::File::open(seed_file.ok_or_else(|| Error::Invalid("seed file required".into()))?).await.map_err(|e| Error::Invalid(e.to_string()))?;
                        let mut bytes = vec![];
                        file.take(runtime.max_upload_bytes.saturating_add(1)).read_to_end(&mut bytes).await.map_err(|e| Error::Invalid(e.to_string()))?;
                        nous_memory_service::subjects::CreateSubject { api_version: 1, label, metadata: serde_json::json!({}), memory_enabled: true, character_seed: nous_memory_service::subjects::SeedInput { content: nous_memory_service::subjects::SeedContent::InlineBytes { bytes, media_type }, authored_by } }
                    };
                    serde_json::to_value(runtime.create_subject(input).await?)
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configuration_is_strict_and_cli_exposes_material_paths() {
        let text = include_str!("../../../config.example.toml");
        let config: Config = toml::from_str(text).unwrap();
        assert_eq!(config.object_store.max_upload_bytes, 8 * 1024 * 1024 * 1024);
        assert_eq!(
            config.retrieval.budgets[&default_effort()].max_candidates,
            64
        );
        let invalid = text.replace(
            "max_upload_bytes = 8589934592",
            "inline_payload_max_bytes = 4096",
        );
        assert!(toml::from_str::<Config>(&invalid).is_err());
        let subject = uuid::Uuid::now_v7().to_string();
        assert!(
            Cli::try_parse_from([
                "nous-wave",
                "material",
                "upload",
                &subject,
                "--file",
                "a.bin",
                "--metadata",
                "meta.json"
            ])
            .is_ok()
        );
        assert!(
            Cli::try_parse_from(["nous-wave", "subject", "create", "--seed-file", "seed.txt"])
                .is_ok()
        );
        assert!(Cli::try_parse_from(["nous-wave", "subject", "create"]).is_err());
    }
}

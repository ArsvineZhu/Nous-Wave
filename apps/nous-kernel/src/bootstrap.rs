use nous_core::{Error, Result};
use nous_kernel::{NousRuntime, RuntimeOptions};
use postgresql_embedded::{PostgreSQL, SettingsBuilder, VersionReq};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    stored_embedding: Option<nous_serving::StoredEmbeddingConfig>,
    server: ServerConfig,
    #[serde(default = "default_true")]
    memory_enabled: bool,
    database: DatabaseConfig,
    object_store: ObjectStoreConfig,
    #[serde(default)]
    retrieval: RetrievalConfig,
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

pub async fn open(path: &Path) -> Result<(NousRuntime, Option<PostgreSQL>)> {
    let text = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let mut config: Config = toml::from_str(&text).map_err(|e| Error::Invalid(e.to_string()))?;
    if let Ok(url) = std::env::var("NOUS_WAVE_POSTGRES_URL") {
        config.database.mode = "external".into();
        config.database.url = url;
    }
    if config.server.remote_access || !config.server.bind.ip().is_loopback() {
        return Err(Error::Invalid("loopback binding required".into()));
    }
    if config.object_store.backend != "fs" {
        return Err(Error::Invalid("object store backend must be fs".into()));
    }
    let absolute = std::path::absolute(path).map_err(|e| Error::Invalid(e.to_string()))?;
    let root = absolute
        .parent()
        .ok_or_else(|| Error::Invalid("config parent required".into()))?;
    let (postgres_url, managed) = open_database(root, &config.database).await?;
    let result = NousRuntime::open(RuntimeOptions {
        postgres_url,
        max_connections: config.database.max_connections,
        object_root: resolve_path(root, &config.object_store.root)
            .to_string_lossy()
            .into_owned(),
        max_upload_bytes: config.object_store.max_upload_bytes,
        resident_limit: config.retrieval.resident_limit,
        memory_enabled: config.memory_enabled,
        serving_options: nous_serving::ServingOptions {
            root: resolve_path(root, &config.retrieval.root),
            lexical: config.retrieval.lexical_enabled,
            dense: config.retrieval.dense_enabled,
            topology: config.retrieval.topology_enabled,
            memory_enabled: config.memory_enabled,
        },
        embedding: None,
        stored_embedding: config.stored_embedding,
    })
    .await;
    match result {
        Ok(runtime) => Ok((runtime, managed)),
        Err(e) => {
            stop_managed(managed).await?;
            Err(e)
        }
    }
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

pub async fn stop_managed(mut postgres: Option<PostgreSQL>) -> Result<()> {
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

use clap::Parser;
use nous_core::{Error, Result};
use nous_kernel::transport::KernelService;
use nous_protocol::kernel::{
    artifact_stream_service_server::ArtifactStreamServiceServer,
    authority_service_server::AuthorityServiceServer,
    model_material_service_server::ModelMaterialServiceServer,
    runtime_store_service_server::RuntimeStoreServiceServer,
};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Status};
mod bootstrap;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    config: PathBuf,
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    if let Err(error) = run().await {
        tracing::error!(%error,"Kernel failed");
        std::process::exit(1);
    }
}
async fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut input = BufReader::new(tokio::io::stdin());
    let mut token = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        (&mut input).take(130).read_line(&mut token),
    )
    .await
    .map_err(|_| Error::Invalid("Kernel bootstrap timed out".into()))?
    .map_err(|e| Error::Invalid(e.to_string()))?;
    let token = token.trim().to_owned();
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::Invalid("invalid Kernel bootstrap credential".into()));
    }
    let (runtime, managed) = bootstrap::open(&cli.config).await?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?;
    let endpoint = listener
        .local_addr()
        .map_err(|e| Error::Infrastructure(e.to_string()))?;
    let auth = move |request: Request<()>| -> std::result::Result<Request<()>, Status> {
        let value = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        let expected = format!("Bearer {token}");
        if value.len() != expected.len()
            || value
                .bytes()
                .zip(expected.bytes())
                .fold(0u8, |a, (x, y)| a | (x ^ y))
                != 0
        {
            return Err(Status::unauthenticated("invalid Kernel credential"));
        }
        Ok(request)
    };
    let service = KernelService(runtime.clone());
    let (reporter, health) = tonic_health::server::health_reporter();
    reporter
        .set_serving::<AuthorityServiceServer<KernelService>>()
        .await;
    let server = tonic::transport::Server::builder()
        .add_service(AuthorityServiceServer::with_interceptor(
            service.clone(),
            auth.clone(),
        ))
        .add_service(RuntimeStoreServiceServer::with_interceptor(
            service.clone(),
            auth.clone(),
        ))
        .add_service(ModelMaterialServiceServer::with_interceptor(
            service.clone(),
            auth.clone(),
        ))
        .add_service(ArtifactStreamServiceServer::with_interceptor(
            service,
            auth.clone(),
        ))
        .add_service(tonic::service::interceptor::InterceptedService::new(
            health, auth,
        ))
        .serve_with_incoming(TcpListenerStream::new(listener));
    println!(
        "{}",
        serde_json::json!({"endpoint":format!("http://{endpoint}")})
    );
    let result = tokio::select! {
        result=server=>result.map_err(|e|Error::Infrastructure(e.to_string())),
        _=async {let mut byte=[0u8;1];loop{match input.read(&mut byte).await{Ok(0)|Err(_)=>break,Ok(_)=>{}}}}=>Ok(()),
        _=tokio::signal::ctrl_c()=>Ok(()),
    };
    runtime.store.close().await;
    bootstrap::stop_managed(managed).await?;
    result
}

use postgresql_embedded::{PostgreSQL, SettingsBuilder, VersionReq};
use tempfile::TempDir;

pub async fn database() -> (PostgreSQL, String, TempDir) {
    let root = tempfile::tempdir().expect("temporary PostgreSQL root");
    let settings = SettingsBuilder::new()
        .version(VersionReq::parse("=18.6.0").expect("PostgreSQL 18.6 version requirement"))
        .host("127.0.0.1")
        .port(0)
        .username("nous_wave")
        .password("nous_wave")
        .installation_dir(root.path().join("installation"))
        .data_dir(root.path().join("postgres"))
        .temporary(false)
        .build();
    let mut postgres = PostgreSQL::new(settings);
    postgres.setup().await.expect("embedded PostgreSQL setup");
    postgres.start().await.expect("embedded PostgreSQL start");
    postgres
        .create_database("nous_wave")
        .await
        .expect("embedded PostgreSQL database");
    let url = postgres.settings().url("nous_wave");
    (postgres, url, root)
}

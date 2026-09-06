use nous_core::{Error, Result};
use opendal::{Operator, services::Fs};

#[derive(Clone)]
pub struct ObjectStore {
    operator: Operator,
    root: std::path::PathBuf,
}

impl ObjectStore {
    /// Shared guards cover byte writes through canonical reference commit. Purge
    /// takes an exclusive guard so it cannot race a shared-hash registration.
    pub async fn reference_guard(&self, exclusive: bool) -> Result<std::fs::File> {
        let path = self.root.join(".reference-lock");
        tokio::task::spawn_blocking(move || {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
            if exclusive {
                file.lock()
            } else {
                file.lock_shared()
            }
            .map_err(|e| Error::Infrastructure(e.to_string()))?;
            Ok(file)
        })
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))?
    }

    pub async fn hashes(&self) -> Result<impl futures::Stream<Item = Result<String>> + use<>> {
        use futures::StreamExt;
        let lister = self
            .operator
            .lister_with("")
            .recursive(true)
            .await
            .map_err(storage_error)?;
        Ok(lister.filter_map(|entry| async move {
            match entry {
                Ok(entry) => {
                    let hash = entry.path().rsplit('/').next().unwrap_or("");
                    if Self::key(hash).is_ok() {
                        Some(Ok(hash.to_string()))
                    } else {
                        None
                    }
                }
                Err(error) => Some(Err(storage_error(error))),
            }
        }))
    }

    pub async fn open(root: &str) -> Result<Self> {
        tokio::fs::create_dir_all(root)
            .await
            .map_err(|e| Error::Infrastructure(e.to_string()))?;
        let root = tokio::fs::canonicalize(root)
            .await
            .map_err(|e| Error::Infrastructure(e.to_string()))?;
        let operator =
            Operator::new(Fs::default().root(&root.to_string_lossy())).map_err(storage_error)?;
        Ok(Self { operator, root })
    }

    fn key(hash: &str) -> Result<String> {
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(Error::Invalid("invalid BLAKE3 digest".into()));
        }
        Ok(format!("{}/{}", &hash[..2], hash))
    }

    pub async fn put(&self, bytes: Vec<u8>) -> Result<String> {
        let hash = blake3::hash(&bytes).to_hex().to_string();
        self.operator
            .write(&Self::key(&hash)?, bytes)
            .await
            .map_err(storage_error)?;
        Ok(hash)
    }

    pub async fn get(&self, hash: &str) -> Result<Vec<u8>> {
        let bytes = self
            .operator
            .read(&Self::key(hash)?)
            .await
            .map_err(storage_error)?
            .to_vec();
        if blake3::hash(&bytes).to_hex().as_str() != hash {
            return Err(Error::Infrastructure(
                "artifact content hash mismatch".into(),
            ));
        }
        Ok(bytes)
    }

    /// The canonical owner must first establish that no retained reference exists.
    pub async fn delete_unreferenced(&self, hash: &str) -> Result<()> {
        self.operator
            .delete(&Self::key(hash)?)
            .await
            .map_err(storage_error)
    }

    pub async fn check(&self) -> Result<()> {
        self.operator.check().await.map_err(storage_error)
    }
}

fn storage_error(error: opendal::Error) -> Error {
    Error::Infrastructure(error.to_string())
}

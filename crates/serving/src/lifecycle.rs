use crate::{
    artifacts::*,
    build::{DenseManifest, implementation},
    *,
};
use std::path::Path;

pub(crate) enum OpenArtifact {
    Lexical(Arc<LexicalGeneration>),
    Dense(Arc<DenseGeneration>, Option<Arc<EpaBasisGeneration>>),
    Topology(Arc<WaveGraphGeneration>),
    Exact(Arc<ExactPostings>),
}

impl ServingService {
    pub(crate) fn config_digest(&self, family: &str) -> Result<String> {
        let mut config =
            serde_json::json!({"schema":1,"memory_enabled":self.options.memory_enabled});
        if family == "topology" {
            config["propagation"] = serde_json::to_value(WaveConfig::default())
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
        }
        if family == "dense"
            && let Some(provider) = &self.embedding
        {
            config["space"] = serde_json::to_value(provider.space())
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
            config["producer"] = serde_json::to_value(provider.producer())
                .map_err(|e| Error::Infrastructure(e.to_string()))?;
        }
        digest(&config)
    }

    pub async fn refresh(&self, subject: SubjectId) -> Result<ProjectionStatus> {
        self.store.require_subject(subject).await?;
        let current = self.store.serving_current(subject).await?;
        let mut requested = vec![("exact", String::new())];
        if self.options.lexical {
            requested.push(("lexical", String::new()));
        }
        if self.options.topology {
            requested.push(("topology", String::new()));
        }
        if self.options.dense {
            if let Some(provider) = &self.embedding {
                requested.push(("dense", provider.space().space_hash));
            } else {
                requested.extend(
                    current
                        .iter()
                        .filter(|record| record.family == "dense")
                        .map(|record| ("dense", record.space.clone())),
                );
            }
        }
        let mut result = ProjectionStatus::default();
        for (family, space) in requested {
            let key = if space.is_empty() {
                family.to_string()
            } else {
                format!("{family}:{space}")
            };
            let existing = current
                .iter()
                .find(|record| record.family == family && record.space == space);
            let watermark = self
                .store
                .projection_watermark(subject, family, &space)
                .await?;
            let compatible = existing.filter(|record| self.compatible(record));
            if let Some(record) =
                compatible.filter(|record| record.watermark >= watermark && self.loaded(record))
            {
                result.generations.insert(key, record.generation_id);
                continue;
            }
            let outcome = if let Some(record) =
                compatible.filter(|record| record.watermark >= watermark)
            {
                match self.open_record(record) {
                    Ok(artifact) => {
                        self.publish_snapshot(record, artifact);
                        result.reopened.push(key.clone());
                        Ok(record.clone())
                    }
                    Err(_) => self
                        .build_and_open(subject, family, &space)
                        .await
                        .inspect(|_| {
                            result.rebuilt.push(key.clone());
                        }),
                }
            } else {
                self.build_and_open(subject, family, &space)
                    .await
                    .inspect(|_| {
                        result.rebuilt.push(key.clone());
                    })
            };
            match outcome {
                Ok(record) => {
                    result.generations.insert(key, record.generation_id);
                }
                Err(error) => {
                    if let Some(record) = compatible
                        && let Ok(artifact) = self.open_record(record)
                    {
                        self.publish_snapshot(record, artifact);
                        result.generations.insert(key.clone(), record.generation_id);
                    }
                    result.degradation.push(Degradation {
                        code: format!("{family}_projection_unavailable"),
                        detail: Some(error.to_string()),
                    });
                }
            }
        }
        Ok(result)
    }

    async fn build_and_open(
        &self,
        subject: SubjectId,
        family: &str,
        space: &str,
    ) -> Result<ServingRecord> {
        let record = self.build_family(subject, family, space).await?;
        let artifact = self.open_record(&record)?;
        self.publish_snapshot(&record, artifact);
        Ok(record)
    }

    fn compatible(&self, record: &ServingRecord) -> bool {
        let identity = record
            .metadata
            .get("implementation")
            .and_then(|v| v.as_str())
            == Some(implementation(&record.family))
            && record
                .metadata
                .get("implementation_revision")
                .and_then(|v| v.as_u64())
                == Some(1);
        if record.family == "dense" && self.embedding.is_none() {
            return identity;
        }
        identity
            && self.config_digest(&record.family).ok().as_deref()
                == record
                    .metadata
                    .get("config_digest")
                    .and_then(|v| v.as_str())
    }

    fn loaded(&self, record: &ServingRecord) -> bool {
        let snapshot = self.publisher.snapshot_for(record.subject);
        match record.family.as_str() {
            "lexical" => snapshot
                .lexical
                .as_ref()
                .is_some_and(|index| index.generation_id == record.generation_id),
            "topology" => snapshot
                .wave
                .as_ref()
                .is_some_and(|graph| graph.generation_id == record.generation_id),
            "dense" => snapshot
                .dense
                .iter()
                .any(|index| index.generation_id == record.generation_id),
            "exact" => snapshot.postings_generation == Some(record.generation_id),
            _ => false,
        }
    }

    fn open_record(&self, record: &ServingRecord) -> Result<OpenArtifact> {
        let path = Path::new(&record.artifact_location);
        let sums = checksums(path)?;
        if digest(&sums)? != record.artifact_hash {
            return Err(Error::Infrastructure(
                "serving generation checksum mismatch".into(),
            ));
        }
        self.readback(path, &record.family, record.generation_id)
    }

    pub(crate) fn readback(
        &self,
        path: &Path,
        family: &str,
        id: ServingGenerationId,
    ) -> Result<OpenArtifact> {
        match family {
            "lexical" => {
                let mut index = LexicalGeneration::open(path)?;
                index.generation_id = id;
                Ok(OpenArtifact::Lexical(Arc::new(index)))
            }
            "exact" => Ok(OpenArtifact::Exact(Arc::new(read_json(
                &path.join("postings.json"),
            )?))),
            "topology" => {
                let artifact: TopologyArtifact = read_json(&path.join("topology.json"))?;
                if artifact.generation_id != id {
                    return Err(Error::Infrastructure(
                        "topology generation identity mismatch".into(),
                    ));
                }
                Ok(OpenArtifact::Topology(Arc::new(
                    WaveGraphGeneration::from_artifact(artifact)?,
                )))
            }
            "dense" => {
                let manifest: DenseManifest = read_json(&path.join("records.json"))?;
                if let Some(basis) = &manifest.basis
                    && (basis.generation_id != id
                        || basis.basis.embedding_space != manifest.space.space_hash)
                {
                    return Err(Error::Infrastructure(
                        "cue basis generation/space mismatch".into(),
                    ));
                }
                let dense = DenseGeneration::open(
                    &path.join("vectors.usearch"),
                    id,
                    manifest.space,
                    manifest.records,
                )?;
                Ok(OpenArtifact::Dense(
                    Arc::new(dense),
                    manifest.basis.map(Arc::new),
                ))
            }
            _ => Err(Error::Invalid("unknown serving family".into())),
        }
    }

    fn publish_snapshot(&self, record: &ServingRecord, artifact: OpenArtifact) {
        self.publisher
            .update_for(record.subject, |snapshot| match &artifact {
                OpenArtifact::Lexical(index) => snapshot.lexical = Some(index.clone()),
                OpenArtifact::Exact(postings) => {
                    snapshot.postings = postings.clone();
                    snapshot.postings_generation = Some(record.generation_id);
                }
                OpenArtifact::Topology(graph) => snapshot.wave = Some(graph.clone()),
                OpenArtifact::Dense(index, basis) => {
                    snapshot
                        .dense
                        .retain(|existing| existing.space.space_hash != index.space.space_hash);
                    snapshot.dense.push(index.clone());
                    snapshot.epa.retain(|existing| {
                        existing.basis.embedding_space != index.space.space_hash
                    });
                    if let Some(basis) = basis {
                        snapshot.epa.push(basis.clone());
                    }
                }
            });
    }
}

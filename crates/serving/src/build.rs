use crate::{artifacts::*, *};
use nous_authority_store::TextProjectionSource;

#[derive(Serialize, Deserialize)]
pub(crate) struct DenseManifest {
    pub space: EmbeddingSpaceSignature,
    pub records: Vec<VectorRecord>,
    pub basis: Option<EpaBasisGeneration>,
}

impl ServingService {
    pub(crate) async fn build_family(
        &self,
        subject: SubjectId,
        family: &str,
        space: &str,
    ) -> Result<ServingRecord> {
        let staging = tempfile::Builder::new()
            .prefix(".staging-")
            .tempdir_in(&self.options.root)
            .map_err(io)?;
        let id = ServingGenerationId::new();
        let watermark = match family {
            "topology" => self.build_topology(subject, id, staging.path()).await?,
            "dense" => self.build_dense(subject, id, space, staging.path()).await?,
            "lexical" | "exact" => self.build_text(subject, family, staging.path()).await?,
            _ => return Err(Error::Invalid("unknown serving family".into())),
        };
        // Reopen the staged native artifact before publishing any durable pointer.
        self.readback(staging.path(), family, id)?;
        let sums = checksums(staging.path())?;
        let hash = digest(&sums)?;
        let target = self.options.root.join(id.0.to_string());
        std::fs::rename(staging.path(), &target).map_err(io)?;
        let record = ServingRecord {
            generation_id: id,
            subject,
            family: family.into(),
            space: space.into(),
            watermark,
            artifact_location: target.to_string_lossy().into_owned(),
            artifact_hash: hash,
            built_at: chrono::Utc::now(),
            metadata: serde_json::json!({ "implementation": implementation(family), "implementation_revision": 1, "config_digest": self.config_digest(family)?, "checksums": sums }),
        };
        self.store.publish_generation(record).await
    }

    async fn build_text(
        &self,
        subject: SubjectId,
        family: &str,
        dir: &std::path::Path,
    ) -> Result<i64> {
        let input = self
            .store
            .text_projection_input(subject, family, "", self.options.memory_enabled)
            .await?;
        if family == "exact" {
            let mut postings = ExactPostings::default();
            for (index, source) in input.sources.iter().enumerate() {
                let id = u32::try_from(index)
                    .map_err(|_| Error::Invalid("exact projection exceeds address space".into()))?;
                postings.insert_reference(
                    source.reference.to_string(),
                    id,
                    source.reference.clone(),
                );
                if let Some(revision) = source.revision {
                    postings.insert_reference(
                        CognitiveRef::MemoryRevision(revision).to_string(),
                        id,
                        source.reference.clone(),
                    );
                }
                for entity in &source.entity_refs {
                    postings.insert_reference(entity.clone(), id, source.reference.clone());
                }
                for tag in &source.tag_ids {
                    postings.insert_reference(format!("tag:{tag}"), id, source.reference.clone());
                }
                for anchor in &source.anchor_ids {
                    postings.insert_reference(
                        format!("anchor:{anchor}"),
                        id,
                        source.reference.clone(),
                    );
                }
            }
            write_json(&dir.join("postings.json"), &postings)?;
        } else {
            let documents = self.documents(input.sources).await?;
            let directory = dir.to_path_buf();
            tokio::task::spawn_blocking(move || {
                let mut lexical = LexicalGeneration::create(directory)?;
                lexical.add_documents(&documents)
            })
            .await
            .map_err(|e| Error::Infrastructure(e.to_string()))??;
        }
        Ok(input.watermark)
    }

    async fn documents(&self, sources: Vec<TextProjectionSource>) -> Result<Vec<LexicalDocument>> {
        let mut documents = Vec::new();
        for source in sources {
            let text = if let Some(text) = source.text {
                text
            } else if is_text(&source.media_type)
                && let Some(hash) = &source.content_hash
            {
                String::from_utf8(self.objects.get(hash).await?)
                    .map_err(|e| Error::Invalid(format!("text source encoding: {e}")))?
            } else {
                continue;
            };
            documents.push(LexicalDocument {
                serving_doc_id: documents.len() as u64,
                reference: source.reference,
                representation_text: text,
                title: source.title,
                entity_refs: source.entity_refs,
                tag_ids: source.tag_ids,
                anchor_ids: source.anchor_ids,
                source_class: source.source_class,
            });
        }
        Ok(documents)
    }

    async fn build_topology(
        &self,
        subject: SubjectId,
        id: ServingGenerationId,
        dir: &std::path::Path,
    ) -> Result<i64> {
        let input = self
            .store
            .topology_projection_input(subject, self.options.memory_enabled)
            .await?;
        let edges: Vec<_> = input
            .edges
            .into_iter()
            .map(|edge| WaveEdgeEvidence {
                from: edge.from,
                to: edge.to,
                support_class: edge.support_class,
                association_kind: edge.association_kind,
                polarity: edge.polarity,
                support_value: edge.support_value,
                bridge_hint: edge.bridge_hint,
            })
            .collect();
        let nodes = input
            .nodes
            .into_iter()
            .enumerate()
            .map(|(index, reference)| {
                let node_kind = match reference {
                    CognitiveRef::Tag(_) => WaveNodeKind::Tag,
                    CognitiveRef::Anchor(_) => WaveNodeKind::Anchor,
                    CognitiveRef::Entity(_) => WaveNodeKind::Entity,
                    CognitiveRef::Resource(_) => WaveNodeKind::Resource,
                    _ => WaveNodeKind::Memory,
                };
                WaveNode {
                    serving_id: index as u32,
                    reference,
                    node_kind,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                }
            })
            .collect();
        let path = dir.join("topology.json");
        tokio::task::spawn_blocking(move || {
            let mut graph = WaveGraphGeneration::build(nodes, &edges, WaveConfig::default())?;
            graph.generation_id = id;
            write_json(&path, &graph.artifact())
        })
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))??;
        Ok(input.watermark)
    }

    async fn build_dense(
        &self,
        subject: SubjectId,
        id: ServingGenerationId,
        space_key: &str,
        dir: &std::path::Path,
    ) -> Result<i64> {
        let provider = self
            .embedding
            .as_ref()
            .ok_or_else(|| Error::Unavailable("text embedding provider is unavailable".into()))?;
        let space = provider.space();
        if space_key != space.space_hash {
            return Err(Error::Unavailable(
                "embedding space has no configured provider".into(),
            ));
        }
        let input = self
            .store
            .text_projection_input(subject, "dense", space_key, self.options.memory_enabled)
            .await?;
        let source_regions: std::collections::HashMap<_, _> = input
            .sources
            .iter()
            .map(|source| (source.reference.clone(), source.source_region))
            .collect();
        let documents = self.documents(input.sources).await?;
        let mut vectors = Vec::new();
        for document in documents {
            let output = provider
                .embed(TextEmbeddingRequest {
                    subject,
                    text: document.representation_text,
                    query: false,
                })
                .await?;
            if !output.space.compatible_with(&space)
                || output.producer.signature_hash != provider.producer().signature_hash
            {
                return Err(Error::Conflict(
                    "embedding output disagrees with configured space/producer".into(),
                ));
            }
            let source_region = source_regions
                .get(&document.reference)
                .copied()
                .flatten()
                .map(CognitiveRef::SourceRegion);
            let record = VectorRecord {
                serving_doc_id: document.serving_doc_id,
                representation_kind: if matches!(document.reference, CognitiveRef::Tag(_)) {
                    "tag"
                } else {
                    "text"
                }
                .into(),
                reference: document.reference,
                embedding_space: space.clone(),
                producer_signature: output.producer.signature_hash,
                source_region,
            };
            vectors.push((record, output.vector));
        }
        let directory = dir.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let mut generation = DenseGeneration::new(space.clone(), vectors.len())?;
            generation.generation_id = id;
            let mut tag_vectors = Vec::new();
            let mut records = Vec::new();
            for (record, vector) in vectors {
                if matches!(record.reference, CognitiveRef::Tag(_)) {
                    tag_vectors.push((
                        record.serving_doc_id,
                        vector.iter().map(|v| f64::from(*v)).collect(),
                    ));
                }
                generation.insert(record.clone(), &vector)?;
                records.push(record);
            }
            generation.save(&directory.join("vectors.usearch"))?;
            let basis =
                build_epa_basis(&tag_vectors, &space.space_hash).map(|basis| EpaBasisGeneration {
                    generation_id: id,
                    basis,
                });
            write_json(
                &directory.join("records.json"),
                &DenseManifest {
                    space,
                    records,
                    basis,
                },
            )
        })
        .await
        .map_err(|e| Error::Infrastructure(e.to_string()))??;
        Ok(input.watermark)
    }
}

pub(crate) fn implementation(family: &str) -> &'static str {
    match family {
        "lexical" => "tantivy",
        "dense" => "usearch",
        "topology" => "petgraph-csr-bounded-wave",
        "exact" => "roaring-postings",
        _ => "unknown",
    }
}

fn is_text(media: &str) -> bool {
    media.starts_with("text/") || media.contains("json") || media.contains("xml")
}

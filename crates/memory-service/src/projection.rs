use super::support::*;
use super::*;

impl LocalRuntime {
    pub async fn rebuild_projection(&self, subject: SubjectId) -> Result<ProjectionStatus> {
        self.require_subject(subject).await?;
        let memory_rows = sqlx::query("SELECT o.memory_id,o.current_revision_id,r.representation_text,r.title FROM memory_objects o JOIN memory_revisions r ON r.memory_revision_id=o.current_revision_id WHERE o.subject_id=$1 AND o.status='active' ORDER BY o.memory_id")
            .bind(subject.0)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)?;
        let mut lexical_documents = Vec::new();
        let mut postings = ExactPostings::default();
        let mut memory_ids = Vec::new();
        for (serving_id, row) in memory_rows.into_iter().enumerate() {
            let memory_id = MemoryId(row.try_get("memory_id").map_err(db)?);
            let revision_id = MemoryRevisionId(row.try_get("current_revision_id").map_err(db)?);
            let reference = CognitiveRef::Memory(memory_id);
            let entity_refs = self.revision_entities(revision_id.0).await?;
            let tag_ids = self.revision_tags(revision_id.0).await?;
            let anchor_ids = self.revision_anchors(revision_id.0).await?;
            let source_class = self
                .memory_source_classes(revision_id.0)
                .await?
                .into_iter()
                .next();
            let document = LexicalDocument {
                serving_doc_id: serving_id as u64,
                reference: reference.clone(),
                representation_text: row.try_get("representation_text").map_err(db)?,
                title: row.try_get("title").map_err(db)?,
                entity_refs: entity_refs.clone(),
                tag_ids: tag_ids.iter().map(|id| id.0.to_string()).collect(),
                anchor_ids: anchor_ids.iter().map(|id| id.0.to_string()).collect(),
                source_class,
            };
            postings.insert_reference(reference.to_string(), serving_id as u32, reference);
            postings.insert_reference(
                CognitiveRef::MemoryRevision(revision_id).to_string(),
                serving_id as u32,
                CognitiveRef::MemoryRevision(revision_id),
            );
            for entity in entity_refs {
                let entity = EntityRef::new(entity)?;
                postings.insert_reference(
                    CognitiveRef::Entity(entity.clone()).to_string(),
                    serving_id as u32,
                    CognitiveRef::Entity(entity),
                );
            }
            for tag in tag_ids {
                postings.insert_reference(
                    CognitiveRef::Tag(tag).to_string(),
                    serving_id as u32,
                    CognitiveRef::Tag(tag),
                );
            }
            for anchor in anchor_ids {
                postings.insert_reference(
                    CognitiveRef::Anchor(anchor).to_string(),
                    serving_id as u32,
                    CognitiveRef::Anchor(anchor),
                );
            }
            memory_ids.push((memory_id, revision_id));
            lexical_documents.push(document);
        }
        let mut next_serving_id = lexical_documents.len() as u32;
        let derived_rows = sqlx::query("SELECT derived_representation_id,representation_kind,payload_text,source_region_id FROM derived_representations WHERE subject_id=$1 AND payload_text IS NOT NULL ORDER BY derived_representation_id")
            .bind(subject.0)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)?;
        for row in derived_rows {
            let reference = CognitiveRef::DerivedRepresentation(DerivedRepresentationId(
                row.try_get("derived_representation_id").map_err(db)?,
            ));
            let document = LexicalDocument {
                serving_doc_id: next_serving_id as u64,
                reference: reference.clone(),
                representation_text: row.try_get("payload_text").map_err(db)?,
                title: Some(row.try_get("representation_kind").map_err(db)?),
                entity_refs: Vec::new(),
                tag_ids: Vec::new(),
                anchor_ids: Vec::new(),
                source_class: Some("derived".into()),
            };
            postings.insert_reference(reference.to_string(), next_serving_id, reference);
            lexical_documents.push(document);
            next_serving_id = next_serving_id.saturating_add(1);
        }
        let observation_rows = sqlx::query("SELECT o.occurrence_id,o.source_class,a.content_hash,a.media_type FROM observation_occurrences o JOIN artifacts a ON a.artifact_id=o.artifact_id WHERE o.subject_id=$1 AND (a.media_type LIKE 'text/%' OR a.media_type LIKE '%json%' OR a.media_type LIKE '%xml%') ORDER BY o.observed_at,o.occurrence_id")
            .bind(subject.0)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)?;
        for row in observation_rows {
            let hash: String = row.try_get("content_hash").map_err(db)?;
            let Ok(bytes) = self.objects.get(&hash).await else {
                continue;
            };
            let Ok(mut text) = String::from_utf8(bytes) else {
                continue;
            };
            if text.len() > 1_000_000 {
                text.truncate(1_000_000);
            }
            if text.trim().is_empty() {
                continue;
            }
            let reference =
                CognitiveRef::Occurrence(OccurrenceId(row.try_get("occurrence_id").map_err(db)?));
            let document = LexicalDocument {
                serving_doc_id: next_serving_id as u64,
                reference: reference.clone(),
                representation_text: text,
                title: None,
                entity_refs: Vec::new(),
                tag_ids: Vec::new(),
                anchor_ids: Vec::new(),
                source_class: Some(row.try_get("source_class").map_err(db)?),
            };
            postings.insert_reference(reference.to_string(), next_serving_id, reference);
            lexical_documents.push(document);
            next_serving_id = next_serving_id.saturating_add(1);
        }
        let tag_labels = sqlx::query(
            "SELECT t.tag_id,tr.label FROM tags t JOIN tag_revisions tr ON tr.tag_revision_id=t.current_revision_id WHERE t.subject_id=$1 AND t.status='active' ORDER BY t.tag_id",
        )
        .bind(subject.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?
        .into_iter()
        .map(|row| {
            Ok::<_, Error>((
                TagId(row.try_get("tag_id").map_err(db)?),
                row.try_get::<String, _>("label").map_err(db)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
        let dense_capacity = lexical_documents.len().saturating_add(tag_labels.len());
        let mut dense_builders =
            BTreeMap::<String, (EmbeddingSpaceSignature, DenseGeneration)>::new();
        let mut next_embedding_doc_id = next_serving_id as u64;
        if let Some(provider) = &self.text_embedding_provider {
            for document in &lexical_documents {
                let output = provider
                    .embed(TextEmbeddingRequest {
                        subject,
                        text: document.representation_text.clone(),
                        query: false,
                    })
                    .await?;
                let space_key = output.space.space_hash.clone();
                if let Some((space, generation)) = dense_builders.get_mut(&space_key) {
                    if !space.compatible_with(&output.space) {
                        return Err(Error::Conflict(
                            "embedding provider returned incompatible spaces with one hash".into(),
                        ));
                    }
                    generation.insert(
                        VectorRecord {
                            serving_doc_id: document.serving_doc_id,
                            reference: document.reference.clone(),
                            embedding_space: space.clone(),
                            producer_signature: output.producer.signature_hash,
                            representation_kind: "text".into(),
                            source_region: None,
                        },
                        &output.vector,
                    )?;
                } else {
                    let space = output.space;
                    let mut generation = DenseGeneration::new(space.clone(), dense_capacity)?;
                    generation.insert(
                        VectorRecord {
                            serving_doc_id: document.serving_doc_id,
                            reference: document.reference.clone(),
                            embedding_space: space.clone(),
                            producer_signature: output.producer.signature_hash,
                            representation_kind: "text".into(),
                            source_region: None,
                        },
                        &output.vector,
                    )?;
                    dense_builders.insert(space_key, (space, generation));
                }
            }
            for (tag_id, label) in &tag_labels {
                let output = provider
                    .embed(TextEmbeddingRequest {
                        subject,
                        text: label.clone(),
                        query: false,
                    })
                    .await?;
                let space_key = output.space.space_hash.clone();
                let record = VectorRecord {
                    serving_doc_id: next_embedding_doc_id,
                    reference: CognitiveRef::Tag(*tag_id),
                    embedding_space: output.space.clone(),
                    producer_signature: output.producer.signature_hash,
                    representation_kind: "tag".into(),
                    source_region: None,
                };
                if let Some((space, generation)) = dense_builders.get_mut(&space_key) {
                    if !space.compatible_with(&output.space) {
                        return Err(Error::Conflict(
                            "embedding provider returned incompatible spaces with one hash".into(),
                        ));
                    }
                    generation.insert(record, &output.vector)?;
                } else {
                    let space = output.space;
                    let mut generation = DenseGeneration::new(space.clone(), dense_capacity)?;
                    generation.insert(record, &output.vector)?;
                    dense_builders.insert(space_key, (space, generation));
                }
                next_embedding_doc_id = next_embedding_doc_id.saturating_add(1);
            }
        } else {
            let persisted = sqlx::query("SELECT v.serving_doc_id,v.ref_kind,v.ref_value,v.embedding_space_hash,v.producer_signature_hash,v.representation_kind,v.source_region_id,v.vector,s.model_identity,s.weights_revision,s.task,s.input_representation,s.preprocessing_identity,s.preprocessing_revision,s.dimension,s.normalization,s.output_semantics FROM vector_records v JOIN serving_current c ON c.generation_id=v.serving_generation_id AND c.subject_id=$1 AND c.kind='dense' AND c.space_signature=v.embedding_space_hash JOIN embedding_spaces s ON s.space_hash=v.embedding_space_hash WHERE v.serving_generation_id=c.generation_id ORDER BY v.embedding_space_hash,v.serving_doc_id")
                .bind(subject.0)
                .fetch_all(self.store.pool())
                .await
                .map_err(db)?;
            for row in persisted {
                let space = EmbeddingSpaceSignature {
                    space_hash: row.try_get("embedding_space_hash").map_err(db)?,
                    model_identity: row.try_get("model_identity").map_err(db)?,
                    weights_revision: row.try_get("weights_revision").map_err(db)?,
                    task: row.try_get("task").map_err(db)?,
                    input_representation: row.try_get("input_representation").map_err(db)?,
                    preprocessing_identity: row.try_get("preprocessing_identity").map_err(db)?,
                    preprocessing_revision: row.try_get("preprocessing_revision").map_err(db)?,
                    dimension: row.try_get::<i32, _>("dimension").map_err(db)? as u32,
                    normalization: row.try_get("normalization").map_err(db)?,
                    output_semantics: row.try_get("output_semantics").map_err(db)?,
                };
                let key = space.space_hash.clone();
                let reference = parse_reference(
                    &row.try_get::<String, _>("ref_kind").map_err(db)?,
                    &row.try_get::<String, _>("ref_value").map_err(db)?,
                )?;
                if !lexical_documents
                    .iter()
                    .any(|document| document.reference == reference)
                    && !matches!(reference, CognitiveRef::Tag(_))
                {
                    continue;
                }
                let vector: Vec<f32> = serde_json::from_value(
                    row.try_get::<serde_json::Value, _>("vector").map_err(db)?,
                )
                .map_err(|error| Error::Infrastructure(error.to_string()))?;
                let record = VectorRecord {
                    serving_doc_id: row.try_get::<i64, _>("serving_doc_id").map_err(db)? as u64,
                    reference,
                    embedding_space: space.clone(),
                    producer_signature: row.try_get("producer_signature_hash").map_err(db)?,
                    representation_kind: row.try_get("representation_kind").map_err(db)?,
                    source_region: row
                        .try_get::<Option<Uuid>, _>("source_region_id")
                        .map_err(db)?
                        .map(|id| CognitiveRef::SourceRegion(SourceRegionId(id))),
                };
                if let Some((existing_space, generation)) = dense_builders.get_mut(&key) {
                    if !existing_space.compatible_with(&space) {
                        return Err(Error::Conflict(
                            "persisted vectors disagree about an embedding space".into(),
                        ));
                    }
                    generation.insert(record, &vector)?;
                } else {
                    let mut generation =
                        DenseGeneration::new(space.clone(), dense_capacity.max(1))?;
                    generation.insert(record, &vector)?;
                    dense_builders.insert(key, (space, generation));
                }
            }
        }
        let lexical = LexicalGeneration::in_memory()?;
        lexical.add_documents(&lexical_documents)?;
        let mut wave_refs = memory_ids
            .iter()
            .map(|(memory, _)| CognitiveRef::Memory(*memory))
            .collect::<HashSet<_>>();
        for tag in sqlx::query_scalar::<_, Uuid>(
            "SELECT tag_id FROM tags WHERE subject_id=$1 AND status='active'",
        )
        .bind(subject.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?
        {
            wave_refs.insert(CognitiveRef::Tag(TagId(tag)));
        }
        for anchor in sqlx::query_scalar::<_, Uuid>(
            "SELECT anchor_id FROM anchors WHERE subject_id=$1 AND status='active'",
        )
        .bind(subject.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?
        {
            wave_refs.insert(CognitiveRef::Anchor(AnchorId(anchor)));
        }
        for resource in sqlx::query_scalar::<_, String>(
            "SELECT resource_ref FROM resources WHERE subject_id=$1",
        )
        .bind(subject.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?
        {
            wave_refs.insert(CognitiveRef::Resource(ResourceRef::new(resource)?));
        }
        for entity in sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT b.entity_ref FROM entity_mentions m JOIN entity_binding_revisions b ON b.mention_id=m.mention_id WHERE m.subject_id=$1 AND b.revision_no=(SELECT max(b2.revision_no) FROM entity_binding_revisions b2 WHERE b2.mention_id=b.mention_id) AND b.binding_state='bound' AND b.entity_ref IS NOT NULL",
        )
        .bind(subject.0)
        .fetch_all(self.store.pool())
        .await
        .map_err(db)?
        {
            wave_refs.insert(CognitiveRef::Entity(EntityRef::new(entity)?));
        }

        let rows=sqlx::query("SELECT from_ref_kind,from_ref,to_ref_kind,to_ref,association_kind,polarity,support_class,support_value FROM association_evidence WHERE subject_id=$1 AND revoked_at IS NULL")
            .bind(subject.0).fetch_all(self.store.pool()).await.map_err(db)?;
        let mut edges = Vec::new();
        for row in rows {
            let from_kind: String = row.try_get("from_ref_kind").map_err(db)?;
            let from_value: String = row.try_get("from_ref").map_err(db)?;
            let to_kind: String = row.try_get("to_ref_kind").map_err(db)?;
            let to_value: String = row.try_get("to_ref").map_err(db)?;
            let from = parse_reference(&from_kind, &from_value)?;
            let to = parse_reference(&to_kind, &to_value)?;
            if !wave_reference_allowed(&from) || !wave_reference_allowed(&to) {
                continue;
            }
            wave_refs.insert(from.clone());
            wave_refs.insert(to.clone());
            edges.push(WaveEdgeEvidence {
                from,
                to,
                association_kind: row.try_get("association_kind").map_err(db)?,
                polarity: row.try_get("polarity").map_err(db)?,
                support_class: row.try_get("support_class").map_err(db)?,
                support_value: row.try_get("support_value").map_err(db)?,
                bridge_hint: false,
            });
        }
        for (memory, revision) in &memory_ids {
            for tag in self.revision_tags(revision.0).await? {
                let memory_ref = CognitiveRef::Memory(*memory);
                let tag_ref = CognitiveRef::Tag(tag);
                wave_refs.insert(tag_ref.clone());
                edges.push(topology_edge(
                    &memory_ref,
                    &tag_ref,
                    "memory_evidence",
                    "structural",
                ));
                edges.push(topology_edge(
                    &tag_ref,
                    &memory_ref,
                    "memory_evidence",
                    "structural",
                ));
            }
            for entity in self.revision_entities(revision.0).await? {
                let entity_ref = CognitiveRef::Entity(EntityRef::new(entity)?);
                let memory_ref = CognitiveRef::Memory(*memory);
                wave_refs.insert(entity_ref.clone());
                edges.push(topology_edge(
                    &memory_ref,
                    &entity_ref,
                    "memory_evidence",
                    "experiential",
                ));
                edges.push(topology_edge(
                    &entity_ref,
                    &memory_ref,
                    "memory_evidence",
                    "experiential",
                ));
            }
        }
        let anchor_rows = sqlx::query("SELECT a.anchor_id,s.support_ref_kind,s.support_ref FROM anchors a JOIN anchor_revisions ar ON ar.anchor_revision_id=a.current_revision_id JOIN anchor_support s ON s.anchor_revision_id=ar.anchor_revision_id WHERE a.subject_id=$1 AND a.status='active'")
            .bind(subject.0)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)?;
        for row in anchor_rows {
            let anchor = CognitiveRef::Anchor(AnchorId(row.try_get("anchor_id").map_err(db)?));
            let kind: String = row.try_get("support_ref_kind").map_err(db)?;
            let value: String = row.try_get("support_ref").map_err(db)?;
            let support = parse_reference(&kind, &value)?;
            if !wave_reference_allowed(&support) {
                continue;
            }
            wave_refs.insert(anchor.clone());
            wave_refs.insert(support.clone());
            edges.push(topology_edge(
                &anchor,
                &support,
                "derived_structure",
                "anchor",
            ));
            edges.push(topology_edge(
                &support,
                &anchor,
                "derived_structure",
                "anchor",
            ));
        }
        let relation_rows = sqlx::query("SELECT left_o.memory_id AS from_memory,right_o.memory_id AS to_memory,rr.relation FROM memory_revision_relations rr JOIN memory_revisions left_r ON left_r.memory_revision_id=rr.from_revision_id JOIN memory_revisions right_r ON right_r.memory_revision_id=rr.to_revision_id JOIN memory_objects left_o ON left_o.memory_id=left_r.memory_id JOIN memory_objects right_o ON right_o.memory_id=right_r.memory_id WHERE left_o.subject_id=$1 AND right_o.subject_id=$1 AND left_o.status='active' AND right_o.status='active'")
            .bind(subject.0)
            .fetch_all(self.store.pool())
            .await
            .map_err(db)?;
        for row in relation_rows {
            let from = CognitiveRef::Memory(MemoryId(row.try_get("from_memory").map_err(db)?));
            let to = CognitiveRef::Memory(MemoryId(row.try_get("to_memory").map_err(db)?));
            let kind: String = row.try_get("relation").map_err(db)?;
            edges.push(topology_edge(&from, &to, "derived_structure", &kind));
        }
        let mut nodes = wave_refs.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|reference| reference.to_string());
        let nodes = nodes
            .into_iter()
            .enumerate()
            .map(|(index, reference)| {
                let node_kind = wave_node_kind(&reference);
                WaveNode {
                    serving_id: index as u32,
                    reference,
                    node_kind,
                    embedding_key: None,
                    posting_key: None,
                    intrinsic_residual_gain: None,
                }
            })
            .collect::<Vec<_>>();
        let wave = WaveGraphGeneration::build(nodes, &edges, WaveConfig::default())?;
        let wave_nodes = wave.nodes.len();
        let wave_edges = (0..wave_nodes as u32)
            .map(|node| wave.outgoing(node).len())
            .sum();
        let lexical_hash = blake3::hash(
            serde_json::to_vec(&lexical_documents)
                .map_err(|error| Error::Infrastructure(error.to_string()))?
                .as_slice(),
        )
        .to_hex()
        .to_string();
        let wave_hash = blake3::hash(format!("{:?}:{:?}", wave.nodes, edges).as_bytes())
            .to_hex()
            .to_string();
        let mut posting_keys = postings.keys().cloned().collect::<Vec<_>>();
        posting_keys.sort();
        let postings_hash = blake3::hash(posting_keys.join("\n").as_bytes())
            .to_hex()
            .to_string();
        let postings_generation = ServingGenerationId::new();
        let dense = dense_builders
            .into_values()
            .map(|(_, generation)| Arc::new(generation))
            .collect::<Vec<_>>();
        let epa = dense
            .iter()
            .filter_map(|generation| {
                let mut vectors = generation
                    .records()
                    .filter_map(|record| {
                        if matches!(record.reference, CognitiveRef::Tag(_)) {
                            generation.vector(record.serving_doc_id).map(|vector| {
                                (
                                    record.serving_doc_id,
                                    vector.iter().map(|value| f64::from(*value)).collect(),
                                )
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                vectors.sort_by_key(|(serving_doc_id, _)| *serving_doc_id);
                build_epa_basis(&vectors, &generation.space.space_hash).map(|basis| {
                    Arc::new(EpaBasisGeneration {
                        generation_id: ServingGenerationId::new(),
                        basis,
                    })
                })
            })
            .collect::<Vec<_>>();
        let snapshot = ServingSnapshot {
            generation: self.publisher.next_generation(),
            lexical: Some(Arc::new(lexical)),
            dense,
            wave: Some(Arc::new(wave)),
            epa,
            postings: Arc::new(postings),
            postings_generation: Some(postings_generation),
        };
        let now = Utc::now();
        let lexical_id = snapshot
            .lexical
            .as_ref()
            .map(|generation| generation.generation_id);
        let wave_id = snapshot
            .wave
            .as_ref()
            .map(|generation| generation.generation_id);
        let mut tx = self.store.begin().await?;
        sqlx::query("UPDATE serving_generations SET state='retired' WHERE subject_id=$1 AND kind IN ('lexical','dense','wave','epa_basis','postings') AND state='ready'")
            .bind(subject.0)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        if let Some(generation_id) = lexical_id {
            sqlx::query("INSERT INTO serving_generations(generation_id,subject_id,kind,input_generation,artifact_location,artifact_hash,state,built_at,published_at) VALUES($1,$2,'lexical',$3,'memory://tantivy',$4,'ready',$5,$5)")
                .bind(generation_id.0).bind(subject.0).bind(snapshot.generation as i64).bind(&lexical_hash).bind(now).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("INSERT INTO serving_current(subject_id,kind,generation_id) VALUES($1,'lexical',$2) ON CONFLICT(subject_id,kind,space_signature) DO UPDATE SET generation_id=excluded.generation_id")
                .bind(subject.0).bind(generation_id.0).execute(&mut *tx).await.map_err(db)?;
        }
        if let Some(generation_id) = wave_id {
            sqlx::query("INSERT INTO serving_generations(generation_id,subject_id,kind,input_generation,artifact_location,artifact_hash,state,built_at,published_at) VALUES($1,$2,'wave',$3,'memory://csr',$4,'ready',$5,$5)")
                .bind(generation_id.0).bind(subject.0).bind(snapshot.generation as i64).bind(&wave_hash).bind(now).execute(&mut *tx).await.map_err(db)?;
            sqlx::query("INSERT INTO serving_current(subject_id,kind,generation_id) VALUES($1,'wave',$2) ON CONFLICT(subject_id,kind,space_signature) DO UPDATE SET generation_id=excluded.generation_id")
                .bind(subject.0).bind(generation_id.0).execute(&mut *tx).await.map_err(db)?;
        }
        for generation in &snapshot.dense {
            let generation_id = generation.generation_id;
            let artifact_hash = blake3::hash(generation.space.space_hash.as_bytes())
                .to_hex()
                .to_string();
            sqlx::query("INSERT INTO embedding_spaces(embedding_space_id,space_hash,model_identity,weights_revision,task,input_representation,preprocessing_identity,preprocessing_revision,dimension,normalization,output_semantics,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) ON CONFLICT(space_hash) DO NOTHING")
                .bind(Uuid::now_v7())
                .bind(&generation.space.space_hash)
                .bind(&generation.space.model_identity)
                .bind(&generation.space.weights_revision)
                .bind(&generation.space.task)
                .bind(&generation.space.input_representation)
                .bind(&generation.space.preprocessing_identity)
                .bind(&generation.space.preprocessing_revision)
                .bind(generation.space.dimension as i32)
                .bind(&generation.space.normalization)
                .bind(&generation.space.output_semantics)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("INSERT INTO serving_generations(generation_id,subject_id,kind,space_signature,input_generation,artifact_location,artifact_hash,state,built_at,published_at) VALUES($1,$2,'dense',$3,$4,'memory://usearch',$5,'ready',$6,$6)")
                .bind(generation_id.0)
                .bind(subject.0)
                .bind(&generation.space.space_hash)
                .bind(snapshot.generation as i64)
                .bind(artifact_hash)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            sqlx::query("INSERT INTO serving_current(subject_id,kind,space_signature,generation_id) VALUES($1,'dense',$2,$3) ON CONFLICT(subject_id,kind,space_signature) DO UPDATE SET generation_id=excluded.generation_id")
                .bind(subject.0)
                .bind(&generation.space.space_hash)
                .bind(generation_id.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            for record in generation.records() {
                let (ref_kind, ref_value) = reference_parts(&record.reference);
                let vector = serde_json::to_value(
                    generation.vector(record.serving_doc_id).unwrap_or_default(),
                )
                .map_err(|error| Error::Infrastructure(error.to_string()))?;
                sqlx::query("INSERT INTO vector_records(serving_generation_id,serving_doc_id,ref_kind,ref_value,embedding_space_hash,producer_signature_hash,representation_kind,source_region_id,vector) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                    .bind(generation_id.0)
                    .bind(record.serving_doc_id as i64)
                    .bind(ref_kind)
                    .bind(ref_value)
                    .bind(&record.embedding_space.space_hash)
                    .bind(&record.producer_signature)
                    .bind(&record.representation_kind)
                    .bind(record.source_region.as_ref().and_then(|reference| match reference {
                        CognitiveRef::SourceRegion(id) => Some(id.0),
                        _ => None,
                    }))
                    .bind(vector)
                    .execute(&mut *tx)
                .await
                .map_err(db)?;
            }
            for generation in &snapshot.epa {
                let generation_id = generation.generation_id;
                let metadata = serde_json::to_value(&generation.basis)
                    .map_err(|error| Error::Infrastructure(error.to_string()))?;
                let artifact_hash = blake3::hash(
                    serde_json::to_vec(&generation.basis)
                        .map_err(|error| Error::Infrastructure(error.to_string()))?
                        .as_slice(),
                )
                .to_hex()
                .to_string();
                sqlx::query("INSERT INTO serving_generations(generation_id,subject_id,kind,space_signature,input_generation,artifact_location,artifact_hash,state,built_at,published_at,metadata) VALUES($1,$2,'epa_basis',$3,$4,'memory://epa',$5,'ready',$6,$6,$7)")
                .bind(generation_id.0)
                .bind(subject.0)
                .bind(&generation.basis.embedding_space)
                .bind(snapshot.generation as i64)
                .bind(artifact_hash)
                .bind(now)
                .bind(metadata)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
                sqlx::query("INSERT INTO serving_current(subject_id,kind,space_signature,generation_id) VALUES($1,'epa_basis',$2,$3) ON CONFLICT(subject_id,kind,space_signature) DO UPDATE SET generation_id=excluded.generation_id")
                .bind(subject.0)
                .bind(&generation.basis.embedding_space)
                .bind(generation_id.0)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
            }
        }
        sqlx::query("INSERT INTO serving_generations(generation_id,subject_id,kind,input_generation,artifact_location,artifact_hash,state,built_at,published_at) VALUES($1,$2,'postings',$3,'memory://postings',$4,'ready',$5,$5)")
            .bind(postings_generation.0)
            .bind(subject.0)
            .bind(snapshot.generation as i64)
            .bind(&postings_hash)
            .bind(now)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        sqlx::query("INSERT INTO serving_current(subject_id,kind,generation_id) VALUES($1,'postings',$2) ON CONFLICT(subject_id,kind,space_signature) DO UPDATE SET generation_id=excluded.generation_id")
            .bind(subject.0)
            .bind(postings_generation.0)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        tx.commit().await.map_err(db)?;
        self.publisher.publish_for(subject, snapshot.clone());
        Ok(ProjectionStatus {
            generation: snapshot.generation,
            wave_nodes,
            wave_edges,
            lexical_ready: snapshot.lexical.is_some(),
            dense_ready: !snapshot.dense.is_empty(),
        })
    }
}

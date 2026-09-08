use nous_core::{CognitiveRef, Error, Result, ServingGenerationId};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::{
    Index, IndexReader, IndexWriter, TantivyDocument,
    collector::TopDocs,
    query::QueryParser,
    schema::{
        Field, IndexRecordOption, STORED, STRING, Schema, TEXT, TextFieldIndexing, TextOptions,
        Value,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalDocument {
    pub serving_doc_id: u64,
    pub reference: CognitiveRef,
    pub representation_text: String,
    pub title: Option<String>,
    pub entity_refs: Vec<String>,
    pub tag_ids: Vec<String>,
    pub anchor_ids: Vec<String>,
    pub source_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalMatch {
    pub serving_doc_id: u64,
    pub score: f32,
    pub reference: Option<CognitiveRef>,
}

pub struct LexicalGeneration {
    pub generation_id: ServingGenerationId,
    index: Index,
    reader: IndexReader,
    body: Field,
    title: Field,
    reference: Field,
    metadata: Field,
    entity_refs: Field,
    tag_ids: Field,
    anchor_ids: Field,
    source_class: Field,
}

impl LexicalGeneration {
    pub fn create(path: impl AsRef<Path>) -> Result<Self> {
        let (schema, ..) = schema();
        Index::create_in_dir(path.as_ref(), schema)
            .map_err(|error| Error::Infrastructure(format!("Tantivy create: {error}")))?;
        Self::open(path)
    }
    pub fn in_memory() -> Result<Self> {
        let (
            schema,
            body,
            title,
            reference,
            metadata,
            entity_refs,
            tag_ids,
            anchor_ids,
            source_class,
        ) = schema();
        let index = Index::create_in_ram(schema);
        let reader = index
            .reader()
            .map_err(|error| Error::Infrastructure(format!("Tantivy reader: {error}")))?;
        Ok(Self {
            generation_id: ServingGenerationId::new(),
            index,
            reader,
            body,
            title,
            reference,
            metadata,
            entity_refs,
            tag_ids,
            anchor_ids,
            source_class,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let index = Index::open_in_dir(path)
            .map_err(|error| Error::Infrastructure(format!("Tantivy open: {error}")))?;
        let schema = index.schema();
        let body = schema
            .get_field("representation_text")
            .map_err(|_| Error::Infrastructure("Tantivy body field missing".into()))?;
        let title = schema
            .get_field("title")
            .map_err(|_| Error::Infrastructure("Tantivy title field missing".into()))?;
        let reference = schema
            .get_field("reference")
            .map_err(|_| Error::Infrastructure("Tantivy reference field missing".into()))?;
        let metadata = schema
            .get_field("metadata")
            .map_err(|_| Error::Infrastructure("Tantivy metadata field missing".into()))?;
        let entity_refs = schema
            .get_field("entity_refs")
            .map_err(|_| Error::Infrastructure("Tantivy entity_refs field missing".into()))?;
        let tag_ids = schema
            .get_field("tag_ids")
            .map_err(|_| Error::Infrastructure("Tantivy tag_ids field missing".into()))?;
        let anchor_ids = schema
            .get_field("anchor_ids")
            .map_err(|_| Error::Infrastructure("Tantivy anchor_ids field missing".into()))?;
        let source_class = schema
            .get_field("source_class")
            .map_err(|_| Error::Infrastructure("Tantivy source_class field missing".into()))?;
        let reader = index
            .reader()
            .map_err(|error| Error::Infrastructure(format!("Tantivy reader: {error}")))?;
        Ok(Self {
            generation_id: ServingGenerationId::new(),
            index,
            reader,
            body,
            title,
            reference,
            metadata,
            entity_refs,
            tag_ids,
            anchor_ids,
            source_class,
        })
    }

    pub fn add_documents(&mut self, documents: &[LexicalDocument]) -> Result<()> {
        let mut writer: IndexWriter = self
            .index
            .writer(15_000_000)
            .map_err(|error| Error::Infrastructure(format!("Tantivy writer: {error}")))?;
        for document in documents {
            let metadata = serde_json::to_string(document)
                .map_err(|error| Error::Infrastructure(format!("lexical metadata: {error}")))?;
            let title = document.title.as_deref().unwrap_or("");
            let mut indexed = TantivyDocument::default();
            indexed.add_text(self.body, document.representation_text.clone());
            indexed.add_text(self.title, title);
            indexed.add_text(self.reference, document.serving_doc_id.to_string());
            indexed.add_text(self.metadata, metadata);
            for value in &document.entity_refs {
                indexed.add_text(self.entity_refs, value);
            }
            for value in &document.tag_ids {
                indexed.add_text(self.tag_ids, value);
            }
            for value in &document.anchor_ids {
                indexed.add_text(self.anchor_ids, value);
            }
            if let Some(value) = &document.source_class {
                indexed.add_text(self.source_class, value);
            }
            writer
                .add_document(indexed)
                .map_err(|error| Error::Infrastructure(format!("Tantivy add: {error}")))?;
        }
        writer
            .commit()
            .map_err(|error| Error::Infrastructure(format!("Tantivy commit: {error}")))?;
        writer
            .wait_merging_threads()
            .map_err(|error| Error::Infrastructure(format!("Tantivy finalize: {error}")))?;
        self.reader
            .reload()
            .map_err(|error| Error::Infrastructure(format!("Tantivy reload: {error}")))?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<LexicalMatch>> {
        if query.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let searcher = self.reader.searcher();
        let parser = QueryParser::for_index(&self.index, vec![self.body, self.title]);
        let parsed = parser
            .parse_query(query)
            .map_err(|error| Error::Invalid(format!("invalid lexical query: {error}")))?;
        let hits = searcher
            .search(&parsed, &TopDocs::with_limit(limit).order_by_score())
            .map_err(|error| Error::Infrastructure(format!("Tantivy search: {error}")))?;
        hits.into_iter()
            .map(|(score, address)| {
                let document: TantivyDocument = searcher
                    .doc(address)
                    .map_err(|error| Error::Infrastructure(format!("Tantivy document: {error}")))?;
                let serving_doc_id = document
                    .get_first(self.reference)
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| {
                        Error::Infrastructure("Tantivy document lacks serving id".into())
                    })?
                    .parse::<u64>()
                    .map_err(|_| Error::Infrastructure("invalid Tantivy serving id".into()))?;
                let reference = document
                    .get_first(self.metadata)
                    .and_then(|value| value.as_str())
                    .and_then(|value| serde_json::from_str::<LexicalDocument>(value).ok())
                    .map(|document| document.reference);
                Ok(LexicalMatch {
                    serving_doc_id,
                    score,
                    reference,
                })
            })
            .collect()
    }
}

fn schema() -> (
    Schema,
    Field,
    Field,
    Field,
    Field,
    Field,
    Field,
    Field,
    Field,
) {
    let mut builder = Schema::builder();
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("default")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();
    let body = builder.add_text_field("representation_text", text.clone());
    let title = builder.add_text_field("title", text);
    let reference = builder.add_text_field("reference", STRING | STORED);
    let metadata = builder.add_text_field("metadata", TEXT | STORED);
    let keyword = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("raw")
                .set_index_option(IndexRecordOption::Basic),
        )
        .set_stored();
    let entity_refs = builder.add_text_field("entity_refs", keyword.clone());
    let tag_ids = builder.add_text_field("tag_ids", keyword.clone());
    let anchor_ids = builder.add_text_field("anchor_ids", keyword.clone());
    let source_class = builder.add_text_field("source_class", keyword);
    (
        builder.build(),
        body,
        title,
        reference,
        metadata,
        entity_refs,
        tag_ids,
        anchor_ids,
        source_class,
    )
}

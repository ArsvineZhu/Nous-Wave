use chrono::{DateTime, Utc};
use nous_core::{Error, Result, SubjectId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OriginClass {
    Human,
    Tool,
    Host,
    ExternalSystem,
    SelfGenerated,
    Imported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EpistemicClass {
    Observed,
    Reported,
    Derived,
    Inferred,
    Narrative,
    Simulated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    pub origin: OriginClass,
    pub semantic: String,
    pub epistemic: EpistemicClass,
}

impl Classification {
    pub fn validate(&self) -> Result<()> {
        const CORE: &[&str] = &[
            "MESSAGE",
            "CONVERSATION",
            "DOCUMENT",
            "TOOL_OBSERVATION",
            "WEB_CONTENT",
            "CODE",
            "NOTE",
            "REPORT",
            "SYSTEM_OBSERVATION",
            "GENERIC_MATERIAL",
        ];
        if self.semantic.len() > 128
            || (!CORE.contains(&self.semantic.as_str())
                && !self
                    .semantic
                    .split_once(':')
                    .is_some_and(|(namespace, name)| !namespace.is_empty() && !name.is_empty()))
        {
            return Err(Error::Invalid(
                "semantic class must be a core class or namespace:name".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub approximate: bool,
}

impl TemporalRange {
    pub fn validate(&self) -> Result<()> {
        if matches!((self.start, self.end), (Some(start), Some(end)) if start > end) {
            return Err(Error::Invalid("time range starts after its end".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub source_id: Uuid,
    pub subject_id: SubjectId,
    pub source_kind: String,
    pub classification: Classification,
    pub scope: String,
    pub occurred: Option<TemporalRange>,
    pub observed_at: Option<DateTime<Utc>>,
    pub recorded_at: DateTime<Utc>,
    pub actor: Option<String>,
    pub invocation_id: Option<String>,
    pub parent_sources: Vec<Uuid>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: Uuid,
    pub subject_id: SubjectId,
    pub content_hash: String,
    pub media_type: String,
    pub byte_size: i64,
    pub classification: Classification,
    pub source_refs: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorProvenance {
    pub identity: String,
    pub revision: String,
    pub preprocessing: String,
    pub config_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Derivation {
    pub derivation_id: Uuid,
    pub subject_id: SubjectId,
    pub input_artifacts: Vec<Uuid>,
    pub output_artifacts: Vec<Uuid>,
    pub processor: ProcessorProvenance,
    pub created_at: DateTime<Utc>,
}

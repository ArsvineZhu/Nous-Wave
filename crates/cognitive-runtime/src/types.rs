use chrono::{DateTime, Utc};
use nous_core::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UseKind {
    Surfaced,
    Inspected,
    Selected,
    Referenced,
    ActedOn,
    Corroborated,
    Corrected,
    Pinned,
    Rejected,
}

impl UseKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Surfaced => "surfaced",
            Self::Inspected => "inspected",
            Self::Selected => "selected",
            Self::Referenced => "referenced",
            Self::ActedOn => "acted_on",
            Self::Corroborated => "corroborated",
            Self::Corrected => "corrected",
            Self::Pinned => "pinned",
            Self::Rejected => "rejected",
        }
    }

    pub fn meaningful(self) -> bool {
        matches!(
            self,
            Self::Referenced | Self::ActedOn | Self::Corroborated | Self::Pinned | Self::Corrected
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveUseEvent {
    pub use_event_id: Uuid,
    pub subject_id: SubjectId,
    pub session_id: Option<SessionId>,
    pub reference: CognitiveRef,
    pub use_kind: UseKind,
    pub consumer_ref: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveSession {
    pub session_id: SessionId,
    pub subject_id: SubjectId,
    pub opened_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub last_meaningful_use_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub state_revision: i64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidentRef {
    pub session_id: SessionId,
    pub reference: CognitiveRef,
    pub entered_at: DateTime<Utc>,
    pub entry_reason: String,
    pub last_meaningful_use_at: Option<DateTime<Utc>>,
    pub hold_until: Option<DateTime<Utc>>,
    pub state: ResidentState,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidentState {
    Resident,
    Provisional,
    Evicted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionView {
    pub session_id: SessionId,
    pub subject_id: SubjectId,
    pub opened_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub last_meaningful_use_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub state_revision: i64,
    pub resident: Vec<ResidentView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidentView {
    pub reference: CognitiveRef,
    pub entry_reason: String,
    pub state: String,
    pub entered_at: DateTime<Utc>,
    pub last_meaningful_use_at: Option<DateTime<Utc>>,
    pub hold_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseFeedback {
    #[serde(default)]
    pub subject: SubjectId,
    pub session_id: Option<SessionId>,
    pub consumer: Option<String>,
    pub events: Vec<UseFeedbackEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseFeedbackEvent {
    pub reference: CognitiveRef,
    pub use_kind: UseKind,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retrieval_exposure_is_not_meaningful_use() {
        assert!(!UseKind::Surfaced.meaningful());
        assert!(!UseKind::Inspected.meaningful());
        assert!(UseKind::Referenced.meaningful());
        assert!(UseKind::ActedOn.meaningful());
    }
}

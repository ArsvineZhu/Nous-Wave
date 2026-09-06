use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UseKind {
    Surfaced,
    Inspected,
    ExposedToContext,
    Followed,
    ReferencedOrActedOn,
    Abandoned,
    Suppressed,
    CycleResolved,
    CycleInsufficient,
}
impl UseKind {
    pub fn strengthens(self) -> bool {
        matches!(self, Self::Followed | Self::ReferencedOrActedOn)
    }
}

/// An operational baseline, not a claim about human forgetting. Uses and days
/// retain their units; a half-power decay penalizes long-unused ordinary cues.
pub fn accessibility(
    uses: u64,
    last_use: DateTime<Utc>,
    now: DateTime<Utc>,
    use_decay: f64,
    retention_hint: f64,
) -> f64 {
    let days = (now - last_use).num_seconds().max(0) as f64 / 86400.0;
    let strength = 1.0 + (uses as f64).ln_1p();
    (strength / (strength + days.powf(use_decay)))
        .max(retention_hint)
        .clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn surfaced_is_not_reinforced_and_time_changes_accessibility() {
        let now = Utc::now();
        let old = now - chrono::Duration::days(365);
        let mut uses = 0;
        for _ in 0..1000 {
            if UseKind::Surfaced.strengthens() {
                uses += 1;
            }
        }
        assert_eq!(uses, 0);
        assert!(!UseKind::ExposedToContext.strengthens());
        assert!(UseKind::Followed.strengthens());
        assert!(accessibility(0, old, now, 0.5, 0.0) < accessibility(0, now, now, 0.5, 0.0));
        assert!(accessibility(10, old, now, 0.5, 0.0) > accessibility(0, old, now, 0.5, 0.0));
    }
}

//! Stable model-facing aliases. Canonical object identity remains owner-owned.
use crate::*;
use nous_core::{CognitiveRef, parse_reference, reference_parts};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::LazyLock;

static WORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    include_str!("../data/lexical-words-v1.txt")
        .lines()
        .collect()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityBinding {
    pub lexical_ref: String,
    pub canonical: CognitiveRef,
    pub display_name: String,
    pub aliases: Vec<String>,
    pub status: String,
}

pub fn lexical_prefix(kind: &str) -> Result<&'static str> {
    Ok(match kind {
        "entity" => "ent",
        "memory" => "mem",
        "memory_revision" => "memrev",
        "tag" => "tag",
        "anchor" => "anchor",
        "artifact" => "art",
        "occurrence" => "obs",
        "source_region" => "src",
        "derived_representation" => "repr",
        "derived_region" => "region",
        "session" => "session",
        "resource" => "res",
        _ => return Err(Error::Invalid("object kind has no lexical address".into())),
    })
}
pub fn validate_lexical(value: &str) -> Result<&str> {
    let (kind, body) = value
        .split_once(':')
        .ok_or_else(|| Error::Invalid("INVALID_LEXICAL_REF".into()))?;
    if !matches!(
        kind,
        "ent"
            | "mem"
            | "memrev"
            | "tag"
            | "anchor"
            | "art"
            | "obs"
            | "src"
            | "repr"
            | "region"
            | "session"
            | "res"
    ) {
        return Err(Error::Invalid("INVALID_LEXICAL_REF".into()));
    }
    let words: Vec<_> = body.split('-').collect();
    if words.len() != 4 || words.iter().any(|w| WORDS.binary_search(w).is_err()) {
        return Err(Error::Invalid("INVALID_LEXICAL_REF".into()));
    }
    Ok(kind)
}
impl AuthorityStore {
    pub async fn bind_identity(
        &self,
        subject: SubjectId,
        reference: CognitiveRef,
        display_name: String,
        mut aliases: Vec<String>,
    ) -> Result<IdentityBinding> {
        self.require_subject(subject).await?;
        if display_name.len() > 256
            || aliases.len() > 32
            || aliases.iter().any(|a| a.is_empty() || a.len() > 256)
        {
            return Err(Error::Invalid("invalid identity label bounds".into()));
        }
        if !matches!(reference, CognitiveRef::Entity(_)) {
            self.validate_reference(subject, &reference).await?;
        }
        let (kind, canonical) = reference_parts(&reference);
        let prefix = lexical_prefix(&kind)?;
        aliases.sort();
        aliases.dedup();
        let mut tx = self.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,1))")
            .bind(format!("{kind}:{canonical}"))
            .execute(&mut *tx)
            .await
            .map_err(database_error)?;
        let row=sqlx::query("SELECT lexical_ref,tombstoned_at IS NOT NULL AS tombstoned FROM lexical_bindings WHERE object_kind=$1 AND canonical_ref=$2")
            .bind(&kind).bind(&canonical).fetch_optional(&mut *tx).await.map_err(database_error)?;
        let lexical = if let Some(row) = row {
            if row
                .try_get::<bool, _>("tombstoned")
                .map_err(database_error)?
            {
                return Err(Error::Conflict("REFERENCE_TOMBSTONED".into()));
            }
            row.try_get::<String, _>("lexical_ref")
                .map_err(database_error)?
        } else {
            let mut inserted = None;
            for _ in 0..32 {
                let mut bytes = [0; 8];
                getrandom::fill(&mut bytes).map_err(|e| Error::Infrastructure(e.to_string()))?;
                let bits = u64::from_le_bytes(bytes);
                let candidate = format!(
                    "{}:{}",
                    prefix,
                    (0..4)
                        .map(|index| WORDS[((bits >> (index * 12)) & 4095) as usize])
                        .collect::<Vec<_>>()
                        .join("-")
                );
                if sqlx::query("INSERT INTO lexical_bindings(lexical_ref,object_kind,canonical_ref,wordlist_version) VALUES($1,$2,$3,1) ON CONFLICT DO NOTHING")
                    .bind(&candidate).bind(&kind).bind(&canonical).execute(&mut *tx).await.map_err(database_error)?.rows_affected()==1 {inserted=Some(candidate);break;}
            }
            inserted.ok_or_else(|| {
                Error::Infrastructure("LexicalRef collision retry bound exceeded".into())
            })?
        };
        sqlx::query("INSERT INTO lexical_visibility(subject_id,lexical_ref,display_name,aliases) VALUES($1,$2,$3,$4) ON CONFLICT(subject_id,lexical_ref) DO UPDATE SET display_name=CASE WHEN excluded.display_name='' THEN lexical_visibility.display_name ELSE excluded.display_name END,aliases=CASE WHEN cardinality(excluded.aliases)=0 THEN lexical_visibility.aliases ELSE excluded.aliases END")
            .bind(subject.0).bind(&lexical).bind(display_name).bind(aliases).execute(&mut *tx).await.map_err(database_error)?;
        tx.commit().await.map_err(database_error)?;
        let (status, mut candidates) = self
            .resolve_identity(subject, &kind, &lexical, true)
            .await?;
        if status != "BOUND" {
            return Err(Error::NotFound(status));
        }
        candidates
            .pop()
            .ok_or_else(|| Error::Infrastructure("committed binding missing".into()))
    }
    pub async fn resolve_identity(
        &self,
        subject: SubjectId,
        kind: &str,
        locator: &str,
        lexical: bool,
    ) -> Result<(String, Vec<IdentityBinding>)> {
        self.require_subject(subject).await?;
        if lexical {
            let prefix = validate_lexical(locator)?;
            if !kind.is_empty() && lexical_prefix(kind)? != prefix {
                return Ok(("REFERENCE_TYPE_MISMATCH".into(), vec![]));
            }
        } else if locator.is_empty() || locator.len() > 256 {
            return Err(Error::Invalid("invalid name locator".into()));
        }
        let rows=sqlx::query("SELECT b.lexical_ref,b.object_kind,b.canonical_ref,b.tombstoned_at IS NOT NULL AS tombstoned,v.display_name,v.aliases FROM lexical_bindings b JOIN lexical_visibility v ON v.lexical_ref=b.lexical_ref WHERE v.subject_id=$1 AND ($2='' OR b.object_kind=$2) AND (($4 AND b.lexical_ref=$3) OR (NOT $4 AND (v.display_name=$3 OR $3=ANY(v.aliases)))) ORDER BY b.lexical_ref LIMIT 65")
            .bind(subject.0).bind(kind).bind(locator).bind(lexical).fetch_all(self.pool()).await.map_err(database_error)?;
        if rows.len() > 64 {
            return Err(Error::Invalid(
                "identity ambiguity exceeds 64 candidates".into(),
            ));
        }
        let mut bindings = Vec::new();
        let mut tombstoned = false;
        for row in rows {
            if row
                .try_get::<bool, _>("tombstoned")
                .map_err(database_error)?
            {
                tombstoned = true;
                continue;
            }
            let reference = parse_reference(
                &row.try_get::<String, _>("object_kind")
                    .map_err(database_error)?,
                &row.try_get::<String, _>("canonical_ref")
                    .map_err(database_error)?,
            )?;
            if !matches!(reference, CognitiveRef::Entity(_))
                && !self.reference_in_subject(subject, &reference).await?
            {
                tombstoned = true;
                continue;
            }
            bindings.push(IdentityBinding {
                lexical_ref: row.try_get("lexical_ref").map_err(database_error)?,
                canonical: reference,
                display_name: row.try_get("display_name").map_err(database_error)?,
                aliases: row.try_get("aliases").map_err(database_error)?,
                status: "LIVE".into(),
            });
        }
        let status = match bindings.len() {
            1 => "BOUND",
            2.. => "AMBIGUOUS_REFERENCE",
            _ if tombstoned => "REFERENCE_TOMBSTONED",
            _ => "UNKNOWN_REFERENCE",
        };
        Ok((status.into(), bindings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_vocabulary_and_reference_validation() {
        assert_eq!(WORDS.len(), 4096);
        assert!(WORDS.windows(2).all(|w| w[0] < w[1]));
        assert!(
            WORDS
                .iter()
                .all(|w| w.bytes().all(|c| c.is_ascii_lowercase()))
        );
        let value = format!("mem:{}-{}-{}-{}", WORDS[0], WORDS[1], WORDS[2], WORDS[3]);
        assert_eq!(validate_lexical(&value).unwrap(), "mem");
        assert!(validate_lexical(&value.to_uppercase()).is_err());
        assert!(validate_lexical("mem:unknown-words-not-valid").is_err());
    }
}

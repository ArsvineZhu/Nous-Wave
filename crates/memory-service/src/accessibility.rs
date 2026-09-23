use super::*;
use nous_authority_store::database_error as db;

#[derive(Debug,Clone,Copy,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessibilityPolicy {pub normal_days:u32,pub explicit_after_days:u32}
impl Default for AccessibilityPolicy {fn default()->Self{Self{normal_days:90,explicit_after_days:365}}}
impl AccessibilityPolicy {
    pub fn validate(self)->Result<Self>{
        if self.normal_days==0||self.explicit_after_days<=self.normal_days{return Err(Error::Invalid("accessibility age thresholds must satisfy 0 < normal_days < explicit_after_days".into()));}Ok(self)
    }
    pub fn level(self,mode:AccessibilityMode,anchor:DateTime<Utc>,now:DateTime<Utc>)->AccessibilityLevel{
        match mode {
            AccessibilityMode::Normal=>AccessibilityLevel::Normal,
            AccessibilityMode::Deep=>AccessibilityLevel::Deep,
            AccessibilityMode::Explicit=>AccessibilityLevel::Explicit,
            AccessibilityMode::Auto=>{
                let elapsed=(now-anchor).num_seconds().max(0);
                if elapsed<=i64::from(self.normal_days)*86400{AccessibilityLevel::Normal}
                else if elapsed<=i64::from(self.explicit_after_days)*86400{AccessibilityLevel::Deep}else{AccessibilityLevel::Explicit}
            }
        }
    }
}

pub fn eligible(level:AccessibilityLevel,effort:CognitiveEffort,exact:bool,direct_topology:bool)->bool{
    match level {AccessibilityLevel::Normal=>true,AccessibilityLevel::Deep=>exact||direct_topology||matches!(effort,CognitiveEffort::Deep|CognitiveEffort::Maximum),AccessibilityLevel::Explicit=>exact}
}
impl MemoryService {
    pub async fn accessibility_level(&self,subject:SubjectId,memory:MemoryId,now:DateTime<Utc>)->Result<AccessibilityLevel>{
        let row=sqlx::query("SELECT m.accessibility_mode,m.created_at,(SELECT max(e.occurred_at) FROM cognitive_use_events e WHERE e.subject_id=m.subject_id AND e.use_kind IN ('referenced','acted_on','corroborated','corrected','pinned') AND ((e.ref_kind='memory' AND e.ref_value=m.memory_id::text) OR (e.ref_kind='memory_revision' AND e.ref_value IN (SELECT memory_revision_id::text FROM memory_revisions WHERE memory_id=m.memory_id)))) AS last_use FROM memory_objects m WHERE m.subject_id=$1 AND m.memory_id=$2")
            .bind(subject.0).bind(memory.0).fetch_one(self.store.pool()).await.map_err(db)?;
        let created:DateTime<Utc>=row.try_get("created_at").map_err(db)?;
        let last:Option<DateTime<Utc>>=row.try_get("last_use").map_err(db)?;
        let mode=super::support::parse_accessibility_mode(&row.try_get::<String,_>("accessibility_mode").map_err(db)?)?;
        Ok(self.accessibility_policy.level(mode,last.map_or(created,|v|v.max(created)),now))
    }
    pub async fn set_accessibility(&self,subject:SubjectId,memory:MemoryId,expected:i64,mode:AccessibilityMode)->Result<MemoryView>{
        let mut tx=self.store.begin().await?;self.fence_memory_head(&mut tx,subject,memory,expected).await?;
        sqlx::query("UPDATE memory_objects SET accessibility_mode=$3 WHERE subject_id=$1 AND memory_id=$2")
            .bind(subject.0).bind(memory.0).bind(format!("{mode:?}").to_lowercase()).execute(&mut *tx).await.map_err(db)?;
        tx.commit().await.map_err(db)?;self.memory(subject,memory,None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accessibility_is_an_eligibility_rule_and_exact_is_not_suppression(){
        let now=Utc::now();let policy=AccessibilityPolicy::default();
        assert_eq!(policy.level(AccessibilityMode::Auto,now-chrono::Duration::days(20),now),AccessibilityLevel::Normal);
        let deep=policy.level(AccessibilityMode::Auto,now-chrono::Duration::days(100),now);
        assert!(!eligible(deep,CognitiveEffort::Normal,false,false));
        assert!(eligible(deep,CognitiveEffort::Deep,false,false));
        assert!(eligible(deep,CognitiveEffort::Light,false,true));
        let explicit=policy.level(AccessibilityMode::Auto,now-chrono::Duration::days(400),now);
        assert!(!eligible(explicit,CognitiveEffort::Maximum,false,true));
        assert!(eligible(explicit,CognitiveEffort::Light,true,false));
    }
}

use super::*;
use nous_core::*;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

impl KernelService {
    pub(super) async fn build_contribution_batch(
        &self,
        input: p::ProjectionRequest,
    ) -> Result<p::Projection> {
        use nous_cognitive_runtime::{
            ConsumerProfile, ContextBudget, MaterializationPolicy, WorkingSetRequest,
        };
        let subject = SubjectId(id(&input.subject_id)?);
        let session = SessionId(id(&input.session_id)?);
        if input.max_items == 0 || input.max_items > 2048 || input.max_text_bytes > 1_048_576 {
            return Err(Error::Invalid("invalid contribution budget".into()));
        }
        let mut refs = input
            .situation_refs
            .into_iter()
            .map(from_ref)
            .collect::<Result<Vec<_>>>()?;
        let mut degradation = Vec::new();
        if let Some(expression) = input.query {
            let query = self
                .query(p::QueryRequest {
                    subject_id: input.subject_id,
                    session_id: Some(input.session_id),
                    expression: Some(expression),
                    nousql: None,
                })
                .await?;
            for hit in query.hits {
                refs.push(from_ref(required(hit.reference, "hit reference")?)?);
            }
            degradation = query.degradation;
        }
        let snapshot = self.0.cognition.session(subject, session).await?;
        let batch = self
            .0
            .cognition
            .working_set(
                WorkingSetRequest {
                    subject,
                    session_id: session,
                    consumer: ConsumerProfile {
                        consumer_id: input.consumer_id.clone(),
                        context_budget: ContextBudget {
                            max_items: input.max_items as usize,
                            max_text_bytes: input.max_text_bytes as usize,
                        },
                        accepted_modalities: vec![
                            Modality::Text,
                            Modality::Structured,
                            Modality::Binary,
                            Modality::Image,
                            Modality::Audio,
                            Modality::Video,
                        ],
                        materialization_policy: MaterializationPolicy::AvailableText,
                    },
                    references: refs,
                    query_results: vec![],
                },
                &self.0,
            )
            .await?;
        degradation.extend(batch.degradation.into_iter().map(|d| p::Degradation {
            code: d.code,
            detail: d.detail.unwrap_or_default(),
        }));
        let segments = batch
            .contributions
            .into_iter()
            .map(|c| {
                let text = c.text.unwrap_or_default();
                let revision = c
                    .source_revision
                    .map(|r| r.0.to_string())
                    .unwrap_or_else(|| blake3::hash(text.as_bytes()).to_hex().to_string());
                p::ContextSegment {
                    segment_id: c.reference.to_string(),
                    text,
                    semantic_role: c.semantic_role,
                    source_refs: vec![to_ref(c.reference)],
                    evidence: c
                        .evidence
                        .into_iter()
                        .map(|e| p::Evidence {
                            reference: Some(to_ref(e.reference)),
                            support_role: e.support_role,
                        })
                        .collect(),
                    authority: enum_name(c.authority),
                    stability: "EPOCH_STABLE".into(),
                    source_revision: Some(revision),
                }
            })
            .collect();
        Ok(p::Projection {
            projection_id: uuid::Uuid::now_v7().to_string(),
            consumer_id: input.consumer_id,
            source_runtime_revision: snapshot.state_revision,
            segments,
            degradation,
        })
    }
    pub(super) async fn query(&self, input: p::QueryRequest) -> Result<p::QueryResponse> {
        if input.nousql.is_some() {
            return Err(Error::Invalid("NousQL must be compiled by Core".into()));
        }
        let subject = SubjectId(id(&input.subject_id)?);
        let session = input
            .session_id
            .as_deref()
            .map(id)
            .transpose()?
            .map(SessionId);
        let expression = required(input.expression, "expression")?;
        let mut nodes = 0;
        validate(&expression, 0, &mut nodes)?;
        let result = self
            .expression(subject, session, expression, vec![])
            .await?;
        Ok(p::QueryResponse {
            query_id: result.query_id.to_string(),
            status: enum_name(result.status),
            hits: result.results.into_iter().map(hit).collect(),
            resource_actions: result
                .resource_actions
                .into_iter()
                .map(|a| p::ResourceAction {
                    resource_ref: a.resource.as_str().into(),
                    action: a.action,
                    reason: a.reason,
                    current_authority: a.current_authority,
                })
                .collect(),
            degradation: result
                .degradation
                .into_iter()
                .map(|d| p::Degradation {
                    code: d.code,
                    detail: d.detail.unwrap_or_default(),
                })
                .collect(),
            bound_query: None,
        })
    }
    fn expression<'a>(&'a self,subject:SubjectId,session:Option<SessionId>,expression:p::QueryExpr,inherited:Vec<p::QueryModifiers>) ->Pin<Box<dyn Future<Output=Result<CognitiveQueryResult>>+Send+'a>> {
        Box::pin(async move {
            let modifiers=expression.modifiers.unwrap_or_default();let mut path=inherited;path.push(modifiers.clone());
            let mut result=match expression.operation.as_str(){
                "atom"=>self.atom(subject,session,expression.cues,&path).await?,
                operation=>self.combine(subject,session,operation,expression.children,&path).await?,
            };
            for domain in &modifiers.domains{retain_domain(&mut result,domain);}
            for preference in modifiers.preferences{self.prefer(subject,session,&mut result,preference).await?;}
            result.results.truncate(modifiers.limit.unwrap_or(2048) as usize);
            Ok(result)
        })
    }
    async fn atom(&self,subject:SubjectId,session:Option<SessionId>,cues:Vec<p::Cue>,path:&[p::QueryModifiers])->Result<CognitiveQueryResult>{
        let mut query=CognitiveQuery{api_version:API_VERSION,subject,session,situation:Default::default(),targets:vec![],cues:vec![],constraints:Default::default(),exploration:Default::default(),resources:Default::default(),result_need:ResultNeed{limit:2048,..Default::default()},effort:Default::default(),capabilities:Default::default(),diagnostics:Default::default()};
        for cue in cues{append_cue(&mut query,cue)?;}
        for modifier in path{if !apply(&mut query,modifier)?{return Ok(empty_result());}}
        let mut result=self.0.query(query).await?;
        if result.results.len()>=2048 {result.status=QueryStatus::Partial;result.degradation.push(Degradation{code:"expression_candidate_bound".into(),detail:Some("expression leaf reached its 2048-result bound".into())});}
        Ok(result)
    }
    async fn combine(&self,subject:SubjectId,session:Option<SessionId>,operation:&str,children:Vec<p::QueryExpr>,path:&[p::QueryModifiers])->Result<CognitiveQueryResult>{
        let mut children=children.into_iter();let first=required(children.next(),"Boolean operands")?;
        let mut result=self.expression(subject,session,first,path.to_vec()).await?;
        for child in children{merge_results(&mut result,self.expression(subject,session,child,path.to_vec()).await?,operation);}
        Ok(result)
    }
    async fn prefer(&self,subject:SubjectId,session:Option<SessionId>,result:&mut CognitiveQueryResult,preference:p::Preference)->Result<()> {
        if preference.key=="recent"{
            let mut dates=HashMap::new();
            for h in &result.results{
                if let CognitiveRef::Memory(memory)=h.reference{dates.insert(h.reference.clone(),self.0.require_memory()?.memory(subject,memory,None).await?.revision.observed_at);}
            }
            result.results.sort_by_key(|h|std::cmp::Reverse(dates.get(&h.reference).copied()));
            if preference.negative{result.results.reverse();}
            return Ok(());
        }
        if let Some(cue)=preference.cue{
            let pref=self.expression(subject,session,p::QueryExpr{operation:"atom".into(),cues:vec![cue],children:vec![],modifiers:None},vec![]).await?;
            let refs:HashSet<_>=pref.results.into_iter().map(|h|h.reference).collect();
            result.results.sort_by_key(|h|refs.contains(&h.reference)==preference.negative);
            result.status=merge_status(result.status,pref.status);result.degradation.extend(pref.degradation);
        }
        Ok(())
    }

}
fn retain_domain(result:&mut CognitiveQueryResult,domain:&str){
    result.results.retain(|h|match domain{
        "memory"=>matches!(h.reference,CognitiveRef::Memory(_)|CognitiveRef::MemoryRevision(_)),
        "evidence"=>matches!(h.authority,AuthorityClass::Evidence|AuthorityClass::Interpretation),
        "resource"=>matches!(h.reference,CognitiveRef::Resource(_)),_=>false,
    });
}
fn merge_status(a:QueryStatus,b:QueryStatus)->QueryStatus{
    match(a,b){(QueryStatus::Partial,_)|(_,QueryStatus::Partial)=>QueryStatus::Partial,(QueryStatus::Degraded,_)|(_,QueryStatus::Degraded)=>QueryStatus::Degraded,_=>QueryStatus::Complete}
}
fn merge_results(result:&mut CognitiveQueryResult,next:CognitiveQueryResult,operation:&str){
    let mut incoming:HashMap<_,_>=next.results.into_iter().map(|h|(h.reference.clone(),h)).collect();
    if operation=="all"{result.results.retain(|h|incoming.contains_key(&h.reference));}
    for hit in &mut result.results{
        if let Some(other)=incoming.remove(&hit.reference){
            for family in other.match_evidence.families{if !hit.match_evidence.families.contains(&family){hit.match_evidence.families.push(family);}}
            for evidence in other.evidence{if !hit.evidence.iter().any(|e|e.reference==evidence.reference&&e.support_role==evidence.support_role){hit.evidence.push(evidence);}}
        }
    }
    if operation=="any"{let mut rest:Vec<_>=incoming.into_values().collect();rest.sort_by_key(|h|h.reference.to_string());result.results.extend(rest);}
    result.resource_actions.extend(next.resource_actions);result.degradation.extend(next.degradation);result.status=merge_status(result.status,next.status);
}
fn validate(e: &p::QueryExpr, depth: usize, count: &mut usize) -> Result<()> {
    *count += 1;
    if depth > 16 || *count > 64 || e.cues.len() > 64 {
        return Err(Error::Invalid("query expression bound exceeded".into()));
    }
    match e.operation.as_str() {
        "atom" if e.children.is_empty() => {}
        "all" | "any" if e.children.len() >= 2 && e.cues.is_empty() => {}
        _ => return Err(Error::Invalid("invalid query expression shape".into())),
    }
    if let Some(m) = &e.modifiers {
        if m.limit.is_some_and(|v| v == 0 || v > 2048) || m.preferences.len() > 16 {
            return Err(Error::Invalid("invalid query modifier bounds".into()));
        }
        for domain in &m.domains {
            if !matches!(domain.as_str(), "memory" | "evidence" | "resource") {
                return Err(Error::Unavailable(format!(
                    "domain {domain} is unavailable"
                )));
            }
        }
        for p in &m.preferences {
            if (!p.key.is_empty() && p.key != "recent") || (p.cue.is_some() == !p.key.is_empty()) {
                return Err(Error::Invalid("invalid preference".into()));
            }
        }
    }
    for child in &e.children {
        validate(child, depth + 1, count)?;
    }
    Ok(())
}
fn append_cue(q: &mut CognitiveQuery, c: p::Cue) -> Result<()> {
    match required(c.cue, "cue")? {
        p::cue::Cue::Text(text) | p::cue::Cue::Concept(text) => {
            q.cues.push(Cue::Text(TextCue { text }))
        }
        p::cue::Cue::Reference(r) => match from_ref(r)? {
            CognitiveRef::Entity(entity_ref) => {
                q.constraints.entity_requirements.push(entity_ref.clone());
                q.cues.push(Cue::Entity(EntityCue { entity_ref }));
            }
            CognitiveRef::Tag(tag) => q.cues.push(Cue::Tag(TagCue { tag })),
            CognitiveRef::Anchor(anchor) => q.cues.push(Cue::Anchor(AnchorCue { anchor })),
            CognitiveRef::Resource(resource) => {
                q.cues.push(Cue::Resource(ResourceCue { resource }))
            }
            CognitiveRef::ExternalObject(object_ref) => {
                q.cues.push(Cue::Object(ObjectCue { object_ref }))
            }
            reference => q.targets.push(QueryTarget::Exact { reference }),
        },
    }
    Ok(())
}
fn interval(input: Option<p::TimeInterval>) -> Result<Option<TimeInterval>> {
    input
        .map(|i| {
            let interval = TimeInterval {
                start: time(i.start)?,
                end: time(i.end)?,
            };
            interval.validate()?;
            Ok(interval)
        })
        .transpose()
}
fn apply(q: &mut CognitiveQuery, m: &p::QueryModifiers) -> Result<bool> {
    if !m.effort.is_empty() {
        q.effort = enum_value(&m.effort)?;
    }
    if !m.exploration.is_empty() {
        q.exploration = enum_value(&m.exploration)?;
    }
    if !m.current_authority.is_empty() {
        q.resources.current_authority = enum_value(&m.current_authority)?;
    }
    if !m.diagnostics.is_empty() {
        q.diagnostics = enum_value(&m.diagnostics)?;
    }
    if m.materialize {
        q.result_need.need_materialization_handles = true;
    }
    if let Some(c) = &m.constraints {
        let target = &mut q.constraints;
        if !merge_include(
            &mut target.source_classes_include,
            c.source_classes_include
                .iter()
                .cloned()
                .map(SourceClass::from)
                .collect(),
        ) {
            return Ok(false);
        }
        target.source_classes_exclude.extend(
            c.source_classes_exclude
                .iter()
                .cloned()
                .map(SourceClass::from),
        );
        if !merge_include(
            &mut target.memory_classes_include,
            c.memory_classes_include.clone(),
        ) {
            return Ok(false);
        }
        target
            .memory_classes_exclude
            .extend(c.memory_classes_exclude.clone());
        target.entity_requirements.extend(
            c.entity_requirements
                .iter()
                .map(|e| EntityRef::new(e.clone()))
                .collect::<Result<Vec<_>>>()?,
        );
        target.occurred = intersect_time(target.occurred, interval(c.occurred)?);
        target.observed = intersect_time(target.observed, interval(c.observed)?);
        target.valid = intersect_time(target.valid, interval(c.valid)?);
        target.include_suppressed = c.include_suppressed;
        if let Some(a) = &c.authority {
            let value = enum_value(a)?;
            if target.authority.is_some_and(|old| old != value) {
                return Ok(false);
            }
            target.authority = Some(value);
        }
        if !merge_include(
            &mut target.modalities,
            c.modalities
                .iter()
                .map(|s| enum_value(s))
                .collect::<Result<_>>()?,
        ) {
            return Ok(false);
        }
        if !merge_include(&mut target.evidence_classes, c.evidence_classes.clone()) {
            return Ok(false);
        }
        if [target.occurred, target.observed, target.valid]
            .into_iter()
            .flatten()
            .any(|i| i.start.zip(i.end).is_some_and(|(a, b)| a > b))
        {
            return Ok(false);
        }
    }
    Ok(true)
}
fn merge_include<T: PartialEq + Clone>(target: &mut Vec<T>, values: Vec<T>) -> bool {
    if target.is_empty() {
        *target = values;
        true
    } else if !values.is_empty() {
        target.retain(|v| values.contains(v));
        !target.is_empty()
    } else {
        true
    }
}
fn empty_result() -> CognitiveQueryResult {
    CognitiveQueryResult {
        query_id: uuid::Uuid::now_v7(),
        generation: Default::default(),
        status: QueryStatus::Complete,
        results: vec![],
        resource_actions: vec![],
        degradation: vec![],
        diagnostics: None,
    }
}
fn intersect_time(a: Option<TimeInterval>, b: Option<TimeInterval>) -> Option<TimeInterval> {
    match (a, b) {
        (Some(a), Some(b)) => Some(TimeInterval {
            start: a.start.max(b.start),
            end: match (a.end, b.end) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            },
        }),
        (a, b) => a.or(b),
    }
}
fn hit(h: CognitiveHit) -> p::Hit {
    p::Hit {
        reference: Some(to_ref(h.reference)),
        revision_id: h.revision.map(|r| r.0.to_string()),
        text: h.representation,
        authority: enum_name(h.authority),
        evidence: h
            .evidence
            .into_iter()
            .map(|e| p::Evidence {
                reference: Some(to_ref(e.reference)),
                support_role: e.support_role,
            })
            .collect(),
        evidence_families: h
            .match_evidence
            .families
            .into_iter()
            .map(enum_name)
            .collect(),
        entity_refs: h
            .entity_refs
            .into_iter()
            .map(|e| e.as_str().into())
            .collect(),
        lexical_ref: None,
        semantic_role: h.semantic_role,
        memory_class: h.memory_class,
    }
}

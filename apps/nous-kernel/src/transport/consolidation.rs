use super::*;
use nous_core::*;
use nous_memory_domain::*;
impl KernelService {
    pub(super) async fn consolidate_memory(&self,input:p::ConsolidateMemoryRequest)->Result<p::ConsolidationResponse>{
        let subject=SubjectId(id(&input.subject_id)?);
        if input.source_revision_ids.len()>256{return Err(Error::Invalid("consolidation source bound exceeded".into()));}
        let topology=input.topology.map(topology_proposal).transpose()?;
        let result=self.0.require_memory()?.consolidate(subject,ConsolidationRequest{subject,source_memories:input.source_revision_ids.iter().map(|r|Ok(MemoryRevisionId(id(r)?))).collect::<Result<_>>()?,target:enum_value(&input.target)?,capability:CapabilityRequirement{operation:CapabilityOperation::MemoryConsolidationText,strength:RequirementStrength::Optional},representation_text:(!input.text.is_empty()).then_some(input.text),semantic_role:(!input.semantic_role.is_empty()).then_some(input.semantic_role),topology}).await?;
        Ok(p::ConsolidationResponse{memory:result.memory.map(super::memory::view),topology_changes:result.topology_changes as u32})
    }
}
fn topology_proposal(input:p::TopologyChanges)->Result<TopologyConsolidationProposal>{
    if input.tags.len()+input.anchors.len()+input.associations.len()+input.revisions.len()>256{return Err(Error::Invalid("topology proposal bound exceeded".into()));}
    Ok(TopologyConsolidationProposal{
        tags:input.tags.into_iter().map(|t|{
            let tag=t.tag.unwrap_or_default();
            Ok(TopologyTagProposal{label:tag.label,description:tag.description,kind_hint:tag.kind_hint,tag_id:t.existing_tag_id.as_deref().map(id).transpose()?.map(TagId),attach_to:t.attach_to_revision_ids.iter().map(|r|Ok(MemoryRevisionId(id(r)?))).collect::<Result<_>>()?})
        }).collect::<Result<_>>()?,
        anchors:input.anchors.into_iter().map(|a|Ok(TopologyAnchorProposal{label:a.label,description:a.description,confirmed:a.confirmed,supports:a.supports.into_iter().map(|s|Ok(TopologyAnchorSupport{reference:from_ref(required(s.reference,"support reference")?)?,role:s.role})).collect::<Result<_>>()?})).collect::<Result<_>>()?,
        associations:input.associations.into_iter().map(|a|Ok(TopologyAssociationProposal{from:from_ref(required(a.from,"from")?)?,to:from_ref(required(a.to,"to")?)?,association_kind:a.kind,polarity:enum_value(&a.polarity)?,support_class:enum_value(&a.support_class)?,support_value:a.support_value,occurrence_id:a.occurrence_id.as_deref().map(id).transpose()?.map(OccurrenceId),memory_revision_id:a.memory_revision_id.as_deref().map(id).transpose()?.map(MemoryRevisionId),bridge_hint:a.bridge_hint})).collect::<Result<_>>()?,
        revisions:input.revisions.into_iter().map(|r|{
            let content=required(r.content,"revision content")?;let valid=content.valid.unwrap_or_default();
            Ok(TopologyRevisionProposal{memory_id:MemoryId(id(&r.memory_id)?),expected_head_revision:super::memory::etag(&r.expected_etag)?,intent:enum_value(&r.intent)?,entity_refs:r.entity_refs.into_iter().map(EntityRef::new).collect::<Result<_>>()?,representation_text:content.text,semantic_role:Some(content.semantic_role),title:content.title,evidence:super::memory::evidence(content.evidence)?,valid_from:time(valid.start)?,valid_to:time(valid.end)?,epistemic_class:enum_value(&content.epistemic_class)?})
        }).collect::<Result<_>>()?,
    })
}

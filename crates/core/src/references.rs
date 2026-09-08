use crate::*;

pub fn reference_parts(reference: &CognitiveRef) -> (String, String) {
    match reference {
        CognitiveRef::Memory(id) => ("memory".into(), id.0.to_string()),
        CognitiveRef::MemoryRevision(id) => ("memory_revision".into(), id.0.to_string()),
        CognitiveRef::Artifact(id) => ("artifact".into(), id.0.to_string()),
        CognitiveRef::SourceRegion(id) => ("source_region".into(), id.0.to_string()),
        CognitiveRef::DerivedRepresentation(id) => {
            ("derived_representation".into(), id.0.to_string())
        }
        CognitiveRef::DerivedRegion(id) => ("derived_region".into(), id.0.to_string()),
        CognitiveRef::Entity(id) => ("entity".into(), id.as_str().into()),
        CognitiveRef::Tag(id) => ("tag".into(), id.0.to_string()),
        CognitiveRef::Anchor(id) => ("anchor".into(), id.0.to_string()),
        CognitiveRef::Resource(id) => ("resource".into(), id.as_str().into()),
        CognitiveRef::ExternalObject(id) => ("external_object".into(), id.as_str().into()),
        CognitiveRef::Occurrence(id) => ("occurrence".into(), id.0.to_string()),
        CognitiveRef::Session(id) => ("session".into(), id.0.to_string()),
    }
}

pub fn parse_reference(kind: &str, value: &str) -> Result<CognitiveRef> {
    Ok(match kind {
        "memory" => CognitiveRef::Memory(MemoryId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid memory ref".into()))?,
        )),
        "memory_revision" => CognitiveRef::MemoryRevision(MemoryRevisionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid memory revision ref".into()))?,
        )),
        "artifact" => CognitiveRef::Artifact(ArtifactId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid artifact ref".into()))?,
        )),
        "source_region" => CognitiveRef::SourceRegion(SourceRegionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid source region ref".into()))?,
        )),
        "derived_representation" => CognitiveRef::DerivedRepresentation(DerivedRepresentationId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid derived ref".into()))?,
        )),
        "derived_region" => CognitiveRef::DerivedRegion(DerivedRegionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid derived region ref".into()))?,
        )),
        "entity" => CognitiveRef::Entity(EntityRef::new(value)?),
        "tag" => CognitiveRef::Tag(TagId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid tag ref".into()))?,
        )),
        "anchor" => CognitiveRef::Anchor(AnchorId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid anchor ref".into()))?,
        )),
        "resource" => CognitiveRef::Resource(ResourceRef::new(value)?),
        "external_object" => CognitiveRef::ExternalObject(ObjectRef::new(value)?),
        "occurrence" => CognitiveRef::Occurrence(OccurrenceId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid occurrence ref".into()))?,
        )),
        "session" => CognitiveRef::Session(SessionId(
            value
                .parse()
                .map_err(|_| Error::Invalid("invalid session ref".into()))?,
        )),
        _ => return Err(Error::Invalid("unknown reference kind".into())),
    })
}

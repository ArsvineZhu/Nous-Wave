use super::*;
use nous_core::{Result, SubjectId};
fn binding(b: nous_authority_store::IdentityBinding) -> p::IdentityBinding {
    p::IdentityBinding {
        lexical_ref: b.lexical_ref,
        canonical: Some(to_ref(b.canonical)),
        display_name: b.display_name,
        aliases: b.aliases,
        status: b.status,
    }
}
impl KernelService {
    pub(super) async fn bind_identity(
        &self,
        input: p::BindIdentityRequest,
    ) -> Result<p::IdentityBinding> {
        Ok(binding(
            self.0
                .store
                .bind_identity(
                    SubjectId(id(&input.subject_id)?),
                    from_ref(required(input.canonical, "canonical")?)?,
                    input.display_name,
                    input.aliases,
                )
                .await?,
        ))
    }
    pub(super) async fn resolve_identity(
        &self,
        input: p::ResolveIdentityRequest,
    ) -> Result<p::ResolveIdentityResponse> {
        let (locator, lexical) = match required(input.locator, "locator")? {
            p::resolve_identity_request::Locator::Name(name) => (name, false),
            p::resolve_identity_request::Locator::LexicalRef(reference) => (reference, true),
        };
        let (status, bindings) = self
            .0
            .store
            .resolve_identity(
                SubjectId(id(&input.subject_id)?),
                &input.kind,
                &locator,
                lexical,
            )
            .await?;
        Ok(p::ResolveIdentityResponse {
            status,
            candidates: bindings.into_iter().map(binding).collect(),
        })
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source/API constructors and selective direct reflection over genuinely typed facts.
use super::pb::pse::code_intelligence::v1 as wire;
use crate::source_read::{Entry, SourceReadError, SourceText};
use buffa_descriptor::reflect::{ReflectMessage, ValueRef};

/// A small borrowed declaration projection. Shape access never establishes native identity.
#[derive(Debug)]
pub struct DeclarationProjection<'a> {
    /// Original source name or artifact-qualified public API route.
    pub name: &'a str,
    /// Original producer declaration category.
    pub kind: &'a str,
}
/// Read selected fields directly through the generated VTable; no wire or JSON transcode.
pub fn project_declaration(message: &impl ReflectMessage) -> Option<DeclarationProjection<'_>> {
    let descriptor = message.message_descriptor();
    let ValueRef::String(name) = message.get(descriptor.field_by_name("name")?) else {
        return None;
    };
    let ValueRef::String(kind) = message.get(descriptor.field_by_name("kind")?) else {
        return None;
    };
    Some(DeclarationProjection { name, kind })
}
/// Construct a source fact only from the original snapshot used by its parser.
pub fn source_declaration(
    source: &SourceText,
    outline_digest: &str,
    entry: &Entry,
) -> Result<wire::SourceFact, SourceReadError> {
    if outline_digest != source.identity().source_sha256() {
        return Err(SourceReadError::StaleSource);
    }
    if source
        .text()
        .get(entry.range.byte_start..entry.range.byte_end)
        .is_none()
    {
        return Err(SourceReadError::InvalidRange);
    }
    let identity = source.identity();
    let guard = wire::ContextGuard {
        source_digest: identity.source_sha256().as_bytes().to_vec(),
        provider: "authored_source".into(),
        ..Default::default()
    };
    Ok(wire::SourceFact {
        name: entry.path.last().cloned().unwrap_or_default(),
        kind: entry.kind.clone(),
        location: Some(wire::Location {
            path: identity.path().as_os_str().as_encoded_bytes().to_vec(),
            byte_start: Some(entry.range.byte_start as u64),
            byte_end: Some(entry.range.byte_end as u64),
            guard: Some(guard).into(),
            ..Default::default()
        })
        .into(),
        source_identity: Some(source_identity(identity)).into(),
        ..Default::default()
    })
}
fn source_identity(source: &crate::source_read::SourceIdentity) -> wire::NativeSourceIdentity {
    use crate::source_read::SourceOrigin;
    let mut result = wire::NativeSourceIdentity {
        path: source.path().as_os_str().as_encoded_bytes().to_vec(),
        source_sha256: source.source_sha256().into(),
        ..Default::default()
    };
    match source.origin() {
        SourceOrigin::Working => {}
        SourceOrigin::Index {
            git_dir,
            index_path,
            blob,
        } => {
            result.origin = wire::native_source_identity::Origin::INDEX_SOURCE.into();
            result.git_dir = Some(git_dir.as_bytes().to_vec());
            result.index_path = Some(index_path.as_bytes().to_vec());
            result.blob = Some(blob.clone());
        }
        SourceOrigin::Blob {
            git_dir,
            revision,
            blob,
        } => {
            result.origin = wire::native_source_identity::Origin::BLOB_SOURCE.into();
            result.git_dir = Some(git_dir.as_bytes().to_vec());
            result.revision = Some(revision.clone());
            result.blob = Some(blob.clone());
        }
    }
    result
}
/// Preserve the exact artifact-local source universe before selective projection.
pub fn api_declaration(
    artifact: &crate::api::Artifact,
    item_id: u32,
    public_path: &str,
    kind: &str,
    definition: Option<&crate::api::query::DefinitionEvidence>,
    qualification_unknown: Option<&str>,
) -> Result<wire::ApiFact, crate::api::ApiError> {
    if !artifact
        .document
        .index
        .contains_key(&rustdoc_types::Id(item_id))
    {
        return Err(crate::api::ApiError::Invalid(
            "selected artifact has no original API item".into(),
        ));
    }
    Ok(wire::ApiFact {
        name: public_path.into(),
        kind: kind.into(),
        item_id,
        artifact: artifact.path.as_os_str().as_encoded_bytes().to_vec(),
        artifact_sha256: artifact.sha256.clone(),
        context_key: artifact.context.key()?,
        package_id: artifact.context.package_id.clone(),
        qualification_unknown: qualification_unknown.map(str::to_owned),
        definition: definition
            .map(|definition| wire::ApiDefinition {
                package_id: definition.package_id.clone(),
                context_key: definition.context_key.clone(),
                artifact: definition.artifact.as_os_str().as_encoded_bytes().to_vec(),
                artifact_sha256: definition.artifact_sha256.clone(),
                item_id: definition.item_id,
                public_path: definition.public_path.clone(),
                ..Default::default()
            })
            .into(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa_descriptor::reflect::{ReflectCow, Reflectable};
    #[test]
    fn actual_source_projection_borrows_and_keeps_same_spelling_distinct() {
        let source = SourceText::new(
            "/source.rs".into(),
            "mod first { fn same() {} } mod second { fn same() {} }".into(),
        )
        .unwrap();
        let outline =
            crate::source_read::rust(source.text(), crate::source_read::RustEdition::E2024);
        let facts = outline
            .entries
            .iter()
            .filter(|entry| entry.path.last().is_some_and(|name| name == "same"))
            .map(|entry| source_declaration(&source, &outline.source_sha256, entry).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(facts.len(), 2);
        assert_eq!(project_declaration(&facts[0]).unwrap().name, "same");
        assert_ne!(facts[0].location.byte_start, facts[1].location.byte_start);
        assert!(matches!(facts[0].reflect(), ReflectCow::Borrowed(_)));
        let changed = SourceText::new("/source.rs".into(), "fn changed() {}".into()).unwrap();
        assert!(matches!(
            source_declaration(&changed, &outline.source_sha256, &outline.entries[0]),
            Err(SourceReadError::StaleSource)
        ));
        assert!(project_declaration(&wire::PredicateFact::default()).is_none());
    }
    #[cfg(unix)]
    #[test]
    fn source_fact_keeps_native_non_utf8_path_bytes_without_json() {
        use std::os::unix::ffi::OsStringExt as _;
        let path = std::ffi::OsString::from_vec(b"/source-\xff.rs".to_vec());
        let source = SourceText::new(path.into(), "fn exact() {}".into()).unwrap();
        let outline =
            crate::source_read::rust(source.text(), crate::source_read::RustEdition::E2024);
        let fact =
            source_declaration(&source, &outline.source_sha256, &outline.entries[0]).unwrap();
        assert_eq!(fact.location.path, b"/source-\xff.rs");
        assert_eq!(fact.source_identity.path, b"/source-\xff.rs");
        assert_eq!(project_declaration(&fact).unwrap().name, "exact");
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Select exact original document bytes before copying, without rebuilding a source inventory.

use super::*;
use pse_model::boundary_projection::{Direction, LocalRule, ValidateProjection as _};
use pse_model::engineering::{LOCAL_REVISION_RULE, SOURCE_FILE_PATH_RULE};

pse_model::projection_struct! {
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    /// One exact-parent original document read; no path addresses the host filesystem.
    pub struct OriginalSourceFileRequest {
        projection: { name: "OriginalSourceFileRequest", rust_path: crate::workflow::OriginalSourceFileRequest,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact immutable local source parent, independent of semantic content equality.
        pub parent: String => { tag: 1, direction: Direction::Input, rules: [LOCAL_REVISION_RULE] };
        /// Original owning package identity.
        pub package: SemanticId => { tag: 2, direction: Direction::Input, rules: [] };
        /// Exact original package-relative document path, interpreted by its admitted owner.
        pub path: String => { tag: 3, direction: Direction::Input, rules: [SOURCE_FILE_PATH_RULE] };
        /// Finite complete encoded response allowance; exhaustion never publishes a prefix.
        pub maximum_bytes: u64 => { tag: 4, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 16777216 }] };
    }
}
pse_model::projection_struct! {
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(deny_unknown_fields)]
    /// Original selected source/data bytes and their actual immutable document attribution.
    pub struct OriginalSourceFile {
        projection: { name: "OriginalSourceFile", rust_path: crate::workflow::OriginalSourceFile,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact local parent from which this document was selected.
        pub local_revision: String => { tag: 1, direction: Direction::Output, rules: [] };
        /// Original package identity.
        pub package: SemanticId => { tag: 2, direction: Direction::Output, rules: [] };
        /// Actual loader-issued document identity.
        pub document: SemanticId => { tag: 3, direction: Direction::Output, rules: [] };
        /// Original package-relative source/data path.
        pub path: String => { tag: 4, direction: Direction::Output, rules: [] };
        /// Exact admitted document member version; no latest member is substituted.
        pub document_version: String => { tag: 5, direction: Direction::Output, rules: [] };
        /// Encoding checksum of these exact raw bytes, separate from scientific source identity.
        pub original_hash: pse_ids::ContentHash => { tag: 6, direction: Direction::Output, rules: [] };
        /// True only for the owner's original UTF-8 text, including comments and whitespace.
        pub is_text: bool => { tag: 7, direction: Direction::Output, rules: [] };
        /// Exact original bytes. Unavailable text after IR edits is never reconstructed.
        pub body: Vec<u8> => { tag: 8, direction: Direction::Output, rules: [] };
    }
}

/// Original selected document with its allocation retained through boundary publication.
#[derive(Debug)]
pub struct OriginalSourceFileOutput {
    pub(crate) document: OriginalSourceFile,
    pub(crate) owner: Arc<pse_columnar::AllocationLease>,
    pub(crate) maximum_bytes: usize,
}
impl OriginalSourceFileOutput {
    /// Borrow the exact selected document while its original allocation remains held.
    pub fn document(&self) -> &OriginalSourceFile {
        &self.document
    }
    /// Keep the original selected allocation through caller encoding and byte consumption.
    pub fn owner(&self) -> Arc<dyn pse_columnar::PayloadOwner> {
        self.owner.clone()
    }
    /// Complete encoded response allowance, independent of native source completeness.
    pub const fn maximum_bytes(&self) -> usize {
        self.maximum_bytes
    }
}

impl ModelingPackage {
    /// Read one original source/data document from this exact admitted parent.
    /// Selection uses the loader's actual named identity before decoding/copying a body.
    /// # Errors
    /// Stale parent, absent original bytes, changed receiving services, cancellation,
    /// malformed membership or a finite response/resource refusal preserves the parent.
    pub async fn original_source_file(
        &self,
        services: &crate::workflow::PreparationServices,
        request: &OriginalSourceFileRequest,
        cancel: &crate::CancelSource,
    ) -> Result<OriginalSourceFileOutput, WorkflowError> {
        request.validate_projection().map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        self.check_service_bindings(services)?;
        if request.parent != self.revision_id() {
            return Err(contract(
                "original document read names another exact local parent",
            ));
        }
        let activity = services.admit_activity()?;
        activity.scope(async {
            checkpoint(cancel)?;
            let id = pse_ids::named_id(request.package, &request.path);
            let object = self.local_object(&format!("document:{id}"))?;
            if object.kind != "document" { return Err(contract("selected source member is not a document")); }
            let maximum_bytes = usize::try_from(request.maximum_bytes).map_err(|_| contract("source response allowance exceeds native address space"))?;
            // A cold native document payload uses JSON's at-most-sixfold text
            // escaping plus bounded metadata. Refuse oversized cold adoption
            // before the original typed decoder can allocate its selected row.
            if object.typed.is_none() && object.payload.len() > maximum_bytes.saturating_mul(6).saturating_add(65536) {
                return Err(WorkflowError::Resource("selected cold document exceeds its finite adoption allowance".into()));
            }
            let decoded;
            let row = if let Some(TypedObjectPayload::Document(row)) = &object.typed { row.as_ref() }
                else { decoded = self.source_decode::<pse_model::generated::authored::documents::Row>(services, &object)?; &decoded.0 };
            if row.document_id != id || row.package_id.as_id() != request.package || row.path != request.path {
                return Err(contract("selected document membership differs from its original package/path"));
            }
            if row.content.is_some() { return Err(contract("original document metadata unexpectedly embeds a second data body")); }
            let data;
            let (body, is_text) = if let Some(text) = &row.source_text { (text.as_bytes(), true) }
                else {
                    data = self.local_object(&format!("data:{id}"))?;
                    if data.kind != "data" { return Err(contract("original source text is unavailable and no original data object exists")); }
                    (data.payload.as_slice(), false)
                };
            if body.len() > maximum_bytes { return Err(WorkflowError::Resource("complete original document exceeds selected response allowance".into())); }
            // Retain room for the owned body, base64/escaped JSON or protobuf
            // encoding and transient encoder growth. This is a per-response
            // grant, never a wider native/host execution allocation.
            let metadata = request.path.len().checked_add(self.revision_id().len())
                .and_then(|bytes| bytes.checked_add(object.version.len()))
                .and_then(|bytes| bytes.checked_mul(18)).ok_or_else(|| contract("selected source metadata extent overflow"))?;
            let extent = body.len().checked_mul(8).and_then(|bytes| bytes.checked_add(metadata))
                .and_then(|bytes| bytes.checked_add(size_of::<OriginalSourceFile>() + 65536))
                .ok_or_else(|| contract("selected original source response extent overflow"))?;
            let owner = services.math().reserve("modeling:original-file-response", extent)?;
            checkpoint(cancel)?;
            let document = OriginalSourceFile { local_revision: self.revision_id().into(), package: request.package,
                document: id, path: row.path.clone(), document_version: object.version.clone(),
                original_hash: pse_ids::encoding_checksum(body).content_hash(), is_text, body: body.to_vec() };
            Ok(OriginalSourceFileOutput { document, owner, maximum_bytes })
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn original_source_file_selects_exact_raw_bytes_and_rejects_stale_or_oversized_reads() {
        let runtime = crate::workflow::tests::runtime();
        let cancel = crate::CancelSource::new();
        let text =
            b"package minimal_named {\n  // preserve original comment\n  param a:Scalar=1;\n}\n";
        let source = BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!(
                    "../../../../../../../tests/fixtures/packages/minimal_named/package.toml"
                )
                .to_vec(),
            ),
            ("models/original.pse".into(), text.to_vec()),
        ]);
        let package = runtime
            .package_from_sources(&[source], crate::workflow::tests::physical(), &cancel)
            .await
            .unwrap();
        let row = package
            .revision
            .admitted
            .objects
            .values()
            .find_map(|object| match &object.typed {
                Some(TypedObjectPayload::Document(row)) if row.path == "models/original.pse" => {
                    Some(row.clone())
                }
                _ => None,
            })
            .unwrap();
        let mut request = OriginalSourceFileRequest {
            parent: package.revision_id().into(),
            package: row.package_id.as_id(),
            path: row.path.clone(),
            maximum_bytes: 4096,
        };
        let result = package
            .original_source_file(&runtime.preparation_services(), &request, &cancel)
            .await
            .unwrap();
        assert_eq!(result.document().body, text);
        assert!(result.document().is_text);
        assert_eq!(result.document().document, row.document_id);
        assert_eq!(
            result.document().original_hash,
            pse_ids::encoding_checksum(text).content_hash()
        );
        request.maximum_bytes = 1;
        let error = package
            .original_source_file(&runtime.preparation_services(), &request, &cancel)
            .await
            .unwrap_err();
        assert_eq!(
            pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
            Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit)
        );
        request.maximum_bytes = 4096;
        request.parent = "stale".into();
        assert!(
            package
                .original_source_file(&runtime.preparation_services(), &request, &cancel)
                .await
                .is_err()
        );
        let mut unavailable = row.as_ref().clone();
        unavailable.source_text = None;
        let changed = package
            .with_delta(
                &runtime.source_services(),
                crate::workflow::SourceDelta {
                    parent: package.revision_id().into(),
                    documents: crate::workflow::SourceDocumentDelta {
                        replacements: vec![unavailable],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        request.parent = changed.revision_id().into();
        assert!(
            changed
                .original_source_file(&runtime.preparation_services(), &request, &cancel)
                .await
                .is_err(),
            "IR/document changes cannot manufacture an original text preimage"
        );
    }
}

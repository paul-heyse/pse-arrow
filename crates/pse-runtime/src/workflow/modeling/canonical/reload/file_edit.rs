// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit file intent delegates membership, source spans and fit/data changes to parsed reload.
use super::*;
use pse_contract::boundary_projection::ValidateProjection as _;
use pse_model::engineering::{SourceFileChange, SourceFileDelta};

impl ModelingPackage {
    /// Apply explicit file edits against this exact parent using the original parsed reload owner.
    /// Omitted files inherit. No IR declaration, fit or data ownership is inferred at the service.
    /// # Errors
    /// Stale parent, duplicate intent, absent/existing membership conflict, unsafe path,
    /// unsupported source reconstruction or contextual loader/delta refusal leaves the parent intact.
    pub async fn with_file_delta(
        &self,
        services: &crate::workflow::SourceServices,
        edit: &SourceFileDelta,
        cancel: &crate::CancelSource,
    ) -> Result<Self, WorkflowError> {
        edit.validate_projection().map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        self.check_service_bindings(services.preparation())?;
        if edit.parent != self.revision_id() {
            return Err(contract(
                "source file delta names a different exact local parent",
            ));
        }
        let activity = services.preparation().admit_activity()?;
        activity
            .scope(async {
                let deadline = services.preparation().math().preparation_deadline(cancel)?;
                let scoped = cancel.with_deadline(Some(deadline));
                checkpoint(&scoped)?;
                let parent = self.delta_parent(services.preparation(), &scoped).await?;
                // Reserve before copying original documents or incoming replacement bytes.
                let extent = parent
                    .documents
                    .values()
                    .try_fold(0usize, |total, document| {
                        let bytes = match &document.source_text {
                            Some(text) => text.len(),
                            None => self
                                .local_object(&format!("data:{}", document.document_id))?
                                .payload
                                .len(),
                        };
                        total
                            .checked_add(
                                bytes
                                    .checked_mul(4)
                                    .ok_or_else(|| contract("source file edit extent overflow"))?,
                            )
                            .and_then(|size| size.checked_add(document.path.len() + 1024))
                            .ok_or_else(|| contract("source file edit extent overflow"))
                    })?;
                let extent = edit.edits.iter().try_fold(extent, |total, edit| {
                    let bytes = match &edit.change {
                        SourceFileChange::Add(bytes) | SourceFileChange::Replace(bytes) => {
                            bytes.len()
                        }
                        SourceFileChange::Remove => 0,
                    };
                    total
                        .checked_add(
                            bytes
                                .checked_mul(4)
                                .ok_or_else(|| contract("source edit ingress extent overflow"))?,
                        )
                        .ok_or_else(|| contract("source edit ingress extent overflow"))
                })?;
                let _owner = services
                    .preparation()
                    .math()
                    .reserve("modeling:explicit-file-edit", extent)?;
                let mut inventory =
                    exact_document_inventory(services.preparation(), self, &parent, &scoped)?;
                let mut seen = BTreeSet::new();
                for change in &edit.edits {
                    checkpoint(&scoped)?;
                    if !seen.insert((change.package, change.path.as_str())) {
                        return Err(contract("source file delta repeats a package/path"));
                    }
                    if std::path::Path::new(&change.path)
                        .components()
                        .any(|part| !matches!(part, std::path::Component::Normal(_)))
                    {
                        return Err(contract("source file edit requires a normal relative path"));
                    }
                    let package = inventory
                        .iter_mut()
                        .find(|package| package.package == change.package)
                        .ok_or_else(|| {
                            contract("source file edit names a package outside its exact parent")
                        })?;
                    let files = &mut package.files;
                    match &change.change {
                        SourceFileChange::Add(bytes) => {
                            if files.contains_key(&change.path) {
                                return Err(contract("source file addition already exists"));
                            }
                            files.insert(change.path.clone(), bytes.clone());
                        }
                        SourceFileChange::Replace(bytes) => {
                            let value = files
                                .get_mut(&change.path)
                                .ok_or_else(|| contract("source file replacement is absent"))?;
                            *value = bytes.clone();
                        }
                        SourceFileChange::Remove => {
                            if files.remove(&change.path).is_none() {
                                return Err(contract("source file removal is absent"));
                            }
                        }
                    }
                }
                let sources = inventory
                    .into_iter()
                    .map(|package| package.files)
                    .collect::<Vec<_>>();
                // This constructs the real SourceDelta with exact parser products, object order,
                // document/fit changes and data owners before original parent CAS admission.
                self.reload_retained(services, &sources, &scoped).await
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_model::engineering::SourceFileEdit;

    #[tokio::test]
    async fn explicit_file_delta_preserves_omitted_fits_and_deletes_only_named_document_members() {
        let runtime = crate::workflow::tests::runtime();
        let cancel = crate::CancelSource::new();
        let fit = "01991d6a13a070008000000000000077";
        let inventory = BTreeMap::from([
            ("package.toml".into(), include_bytes!("../../../../../../../tests/fixtures/packages/minimal_named/package.toml").to_vec()),
            ("models/bank.pse".into(), b"package minimal_named {param a:Scalar=1;}".to_vec()),
            ("cases/fit.yaml".into(), format!("fit_cases:\n  - fit_id: \"{fit}\"\n    parameters: []\n    experiments: []\n    observations: []\n").into_bytes()),
        ]);
        let original = runtime
            .package_from_sources(&[inventory], crate::workflow::tests::physical(), &cancel)
            .await
            .unwrap();
        let parent = original
            .delta_parent(&runtime.preparation_services(), &cancel)
            .await
            .unwrap();
        let package = parent.documents.values().next().unwrap().package_id.as_id();
        let mut edit = SourceFileDelta {
            parent: original.revision_id().into(),
            edits: vec![SourceFileEdit {
                package,
                path: "models/bank.pse".into(),
                change: SourceFileChange::Replace(
                    b"package minimal_named {param a:Scalar=2;param b:Scalar=3;}".to_vec(),
                ),
            }],
        };
        let updated = original
            .with_file_delta(&runtime.source_services(), &edit, &cancel)
            .await
            .unwrap();
        let current = updated
            .delta_parent(&runtime.preparation_services(), &cancel)
            .await
            .unwrap();
        assert!(
            current.members.contains_key(&format!("fit:{fit}")),
            "omission retains actual fit ownership"
        );
        assert_eq!(current.rows.iter().filter(|row| row.name == "b").count(), 1);
        assert!(
            updated
                .with_file_delta(&runtime.source_services(), &edit, &cancel)
                .await
                .is_err(),
            "the original parent cannot edit a different immutable receiver"
        );
        edit.parent = updated.revision_id().into();
        edit.edits = vec![SourceFileEdit {
            package,
            path: "cases/fit.yaml".into(),
            change: SourceFileChange::Remove,
        }];
        let removed = updated
            .with_file_delta(&runtime.source_services(), &edit, &cancel)
            .await
            .unwrap();
        let final_parent = removed
            .delta_parent(&runtime.preparation_services(), &cancel)
            .await
            .unwrap();
        assert!(!final_parent.members.contains_key(&format!("fit:{fit}")));
        assert!(
            !final_parent
                .documents
                .values()
                .any(|document| document.path == "cases/fit.yaml")
        );
        assert_eq!(
            final_parent
                .rows
                .iter()
                .filter(|row| row.name == "b")
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn explicit_file_delta_rejects_conflicts_and_invalid_source_without_mutating_parent() {
        let runtime = crate::workflow::tests::runtime();
        let cancel = crate::CancelSource::new();
        let sources = BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!(
                    "../../../../../../../tests/fixtures/packages/minimal_named/package.toml"
                )
                .to_vec(),
            ),
            (
                "models/bank.pse".into(),
                b"package minimal_named {param a:Scalar=1;}".to_vec(),
            ),
        ]);
        let original = runtime
            .package_from_sources(&[sources], crate::workflow::tests::physical(), &cancel)
            .await
            .unwrap();
        let before = original
            .delta_parent(&runtime.preparation_services(), &cancel)
            .await
            .unwrap();
        let package = before.documents.values().next().unwrap().package_id.as_id();
        let replacement = SourceFileEdit {
            package,
            path: "models/bank.pse".into(),
            change: SourceFileChange::Replace(b"not valid authored source".to_vec()),
        };
        for edits in [
            vec![replacement.clone()],
            vec![replacement.clone(), replacement],
            vec![SourceFileEdit {
                package,
                path: "models/absent.pse".into(),
                change: SourceFileChange::Remove,
            }],
            vec![SourceFileEdit {
                package,
                path: "models/bank.pse".into(),
                change: SourceFileChange::Add(vec![]),
            }],
            vec![SourceFileEdit {
                package,
                path: "../outside.pse".into(),
                change: SourceFileChange::Add(vec![]),
            }],
        ] {
            let edit = SourceFileDelta {
                parent: original.revision_id().into(),
                edits,
            };
            assert!(
                original
                    .with_file_delta(&runtime.source_services(), &edit, &cancel)
                    .await
                    .is_err()
            );
            let after = original
                .delta_parent(&runtime.preparation_services(), &cancel)
                .await
                .unwrap();
            assert_eq!(after.rows, before.rows);
            assert_eq!(after.documents, before.documents);
        }
    }
}

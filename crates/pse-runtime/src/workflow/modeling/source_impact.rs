// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded selected-source discovery and conservative authored update impact.
use super::*;
use pse_model::diagnostic::BoundaryClass;

/// Exact admitted lexical source membership. This does not grant execution permission.
#[derive(Clone, Debug)]
pub struct ModelingSourceClosure {
    admission: Arc<SelectedAdmission>,
}
impl ModelingSourceClosure {
    /// Exact source/physical/data revision of this selected closure.
    pub fn source_revision(&self) -> pse_ids::roles::SourceRevisionHash {
        self.admission.revision.identity()
    }
    /// Original logical object versions, including selected data documents.
    pub fn members(&self) -> &BTreeMap<String, String> {
        &self.admission.versions
    }
}
/// Conservative source impact for one explicitly selected root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelingSourceImpactStatus {
    /// All original positive/absent lookup premises and physical/provider context still match.
    Unchanged,
    /// Selected source premises or context changed; numerical equivalence is unassessed.
    Changed,
    /// At least one selected source query refused. Inspect its original error.
    Refused,
}
/// Both original selection results are retained; a refusal never implies global absence.
#[derive(Clone, Debug)]
pub struct ModelingRootSourceImpact {
    /// The exact requested authored root, without an inferred reverse-consumer population.
    pub root: DeclarationId,
    /// Original checked closure, or its original source-query refusal.
    pub before: Result<ModelingSourceClosure, Arc<WorkflowError>>,
    /// Proposed checked closure, or its original source-query refusal.
    pub after: Result<ModelingSourceClosure, Arc<WorkflowError>>,
    /// Conservative effect on this selected root's source premises only.
    pub status: ModelingSourceImpactStatus,
}
/// Read-only exact source bases and the complete explicitly requested root population.
#[derive(Clone, Debug)]
pub struct ModelingSourceImpact {
    /// Original immutable source, physical and optional research/context basis.
    pub from: super::super::OperationSource,
    /// Compared immutable source, physical and optional research/context basis.
    pub to: super::super::OperationSource,
    /// Results in requested order; no global-consumer coverage is implied.
    pub roots: Vec<ModelingRootSourceImpact>,
    _owner: Arc<pse_columnar::AllocationLease>,
}

fn selection_result(
    result: Result<canonical::CheckedSelection, WorkflowError>,
) -> Result<Result<ModelingSourceClosure, Arc<WorkflowError>>, WorkflowError> {
    match result {
        Ok(selected) => Ok(Ok(ModelingSourceClosure {
            admission: selected.admission,
        })),
        Err(error) => match error.boundary_diagnostic().class {
            BoundaryClass::ResourceLimit
            | BoundaryClass::Cancelled
            | BoundaryClass::Infrastructure
            | BoundaryClass::Internal => Err(error),
            _ => Ok(Err(Arc::new(error))),
        },
    }
}

impl ModelingPackage {
    /// Discover exact selected dependencies and compare their original lookup premises.
    /// Positive and absent namespace observations use the existing source-selection owner.
    /// Source admission/inspection does not establish ordinary scientific eligibility.
    ///
    /// # Errors
    /// Invalid finite bounds, cancellation, resource/infrastructure failures and service
    /// mismatches propagate. Individual source-query refusals remain in the root results.
    pub async fn selected_source_impact(
        &self,
        services: &crate::workflow::PreparationServices,
        other: &Self,
        roots: &[DeclarationId],
        maximum_members: usize,
        maximum_bytes: usize,
        cancel: &crate::CancelSource,
    ) -> Result<ModelingSourceImpact, WorkflowError> {
        self.check_service_bindings(services)?;
        other.check_service_bindings(services)?;
        if roots.is_empty()
            || roots.len() > 32
            || roots
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != roots.len()
            || maximum_members == 0
            || maximum_members > 1_000_000
            || maximum_bytes == 0
        {
            return Err(contract(
                "selected source impact requires distinct bounded roots and finite output",
            ));
        }
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let deadline = services.math().preparation_deadline(cancel)?;
                let phase = cancel.with_deadline(Some(deadline));
                preparation::scoped(&phase, deadline, async {
                    // Charge new observation metadata before owned growth. The admitted
                    // closure members are shared under their original retained owners.
                    let owner = services
                        .math()
                        .reserve("modeling:selected-source-impact", maximum_bytes)?;
                    let mut bytes = size_of::<ModelingSourceImpact>().saturating_add(
                        roots
                            .len()
                            .saturating_mul(size_of::<ModelingRootSourceImpact>() + 256),
                    );
                    if bytes > maximum_bytes {
                        return Err(contract(
                            "selected source impact metadata exceeds maximum_bytes",
                        ));
                    }
                    let mut members = 0usize;
                    let mut selected_roots = Vec::with_capacity(roots.len());
                    let context_changed = self.physical.key != other.physical.key
                        || self.research_source != other.research_source
                        || self.project_context != other.project_context
                        || self.providers.len() != other.providers.len()
                        || self.providers.iter().zip(other.providers.iter()).any(
                            |((a, x), (b, y))| {
                                a != b
                                    || x.spec().key() != y.spec().key()
                                    || x.configuration_key() != y.configuration_key()
                            },
                        );
                    for root in roots {
                        let before = selection_result(
                            self.checked_selection(services, &[*root], &phase).await,
                        )?;
                        let after = selection_result(
                            other.checked_selection(services, &[*root], &phase).await,
                        )?;
                        let status = match (&before, &after) {
                            (Ok(before), Ok(_)) => {
                                if context_changed
                                    || !before
                                        .admission
                                        .dependencies
                                        .matches_scoped(&other.revision.admitted, &phase)?
                                {
                                    ModelingSourceImpactStatus::Changed
                                } else {
                                    ModelingSourceImpactStatus::Unchanged
                                }
                            }
                            _ => ModelingSourceImpactStatus::Refused,
                        };
                        for closure in [&before, &after]
                            .into_iter()
                            .filter_map(|value| value.as_ref().ok())
                        {
                            members = members.saturating_add(closure.members().len());
                            if members > maximum_members {
                                return Err(contract(
                                    "selected source impact exceeds maximum_members",
                                ));
                            }
                            for (logical, version) in closure.members() {
                                if phase.token().is_cancelled() {
                                    return Err(crate::math::MathRuntimeError::Cancelled.into());
                                }
                                if std::time::Instant::now() >= deadline {
                                    return Err(super::super::deadline_limit(
                                        "selected source impact deadline",
                                    ));
                                }
                                bytes = bytes
                                    .saturating_add(logical.len())
                                    .saturating_add(version.len())
                                    .saturating_add(128);
                            }
                        }
                        if bytes > maximum_bytes {
                            return Err(contract("selected source impact exceeds maximum_bytes"));
                        }
                        selected_roots.push(ModelingRootSourceImpact {
                            root: *root,
                            before,
                            after,
                            status,
                        });
                    }
                    Ok(ModelingSourceImpact {
                        from: super::super::OperationSource::of(self),
                        to: super::super::OperationSource::of(other),
                        roots: selected_roots,
                        _owner: owner,
                    })
                })
                .await
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn selected_source_impact_is_bounded_and_preserves_exact_requested_coverage() {
        let runtime = crate::workflow::tests::runtime();
        let rows = pse_authoring::language::parse(
            "package p {def A {param k:Scalar=1; var x:Scalar; eq e:x==k;}} package q {def B {var y:Scalar; eq e:y==2;}}",
            SemanticId::NIL, pse_authoring::language::IdentityPolicy::Named, Default::default(),
        ).unwrap();
        let a = rows
            .iter()
            .find(|row| row.name == "A")
            .unwrap()
            .declaration_id;
        let b = rows
            .iter()
            .find(|row| row.name == "B")
            .unwrap()
            .declaration_id;
        let mut replacement = rows.iter().find(|row| row.name == "k").unwrap().clone();
        replacement.value.binding.as_mut().unwrap().expression = Some("2".into());
        let package = runtime
            .modeling_package(rows, crate::workflow::tests::physical())
            .await
            .unwrap();
        let cancel = crate::CancelSource::new();
        let changed = package
            .with_delta(
                &runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    replacements: vec![replacement],
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        let report = package
            .selected_source_impact(
                &runtime.preparation_services(),
                &changed,
                &[a, b, SemanticId::NIL.into()],
                10_000,
                4 << 20,
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(report.from, crate::workflow::OperationSource::of(&package));
        assert_eq!(report.to, crate::workflow::OperationSource::of(&changed));
        assert_eq!(report.roots.len(), 3);
        assert_eq!(report.roots[0].status, ModelingSourceImpactStatus::Changed);
        assert_eq!(
            report.roots[1].status,
            ModelingSourceImpactStatus::Unchanged
        );
        assert_eq!(report.roots[2].status, ModelingSourceImpactStatus::Refused);
        assert!(report.roots[2].before.is_err());
        assert!(report.roots[2].after.is_err());
        assert_ne!(
            report.roots[0].before.as_ref().unwrap().source_revision(),
            report.roots[0].after.as_ref().unwrap().source_revision()
        );
        let unchanged = package
            .selected_source_impact(
                &runtime.preparation_services(),
                &package,
                &[a, b],
                10_000,
                4 << 20,
                &cancel,
            )
            .await
            .unwrap();
        assert!(
            unchanged
                .roots
                .iter()
                .all(|root| root.status == ModelingSourceImpactStatus::Unchanged)
        );
        assert!(
            package
                .selected_source_impact(
                    &runtime.preparation_services(),
                    &changed,
                    &[a],
                    1,
                    4 << 20,
                    &cancel
                )
                .await
                .is_err()
        );
        assert!(
            package
                .selected_source_impact(
                    &runtime.preparation_services(),
                    &changed,
                    &[a],
                    10_000,
                    1,
                    &cancel
                )
                .await
                .is_err()
        );
        assert!(
            package
                .selected_source_impact(
                    &runtime.preparation_services(),
                    &changed,
                    &[a, a],
                    10_000,
                    4 << 20,
                    &cancel
                )
                .await
                .is_err()
        );
        cancel.cancel();
        assert!(
            package
                .selected_source_impact(
                    &runtime.preparation_services(),
                    &changed,
                    &[a],
                    10_000,
                    4 << 20,
                    &cancel
                )
                .await
                .is_err()
        );
    }

    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    async fn dataset_cells(
        package: &ModelingPackage,
        services: &crate::workflow::PreparationServices,
        dataset: DeclarationId,
        cancel: &crate::CancelSource,
    ) -> Vec<pse_model::generated::runtime::modeling_knowledge::Row> {
        use pse_relations::columnar::RelationRow;
        let knowledge = package
            .knowledge(services, Some(dataset), 10_000, 16 << 20, &cancel.token())
            .await
            .unwrap();
        pse_model::generated::runtime::modeling_knowledge::Row::rows(knowledge.table()).unwrap()
    }

    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn ordinary_selected_source_discovery_distinguishes_reimport_and_changed_science() {
        let (runtime, package, _spill, _) = pr_jacobian_tests::ordinary_delivery_package(false)
            .await
            .unwrap();
        let services = runtime.preparation_services();
        let cancel = crate::CancelSource::new();
        let rows = package
            .declaration_path(&services, &["ordinary_bt", "bt_vapor_caloric"], &cancel)
            .await
            .unwrap()
            .unwrap();
        let target = &rows[0];
        let mut roots = Vec::new();
        for name in ["ordinary_recycle", "ordinary_exchanger"] {
            let selected = package
                .declaration_path(&services, &["ordinary_delivery", name], &cancel)
                .await
                .unwrap()
                .unwrap();
            roots.push(selected[0].declaration_id);
        }
        let original_cells =
            dataset_cells(&package, &services, target.declaration_id, &cancel).await;
        assert!(!original_cells.is_empty());
        assert!(
            original_cells.iter().all(|row| row.source_id.is_some()
                && row.role_member_id.is_some()
                && !row.test_only)
        );
        assert!(
            original_cells
                .iter()
                .flat_map(|row| &row.value)
                .any(|node| node
                    .magnitude
                    .is_some_and(|value| (value - 29.524656757062152).abs() < 1e-10)
                    && node.quantity_type_id.is_some()
                    && node.canonical_unit_id.is_some()),
            "published 29524.656757062152 J/(kmol K) must independently read as29.524656757062152 J/(mol K)"
        );
        let artifact = include_bytes!(
            "../../../../../packages/reference/data/ordinary-bt/source-artifacts/feos-poling2000.json"
        );
        let delta = scientific_intake::feos_poling2000_bt_caloric_delta(
            package.revision_id(),
            target,
            artifact,
        )
        .unwrap();
        let reimported = package
            .with_delta(&runtime.source_services(), delta, &cancel)
            .await
            .unwrap();
        assert!(
            package
                .project_compare(&runtime.source_services(), &reimported, &cancel)
                .await
                .unwrap()
                .changes
                .is_empty()
        );
        let unchanged = package
            .selected_source_impact(&services, &reimported, &roots, 100_000, 16 << 20, &cancel)
            .await
            .unwrap();
        assert!(
            unchanged
                .roots
                .iter()
                .all(|root| root.status == ModelingSourceImpactStatus::Unchanged)
        );
        for root in &unchanged.roots {
            assert!(
                root.before
                    .as_ref()
                    .unwrap()
                    .members()
                    .contains_key(&format!("modeling:{}", target.declaration_id))
            );
        }

        // A deliberate candidate interpretation changes both coefficient and role.
        // This establishes source/update reporting, not empirical qualification.
        let mut replacement = target.clone();
        let dataset = replacement.value.dataset.as_mut().unwrap();
        dataset.rows[0].values[0]
            .value
            .quantity
            .as_mut()
            .unwrap()
            .magnitude += 1.;
        *dataset.provenance.role.last_mut().unwrap() = "derived".into();
        let changed = package
            .with_delta(
                &runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    replacements: vec![replacement],
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        let comparison = package
            .comparison_observation(
                &runtime.source_services(),
                &changed,
                &ModelingComparisonRequest {
                    source_only: false,
                    offset: 0,
                    limit: 16,
                    logical: vec![
                        format!("modeling:{}", target.declaration_id),
                        format!("document:{}", target.document_id),
                    ],
                },
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(comparison.total, 2, "{:?}", comparison.comparison.changes);
        assert_eq!(
            comparison
                .comparison
                .changes
                .iter()
                .map(|change| change.logical.clone())
                .collect::<std::collections::BTreeSet<_>>(),
            std::collections::BTreeSet::from([
                format!("modeling:{}", target.declaration_id),
                format!("document:{}", target.document_id),
            ]),
            "a typed scientific edit changes its declaration and retires its stale authoritative text"
        );
        let scientific_change = comparison
            .comparison
            .changes
            .iter()
            .find(|change| change.logical == format!("modeling:{}", target.declaration_id))
            .unwrap();
        assert!(
            scientific_change
                .impacts
                .contains(&ProjectChangeImpact::Structural)
        );
        assert!(!scientific_change.added && !scientific_change.removed);
        for (details, text_retained) in [(&comparison.before, true), (&comparison.after, false)] {
            let document = details
                .iter()
                .find(|detail| detail.logical == format!("document:{}", target.document_id))
                .unwrap();
            let Some(ProjectSourceObject::Document(document)) = &document.object else {
                panic!("exact source document metadata must remain inspectable");
            };
            assert_eq!(document.document.source_text.is_some(), text_retained);
        }
        let impact = package
            .selected_source_impact(&services, &changed, &roots, 100_000, 16 << 20, &cancel)
            .await
            .unwrap();
        assert_eq!(
            impact
                .roots
                .iter()
                .map(|root| root.root)
                .collect::<Vec<_>>(),
            roots
        );
        assert!(
            impact
                .roots
                .iter()
                .all(|root| root.status == ModelingSourceImpactStatus::Changed)
        );
        let changed_cells =
            dataset_cells(&changed, &services, target.declaration_id, &cancel).await;
        assert_ne!(
            original_cells[0].role_member_id,
            changed_cells[0].role_member_id
        );
        assert_ne!(original_cells, changed_cells);
        assert_eq!(
            dataset_cells(&package, &services, target.declaration_id, &cancel).await,
            original_cells,
            "a proposed revision cannot rewrite original admitted cells or source attribution"
        );
    }

    #[cfg(all(feature = "solver-kinsol", feature = "solver-root-isolation"))]
    #[tokio::test]
    async fn ordinary_source_discovery_keeps_absent_and_test_only_inputs_ineligible() {
        let (runtime, package, _spill, preparation) =
            pr_jacobian_tests::ordinary_source_package().await.unwrap();
        let services = runtime.preparation_services();
        let cancel = crate::CancelSource::new();
        let rows = package
            .declaration_path(&services, &["ordinary_bt", "bt_vapor_caloric"], &cancel)
            .await
            .unwrap()
            .unwrap();
        let target = &rows[0];
        let selected = package
            .declaration_path(&services, &["ordinary_bt", "Ideal"], &cancel)
            .await
            .unwrap()
            .unwrap();
        let root = selected[0].declaration_id;
        let mut replacement = target.clone();
        *replacement
            .value
            .dataset
            .as_mut()
            .unwrap()
            .provenance
            .role
            .last_mut()
            .unwrap() = "oracle_input".into();
        let ineligible = package
            .with_delta(
                &runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    replacements: vec![replacement],
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        let cells = dataset_cells(&ineligible, &services, target.declaration_id, &cancel).await;
        assert!(
            cells.iter().all(|row| row.test_only),
            "inspection must retain the original source-role restriction"
        );
        let refused = ineligible
            .prepare(
                &services,
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                preparation.limits,
                &cancel,
            )
            .await
            .unwrap_err();
        assert_eq!(
            refused.boundary_diagnostic().rule,
            pse_diagnostics::DiagnosticRule::ModelingProvenance,
            "the actual production read must refuse its restricted source: {refused}"
        );

        let mut replacement = target.clone();
        replacement.value.dataset.as_mut().unwrap().rows.remove(0);
        let absent = package
            .with_delta(
                &runtime.source_services(),
                SourceDelta {
                    parent: package.revision_id().into(),
                    replacements: vec![replacement],
                    ..Default::default()
                },
                &cancel,
            )
            .await
            .unwrap();
        let refused = absent
            .prepare(
                &services,
                root,
                pse_modeling::specialize::root_instance(root),
                Bindings::default(),
                preparation.limits,
                &cancel,
            )
            .await
            .unwrap_err();
        assert_eq!(
            refused.boundary_diagnostic().class,
            BoundaryClass::InvalidModel
        );
        assert!(
            !refused.boundary_diagnostic().sources.is_empty(),
            "missing demanded record must retain authored attribution: {refused}"
        );
        assert!(
            dataset_cells(&package, &services, target.declaration_id, &cancel)
                .await
                .iter()
                .all(|row| !row.test_only)
        );
    }
}

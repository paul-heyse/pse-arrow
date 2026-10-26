// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded reconstructible structural inputs. Only a selected ready ordinal constructs science.
use super::{
    ModelingPackage, OperationSource, SourceDelta, StudyOutputDemand, StudyPoint,
    StudyPointDefinition, StudyPointPolicy, WorkflowError, contract,
};
use pse_model::{
    document::Version,
    study::{OccurrenceGraph, PointPolicy, SeedNeed},
};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

/// Count serialization without retaining a second descriptor, and stop at the bound.
pub(in crate::workflow) fn document_extent(
    value: &impl serde::Serialize,
    limit: usize,
) -> Result<usize, WorkflowError> {
    struct Counter {
        bytes: usize,
        limit: usize,
        exceeded: bool,
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let Some(total) = self
                .bytes
                .checked_add(bytes.len())
                .filter(|n| *n <= self.limit)
            else {
                self.exceeded = true;
                return Err(io::Error::other("bounded study descriptor bytes"));
            };
            self.bytes = total;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        bytes: 0,
        limit,
        exceeded: false,
    };
    if let Err(error) = serde_json::to_writer(&mut counter, value) {
        return Err(if counter.exceeded {
            WorkflowError::Resource("bounded study descriptor bytes".into())
        } else {
            WorkflowError::Internal(format!("study descriptor serialization: {error}"))
        });
    }
    Ok(counter.bytes)
}

/// A finite structural dimension. Alternatives are edits against the one admitted base.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceChoice {
    /// Ordered alternatives; ordering is part of ordinal reconstruction.
    pub alternatives: Vec<SourceDelta>,
}
/// One selected Cartesian ordinal and its explicit scientific intent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectedStudyVariant {
    /// Last-choice-fast mixed-radix ordinal, independent of visible occurrence order.
    pub ordinal: u64,
    /// Operation, physically typed assignments, dependencies and original start policy.
    pub point: StudyPoint,
}
/// Immutable finite structural alternatives, without Cartesian materialization.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StudyVariants {
    /// Descriptor interpretation.
    pub version: Version<2>,
    /// Complete admitted base source and physical identity.
    pub source: OperationSource,
    /// Independent structural dimensions, applied in declared order.
    pub choices: Vec<SourceChoice>,
    /// Explicit selected ordinals in original visible occurrence order.
    pub selected: Vec<SelectedStudyVariant>,
}
/// Supported study inputs share the executor; neither form expands the other.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum StudyInputs {
    /// Explicit operation points admitted at the input boundary.
    Explicit(Vec<StudyPoint>),
    /// Structural variants admitted only when their selected occurrence becomes ready.
    Lazy(Box<StudyVariants>),
}
/// Frozen lazy input and its common output demand, retained once per study.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LazyStudyDefinition {
    /// Bounded immutable reconstructible science.
    pub variants: StudyVariants,
    /// Original completion/detail demand.
    pub output: StudyOutputDemand,
}
impl StudyVariants {
    /// Validate finite descriptor/cardinality and ordinals without constructing a variant.
    pub(in crate::workflow) fn validate(&self) -> Result<(), WorkflowError> {
        document_extent(self, pse_operations::canonical::PAYLOAD_BYTES / 2)?;
        if self.choices.is_empty() || self.selected.is_empty() {
            return Err(contract(
                "bounded lazy study dimensions and selected occurrences",
            ));
        }
        if self.choices.len() > 64 || self.selected.len() > super::MAXIMUM_STUDY_POINTS {
            return Err(WorkflowError::Resource(
                "lazy study dimensions or selected occurrences".into(),
            ));
        }
        let mut cardinality = 1_u64;
        let mut alternatives = 0_usize;
        let mut prior_objects = BTreeSet::new();
        for choice in &self.choices {
            if choice.alternatives.is_empty() {
                return Err(contract("empty structural choice"));
            }
            cardinality = cardinality
                .checked_mul(choice.alternatives.len() as u64)
                .ok_or_else(|| {
                    WorkflowError::Resource("structural ordinal cardinality overflow".into())
                })?;
            alternatives = alternatives
                .checked_add(choice.alternatives.len())
                .filter(|n| *n <= super::MAXIMUM_STUDY_POINTS)
                .ok_or_else(|| WorkflowError::Resource("structural alternative count".into()))?;
            let mut objects = BTreeSet::new();
            for delta in &choice.alternatives {
                let mut touched = BTreeSet::new();
                for (kind, ids) in [
                    (
                        "declaration",
                        delta
                            .additions
                            .iter()
                            .map(|r| r.declaration_id.to_string())
                            .chain(
                                delta
                                    .replacements
                                    .iter()
                                    .map(|r| r.declaration_id.to_string()),
                            )
                            .chain(delta.removals.iter().map(ToString::to_string))
                            .collect::<Vec<_>>(),
                    ),
                    (
                        "document",
                        delta
                            .documents
                            .additions
                            .iter()
                            .map(|r| r.document_id.to_string())
                            .chain(
                                delta
                                    .documents
                                    .replacements
                                    .iter()
                                    .map(|r| r.document_id.to_string()),
                            )
                            .chain(delta.documents.removals.iter().map(ToString::to_string))
                            .collect(),
                    ),
                    (
                        "fit",
                        delta
                            .fits
                            .additions
                            .iter()
                            .map(|r| r.fit_id.to_string())
                            .chain(delta.fits.replacements.iter().map(|r| r.fit_id.to_string()))
                            .chain(delta.fits.removals.iter().map(ToString::to_string))
                            .collect(),
                    ),
                ] {
                    for id in ids {
                        let key = (kind, id);
                        if !touched.insert(key.clone()) {
                            return Err(contract("overlapping source alternative intents"));
                        }
                        objects.insert(key);
                    }
                }
            }
            if !prior_objects.is_disjoint(&objects) {
                return Err(contract(
                    "structural dimensions edit overlapping source objects",
                ));
            }
            prior_objects.extend(objects);
        }
        let mut ordinals = BTreeSet::new();
        for selected in &self.selected {
            if selected.ordinal >= cardinality || !ordinals.insert(selected.ordinal) {
                return Err(contract(
                    "selected structural ordinal is out of range or repeated",
                ));
            }
        }
        pse_model::study_policy::admit(&self.graph())
            .map_err(|e| WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(e)))?;
        Ok(())
    }
    pub(in crate::workflow) fn check_parent(
        &self,
        package: &ModelingPackage,
    ) -> Result<(), WorkflowError> {
        self.validate()?;
        self.source.check(package)?;
        if self
            .choices
            .iter()
            .flat_map(|c| &c.alternatives)
            .any(|d| d.parent != package.revision_id())
        {
            return Err(contract(
                "structural alternative names a different admitted base token",
            ));
        }
        Ok(())
    }
    pub(in crate::workflow) fn retain_parent(&mut self, revision: &str) {
        for delta in self
            .choices
            .iter_mut()
            .flat_map(|choice| &mut choice.alternatives)
        {
            delta.parent = revision.into();
        }
    }
    pub(in crate::workflow) fn check_retained_parent(
        &self,
        revision: &str,
    ) -> Result<(), WorkflowError> {
        if self
            .choices
            .iter()
            .flat_map(|choice| &choice.alternatives)
            .any(|delta| delta.parent != revision)
        {
            return Err(contract(
                "lazy descriptor differs from its retained base revision",
            ));
        }
        Ok(())
    }
    pub(in crate::workflow) fn selected_policy(
        &self,
        selection: usize,
    ) -> Result<PointPolicy, WorkflowError> {
        self.selected
            .get(selection)
            .map(|selected| policy(&selected.point.policy))
            .ok_or_else(|| contract("selected structural occurrence absent"))
    }
    pub(in crate::workflow) fn graph(&self) -> OccurrenceGraph {
        OccurrenceGraph {
            points: self
                .selected
                .iter()
                .map(|selection| policy(&selection.point.policy))
                .collect(),
        }
    }
    /// Combine only the selected alternatives; cross-dimension object ownership is disjoint.
    pub(in crate::workflow) fn delta(
        &self,
        selection: usize,
    ) -> Result<SourceDelta, WorkflowError> {
        let selected = self
            .selected
            .get(selection)
            .ok_or_else(|| contract("selected variant index"))?;
        let mut ordinal = selected.ordinal;
        let mut indices = vec![0_usize; self.choices.len()];
        for (index, choice) in self.choices.iter().enumerate().rev() {
            let radix = choice.alternatives.len() as u64;
            if radix == 0 {
                return Err(contract("empty structural choice"));
            }
            indices[index] = (ordinal % radix) as usize;
            ordinal /= radix;
        }
        if ordinal != 0 {
            return Err(contract("structural ordinal out of range"));
        }
        let mut combined = SourceDelta::default();
        for (choice, index) in self.choices.iter().zip(indices) {
            let delta = &choice.alternatives[index];
            combined.additions.extend(delta.additions.clone());
            combined.replacements.extend(delta.replacements.clone());
            combined.removals.extend(delta.removals.clone());
            combined
                .documents
                .additions
                .extend(delta.documents.additions.clone());
            combined
                .documents
                .replacements
                .extend(delta.documents.replacements.clone());
            combined
                .documents
                .removals
                .extend(delta.documents.removals.clone());
            combined.fits.additions.extend(delta.fits.additions.clone());
            combined
                .fits
                .replacements
                .extend(delta.fits.replacements.clone());
            combined.fits.removals.extend(delta.fits.removals.clone());
        }
        Ok(combined)
    }
}
fn policy(requested: &StudyPointPolicy) -> PointPolicy {
    PointPolicy {
        key: requested.key,
        dependencies: requested.dependencies.clone(),
        seed_need: SeedNeed::Pending,
        start: requested.start.clone(),
        attempt_limit: requested.attempt_limit,
    }
}
impl LazyStudyDefinition {
    pub(in crate::workflow) async fn admit_selected(
        &self,
        services: &super::SourceServices,
        base: &ModelingPackage,
        selection: usize,
        occurrence: Option<&pse_operations::canonical_studies::StudyAdmissionAuthority>,
        cancel: &crate::CancelSource,
    ) -> Result<(ModelingPackage, StudyPointDefinition), WorkflowError> {
        let no_publication = |cause| WorkflowError::Admission {
            effect: super::AdmissionEffect::NoPublication,
            cause: Box::new(cause),
        };
        self.variants.source.check(base).map_err(no_publication)?;
        let requested = self.variants.selected.get(selection).ok_or_else(|| {
            no_publication(WorkflowError::Internal(
                "selected admitted variant absent".into(),
            ))
        })?;
        let delta = self.variants.delta(selection).map_err(no_publication)?;
        let mut package = base
            .construct_study_variant(services, &self.variants.source, delta, cancel)
            .await
            .map_err(no_publication)?;
        let effect = if let Some(occurrence) = occurrence {
            package = package
                .record_study_variant(services, occurrence, cancel)
                .await
                .map_err(|cause| WorkflowError::Admission {
                    effect: super::diagnostics::publication_failure_effect(&cause),
                    cause: Box::new(cause),
                })?;
            super::AdmissionEffect::SourceAcknowledged {
                revision: package
                    .recorded_revision()
                    .map_err(|cause| WorkflowError::Admission {
                        effect: super::AdmissionEffect::UnresolvedPublication,
                        cause: Box::new(cause),
                    })?
                    .key
                    .clone(),
            }
        } else {
            super::AdmissionEffect::NoPublication
        };
        let point = package
            .admit_one_study_point(
                services.preparation(),
                &requested.point,
                &self.output,
                cancel,
            )
            .await
            .map_err(|cause| WorkflowError::Admission {
                effect,
                cause: Box::new(cause),
            })?;
        Ok((package, point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{CaseOperation, OperationRequest, PointOverlay, PreparationSettings};
    use pse_ids::ContentHash;
    use pse_model::{
        generated::{enums::ModelingAnalysisRoute, identities::DeclarationId},
        study::{OccurrenceKey, StartPolicy},
    };
    fn descriptor() -> StudyVariants {
        let hash = ContentHash::from_bytes([1; 32]);
        let point = StudyPoint {
            operation: OperationRequest::DeclaredCase(CaseOperation {
                case: DeclarationId::from_bytes([8; 16]),
                route: ModelingAnalysisRoute::Steady,
                settings: Default::default(),
            }),
            preparation: PreparationSettings::default(),
            overlay: PointOverlay::default(),
            policy: StudyPointPolicy {
                key: OccurrenceKey(7),
                dependencies: vec![],
                start: StartPolicy::Fresh,
                attempt_limit: 1,
            },
        };
        let alternative = |id| SourceDelta {
            parent: "admitted-token".into(),
            removals: vec![DeclarationId::from_bytes([id; 16])],
            ..Default::default()
        };
        StudyVariants {
            version: Version,
            source: OperationSource {
                research: None,
                context: None,
                revision: hash.into(),
                physical_context: hash,
            },
            choices: vec![
                SourceChoice {
                    alternatives: vec![alternative(1), alternative(2)],
                },
                SourceChoice {
                    alternatives: vec![alternative(3), alternative(4), alternative(5)],
                },
            ],
            selected: vec![SelectedStudyVariant { ordinal: 4, point }],
        }
    }
    #[test]
    fn lazy_ordinals_reconstruct_only_selected_disjoint_alternatives() {
        let variants = descriptor();
        variants.validate().unwrap();
        let delta = variants.delta(0).unwrap();
        assert_eq!(
            delta.removals,
            vec![
                DeclarationId::from_bytes([2; 16]),
                DeclarationId::from_bytes([4; 16])
            ]
        );
        assert_eq!(variants.graph().points[0].seed_need, SeedNeed::Pending);
        let bytes = serde_json::to_vec(&variants).unwrap();
        let reopened: StudyVariants = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(reopened.delta(0).unwrap().removals, delta.removals);
        assert_eq!(reopened.graph(), variants.graph());
    }
    #[test]
    fn lazy_descriptor_counting_refuses_at_bound_without_a_serialized_copy() {
        assert_eq!(document_extent(&vec![1_u8, 2, 3], 7).unwrap(), 7);
        let error = document_extent(&vec![1_u8, 2, 3], 6).unwrap_err();
        assert_eq!(
            error.boundary_diagnostic().class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit
        );
        let variants = descriptor();
        let length = serde_json::to_vec(&variants).unwrap().len();
        assert_eq!(document_extent(&variants, length).unwrap(), length);
        assert!(document_extent(&variants, length - 1).is_err());
    }
    #[test]
    fn lazy_descriptor_refuses_overlap_duplicates_out_of_range_and_overflow() {
        let original = descriptor();
        let mut overlap = original.clone();
        overlap.choices[1].alternatives[0] = overlap.choices[0].alternatives[0].clone();
        assert!(overlap.validate().is_err());
        let mut duplicate = original.clone();
        let mut second = duplicate.selected[0].clone();
        second.point.policy.key = OccurrenceKey(8);
        duplicate.selected.push(second);
        assert!(duplicate.validate().is_err());
        let mut outside = original.clone();
        outside.selected[0].ordinal = 6;
        assert!(outside.validate().is_err());
        let mut overflow = original;
        let empty = SourceChoice {
            alternatives: vec![SourceDelta::default(), SourceDelta::default()],
        };
        overflow.choices = vec![empty; 64];
        let error = overflow.validate().unwrap_err();
        assert_eq!(
            error.boundary_diagnostic().class,
            pse_model::diagnostic::BoundaryClass::ResourceLimit
        );
    }
}

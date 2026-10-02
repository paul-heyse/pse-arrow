// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical assignments are resolved once against an immutable selected revision.
use super::{ModelingPackage, WorkflowError};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_model::scalars::FiniteBound;
use pse_quantity::{CanonicalConversionPlan, IndexSet, ResolvedPhysicalContract};
use std::collections::BTreeMap;

/// A submitted reference; paths are attribution after admission.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum BindingTarget {
    /// Authored member path in the selected revision.
    Path(String),
    /// Stable instantiated member identity.
    Member(SemanticId),
}
/// One finite quantity with complete declared physical meaning and a representation unit.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BindingQuantity {
    /// Supplied finite magnitude.
    pub magnitude: FiniteBound,
    /// Complete quantity type, including basis, datum and subject.
    pub quantity: SemanticId,
    /// Registered representation unit.
    pub unit: SemanticId,
}
/// An assignment before contextual admission.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BindingAssignment {
    /// Supplied member reference.
    pub target: BindingTarget,
    /// Physically typed value; bare numbers are not input values.
    pub value: BindingQuantity,
}
/// Submitted replacements; the list preserves duplicate references for validation.
#[derive(
    Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PointOverlay {
    /// Each assignment is admitted against the same selected revision.
    #[serde(default)]
    pub assignments: Vec<BindingAssignment>,
}
/// Canonical input with its expected physical contract and original attribution.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdmittedBindingEntry {
    /// Stable selected member.
    pub member: SemanticId,
    /// Expected complete physical quantity.
    pub quantity: SemanticId,
    /// Finite canonical coordinate.
    pub canonical: FiniteBound,
    /// Whether the target is an authored parameter rather than a variable start/value.
    pub parameter: bool,
    /// Original input reference, retained only for attribution.
    pub supplied: BindingTarget,
    /// Original unit, retained only for attribution.
    pub supplied_unit: SemanticId,
}
/// Immutable selected-revision binding; attribution is excluded from content identity.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AdmittedBinding {
    /// Exact revision that admitted member identity and role.
    pub revision: ContentHash,
    /// Physical interpretation context.
    pub context: ContentHash,
    /// Entries in member identity order.
    pub entries: BTreeMap<SemanticId, AdmittedBindingEntry>,
}
impl AdmittedBinding {
    /// Content identity is independent of source spelling, supplied representation and occurrence.
    pub fn identity(&self) -> ContentHash {
        let mut h = FramedHasher::new(pse_ids::Frame::StudyBindingV1);
        h.hash(&self.context).u64(self.entries.len() as u64);
        for (member, entry) in &self.entries {
            h.id(member)
                .id(&entry.quantity)
                .bool(entry.parameter)
                .u64(entry.canonical.into_inner().to_bits());
        }
        h.finish_hash()
    }
    /// Canonical scalar projection; consumers perform no unit conversion.
    pub fn values(&self) -> BTreeMap<SemanticId, f64> {
        self.entries
            .iter()
            .map(|(id, entry)| (*id, entry.canonical.into_inner()))
            .collect()
    }
}
impl ModelingPackage {
    /// Resolve submitted paths through the compiler's lexical/indexed path owner at admission.
    pub(in crate::workflow) async fn admit_operation_overlay(
        &self,
        operation: &super::StudyOperation,
        overlay: &PointOverlay,
        cancel: &crate::CancelSource,
    ) -> Result<AdmittedBinding, WorkflowError> {
        if let super::OperationRequest::DeclaredCase(case) = &operation.operation {
            let execution = self
                .declared_execution(
                    case.case,
                    operation.preparation.compiler,
                    case.settings.clone().profile()?,
                    Default::default(),
                    operation.preparation.limits,
                    cancel,
                )
                .await?;
            let mut bindings = execution.analysis.bindings.clone();
            bindings
                .demand
                .extend(overlay.assignments.iter().filter_map(
                    |assignment| match &assignment.target {
                        BindingTarget::Path(path) => Some(path.clone()),
                        BindingTarget::Member(_) => None,
                    },
                ));
            bindings.demand.sort();
            bindings.demand.dedup();
            let model = self
                .prepare(
                    execution.analysis.root,
                    execution.analysis.instance,
                    bindings,
                    execution.analysis.limits,
                    cancel,
                )
                .await?;
            return self.admit_overlay(model.compiled(), overlay);
        }
        if !overlay.assignments.is_empty() {
            return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::Unsupported,
                pse_diagnostics::DiagnosticStage::StudyBinding,
                [],
                pse_diagnostics::DiagnosticRule::StudyOperationUnsupported,
            )
            .into());
        }
        Ok(AdmittedBinding {
            revision: self.revision.identity(),
            context: self.physical.identity(),
            entries: BTreeMap::new(),
        })
    }
    /// Reconstruct the existing operation and apply its admitted coordinates once.
    pub(in crate::workflow) async fn prepare_bound_operation(
        &self,
        operation: &super::StudyOperation,
        binding: &AdmittedBinding,
        cancel: &crate::CancelSource,
    ) -> Result<super::PreparedStudyOperation, WorkflowError> {
        operation.source.check(self)?;
        if let super::OperationRequest::DeclaredCase(case) = &operation.operation {
            let mut execution = self
                .declared_execution(
                    case.case,
                    operation.preparation.compiler,
                    case.settings.clone().profile()?,
                    Default::default(),
                    operation.preparation.limits,
                    cancel,
                )
                .await?;
            if execution.route != case.route
                || !matches!(execution.procedure, super::DeclaredProcedure::Solve)
            {
                return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                    pse_model::diagnostic::BoundaryClass::Unsupported,
                    pse_diagnostics::DiagnosticStage::StudyAdmission,
                    [case.case.as_id()],
                    pse_diagnostics::DiagnosticRule::StudyOperationUnsupported,
                )
                .into());
            }
            self.validate_binding(execution.model.compiled(), binding)?;
            execution.analysis.case.members = binding.values();
            return Ok(super::PreparedStudyOperation::DeclaredCase(Box::new(
                self.prepare_declared(&execution, cancel).await?,
            )));
        }
        if !binding.entries.is_empty()
            || binding.revision != self.revision.identity()
            || binding.context != self.physical.identity()
        {
            return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::Unsupported,
                pse_diagnostics::DiagnosticStage::StudyBinding,
                binding.entries.keys().copied(),
                pse_diagnostics::DiagnosticRule::StudyOperationUnsupported,
            )
            .into());
        }
        operation.prepare(self, cancel).await
    }
    /// Admit one operation-owned parameter or prior coordinate through the same physical contract.
    pub(in crate::workflow) fn admit_target_quantity(
        &self,
        model: &pse_compiler::workspace::PreparedModeling,
        target: SemanticId,
        quantity: &BindingQuantity,
    ) -> Result<AdmittedBindingEntry, WorkflowError> {
        let mut binding = self.admit_overlay(
            model,
            &PointOverlay {
                assignments: vec![BindingAssignment {
                    target: BindingTarget::Member(target),
                    value: quantity.clone(),
                }],
            },
        )?;
        binding
            .entries
            .pop_first()
            .map(|(_, entry)| entry)
            .ok_or_else(|| {
                WorkflowError::Internal("admitted single assignment has no entry".into())
            })
    }
    /// Resolve references, physical meaning and duplicate assignments exactly once.
    pub fn admit_overlay(
        &self,
        model: &pse_compiler::workspace::PreparedModeling,
        overlay: &PointOverlay,
    ) -> Result<AdmittedBinding, WorkflowError> {
        use pse_diagnostics::{DiagnosticRule as R, DiagnosticStage as S};
        use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic};
        let failure = |member, rule, target: &BindingTarget| {
            let mut diagnostic = BoundaryDiagnostic::new(
                BoundaryClass::InvalidModel,
                S::StudyBinding,
                [member],
                rule,
            );
            diagnostic
                .locations
                .push(pse_model::diagnostic::SourceLocation {
                    source: member,
                    revision: Some(self.revision.identity()),
                    path: match target {
                        BindingTarget::Path(path) => path.clone(),
                        BindingTarget::Member(id) => id.to_string(),
                    },
                    name: None,
                    start: None,
                    end: None,
                });
            diagnostic
        };
        let mut entries: BTreeMap<SemanticId, AdmittedBindingEntry> = BTreeMap::new();
        for assignment in &overlay.assignments {
            let member = match &assignment.target {
                BindingTarget::Member(id) => *id,
                BindingTarget::Path(path) => *model.model.paths.get(path).ok_or_else(|| {
                    WorkflowError::from(failure(
                        SemanticId::NIL,
                        R::StudyBindingTarget,
                        &assignment.target,
                    ))
                })?,
            };
            let symbol = model
                .model
                .symbols
                .get(&member)
                .filter(|symbol| {
                    model.admitted.inputs.contains(&member)
                        && matches!(
                            symbol.role,
                            pse_model::generated::enums::ModelingDeclarationKind::Variable
                                | pse_model::generated::enums::ModelingDeclarationKind::Parameter
                        )
                })
                .ok_or_else(|| {
                    WorkflowError::from(failure(member, R::StudyBindingTarget, &assignment.target))
                })?;
            let expected = symbol
                .ty
                .quantity_scheme()
                .ok_or_else(|| {
                    WorkflowError::from(failure(
                        member,
                        R::StudyBindingPhysical,
                        &assignment.target,
                    ))
                })?
                .resolve_contract_with_evidence(
                    &self.quantities,
                    &BTreeMap::new(),
                    &pse_quantity::infer::NoInvariantFacts,
                )
                .map_err(|cause| {
                    let mut diagnostic =
                        failure(member, R::StudyBindingPhysical, &assignment.target);
                    diagnostic.causes.push(pse_model::diagnostic::project_typed(
                        &cause,
                        S::StudyBinding,
                    ));
                    WorkflowError::from(diagnostic)
                })?;
            let supplied = ResolvedPhysicalContract::named(
                assignment.value.quantity.into(),
                IndexSet::new(),
                &self.quantities,
            )
            .map_err(|cause| super::math(pse_math::MathError::Quantity(cause)))?;
            if !expected.same_meaning(&supplied) {
                let mut diagnostic = failure(member, R::StudyBindingPhysical, &assignment.target);
                diagnostic.observations.insert(
                    "operands".into(),
                    pse_model::diagnostic::Observation::Contracts(vec![
                        operand(&expected),
                        operand(&supplied),
                    ]),
                );
                return Err(diagnostic.into());
            }
            let quantity = expected.named_id().ok_or_else(|| {
                WorkflowError::from(failure(member, R::StudyBindingPhysical, &assignment.target))
            })?;
            let canonical_unit = self
                .quantities
                .quantity_type(quantity)
                .map_err(|cause| super::math(pse_math::MathError::Quantity(cause)))?
                .canonical_unit;
            let plan = if assignment.value.unit == canonical_unit.as_id() {
                // A's already-canonical admission retains exact bits, including signed zero.
                CanonicalConversionPlan::canonical(&self.quantities, quantity)
            } else {
                CanonicalConversionPlan::registered(
                    &self.quantities,
                    quantity,
                    assignment.value.unit.into(),
                )
            };
            let canonical = plan
                .and_then(|plan| plan.apply(assignment.value.magnitude.into_inner()))
                .map_err(|cause| super::math(pse_math::MathError::Quantity(cause)))?;
            let canonical = FiniteBound::try_new(canonical.value()).map_err(|_| {
                WorkflowError::from(failure(member, R::StudyBindingPhysical, &assignment.target))
            })?;
            let entry = AdmittedBindingEntry {
                member,
                quantity: quantity.as_id(),
                canonical,
                parameter: symbol.role
                    == pse_model::generated::enums::ModelingDeclarationKind::Parameter,
                supplied: assignment.target.clone(),
                supplied_unit: assignment.value.unit,
            };
            if let Some(previous) = entries.get(&member) {
                if previous.quantity != entry.quantity
                    || previous.canonical.into_inner().to_bits()
                        != entry.canonical.into_inner().to_bits()
                    || previous.parameter != entry.parameter
                {
                    return Err(
                        failure(member, R::StudyBindingDuplicate, &assignment.target).into(),
                    );
                }
            } else {
                entries.insert(member, entry);
            }
        }
        Ok(AdmittedBinding {
            revision: self.revision.identity(),
            context: self.physical.identity(),
            entries,
        })
    }
    /// Validate recorded admitted coordinates without resolving attribution or converting units.
    pub(in crate::workflow) fn validate_binding(
        &self,
        model: &pse_compiler::workspace::PreparedModeling,
        binding: &AdmittedBinding,
    ) -> Result<(), WorkflowError> {
        if binding.revision != self.revision.identity()
            || binding.context != self.physical.identity()
        {
            return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::Incompatible,
                pse_diagnostics::DiagnosticStage::StudyBinding,
                [],
                pse_diagnostics::DiagnosticRule::StudyBindingRevision,
            )
            .into());
        }
        for (member, entry) in &binding.entries {
            let symbol = model.model.symbols.get(member).filter(|symbol| {
                model.admitted.inputs.contains(member)
                    && matches!(
                        symbol.role,
                        pse_model::generated::enums::ModelingDeclarationKind::Variable
                            | pse_model::generated::enums::ModelingDeclarationKind::Parameter
                    )
            });
            let valid = symbol.is_some_and(|symbol| {
                *member == entry.member
                    && (symbol.role
                        == pse_model::generated::enums::ModelingDeclarationKind::Parameter)
                        == entry.parameter
                    && symbol
                        .ty
                        .quantity_scheme()
                        .and_then(|scheme| {
                            scheme
                                .resolve_contract_with_evidence(
                                    &self.quantities,
                                    &BTreeMap::new(),
                                    &pse_quantity::infer::NoInvariantFacts,
                                )
                                .ok()
                        })
                        .and_then(|contract| contract.named_id())
                        == Some(entry.quantity.into())
            });
            if !valid {
                return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                    pse_model::diagnostic::BoundaryClass::Incompatible,
                    pse_diagnostics::DiagnosticStage::StudyBinding,
                    [*member],
                    pse_diagnostics::DiagnosticRule::StudyBindingPhysical,
                )
                .into());
            }
        }
        Ok(())
    }
}

fn operand(contract: &ResolvedPhysicalContract) -> pse_diagnostics::OperandContract {
    pse_diagnostics::OperandContract {
        quantity: contract
            .named_id()
            .map_or([0; 16], |id| *id.as_id().as_bytes()),
        indices: contract
            .indices()
            .iter()
            .map(|index| {
                [
                    *index.bound_index.as_id().as_bytes(),
                    *index.domain.as_id().as_bytes(),
                    *index.kind.as_id().as_bytes(),
                ]
            })
            .collect(),
    }
}

#[cfg(test)]
mod binding_admission_unit {
    use super::super::tests::{physical, runtime};
    use super::*;
    use pse_diagnostics::DiagnosticRule;
    use pse_model::diagnostic::Observation;

    async fn fixture(name: &str) -> (ModelingPackage, crate::math::modeling::ModelingPreparation) {
        let mut context = physical();
        let mut builder = context.quantities.to_builder();
        let mut bar = context
            .quantities
            .units()
            .find(|unit| unit.symbol == "Pa")
            .unwrap()
            .clone();
        bar.id =
            pse_quantity::UnitId::from_id(pse_ids::named_id(bar.id.as_id(), "binding-test-bar"));
        bar.symbol = "bar".into();
        bar.scale_to_canonical = 100000.;
        builder.unit(bar);
        context.quantities = std::sync::Arc::new(builder.build().unwrap());
        context.key =
            pse_compiler::workspace::physical_identity(&context.quantities, &context.preconditions);
        fixture_with(name, true, context).await
    }
    async fn fixture_with(
        name: &str,
        explicit: bool,
        physical: crate::workflow::PhysicalContext,
    ) -> (ModelingPackage, crate::math::modeling::ModelingPreparation) {
        let identity = if explicit {
            "@id(\"11111111111111111111111111111111\")"
        } else {
            ""
        };
        let source = format!(
            "package p {{ def Root {{ {identity} param {name}: Temperature = 300{{K}}; param pressure: Pressure = 100000{{Pa}}; let derived: Temperature = {name}; }} }}"
        );
        let rows = pse_authoring::language::parse(
            &source,
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime().modeling_package(rows, physical).unwrap();
        let model = package
            .prepare(
                root,
                pse_modeling::specialize::root_instance(root),
                pse_modeling::Bindings {
                    demand: vec![name.into(), "pressure".into(), "derived".into()],
                    ..Default::default()
                },
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        (package, model)
    }
    fn assignment(
        package: &ModelingPackage,
        target: BindingTarget,
        magnitude: f64,
        quantity: &str,
        unit: &str,
    ) -> BindingAssignment {
        BindingAssignment {
            target,
            value: BindingQuantity {
                magnitude: FiniteBound::try_new(magnitude).unwrap(),
                quantity: package
                    .quantities
                    .quantity_types()
                    .find(|q| q.name.as_deref() == Some(quantity))
                    .unwrap()
                    .id
                    .as_id(),
                unit: package
                    .quantities
                    .units()
                    .find(|u| u.symbol == unit)
                    .unwrap()
                    .id
                    .as_id(),
            },
        }
    }
    #[tokio::test]
    async fn affine_and_equivalent_pressure_units_admit_once() {
        let (package, model) = fixture("temperature").await;
        let a = package
            .admit_overlay(
                model.compiled(),
                &PointOverlay {
                    assignments: vec![
                        assignment(
                            &package,
                            BindingTarget::Path("temperature".into()),
                            80.,
                            "Temperature",
                            "degC",
                        ),
                        assignment(
                            &package,
                            BindingTarget::Path("pressure".into()),
                            1.,
                            "Pressure",
                            "bar",
                        ),
                    ],
                },
            )
            .unwrap();
        let b = package
            .admit_overlay(
                model.compiled(),
                &PointOverlay {
                    assignments: vec![
                        assignment(
                            &package,
                            BindingTarget::Path("pressure".into()),
                            100000.,
                            "Pressure",
                            "Pa",
                        ),
                        assignment(
                            &package,
                            BindingTarget::Member(model.compiled().model.paths["temperature"]),
                            353.15,
                            "Temperature",
                            "K",
                        ),
                    ],
                },
            )
            .unwrap();
        assert_eq!(a.identity(), b.identity());
        assert_eq!(
            a.values()[&model.compiled().model.paths["temperature"]],
            353.15
        );
        assert!(a.entries.values().all(|entry| entry.parameter));
        let encoded = serde_json::to_vec(&a).unwrap();
        let decoded = serde_json::from_slice(&encoded).unwrap();
        package
            .validate_binding(model.compiled(), &decoded)
            .unwrap();
    }
    #[tokio::test]
    async fn conflicting_path_and_member_aliases_refuse_without_last_write_wins() {
        let (package, model) = fixture("temperature").await;
        let first = assignment(
            &package,
            BindingTarget::Path("temperature".into()),
            80.,
            "Temperature",
            "degC",
        );
        let identical = assignment(
            &package,
            BindingTarget::Member(model.compiled().model.paths["temperature"]),
            353.15,
            "Temperature",
            "K",
        );
        assert_eq!(
            package
                .admit_overlay(
                    model.compiled(),
                    &PointOverlay {
                        assignments: vec![first.clone(), identical]
                    }
                )
                .unwrap()
                .entries
                .len(),
            1
        );
        let conflicting = assignment(
            &package,
            BindingTarget::Member(model.compiled().model.paths["temperature"]),
            353.,
            "Temperature",
            "K",
        );
        let error = package
            .admit_overlay(
                model.compiled(),
                &PointOverlay {
                    assignments: vec![first, conflicting],
                },
            )
            .unwrap_err()
            .boundary_diagnostic();
        assert_eq!(error.rule, DiagnosticRule::StudyBindingDuplicate);
        assert_eq!(
            error.locations[0].revision,
            Some(package.revision.identity())
        );
    }
    #[tokio::test]
    async fn conflicting_signed_zero_assignments_refuse_in_either_order() {
        let (package, model) = fixture("temperature").await;
        for magnitudes in [[0.0, -0.0], [-0.0, 0.0]] {
            let overlay = PointOverlay {
                assignments: magnitudes
                    .into_iter()
                    .map(|value| {
                        assignment(
                            &package,
                            BindingTarget::Path("temperature".into()),
                            value,
                            "Temperature",
                            "K",
                        )
                    })
                    .collect(),
            };
            assert_eq!(
                package
                    .admit_overlay(model.compiled(), &overlay)
                    .unwrap_err()
                    .boundary_diagnostic()
                    .rule,
                DiagnosticRule::StudyBindingDuplicate
            );
        }
    }
    #[tokio::test]
    async fn unit_mismatch_retains_both_contracts_and_revision_location() {
        let (package, model) = fixture("temperature").await;
        let error = package
            .admit_overlay(
                model.compiled(),
                &PointOverlay {
                    assignments: vec![assignment(
                        &package,
                        BindingTarget::Path("temperature".into()),
                        1.,
                        "Pressure",
                        "Pa",
                    )],
                },
            )
            .unwrap_err()
            .boundary_diagnostic();
        assert_eq!(error.rule, DiagnosticRule::StudyBindingPhysical);
        assert!(
            matches!(&error.observations["operands"],Observation::Contracts(values) if values.len()==2 && values[0].quantity != values[1].quantity)
        );
        let decoded: pse_model::diagnostic::BoundaryDiagnostic =
            serde_json::from_slice(&serde_json::to_vec(&error).unwrap()).unwrap();
        assert_eq!(decoded.locations[0].path, "temperature");
        assert_eq!(
            decoded.locations[0].revision,
            Some(package.revision.identity())
        );
    }
    #[tokio::test]
    async fn derived_unknown_and_tampered_targets_cannot_be_admitted() {
        let (package, model) = fixture("temperature").await;
        for path in ["derived", "unknown"] {
            let error = package
                .admit_overlay(
                    model.compiled(),
                    &PointOverlay {
                        assignments: vec![assignment(
                            &package,
                            BindingTarget::Path(path.into()),
                            300.,
                            "Temperature",
                            "K",
                        )],
                    },
                )
                .unwrap_err()
                .boundary_diagnostic();
            assert_eq!(error.rule, DiagnosticRule::StudyBindingTarget);
        }
        let mut binding = package
            .admit_overlay(
                model.compiled(),
                &PointOverlay {
                    assignments: vec![assignment(
                        &package,
                        BindingTarget::Path("temperature".into()),
                        300.,
                        "Temperature",
                        "K",
                    )],
                },
            )
            .unwrap();
        binding.entries.values_mut().next().unwrap().parameter = false;
        assert!(
            package
                .validate_binding(model.compiled(), &binding)
                .is_err()
        );
    }
    #[tokio::test]
    async fn compatible_explicit_member_rename_retains_content_but_old_revision_refuses() {
        let (before, old_model) = fixture("temperature").await;
        let (after, new_model) = fixture("renamed").await;
        let old = before
            .admit_overlay(
                old_model.compiled(),
                &PointOverlay {
                    assignments: vec![assignment(
                        &before,
                        BindingTarget::Path("temperature".into()),
                        300.,
                        "Temperature",
                        "K",
                    )],
                },
            )
            .unwrap();
        let new = after
            .admit_overlay(
                new_model.compiled(),
                &PointOverlay {
                    assignments: vec![assignment(
                        &after,
                        BindingTarget::Path("renamed".into()),
                        300.,
                        "Temperature",
                        "K",
                    )],
                },
            )
            .unwrap();
        assert_eq!(
            old.entries.keys().collect::<Vec<_>>(),
            new.entries.keys().collect::<Vec<_>>()
        );
        assert_eq!(old.identity(), new.identity());
        assert_ne!(old.revision, new.revision);
        assert!(after.validate_binding(new_model.compiled(), &old).is_err());
    }
    #[tokio::test]
    async fn same_dimension_wrong_basis_datum_and_subject_refuse_complete_contracts() {
        use pse_quantity::QuantityTypeId;
        let mut context = physical();
        let original = context
            .quantities
            .quantity_types()
            .find(|q| q.name.as_deref() == Some("Temperature"))
            .unwrap()
            .clone();
        let mut builder = context.quantities.to_builder();
        let controls: Vec<_> = ["basis", "datum", "subject"]
            .into_iter()
            .map(|difference| {
                let mut control = original.clone();
                control.id =
                    QuantityTypeId::from_id(pse_ids::named_id(original.id.as_id(), difference));
                control.name = Some(format!("Wrong{difference}"));
                match difference {
                    "basis" => {
                        control.key.basis = Some(
                            context
                                .quantities
                                .bases()
                                .find(|b| Some(b.id) != original.key.basis)
                                .unwrap()
                                .id,
                        )
                    }
                    "datum" => {
                        control.key.reference_state = Some(
                            context
                                .quantities
                                .reference_states()
                                .find(|r| Some(r.id) != original.key.reference_state)
                                .unwrap()
                                .id,
                        )
                    }
                    "subject" => {
                        control.key.subject_kind = Some(
                            context
                                .quantities
                                .entity_kinds()
                                .find(|e| Some(e.id) != original.key.subject_kind)
                                .unwrap()
                                .id,
                        )
                    }
                    _ => unreachable!(),
                }
                assert_eq!(control.key.kind, original.key.kind);
                assert_eq!(control.canonical_unit, original.canonical_unit);
                builder.quantity_type(control.clone());
                control
            })
            .collect();
        context.quantities = std::sync::Arc::new(builder.build().unwrap());
        context.key =
            pse_compiler::workspace::physical_identity(&context.quantities, &context.preconditions);
        let (package, model) = fixture_with("temperature", true, context).await;
        for control in controls {
            let mut supplied = assignment(
                &package,
                BindingTarget::Path("temperature".into()),
                300.,
                "Temperature",
                "K",
            );
            supplied.value.quantity = control.id.as_id();
            let diagnostic = package
                .admit_overlay(
                    model.compiled(),
                    &PointOverlay {
                        assignments: vec![supplied],
                    },
                )
                .unwrap_err()
                .boundary_diagnostic();
            assert_eq!(
                diagnostic.rule,
                DiagnosticRule::StudyBindingPhysical,
                "{:?}",
                control.name
            );
            let Observation::Contracts(operands) = &diagnostic.observations["operands"] else {
                panic!("{diagnostic:?}")
            };
            assert_eq!(operands.len(), 2);
            assert_eq!(operands[0].quantity, *original.id.as_id().as_bytes());
            assert_eq!(operands[1].quantity, *control.id.as_id().as_bytes());
            assert_eq!(
                diagnostic.locations[0].revision,
                Some(package.revision.identity())
            );
            assert_eq!(diagnostic.locations[0].path, "temperature");
        }
    }
    #[tokio::test]
    async fn named_member_rename_changes_binding_identity_and_cannot_reuse_member() {
        let (before, old_model) = fixture_with("temperature", false, physical()).await;
        let (after, new_model) = fixture_with("renamed", false, physical()).await;
        let old = before
            .admit_overlay(
                old_model.compiled(),
                &PointOverlay {
                    assignments: vec![assignment(
                        &before,
                        BindingTarget::Path("temperature".into()),
                        300.,
                        "Temperature",
                        "K",
                    )],
                },
            )
            .unwrap();
        let new = after
            .admit_overlay(
                new_model.compiled(),
                &PointOverlay {
                    assignments: vec![assignment(
                        &after,
                        BindingTarget::Path("renamed".into()),
                        300.,
                        "Temperature",
                        "K",
                    )],
                },
            )
            .unwrap();
        assert_ne!(
            old.entries.keys().collect::<Vec<_>>(),
            new.entries.keys().collect::<Vec<_>>()
        );
        assert_ne!(old.identity(), new.identity());
        let assignment = assignment(
            &after,
            BindingTarget::Member(old_model.compiled().model.paths["temperature"]),
            300.,
            "Temperature",
            "K",
        );
        assert_eq!(
            after
                .admit_overlay(
                    new_model.compiled(),
                    &PointOverlay {
                        assignments: vec![assignment]
                    }
                )
                .unwrap_err()
                .boundary_diagnostic()
                .rule,
            DiagnosticRule::StudyBindingTarget
        );
    }
}

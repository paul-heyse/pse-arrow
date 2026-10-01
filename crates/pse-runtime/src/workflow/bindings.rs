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
#[serde(tag = "kind", content = "value", rename_all = "snake_case", deny_unknown_fields)]
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
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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
            h.id(member).id(&entry.quantity).bool(entry.parameter)
                .u64(entry.canonical.into_inner().to_bits());
        }
        h.finish_hash()
    }
    /// Canonical scalar projection; consumers perform no unit conversion.
    pub fn values(&self) -> BTreeMap<SemanticId, f64> {
        self.entries.iter().map(|(id, entry)| (*id, entry.canonical.into_inner())).collect()
    }
}
impl ModelingPackage {
    /// Resolve references, physical meaning and duplicate assignments exactly once.
    pub fn admit_overlay(
        &self,
        model: &pse_compiler::workspace::PreparedModeling,
        overlay: &PointOverlay,
    ) -> Result<AdmittedBinding, WorkflowError> {
        use pse_model::diagnostic::{BoundaryClass, BoundaryDiagnostic};
        use pse_diagnostics::{DiagnosticRule as R, DiagnosticStage as S};
        let failure = |member, rule| WorkflowError::from(BoundaryDiagnostic::new(
            BoundaryClass::InvalidModel, S::StudyBinding, [member], rule));
        let mut entries: BTreeMap<SemanticId, AdmittedBindingEntry> = BTreeMap::new();
        for assignment in &overlay.assignments {
            let member = match &assignment.target {
                BindingTarget::Member(id) => *id,
                BindingTarget::Path(path) => *model.model.paths.get(path)
                    .ok_or_else(|| failure(SemanticId::NIL, R::StudyBindingTarget))?,
            };
            let symbol = model.model.symbols.get(&member)
                .filter(|_| model.admitted.inputs.contains(&member))
                .ok_or_else(|| failure(member, R::StudyBindingTarget))?;
            let expected = symbol.ty.quantity_scheme()
                .ok_or_else(|| failure(member, R::StudyBindingPhysical))?
                .resolve_contract_with_evidence(&self.quantities, &BTreeMap::new(), &pse_quantity::infer::NoInvariantFacts)
                .map_err(|_| failure(member, R::StudyBindingPhysical))?;
            let supplied = ResolvedPhysicalContract::named(assignment.value.quantity.into(), IndexSet::new(), &self.quantities)
                .map_err(|cause| super::math(pse_math::MathError::Quantity(cause)))?;
            if !expected.same_meaning(&supplied) {
                return Err(failure(member, R::StudyBindingPhysical));
            }
            let quantity = expected.named_id().ok_or_else(|| failure(member, R::StudyBindingPhysical))?;
            let canonical = CanonicalConversionPlan::registered(&self.quantities, quantity, assignment.value.unit.into())
                .and_then(|plan| plan.apply(assignment.value.magnitude.into_inner()))
                .map_err(|cause| super::math(pse_math::MathError::Quantity(cause)))?;
            let canonical = FiniteBound::try_new(canonical.value())
                .map_err(|_| failure(member, R::StudyBindingPhysical))?;
            let entry = AdmittedBindingEntry {
                member, quantity: quantity.as_id(), canonical,
                parameter: symbol.role == pse_model::generated::enums::ModelingDeclarationKind::Param,
                supplied: assignment.target.clone(), supplied_unit: assignment.value.unit,
            };
            if let Some(previous) = entries.get(&member) {
                if previous.quantity != entry.quantity || previous.canonical != entry.canonical || previous.parameter != entry.parameter {
                    return Err(failure(member, R::StudyBindingDuplicate));
                }
            } else { entries.insert(member, entry); }
        }
        Ok(AdmittedBinding { revision: self.revision.identity(), context: self.physical.identity(), entries })
    }
    /// Validate recorded admitted coordinates without resolving attribution or converting units.
    pub(in crate::workflow) fn validate_binding(&self, binding: &AdmittedBinding) -> Result<(), WorkflowError> {
        if binding.revision != self.revision.identity() || binding.context != self.physical.identity() {
            return Err(pse_model::diagnostic::BoundaryDiagnostic::new(
                pse_model::diagnostic::BoundaryClass::Incompatible,
                pse_diagnostics::DiagnosticStage::StudyBinding, [],
                pse_diagnostics::DiagnosticRule::StudyBindingRevision).into());
        }
        Ok(())
    }
}

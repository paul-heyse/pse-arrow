// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Catalogue of semantic derivations. Producers publish facts; consumers require those facts.
use crate::{DeclarationId, InstanceId, Result, invalid};
use pse_ids::{FramedHasher, SemanticId};
use std::collections::BTreeMap;

/// Component of a checked source expectation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TestComponent {
    /// Observed result.
    Actual,
    /// Independent reference result.
    Expected,
    /// Absolute physical allowance.
    Tolerance,
    /// Dimensionless relative allowance.
    RelativeTolerance,
}
/// Component within the declared annotation kind, independent of output presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnnotationComponent {
    /// Single annotation argument.
    Value,
    /// Lower endpoint.
    Lower,
    /// Upper endpoint.
    Upper,
    /// Lower objective boundary.
    ObjectiveLower { /// Whether equality is excluded.
        strict: bool },
    /// Upper objective boundary.
    ObjectiveUpper { /// Whether equality is excluded.
        strict: bool },
    /// Physical time.
    Time,
    /// Physical resolution.
    Resolution,
    /// Lower convergence criterion.
    CriterionLower,
    /// Upper convergence criterion.
    CriterionUpper,
}
/// A canonical body-local role. Parameters are never free derivative coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LocalRole {
    /// Independent derivative input.
    Coordinate,
    /// Parameter input, separate from derivative coordinates.
    Parameter,
    /// Defined expression member.
    Member,
}
/// Meaning of a catalogue payload field; identity-shaped fields retain distinct semantic roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoleKind {
    /// Scalar symbol.
    Symbol,
    /// Equation row.
    Row,
    /// Conservation or accounting closure.
    Closure,
    /// Physical state.
    State,
    /// Conserved inventory balance.
    Inventory,
    /// Discrete event occurrence.
    Event,
    /// Checked expectation.
    Expectation,
    /// Original contribution.
    Contribution,
    /// Authored declaration.
    Declaration,
    /// Actual definition instance.
    Instance,
    /// Ordered local component.
    Ordinal,
    /// Checked expectation component.
    TestComponent,
    /// Registry-owned annotation kind.
    AnnotationKind,
    /// Component of an annotation.
    AnnotationComponent,
    /// Body-local coordinate role.
    LocalRole,
}
/// One ordered field in a relationship's framing contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleSpec {
    /// Exact declared field name.
    pub name: &'static str,
    /// Semantic meaning of this field.
    pub kind: RoleKind,
}

trait FrameValue { fn frame(&self, hash: &mut FramedHasher); }
impl FrameValue for SemanticId { fn frame(&self, hash: &mut FramedHasher) { hash.id(self); } }
impl FrameValue for DeclarationId { fn frame(&self, hash: &mut FramedHasher) { hash.id(&self.as_id()); } }
impl FrameValue for InstanceId { fn frame(&self, hash: &mut FramedHasher) { hash.id(&self.as_id()); } }
impl FrameValue for u64 { fn frame(&self, hash: &mut FramedHasher) { hash.u64(*self); } }
impl FrameValue for TestComponent {
    fn frame(&self, hash: &mut FramedHasher) { hash.str(match self {
        Self::Actual => "actual", Self::Expected => "expected", Self::Tolerance => "tolerance",
        Self::RelativeTolerance => "relative_tolerance",
    }); }
}
impl FrameValue for AnnotationComponent {
    fn frame(&self, hash: &mut FramedHasher) {
        match self {
            Self::Value => { hash.str("value"); }, Self::Lower => { hash.str("lower"); }, Self::Upper => { hash.str("upper"); },
            Self::ObjectiveLower { strict } => { hash.str("objective_lower").bool(*strict); },
            Self::ObjectiveUpper { strict } => { hash.str("objective_upper").bool(*strict); },
            Self::Time => { hash.str("time"); }, Self::Resolution => { hash.str("resolution"); },
            Self::CriterionLower => { hash.str("criterion_lower"); }, Self::CriterionUpper => { hash.str("criterion_upper"); },
        }
    }
}
impl FrameValue for LocalRole {
    fn frame(&self, hash: &mut FramedHasher) { hash.str(match self {
        Self::Coordinate => "coordinate", Self::Parameter => "parameter", Self::Member => "member",
    }); }
}
impl FrameValue for pse_model::generated::enums::ModelingAnnotationKind {
    fn frame(&self, hash: &mut FramedHasher) { hash.str(self.as_str()); }
}
macro_rules! field_type {
    (Declaration) => { DeclarationId }; (Instance) => { InstanceId }; (Ordinal) => { u64 };
    (TestComponent) => { TestComponent }; (AnnotationKind) => { pse_model::generated::enums::ModelingAnnotationKind };
    (AnnotationComponent) => { AnnotationComponent }; (LocalRole) => { LocalRole };
    ($identity:ident) => { SemanticId };
}
macro_rules! relationships {
    ($( $variant:ident => $tag:literal { $( $field:ident : $role:ident ),* $(,)? } ),* $(,)?) => {
        /// Typed derived relationship and its complete ordered payload.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Relationship { $(#[doc = concat!("Declared semantic derivation `", $tag, "`.")]
            $variant { $(#[doc = concat!("Typed ", stringify!($role), " role.")] $field: field_type!($role),)* },)* }
        impl Relationship {
            /// Exact catalogue spelling; no Debug or display representation enters identity.
            pub const fn name(&self) -> &'static str { match self { $(Self::$variant { .. } => $tag,)* } }
            /// Ordered role metadata generated from the same declaration as framing.
            pub const fn roles(&self) -> &'static [RoleSpec] { match self {
                $(Self::$variant { .. } => &[$(RoleSpec { name: stringify!($field), kind: RoleKind::$role },)*],)*
            } }
            /// Construct a producer-issued result identity in the explicit new versioned domain.
            pub fn identity(&self) -> SemanticId {
                let mut hash = FramedHasher::new(pse_ids::Frame::ModelingDerivedRelationshipV1);
                match self { $(Self::$variant { $($field),* } => {
                    hash.str($tag); $(hash.str(stringify!($field)); FrameValue::frame($field, &mut hash);)*
                },)* }
                hash.finish_id()
            }
        }
    };
}
relationships! {
    ClosureConservation => "closure_conservation" { closure: Closure },
    ConditionalBoundaryValue => "conditional_boundary_value" { symbol: Symbol },
    ConditionalBoundaryResidual => "conditional_boundary_residual" { symbol: Symbol },
    ConservedInventoryOutput => "conserved_inventory_output" { balance: Inventory },
    InventoryTransferOutput => "inventory_transfer_output" { balance: Inventory, event: Event },
    InitialStateOutput => "initial_state_output" { state: State, equation: Row },
    DynamicRateOutput => "dynamic_rate_output" { state: State },
    TestOutput => "test_output" { test: Expectation, component: TestComponent },
    AnnotationOutput => "annotation_output" { target: Symbol, declaration: Declaration, kind: AnnotationKind, component: AnnotationComponent },
    OriginalEquationResidual => "original_equation_residual" { equation: Row },
    ElasticPenalty => "elastic_penalty" { equation: Row },
    OriginalTerm => "original_term" { equation: Row, ordinal: Ordinal },
    MemberOutput => "member_output" { symbol: Symbol },
    ContributionOutput => "contribution_output" { contribution: Contribution },
    TransportAgreement => "transport_agreement" { closure: Closure },
    DifferenceClosure => "difference_closure" { closure: Closure },
    InventoryObservation => "inventory_observation" { balance: Inventory },
    OriginalFluxIntegral => "original_flux_integral" { balance: Inventory },
    InventoryState => "inventory_state" { balance: Inventory },
    InventoryInitial => "inventory_initial" { balance: Inventory },
    InventoryDefinition => "inventory_definition" { balance: Inventory },
    InventoryRate => "inventory_rate" { balance: Inventory },
    TemporalClosure => "temporal_closure" { closure: Closure },
    CheckedMember => "checked_member" { role: LocalRole, ordinal: Ordinal },
}

/// Explicit reached derivations retained with a resolved product, independently of numerical values.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts { values: BTreeMap<Relationship, SemanticId>, owners: BTreeMap<SemanticId, Relationship> }
impl Facts {
    /// Publish exactly one complete typed derivation, detecting an identity collision.
    pub fn publish(&mut self, relationship: Relationship) -> Result<SemanticId> {
        let identity = relationship.identity();
        if self.owners.get(&identity).is_some_and(|prior| *prior != relationship) {
            return Err(invalid(identity, "derived relationship identity collision"));
        }
        self.owners.insert(identity, relationship);
        self.values.insert(relationship, identity);
        Ok(identity)
    }
    /// A required reached relationship cannot become a silent missing lookup.
    pub fn require(&self, relationship: Relationship) -> Result<SemanticId> {
        self.values.get(&relationship).copied().ok_or_else(|| invalid(SemanticId::NIL,
            format!("required derived relationship {} absent", relationship.name())))
    }
    /// Exact reached inventory, for inspection and retained allocation accounting.
    pub fn iter(&self) -> impl Iterator<Item = (&Relationship, &SemanticId)> { self.values.iter() }
    /// Conservative map allocation extent; no external Arc is claimed as residency.
    pub fn retained_bytes(&self) -> usize { self.values.len().saturating_add(self.owners.len()).saturating_mul(size_of::<(Relationship, SemanticId)>() + 96) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derived_facts_require_actual_publication_and_keep_semantic_roles_distinct() {
        let id = SemanticId::from_bytes([5; 16]);
        let row = Relationship::ClosureConservation { closure: id };
        let output = Relationship::MemberOutput { symbol: id };
        assert_ne!(row.identity(), output.identity());
        assert_eq!(row.roles(), &[RoleSpec {name: "closure", kind: RoleKind::Closure}]);
        let mut facts = Facts::default();
        assert!(facts.require(row).is_err());
        assert_eq!(facts.publish(row).unwrap(), row.identity());
        assert_eq!(facts.require(row).unwrap(), row.identity());
        assert!(facts.require(output).is_err());
        assert_ne!(Relationship::CheckedMember {role: LocalRole::Coordinate, ordinal: 0}.identity(), Relationship::CheckedMember {role: LocalRole::Parameter, ordinal: 0}.identity());
    }
}

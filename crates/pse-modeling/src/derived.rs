// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Catalogue of semantic derivations. Producers publish facts; consumers require those facts.
use crate::{DeclarationId, InstanceId, Result, invalid};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use std::collections::BTreeMap;

pse_contract::projection_enum! {
/// Component of a checked source expectation.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum TestComponent {
    projection: { name: "ModelingTestComponent", rust_path: crate::derived::TestComponent,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Observed result.
    Actual => 1;
    /// Independent reference result.
    Expected => 2;
    /// Absolute physical allowance.
    Tolerance => 3;
    /// Dimensionless relative allowance.
    RelativeTolerance => 4;
}
}
pse_contract::projection_enum! {
/// Component within the declared annotation kind, independent of output presentation.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum AnnotationComponent {
    projection: { name: "ModelingAnnotationComponent", rust_path: crate::derived::AnnotationComponent,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Single annotation argument.
    Value => 1;
    /// Lower endpoint.
    Lower => 2;
    /// Upper endpoint.
    Upper => 3;
    /// Lower objective boundary.
    ObjectiveLower {
        /// Whether equality is excluded.
        strict: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 4;
    /// Upper objective boundary.
    ObjectiveUpper {
        /// Whether equality is excluded.
        strict: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 5;
    /// Physical time.
    Time => 6;
    /// Physical resolution.
    Resolution => 7;
    /// Lower convergence criterion.
    CriterionLower => 8;
    /// Upper convergence criterion.
    CriterionUpper => 9;
    /// Local caloric response.
    CaloricResponse => 10;
    /// Useful temperature difference.
    CaloricThermalResolution => 11;
    /// Consuming power allowance.
    CaloricPowerAllowance => 12;
    /// Aggregate consuming molar exposure.
    CaloricExposure => 13;
}
}
pse_contract::projection_enum! {
/// A canonical body-local role. Parameters are never free derivative coordinates.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum LocalRole {
    projection: { name: "ModelingLocalRole", rust_path: crate::derived::LocalRole,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Independent derivative input.
    Coordinate => 1;
    /// Parameter input, separate from derivative coordinates.
    Parameter => 2;
    /// Defined expression member.
    Member => 3;
}
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
    /// Complete framed consumed mathematical content.
    Content,
}
/// One ordered field in a relationship's framing contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleSpec {
    /// Exact declared field name.
    pub name: &'static str,
    /// Semantic meaning of this field.
    pub kind: RoleKind,
}

trait FrameValue {
    fn frame(&self, hash: &mut FramedHasher);
}
impl FrameValue for ContentHash {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.hash(self);
    }
}
impl FrameValue for SemanticId {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.id(self);
    }
}
impl FrameValue for DeclarationId {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.id(&self.as_id());
    }
}
impl FrameValue for InstanceId {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.id(&self.as_id());
    }
}
impl FrameValue for u64 {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.u64(*self);
    }
}
impl FrameValue for TestComponent {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.str(match self {
            Self::Actual => "actual",
            Self::Expected => "expected",
            Self::Tolerance => "tolerance",
            Self::RelativeTolerance => "relative_tolerance",
        });
    }
}
impl FrameValue for AnnotationComponent {
    fn frame(&self, hash: &mut FramedHasher) {
        match self {
            Self::Value => {
                hash.str("value");
            }
            Self::Lower => {
                hash.str("lower");
            }
            Self::Upper => {
                hash.str("upper");
            }
            Self::ObjectiveLower { strict } => {
                hash.str("objective_lower").bool(*strict);
            }
            Self::ObjectiveUpper { strict } => {
                hash.str("objective_upper").bool(*strict);
            }
            Self::Time => {
                hash.str("time");
            }
            Self::Resolution => {
                hash.str("resolution");
            }
            Self::CriterionLower => {
                hash.str("criterion_lower");
            }
            Self::CriterionUpper => {
                hash.str("criterion_upper");
            }
            Self::CaloricResponse => {
                hash.str("caloric_response");
            }
            Self::CaloricThermalResolution => {
                hash.str("caloric_thermal_resolution");
            }
            Self::CaloricPowerAllowance => {
                hash.str("caloric_power_allowance");
            }
            Self::CaloricExposure => {
                hash.str("caloric_exposure");
            }
        }
    }
}
impl FrameValue for LocalRole {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.str(match self {
            Self::Coordinate => "coordinate",
            Self::Parameter => "parameter",
            Self::Member => "member",
        });
    }
}
impl FrameValue for pse_model::generated::enums::ModelingAnnotationKind {
    fn frame(&self, hash: &mut FramedHasher) {
        hash.str(self.as_str());
    }
}
macro_rules! field_type {
    (Declaration) => {
        DeclarationId
    };
    (Instance) => {
        InstanceId
    };
    (Ordinal) => {
        u64
    };
    (Content) => {
        ContentHash
    };
    (TestComponent) => {
        TestComponent
    };
    (AnnotationKind) => {
        pse_model::generated::enums::ModelingAnnotationKind
    };
    (AnnotationComponent) => {
        AnnotationComponent
    };
    (LocalRole) => {
        LocalRole
    };
    ($identity:ident) => {
        SemanticId
    };
}
macro_rules! relationships {
    ($( $variant:ident ($wire:literal) => $tag:literal { $( $field:ident : $role:ident => $field_tag:literal ),* $(,)? } ),* $(,)?) => {
        pse_contract::projection_enum! {
        /// Typed derived relationship and its complete ordered payload.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub enum Relationship {
            projection: { name: "ModelingDerivedRelationship", rust_path: crate::derived::Relationship,
                version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
            $(#[doc = concat!("Declared semantic derivation `", $tag, "`.")]
            $variant { $(#[doc = concat!("Typed ", stringify!($role), " role.")]
                $field: field_type!($role) => { tag: $field_tag, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
            )* } => $wire;)*
        }
        }
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
    SubstitutedMember(1) => "substituted_member" {member:Symbol=>1,meaning:Content=>2},
    ReconstructionResultView(2) => "reconstruction_result_view" {owner:Declaration=>1,callee:Content=>2},
    FunctionBodyInputView(3) => "function_body_input_view" {owner:Declaration=>1,ordinal:Ordinal=>2,meaning:Content=>3},
    DefiningEquation(4) => "defining_equation" {symbol:Symbol=>1},
    ContinuityRow(5) => "continuity_row" {family:Symbol=>1,element:Ordinal=>2},
    ContinuousIntegral(6) => "continuous_integral" {content:Content=>1},
    IntegralFlux(7) => "integral_flux" {result:Symbol=>1},
    ClosureConservation(8) => "closure_conservation" { closure: Closure=>1 },
    ConditionalBoundaryValue(9) => "conditional_boundary_value" { symbol: Symbol=>1 },
    ConditionalBoundaryResidual(10) => "conditional_boundary_residual" { symbol: Symbol=>1 },
    ConservedInventoryOutput(11) => "conserved_inventory_output" { balance: Inventory=>1 },
    InventoryTransferOutput(12) => "inventory_transfer_output" { balance: Inventory=>1, event: Event=>2 },
    InitialStateOutput(13) => "initial_state_output" { state: State=>1, equation: Row=>2 },
    IntegratedRateCoordinate(14) => "integrated_rate_coordinate" { state: State=>1 },
    DynamicRateOutput(15) => "dynamic_rate_output" { state: State=>1 },
    TestOutput(16) => "test_output" { test: Expectation=>1, component: TestComponent=>2 },
    AnnotationOutput(17) => "annotation_output" { target: Symbol=>1, declaration: Declaration=>2, kind: AnnotationKind=>3, component: AnnotationComponent=>4 },
    OriginalEquationResidual(18) => "original_equation_residual" { equation: Row=>1 },
    ElasticPenalty(19) => "elastic_penalty" { equation: Row=>1 },
    OriginalTerm(20) => "original_term" { equation: Row=>1, ordinal: Ordinal=>2 },
    MemberOutput(21) => "member_output" { symbol: Symbol=>1 },
    ContributionOutput(22) => "contribution_output" { contribution: Contribution=>1 },
    TransportAgreement(23) => "transport_agreement" { closure: Closure=>1 },
    DifferenceClosure(24) => "difference_closure" { closure: Closure=>1 },
    InventoryObservation(25) => "inventory_observation" { balance: Inventory=>1 },
    OriginalFluxIntegral(26) => "original_flux_integral" { balance: Inventory=>1 },
    InventoryState(27) => "inventory_state" { balance: Inventory=>1 },
    InventoryInitial(28) => "inventory_initial" { balance: Inventory=>1 },
    InventoryDefinition(29) => "inventory_definition" { balance: Inventory=>1 },
    InventoryRate(30) => "inventory_rate" { balance: Inventory=>1 },
    TemporalClosure(31) => "temporal_closure" { closure: Closure=>1 },
    CheckedMember(32) => "checked_member" { role: LocalRole=>1, ordinal: Ordinal=>2 },
    CheckedValiditySource(33) => "checked_validity_source" { role: LocalRole=>1, ordinal: Ordinal=>2 },
}

/// Numerical obligation recipes retain the established named-identity byte contract.
/// They belong to the same derivation catalogue, without changing the framing of existing facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericalRequirement {
    /// Tolerance on an original state reconstruction equation.
    StateReconstruction {
        /// Source declaration.
        declaration: DeclarationId,
        /// Original reconstruction equation.
        row: SemanticId,
    },
    /// Tolerance on an original conservation equation.
    ConservationRow {
        /// Source declaration.
        declaration: DeclarationId,
        /// Original closure.
        closure: SemanticId,
    },
    /// One original physical closure obligation in declaration order.
    PhysicalClosure {
        /// Original closure.
        closure: SemanticId,
        /// Source order.
        ordinal: usize,
    },
    /// The tighter separately resolved physical endpoint budget.
    TighterEndpoint {
        /// Original closure.
        closure: SemanticId,
    },
    /// Original conserved-inventory closure obligation.
    InventoryClosure {
        /// Original inventory balance.
        balance: SemanticId,
    },
}
impl NumericalRequirement {
    /// Exact established identity; callers never restate these spellings or compositions.
    pub fn identity(self) -> SemanticId {
        use pse_ids::named_id;
        match self {
            Self::StateReconstruction { declaration, row } => named_id(
                named_id(declaration.as_id(), &row.to_string()),
                "state-reconstruction-tolerance",
            ),
            Self::ConservationRow {
                declaration,
                closure,
            } => named_id(
                named_id(declaration.as_id(), &closure.to_string()),
                "conservation-row-tolerance",
            ),
            Self::PhysicalClosure { closure, ordinal } => {
                named_id(closure, &format!("physical-closure-requirement:{ordinal}"))
            }
            Self::TighterEndpoint { closure } => named_id(closure, "tighter-endpoint-budget"),
            Self::InventoryClosure { balance } => {
                named_id(balance, "inventory-closure-requirement")
            }
        }
    }
}
/// Explicit reached derivations retained with a resolved product, independently of numerical values.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Facts {
    values: BTreeMap<Relationship, SemanticId>,
    owners: BTreeMap<SemanticId, Relationship>,
}
impl Facts {
    /// Publish exactly one complete typed derivation, detecting an identity collision.
    pub fn publish(&mut self, relationship: Relationship) -> Result<SemanticId> {
        let identity = relationship.identity();
        if self
            .owners
            .get(&identity)
            .is_some_and(|prior| *prior != relationship)
        {
            return Err(invalid(identity, "derived relationship identity collision"));
        }
        self.owners.insert(identity, relationship);
        self.values.insert(relationship, identity);
        Ok(identity)
    }
    /// A required reached relationship cannot become a silent missing lookup.
    pub fn require(&self, relationship: Relationship) -> Result<SemanticId> {
        self.values.get(&relationship).copied().ok_or_else(|| {
            invalid(
                SemanticId::NIL,
                format!(
                    "required derived relationship {} absent",
                    relationship.name()
                ),
            )
        })
    }
    /// Exact reached inventory, for inspection and retained allocation accounting.
    pub fn iter(&self) -> impl Iterator<Item = (&Relationship, &SemanticId)> {
        self.values.iter()
    }
    /// Conservative map allocation extent; no external Arc is claimed as residency.
    pub fn retained_bytes(&self) -> usize {
        self.values
            .len()
            .saturating_add(self.owners.len())
            .saturating_mul(size_of::<(Relationship, SemanticId)>() + 96)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derived_relationship_projection_preserves_exact_semantic_framing_and_data_scope() {
        use pse_contract::boundary_projection::{ConstructionRole, ProjectionAlternative, ProjectionGraph, Shape};
        let owner = DeclarationId::from(SemanticId::from_bytes([41; 16]));
        let meaning = ContentHash::from_bytes([42; 32]);
        let value = Relationship::FunctionBodyInputView { owner, ordinal: 11, meaning };
        let mut original = FramedHasher::new(pse_ids::Frame::ModelingDerivedRelationshipV1);
        original.str("function_body_input_view").str("owner").id(&owner.as_id())
            .str("ordinal").u64(11).str("meaning").hash(&meaning);
        assert_eq!(value.identity(), original.finish_id());
        let expected = format!("{{\"FunctionBodyInputView\":{{\"owner\":\"{}\",\"ordinal\":11,\"meaning\":\"{}\"}}}}", owner.as_id(), meaning.to_prefixed());
        assert_eq!(serde_json::to_vec(&value).unwrap(), expected.as_bytes());
        let fields = <Relationship as ProjectionAlternative<3>>::into_projection_alternative(value).unwrap();
        let restored = <Relationship as ProjectionAlternative<3>>::from_projection_alternative(fields);
        assert_eq!(restored, value);
        assert_eq!(serde_json::to_vec(&restored).unwrap(), expected.as_bytes());
        assert!(Facts::default().require(restored).is_err());

        let graph = ProjectionGraph::of::<Relationship>().unwrap();
        let catalogue = &graph.definitions()["ModelingDerivedRelationship"];
        assert_eq!(catalogue.construction, ConstructionRole::Data);
        let Shape::Alternatives(variants) = &catalogue.shape else { panic!("original complete relationship catalogue") };
        assert_eq!(variants.len(), 33);
        assert_eq!((variants[0].name.as_str(), variants[0].tag), ("SubstitutedMember", 1));
        assert_eq!((variants[32].name.as_str(), variants[32].tag), ("CheckedValiditySource", 33));
        assert_eq!(variants[2].fields.iter().map(|field| (field.name.as_str(), field.tag)).collect::<Vec<_>>(), vec![("owner", 1), ("ordinal", 2), ("meaning", 3)]);
        for name in ["ModelingTestComponent", "ModelingAnnotationComponent", "ModelingLocalRole"] {
            assert_eq!(graph.definitions()[name].construction, ConstructionRole::Data);
        }
        let annotation = AnnotationComponent::ObjectiveLower { strict: true };
        assert_eq!(serde_json::to_vec(&annotation).unwrap(), br#"{"ObjectiveLower":{"strict":true}}"#);
    }
    #[test]
    fn numerical_requirement_recipes_preserve_established_named_bytes() {
        let id = SemanticId::from_bytes([5; 16]);
        let declaration = DeclarationId::from(SemanticId::from_bytes([6; 16]));
        for recipe in [
            NumericalRequirement::StateReconstruction {
                declaration,
                row: id,
            },
            NumericalRequirement::ConservationRow {
                declaration,
                closure: id,
            },
            NumericalRequirement::PhysicalClosure {
                closure: id,
                ordinal: 2,
            },
            NumericalRequirement::TighterEndpoint { closure: id },
            NumericalRequirement::InventoryClosure { balance: id },
        ]
        .into_iter()
        .zip([
            "0d7c8d79084e59fea16d33bc180037c3",
            "513551f434ba840103443e10707aca2e",
            "2fd51aa4e5366a36726742e283576056",
            "a65e19bc2725019846ada0505a3186d3",
            "9d3b0f8b1765ed27ab11a68698442ded",
        ]) {
            assert_eq!(recipe.0.identity().to_string(), recipe.1);
        }
    }
    #[test]
    fn derived_facts_require_actual_publication_and_keep_semantic_roles_distinct() {
        let id = SemanticId::from_bytes([5; 16]);
        let row = Relationship::ClosureConservation { closure: id };
        let output = Relationship::MemberOutput { symbol: id };
        assert_ne!(row.identity(), output.identity());
        assert_ne!(
            Relationship::IntegratedRateCoordinate { state: id }.identity(),
            Relationship::DynamicRateOutput { state: id }.identity()
        );
        assert_ne!(
            Relationship::CheckedMember {
                role: LocalRole::Coordinate,
                ordinal: 0
            }
            .identity(),
            Relationship::CheckedValiditySource {
                role: LocalRole::Coordinate,
                ordinal: 0
            }
            .identity()
        );
        assert_eq!(
            row.roles(),
            &[RoleSpec {
                name: "closure",
                kind: RoleKind::Closure
            }]
        );
        let mut facts = Facts::default();
        assert!(facts.require(row).is_err());
        assert_eq!(facts.publish(row).unwrap(), row.identity());
        assert_eq!(facts.require(row).unwrap(), row.identity());
        assert!(facts.require(output).is_err());
        assert_ne!(
            Relationship::CheckedMember {
                role: LocalRole::Coordinate,
                ordinal: 0
            }
            .identity(),
            Relationship::CheckedMember {
                role: LocalRole::Parameter,
                ordinal: 0
            }
            .identity()
        );
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Resolved finite syntax. Source lookup ends at this boundary; inner edges are typed indexes.
use crate::{DeclarationId, Result, Type, TypeLookup, derived::LocalRole, invalid};
use pse_authoring::dsl::{self, BinaryOp, CompareOp, Expr, ExprKind, Predicate, PredicateKind};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_model::generated::enums::{ModelingDeclarationKind, ModelingVariableDomain};
use std::{collections::BTreeMap, sync::Arc};
pub use typed_index_collections::{TiSlice, TiVec};
pub(crate) mod construction;
pub(crate) mod extent;
pub(crate) mod fragment;
mod framing;
pub(crate) mod functions;
#[path = "resolved/functions-constructed.rs"]
mod functions_constructed;
mod import;
mod occurrences;
pub(crate) mod operation;
mod records;
mod selection;
mod support;
pub use construction::Relation;
pub use functions::{EvidenceUse, FunctionProduct};
pub use occurrences::occurrence_token;
pub use operation::{Anchor, OperationMeaning, Translation};
pub use records::ResolvedRecord;
pub use selection::{
    CheckedRange, PreparedTerm, ProviderSubstitution, RegionAttribution, RegionBinding,
    RegionOutput, RegionReads, RegionRequest, SelectedRegion,
};
pub use support::{
    BodyInputSupport, BooleanSupport, CallSupport, LiteralSupport, NodeSupport, OutputSupport,
    PhysicalEvidence, PhysicalEvidenceRecord, PhysicalSupport, RangeSupport, SupportRecord,
};
pub(crate) use support::{FunctionSupportRequest, InferredView};

macro_rules! indexes {
    ($(#[doc = $doc:literal] $name:ident;)*) => {$(
        pse_contract::projection_newtype! {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,serde::Serialize,serde::Deserialize)]
        #[repr(transparent)]
        pub struct $name(usize) {
            projection: { name: concat!("ModelingResolved", stringify!($name)), version: 1,
                direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
            value: { tag: 1, rules: [] };
        }
        }
        impl $name {
            /// Position inside this product's index space.
            pub const fn position(self) -> usize { self.0 }
        }
        impl From<usize> for $name { fn from(value: usize) -> Self { Self(value) } }
        impl From<$name> for usize { fn from(value: $name) -> Self { value.0 } }
    )*};
}
indexes! {
    /// Scalar in one authoritative resolved symbol table.
    SymbolIx;
    /// Expression operation in one immutable resolved arena.
    NodeIx;
    /// Predicate in one immutable resolved arena.
    PredicateIx;
    /// Lexical binding, distinct from a physical parameter or coordinate.
    BinderIx;
    /// Selected complete function contract.
    FunctionIx;
    /// One exact demanded scientific evidence capture.
    EvidenceIx;
    /// Ordered typed finite carrier.
    CarrierIx;
    /// Indexed member layout, including explicit missing tuples.
    GroupIx;
}

/// The specialization and executable syntax share one scalar declaration type and owner.
pub use crate::specialize::Symbol as ResolvedSymbol;

/// Borrowed source-boundary view over the authoritative symbol table. Types are never cloned.
pub struct SymbolTypes<'a> {
    /// Single symbol owner.
    pub symbols: &'a crate::specialize::Symbols,
    /// Resolved source names, prepared once by the owner.
    pub names: &'a BTreeMap<String, SymbolIx>,
}
impl std::fmt::Debug for SymbolTypes<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolTypes")
            .field("symbols", &self.symbols.len())
            .field("names", &self.names.len())
            .finish()
    }
}
impl TypeLookup for SymbolTypes<'_> {
    fn get(&self, name: &str) -> Option<&Type> {
        self.names
            .get(name)
            .and_then(|slot| self.symbols.at(*slot))
            .map(|symbol| &symbol.ty)
    }
}

pse_contract::projection_struct! {
/// Actual admitted finite values, retained before specialization discards lexical coordinates.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Carrier {
    projection: { name: "ModelingCarrier", rust_path: crate::resolved::Carrier,
        version: 1, direction: pse_contract::boundary_projection::Direction::Output,
        reserved_tags: [], reserved_names: [] };
    /// A finite set and a realized continuous axis are different physical roles.
    pub kind: CarrierKind => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Complete element type, including its actual entity or enumeration kind.
    pub element: Type => { tag: 2, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    /// Semantic order, with exact typed values and no numerical approximation.
    pub members: Arc<[crate::specialize::Value]> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
}
}
impl Carrier {
    /// Complete conservative selected metadata extent before boundary materialization.
    #[must_use]
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.element.retained_bytes())
            .saturating_add(
                self.members
                    .len()
                    .saturating_mul(size_of::<crate::specialize::Value>()),
            )
            .saturating_add(self.members.iter().fold(0usize, |size, value| {
                size.saturating_add(value.retained_bytes())
            }))
    }
}

pse_contract::projection_enum! {
/// The declared interpretation survives even an empty carrier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum CarrierKind {
    projection: { name: "ModelingCarrierKind", rust_path: crate::resolved::CarrierKind,
        version: 1, direction: pse_contract::boundary_projection::Direction::Output,
        reserved_tags: [], reserved_names: [] };
    /// An ordinary typed ordered finite set.
    OrderedSet => 1;
    /// A realized continuous coordinate axis.
    Continuous {
        /// Actual continuous domain declaration.
        domain: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Output, rules: [] };
    } => 2;
}
}

/// One exact indexed layout. `None` is absence, never a zero-valued member.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedGroup {
    /// Declared axes in order.
    pub axes: Vec<CarrierIx>,
    /// Complete member tuples, with their exact local scalar selection or explicit absence.
    pub members: Vec<(Vec<crate::specialize::Value>, Option<SymbolIx>)>,
}

/// Complete selected contract identity. Diagnostics retain the declaration separately.
#[derive(Clone, Debug)]
pub struct FunctionSelection {
    /// Semantic identity produced by the resolved function owner.
    pub identity: ContentHash,
    /// Original declaration used for diagnostics.
    pub declaration: DeclarationId,
    /// Ordered formal names are resolved once at this source boundary.
    pub formals: Vec<String>,
    /// Complete actual product; absent only in construction-only identity controls.
    pub product: Arc<FunctionProduct>,
}

impl PartialEq for FunctionSelection {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.declaration == other.declaration
            && self.formals == other.formals
    }
}
impl Eq for FunctionSelection {}

pse_contract::projection_struct! {
/// One output of an actual complete executable provider descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProviderSelection {
    projection: { name: "ModelingProviderSelection", rust_path: crate::resolved::ProviderSelection,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Full descriptor content identity; it is never narrowed to a semantic identifier.
    pub identity: ContentHash => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual selected output ordinal.
    pub output: usize => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

/// An actual finite domain or scalar selected before entering the resolved arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    /// Independent or defined scalar.
    Symbol(SymbolIx),
    /// A complete finite indexed group.
    Group(GroupIx),
    /// A finite domain, used by reductions and membership predicates.
    Carrier(CarrierIx),
}

/// Source lookup is an adapter into the resolved graph, not a second inner representation.
pub trait Names {
    /// Resolve a complete unindexed path against its actual admitted owner.
    fn selection(&self, path: &dsl::Path) -> Option<Selection>;
    /// Resolve a selected pure function with its complete contract identity.
    fn function(&self, path: &dsl::Path) -> Option<FunctionIx>;
    /// Resolve an executable provider output to its actual descriptor identity.
    fn provider(&self, name: &str) -> Option<ProviderSelection>;
}

pse_contract::projection_tuple_enum! {
/// A resolved reference preserves lexical and scalar/indexed roles.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Reference {
    projection: { name: "ModelingResolvedReference", rust_path: crate::resolved::Reference,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// A scalar from the single symbol table.
    Symbol [symbol: SymbolIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 1;
    /// A lexical binder or local value.
    Lexical [binder: BinderIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 2;
    /// Ordered subscripts select a member of an actual indexed layout.
    Indexed {
        /// Actual typed indexed layout.
        group: GroupIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Subscript expressions in declared axis order.
        indices: Vec<NodeIx> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 3;
}
}

pse_contract::projection_tuple_enum! {
/// One lexical declaration; names and source spans are separate attribution data.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Binding {
    projection: { name: "ModelingResolvedBinding", rust_path: crate::resolved::Binding,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// A local value whose definition precedes its uses.
    Value [value: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 1;
    /// A bound coordinate from an actual typed finite carrier.
    Coordinate [carrier: CarrierIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 2;
    /// A fold accumulator, initialized from the first selected value.
    Accumulator => 3;
    /// The current fold value.
    Item => 4;
}
}

pse_contract::projection_tuple_enum! {
/// An immutable operation with explicit child edges and no unresolved identifier strings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Node {
    projection: { name: "ModelingResolvedNode", rust_path: crate::resolved::Node,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact authored scalar, including integer token and canonical unit factors.
    Number [number: dsl::Number => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 1;
    /// Resolved scalar, lexical value or indexed member.
    Reference [reference: Reference => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 2;
    /// Unary negation.
    Neg [value: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 3;
    /// Ordered binary arithmetic.
    Binary {
        /// Actual operation.
        op: BinaryOp => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Left child.
        lhs: NodeIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Right child.
        rhs: NodeIx => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 4;
    /// Closed builtin application.
    Call {
        /// Library-owned builtin.
        function: dsl::Function => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Ordered arguments.
        args: Vec<NodeIx> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 5;
    /// Selected pure function application.
    Apply {
        /// Complete selected contract.
        function: FunctionIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Ordered arguments.
        args: Vec<NodeIx> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 6;
    /// A derivative of a selected function at ordered formal ordinals.
    Partial {
        /// Complete selected contract.
        function: FunctionIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Ordered differentiated formal ordinals.
        wrt: Vec<usize> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Applied arguments.
        args: Vec<NodeIx> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 7;
    /// Executable provider application.
    Provider {
        /// Actual admitted descriptor selection.
        selection: ProviderSelection => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Ordered arguments.
        args: Vec<NodeIx> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 8;
    /// Original finite or continuous reduction, before library normalization.
    Reduce {
        /// Mathematical reduction kind.
        kind: dsl::ReduceKind => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Actual lexical coordinate.
        binder: BinderIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original membership filter.
        filter: Option<PredicateIx> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Element expression.
        body: NodeIx => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 9;
    /// Required observed effects precede the selected numerical value.
    /// Unlike an ordinary local binding, every listed prerequisite is demanded.
    Sequence {
        /// Returned value.
        value: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Ordered demanded source observations.
        prerequisites: Vec<NodeIx> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 10;
    /// Scientific evidence demanded only when this exact value is read.
    Evidence {
        /// Returned value.
        value: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Actual claim, predicates and numerical observations.
        evidence: EvidenceIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 11;
    /// Nonempty ordered fold.
    Fold {
        /// Actual lexical coordinate.
        binder: BinderIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Original membership filter.
        filter: Option<PredicateIx> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Value expression.
        value: NodeIx => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Accumulator local.
        accumulator: BinderIx => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Current value local.
        item: BinderIx => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Combination expression.
        step: NodeIx => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 12;
    /// Derivative in the explicitly selected physical domain.
    Derivative {
        /// Differentiated value.
        body: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Actual physical domain.
        domain: CarrierIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 13;
    /// Numerical zero/one indicator of an actual original predicate.
    Indicator [predicate: PredicateIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 14;
    /// Original branch order, before simplification.
    Conditional {
        /// Strict original guard.
        guard: PredicateIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// True branch.
        then: NodeIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// False branch.
        otherwise: NodeIx => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 15;
    /// Local definitions, visited once in declaration order.
    Let {
        /// Sequential local declarations.
        bindings: Vec<BinderIx> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Returned value.
        body: NodeIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 16;
}
}

pse_contract::projection_tuple_enum! {
/// Resolved predicates retain original validity and comparison order.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ResolvedPredicate {
    projection: { name: "ModelingResolvedPredicate", rust_path: crate::resolved::ResolvedPredicate,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Ordered scalar comparison.
    Compare {
        /// Actual comparison.
        op: CompareOp => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Left child.
        lhs: NodeIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Right child.
        rhs: NodeIx => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 1;
    /// Exact typed domain membership.
    In {
        /// Tested value.
        value: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
        /// Actual finite carrier.
        domain: CarrierIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    } => 2;
    /// Ordered conjunction.
    And [left: PredicateIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] }; right: PredicateIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 3;
    /// Ordered disjunction.
    Or [left: PredicateIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] }; right: PredicateIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 4;
    /// Logical negation.
    Not [value: PredicateIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 5;
    /// Actual boolean expression.
    Atom [value: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 6;
    /// Boolean literal.
    Bool [value: bool => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };] => 7;
    /// Explicit unknown, never false.
    Null => 8;
}
}

/// Which immutable inventory issued a source coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryStage {
    /// Exact authored syntax, registered before specialization changes it.
    Original,
    /// The actual specialized occurrence inventory used by its admission producer.
    Specialized,
}
/// The node grammar owning this exact source coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryGrammar {
    /// Arithmetic expression preorder within an original field.
    Expression,
    /// Logical predicate preorder, separate from embedded arithmetic expressions.
    Predicate,
    /// Original equation grammar, including conditional equation arms.
    Equation,
}
/// Exact field-local grammar part and expression preorder issued by its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryCoordinate {
    /// Original source or an explicitly distinguished specialized operation inventory.
    pub stage: InventoryStage,
    /// Arithmetic and predicate inventories have distinct coordinate spaces.
    pub grammar: InventoryGrammar,
    /// Grammar expression-root part and preorder position inside that root.
    pub occurrence: crate::expression::admission::ExpressionOccurrence,
}

/// A diagnostic coordinate, independent of canonical executable identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    /// Complete authored field role and repeated ordinal.
    pub field: crate::expression::occurrences::OccurrenceKey,
    /// Exact source byte range.
    pub span: dsl::Span,
    /// Explicit original or specialized inventory coordinate; generated nodes have none.
    pub coordinate: Option<InventoryCoordinate>,
}

/// Actual ordered finite membership consumed by one constructed fold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FiniteFold {
    /// Complete declared carrier, including its semantic order.
    pub carrier: CarrierIx,
    /// Actually selected values, without converting missing members into zero.
    pub selected: Arc<[crate::specialize::Value]>,
    /// Original admitted fold prototype, independent of numerical normalization.
    pub prototype: Type,
}

/// A borrowed construction or immutable owner view; no arena or scalar table is cloned.
#[derive(Clone, Copy)]
pub(crate) struct View<'a> {
    pub(crate) symbols: &'a crate::specialize::Symbols,
    pub(crate) arena: &'a Arena,
    pub(crate) outputs: &'a [NodeIx],
    pub(crate) validity: &'a [PredicateIx],
}
impl std::ops::Deref for View<'_> {
    type Target = Arena;
    fn deref(&self) -> &Arena {
        self.arena
    }
}
pse_contract::projection_struct! {
/// Source-issued hard range, applied whenever the original scalar is demanded.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemberRange {
    projection: { name: "ModelingMemberRange", rust_path: crate::resolved::MemberRange,
        version: 1, direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Actual typed lower endpoint.
    pub lower: NodeIx => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Actual typed upper endpoint.
    pub upper: NodeIx => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original validity owner, never inferred from source spelling.
    pub layer: pse_model::generated::enums::ModelingValidityLayer => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
/// One immutable operation owner, shared by cheap reached-root views.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Arena {
    /// Expression operations.
    pub nodes: TiVec<NodeIx, Node>,
    /// Original predicates.
    pub predicates: TiVec<PredicateIx, ResolvedPredicate>,
    /// Lexical bindings.
    pub binders: TiVec<BinderIx, Binding>,
    /// Complete selected functions.
    pub functions: TiVec<FunctionIx, FunctionSelection>,
    /// Exact scientific captures issued before numerical normalization.
    pub evidence: TiVec<EvidenceIx, EvidenceUse>,
    /// Actual typed carriers.
    pub carriers: TiVec<CarrierIx, Carrier>,
    /// Exact finite membership.
    pub groups: TiVec<GroupIx, IndexedGroup>,
    /// Original hard ranges retain transitive endpoint dependencies on the scalar owner.
    pub member_ranges: BTreeMap<SymbolIx, MemberRange>,
    /// Exact finite-fold facts owned by their direct result roots.
    pub finite_folds: BTreeMap<NodeIx, FiniteFold>,
    /// Attribution independent of canonical executable identity.
    pub node_sources: BTreeMap<NodeIx, Source>,
    /// Original predicate attribution.
    pub predicate_sources: BTreeMap<PredicateIx, Source>,
}
/// A reached-root view sharing one symbol table and immutable operation arena.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResolvedSyntax {
    /// Single authoritative symbol owner, shared with specialization.
    pub symbols: crate::specialize::Symbols,
    arena: Arc<Arena>,
    /// Ordered requested outputs.
    pub outputs: Vec<NodeIx>,
    /// Original validity guards in source order.
    pub validity: Vec<PredicateIx>,
}
impl std::ops::Deref for ResolvedSyntax {
    type Target = Arena;
    fn deref(&self) -> &Arena {
        &self.arena
    }
}
impl std::ops::DerefMut for ResolvedSyntax {
    fn deref_mut(&mut self) -> &mut Arena {
        Arc::make_mut(&mut self.arena)
    }
}

/// The framer-issued correspondence between an actual symbol and one canonical local role.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalSymbol {
    /// Actual selected symbol; its global identity stays in the symbol table outside the frame.
    pub symbol: SymbolIx,
    /// Coordinate, parameter or defined-member role.
    pub role: LocalRole,
    /// First-use ordinal within that role, shared by lowering and checked-member binding.
    pub ordinal: usize,
}

/// Hash and exact local correspondence issued by one canonical framing walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalLayout {
    /// Complete reached syntax identity.
    pub identity: ContentHash,
    /// Actual symbols in the same first-use order that established identity.
    pub symbols: Vec<CanonicalSymbol>,
    /// Reached operation coordinates in canonical declaration order, not an execution schedule.
    pub nodes: Vec<NodeIx>,
    /// Reached predicates in the exact same framing walk.
    pub predicates: Vec<PredicateIx>,
    /// Reached lexical bindings in declaration order.
    pub binders: Vec<BinderIx>,
    /// Reached selected function contracts in first-use order.
    pub functions: Vec<FunctionIx>,
    /// Reached evidence captures in first-use order.
    pub evidence: Vec<EvidenceIx>,
    /// Reached finite carriers in first-use order.
    pub carriers: Vec<CarrierIx>,
    /// Reached indexed families in first-use order.
    pub groups: Vec<GroupIx>,
}
impl CanonicalLayout {
    /// Known correspondence-vector extent, excluding the separately retained syntax arena.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(
                self.symbols
                    .capacity()
                    .saturating_mul(size_of::<CanonicalSymbol>()),
            )
            .saturating_add(self.nodes.capacity().saturating_mul(size_of::<NodeIx>()))
            .saturating_add(
                self.predicates
                    .capacity()
                    .saturating_mul(size_of::<PredicateIx>()),
            )
            .saturating_add(
                self.binders
                    .capacity()
                    .saturating_mul(size_of::<BinderIx>()),
            )
            .saturating_add(
                self.functions
                    .capacity()
                    .saturating_mul(size_of::<FunctionIx>()),
            )
            .saturating_add(
                self.evidence
                    .capacity()
                    .saturating_mul(size_of::<EvidenceIx>()),
            )
            .saturating_add(
                self.carriers
                    .capacity()
                    .saturating_mul(size_of::<CarrierIx>()),
            )
            .saturating_add(self.groups.capacity().saturating_mul(size_of::<GroupIx>()))
    }
}

/// One bounded construction walk over source syntax. Lexical names cannot escape their scope.
pub struct Resolver<'a> {
    syntax: ResolvedSyntax,
    names: &'a dyn Names,
    lexical: Vec<(String, BinderIx)>,
    source: crate::expression::occurrences::OccurrenceKey,
    allowance: usize,
    depth: usize,
    inventory: BTreeMap<usize, (Source, crate::expression::admission::ExpressionOccurrence)>,
    mapped: BTreeMap<crate::expression::admission::ExpressionOccurrence, NodeIx>,
}
impl std::fmt::Debug for Resolver<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Resolver")
            .field("syntax", &self.syntax)
            .field("source", &self.source)
            .field("allowance", &self.allowance)
            .finish_non_exhaustive()
    }
}
impl<'a> Resolver<'a> {
    /// Begin with actual admitted symbol/carrier/function inventories, never inferred placeholders.
    pub fn new(
        syntax: ResolvedSyntax,
        names: &'a dyn Names,
        source: crate::expression::occurrences::OccurrenceKey,
        allowance: usize,
    ) -> Self {
        Self {
            syntax,
            names,
            lexical: Vec::new(),
            source,
            allowance,
            depth: 0,
            inventory: BTreeMap::new(),
            mapped: BTreeMap::new(),
        }
    }
    /// Set the original field owner for subsequent requested roots.
    pub fn source(&mut self, source: crate::expression::occurrences::OccurrenceKey) {
        self.source = source;
    }
    /// Register actual source-bearing roots before their typed resolution. Addresses are temporary.
    pub fn inventory<'b>(
        &mut self,
        roots: impl IntoIterator<
            Item = (
                crate::expression::occurrences::OccurrenceKey,
                usize,
                &'b Expr,
            ),
        >,
        stage: InventoryStage,
    ) -> Result<()> {
        self.inventory.clear();
        self.mapped.clear();
        for (body, (field, part, root)) in roots.into_iter().enumerate() {
            let mut position = 0;
            let mut conflict = false;
            root.walk(|expression| {
                let coordinate = InventoryCoordinate {
                    stage,
                    grammar: InventoryGrammar::Expression,
                    occurrence: crate::expression::admission::ExpressionOccurrence::Node {
                        body: part,
                        position,
                    },
                };
                let source = Source {
                    field: field.clone(),
                    span: expression.span,
                    coordinate: Some(coordinate),
                };
                let prior = self.inventory.insert(
                    std::ptr::from_ref(expression) as usize,
                    (
                        source,
                        crate::expression::admission::ExpressionOccurrence::Node { body, position },
                    ),
                );
                conflict |= prior.is_some();
                position += 1;
            });
            if conflict {
                return Err(invalid(
                    field.declaration,
                    "source root occurs twice in one resolved inventory",
                ));
            }
        }
        Ok(())
    }
    /// Actual selected operation correspondence for checker-owned admissions.
    pub fn mapped(&self) -> &BTreeMap<crate::expression::admission::ExpressionOccurrence, NodeIx> {
        &self.mapped
    }
    fn checkpoint(&self) -> Result<()> {
        if self
            .syntax
            .nodes
            .len()
            .saturating_add(self.syntax.predicates.len())
            .saturating_add(self.syntax.binders.len())
            >= self.allowance
        {
            return Err(crate::ModelingError::Budget(
                "resolved syntax node allowance exceeded".into(),
            ));
        }
        Ok(())
    }
    fn node(&mut self, node: Node, expression: &Expr) -> Result<NodeIx> {
        self.checkpoint()?;
        let slot = NodeIx::from(self.syntax.nodes.len());
        self.syntax.nodes.push(node);
        let source = if let Some((source, occurrence)) = self
            .inventory
            .get(&(std::ptr::from_ref(expression) as usize))
        {
            self.mapped.insert(*occurrence, slot);
            source.clone()
        } else {
            Source {
                field: self.source.clone(),
                span: expression.span,
                coordinate: None,
            }
        };
        self.syntax.node_sources.insert(slot, source);
        Ok(slot)
    }
    fn binding(&mut self, name: String, binding: Binding) -> Result<BinderIx> {
        self.checkpoint()?;
        let slot = BinderIx::from(self.syntax.binders.len());
        self.syntax.binders.push(binding);
        self.lexical.push((name, slot));
        Ok(slot)
    }
    fn carrier(&self, path: &dsl::Path) -> Result<CarrierIx> {
        match self.names.selection(path) {
            Some(Selection::Carrier(slot)) if self.syntax.carriers.get(slot).is_some() => Ok(slot),
            _ => Err(invalid(
                self.source.declaration,
                "resolved carrier is absent or has the wrong role",
            )),
        }
    }
    fn function(&self, path: &dsl::Path) -> Result<FunctionIx> {
        self.names
            .function(path)
            .filter(|slot| self.syntax.functions.get(*slot).is_some())
            .ok_or_else(|| {
                invalid(
                    self.source.declaration,
                    "resolved function contract is absent",
                )
            })
    }
    fn arguments(&mut self, args: &[Expr]) -> Result<Vec<NodeIx>> {
        args.iter().map(|arg| self.expression(arg)).collect()
    }
    /// Resolve a reached expression once, retaining exact literal facts and original branch order.
    pub fn expression(&mut self, expression: &Expr) -> Result<NodeIx> {
        if self.depth >= 128 {
            return Err(crate::ModelingError::Budget(
                "resolved syntax depth exceeded".into(),
            ));
        }
        self.depth += 1;
        let result = self.expression_inner(expression);
        self.depth -= 1;
        result
    }
    fn expression_inner(&mut self, expression: &Expr) -> Result<NodeIx> {
        self.checkpoint()?;
        let node = match &expression.kind {
            ExprKind::Number(number) if number.value.is_finite() => Node::Number(number.clone()),
            ExprKind::Number(_) => {
                return Err(invalid(
                    self.source.declaration,
                    "nonfinite resolved literal",
                ));
            }
            ExprKind::Path(path) => {
                let lexical = path.ident().and_then(|name| {
                    self.lexical
                        .iter()
                        .rev()
                        .find(|(candidate, _)| candidate == name)
                        .map(|(_, slot)| *slot)
                });
                if let Some(slot) = lexical {
                    Node::Reference(Reference::Lexical(slot))
                } else {
                    let mut bare = path.clone();
                    let indices = bare
                        .segments
                        .iter_mut()
                        .flat_map(|segment| std::mem::take(&mut segment.indices))
                        .collect::<Vec<_>>();
                    match self.names.selection(&bare) {
                        Some(Selection::Symbol(slot))
                            if indices.is_empty() && self.syntax.symbols.at(slot).is_some() =>
                        {
                            Node::Reference(Reference::Symbol(slot))
                        }
                        Some(Selection::Group(group))
                            if self.syntax.groups.get(group).is_some() =>
                        {
                            if indices.len() != self.syntax.groups[group].axes.len() {
                                return Err(invalid(
                                    self.source.declaration,
                                    "resolved indexed reference has wrong rank",
                                ));
                            }
                            Node::Reference(Reference::Indexed {
                                group,
                                indices: self.arguments(&indices)?,
                            })
                        }
                        _ => {
                            return Err(invalid(
                                self.source.declaration,
                                "source reference has no actual resolved scalar or indexed owner",
                            ));
                        }
                    }
                }
            }
            ExprKind::Neg(value) => Node::Neg(self.expression(value)?),
            ExprKind::Binary { op, lhs, rhs } => Node::Binary {
                op: *op,
                lhs: self.expression(lhs)?,
                rhs: self.expression(rhs)?,
            },
            ExprKind::Call { function, args } => Node::Call {
                function: *function,
                args: self.arguments(args)?,
            },
            ExprKind::NamedCall { name, args } => Node::Apply {
                function: self.function(name)?,
                args: self.arguments(args)?,
            },
            ExprKind::Partial {
                function,
                wrt,
                args,
            } => {
                let function = self.function(function)?;
                let wrt = wrt
                    .iter()
                    .map(|path| {
                        path.ident()
                            .and_then(|name| {
                                self.syntax.functions[function]
                                    .formals
                                    .iter()
                                    .position(|candidate| candidate == name)
                            })
                            .ok_or_else(|| {
                                invalid(
                                    self.source.declaration,
                                    "partial argument is not an actual selected formal",
                                )
                            })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Node::Partial {
                    function,
                    wrt,
                    args: self.arguments(args)?,
                }
            }
            ExprKind::Kernel { name, args } => Node::Provider {
                selection: self.names.provider(name).ok_or_else(|| {
                    invalid(
                        self.source.declaration,
                        "resolved provider descriptor is absent",
                    )
                })?,
                args: self.arguments(args)?,
            },
            ExprKind::Reduce { kind, binder, body } => {
                let domain = self.carrier(&binder.domain)?;
                let depth = self.lexical.len();
                let binder_slot = self.binding(binder.var.clone(), Binding::Coordinate(domain))?;
                let filter = binder
                    .filter
                    .as_ref()
                    .map(|filter| self.predicate(filter))
                    .transpose()?;
                let body = self.expression(body)?;
                self.lexical.truncate(depth);
                Node::Reduce {
                    kind: *kind,
                    binder: binder_slot,
                    filter,
                    body,
                }
            }
            ExprKind::Fold {
                accumulator,
                item,
                binder,
                value,
                step,
            } => {
                let domain = self.carrier(&binder.domain)?;
                let depth = self.lexical.len();
                let binder_slot = self.binding(binder.var.clone(), Binding::Coordinate(domain))?;
                let filter = binder
                    .filter
                    .as_ref()
                    .map(|filter| self.predicate(filter))
                    .transpose()?;
                let value = self.expression(value)?;
                let accumulator = self.binding(accumulator.clone(), Binding::Accumulator)?;
                let item = self.binding(item.clone(), Binding::Item)?;
                let step = self.expression(step)?;
                self.lexical.truncate(depth);
                Node::Fold {
                    binder: binder_slot,
                    filter,
                    value,
                    accumulator,
                    item,
                    step,
                }
            }
            ExprKind::Derivative { body, wrt } => Node::Derivative {
                body: self.expression(body)?,
                domain: self.carrier(wrt)?,
            },
            ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => Node::Conditional {
                guard: self.predicate(guard)?,
                then: self.expression(then)?,
                otherwise: self.expression(otherwise)?,
            },
            ExprKind::Let { bindings, body } => {
                let depth = self.lexical.len();
                let mut slots = Vec::with_capacity(bindings.len());
                let mut names = std::collections::BTreeSet::new();
                for (name, value) in bindings {
                    if !names.insert(name) {
                        return Err(invalid(
                            self.source.declaration,
                            "duplicate resolved local binding",
                        ));
                    }
                    let value = self.expression(value)?;
                    slots.push(self.binding(name.clone(), Binding::Value(value))?);
                }
                let body = self.expression(body)?;
                self.lexical.truncate(depth);
                Node::Let {
                    bindings: slots,
                    body,
                }
            }
        };
        self.node(node, expression)
    }
    /// Resolve one original guard without normalizing away any source reads.
    pub fn predicate(&mut self, predicate: &Predicate) -> Result<PredicateIx> {
        if self.depth >= 128 {
            return Err(crate::ModelingError::Budget(
                "resolved predicate depth exceeded".into(),
            ));
        }
        self.depth += 1;
        let result = self.predicate_inner(predicate);
        self.depth -= 1;
        result
    }
    fn predicate_inner(&mut self, predicate: &Predicate) -> Result<PredicateIx> {
        self.checkpoint()?;
        let node = match &predicate.kind {
            PredicateKind::Compare { op, lhs, rhs } => ResolvedPredicate::Compare {
                op: *op,
                lhs: self.expression(lhs)?,
                rhs: self.expression(rhs)?,
            },
            PredicateKind::In { expr, domain } => ResolvedPredicate::In {
                value: self.expression(expr)?,
                domain: self.carrier(domain)?,
            },
            PredicateKind::And(lhs, rhs) => {
                ResolvedPredicate::And(self.predicate(lhs)?, self.predicate(rhs)?)
            }
            PredicateKind::Or(lhs, rhs) => {
                ResolvedPredicate::Or(self.predicate(lhs)?, self.predicate(rhs)?)
            }
            PredicateKind::Not(value) => ResolvedPredicate::Not(self.predicate(value)?),
            PredicateKind::Atom(value) => ResolvedPredicate::Atom(self.expression(value)?),
            PredicateKind::Bool(value) => ResolvedPredicate::Bool(*value),
            PredicateKind::Null => ResolvedPredicate::Null,
        };
        self.checkpoint()?;
        let slot = PredicateIx::from(self.syntax.predicates.len());
        self.syntax.predicates.push(node);
        self.syntax.predicate_sources.insert(
            slot,
            Source {
                field: self.source.clone(),
                span: predicate.span,
                coordinate: None,
            },
        );
        Ok(slot)
    }
    /// Publish a demanded root in caller order.
    pub fn output(&mut self, expression: &Expr) -> Result<NodeIx> {
        let root = self.expression(expression)?;
        self.syntax.outputs.push(root);
        Ok(root)
    }
    /// Publish original validity in source order, independently of branch simplification.
    pub fn validity(&mut self, predicate: &Predicate) -> Result<PredicateIx> {
        let root = self.predicate(predicate)?;
        self.syntax.validity.push(root);
        Ok(root)
    }
    /// Finish the immutable construction product; no source tree is reconstructed.
    pub fn finish(self) -> ResolvedSyntax {
        self.syntax
    }
}

impl ResolvedSyntax {
    pub(crate) fn view(&self) -> View<'_> {
        View {
            symbols: &self.symbols,
            arena: &self.arena,
            outputs: &self.outputs,
            validity: &self.validity,
        }
    }
    /// Select reached roots without copying operation tables, source maps or physical types.
    pub fn select(&self, outputs: Vec<NodeIx>, validity: Vec<PredicateIx>) -> Self {
        Self {
            symbols: self.symbols.clone(),
            arena: self.arena.clone(),
            outputs,
            validity,
        }
    }
    /// Canonical reached syntax identity. Global member IDs, spans and unused table rows stay out.
    pub fn identity(&self) -> Result<ContentHash> {
        framing::identity(self)
    }
    /// Issue canonical identity and local-role correspondence together; consumers do not recanonicalize.
    pub fn canonical_layout(&self) -> Result<CanonicalLayout> {
        framing::layout(self)
    }
    /// Original numerical/guard dependencies from issued typed edges only.
    pub fn reads(&self, request: RegionRequest<'_>) -> Result<RegionReads> {
        selection::reads_view(self.view(), request)
    }
    /// Conservative known owned extent, including source maps and typed finite facts.
    pub fn retained_bytes(&self) -> usize {
        extent::syntax(self)
    }
}

/// Frame every complete language type explicitly; no Debug rendering establishes physical meaning.
pub fn frame_type(ty: &Type, hash: &mut FramedHasher) {
    match ty {
        Type::Boolean => {
            hash.str("boolean");
        }
        Type::Integer => {
            hash.str("integer");
        }
        Type::Text => {
            hash.str("text");
        }
        Type::QuantityType => {
            hash.str("quantity_type");
        }
        Type::ReferenceState => {
            hash.str("reference_state");
        }
        Type::Applicability => {
            hash.str("applicability");
        }
        Type::Boundary(id) => {
            hash.str("boundary").id(&id.as_id());
        }
        Type::CoordinateMap(id) => {
            hash.str("coordinate_map").id(&id.as_id());
        }
        Type::Reconstruction { declaration, map } => {
            hash.str("reconstruction")
                .id(&declaration.as_id())
                .id(&map.as_id());
        }
        Type::Quantity(scheme) => {
            hash.str("quantity");
            scheme.frame(hash);
        }
        Type::RefinedQuantity {
            quantity,
            refinement,
        } => {
            hash.str("refined_quantity");
            quantity.frame(hash);
            refinement.frame(hash);
        }
        Type::Entity(id) => {
            hash.str("entity").id(&id.as_id());
        }
        Type::Enum(id) => {
            hash.str("enum").id(&id.as_id());
        }
        Type::Identifier(id) => {
            hash.str("identifier").id(&id.as_id());
        }
        Type::Set(element) => {
            hash.str("set");
            frame_type(element, hash);
        }
        Type::Continuous(id, element) => {
            hash.str("continuous").id(&id.as_id());
            frame_type(element, hash);
        }
        Type::Tuple(elements) => {
            hash.str("tuple").u64(elements.len() as u64);
            for element in elements {
                frame_type(element, hash);
            }
        }
        Type::Table(id) => {
            hash.str("table").id(&id.as_id());
        }
        Type::Row(id) => {
            hash.str("row").id(&id.as_id());
        }
        Type::Definition(id) => {
            hash.str("definition").id(&id.as_id());
        }
        Type::Interface(id) => {
            hash.str("interface").id(&id.as_id());
        }
        Type::Function { arguments, result } => {
            hash.str("function").u64(arguments.len() as u64);
            for (_, ty) in arguments {
                frame_type(ty, hash);
            }
            frame_type(result, hash);
        }
        Type::Optional(element) => {
            hash.str("optional");
            frame_type(element, hash);
        }
        Type::Indexed { element, axes } => {
            hash.str("indexed").u64(axes.len() as u64);
            for axis in axes {
                hash.id(&axis.as_id());
            }
            frame_type(element, hash);
        }
    }
}

#[cfg(test)]
#[path = "resolved/projection_tests.rs"]
mod projection_tests;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolved_index_projection_preserves_nominal_native_position_and_json() {
        use pse_contract::boundary_projection::{ConstructionRole, ProjectionGraph, ProjectionRecord};
        let native = NodeIx::from(usize::MAX);
        let original = serde_json::to_vec(&native).unwrap();
        let reconstructed = NodeIx::from_projection_fields(native.into_projection_fields()).unwrap();
        assert_eq!(reconstructed.position(), usize::MAX);
        assert_eq!(serde_json::to_vec(&reconstructed).unwrap(), original);
        let graph = ProjectionGraph::of::<NodeIx>().unwrap();
        assert_eq!(graph.definitions()["ModelingResolvedNodeIx"].construction, ConstructionRole::Data);
        assert_eq!(serde_json::to_vec(&SymbolIx::from(7)).unwrap(), b"7");
    }
    #[test]
    fn scientific_number_projection_preserves_finite_bits_and_full_integer_data() {
        use pse_contract::boundary_projection::{ProjectionGraph, ProjectionRecord, Scalar, Shape, TypeRef, ValidateProjection};
        for (bits, integer) in [(0, i128::MIN), (1, i128::MAX), (1u64 << 63, 0), (f64::MAX.to_bits(), -1)] {
            let number = dsl::Number {
                value: f64::from_bits(bits), exact_integer: Some(integer),
                unit: Some(pse_quantity::UnitProduct::symbol("m")),
            };
            let original = serde_json::to_vec(&number).unwrap();
            let restored = dsl::Number::from_projection_fields(number.into_projection_fields()).unwrap();
            assert_eq!(restored.value.to_bits(), bits);
            assert_eq!(restored.exact_integer, Some(integer));
            assert_eq!(serde_json::to_vec(&restored).unwrap(), original);
        }
        let graph = ProjectionGraph::of::<dsl::Number>().unwrap();
        let Shape::Record(fields) = &graph.definitions()["AuthoredScientificNumber"].shape else {
            panic!("actual authored literal fields")
        };
        assert_eq!(fields[0].ty, TypeRef::Scalar(Scalar::F64Bits));
        assert_eq!(fields[1].ty, TypeRef::Optional(Box::new(TypeRef::Scalar(Scalar::I128Bits))));
        assert!(graph.definitions().contains_key("PhysicalUnitProduct"));
        // Raw exact integer transport does not establish supported membership normalization.
        // Plain field reconstruction preserves data. Boundary validation and the original
        // scientific Serde codec both enforce the declared finite literal condition.
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let data = dsl::Number::from_projection_fields((value, None, None)).unwrap();
            assert!(data.validate_projection().is_err());
            assert!(serde_json::to_vec(&data).is_err());
        }
    }
    #[derive(Default)]
    struct Lookup(BTreeMap<String, Selection>);
    impl Names for Lookup {
        fn selection(&self, path: &dsl::Path) -> Option<Selection> {
            path.ident().and_then(|name| self.0.get(name)).copied()
        }
        fn function(&self, _: &dsl::Path) -> Option<FunctionIx> {
            None
        }
        fn provider(&self, _: &str) -> Option<ProviderSelection> {
            None
        }
    }
    fn source() -> crate::expression::occurrences::OccurrenceKey {
        crate::expression::occurrences::OccurrenceKey {
            declaration: SemanticId::from_bytes([1; 16]).into(),
            role: "function.body".into(),
            position: 0,
        }
    }
    fn syntax(text: &str) -> ResolvedSyntax {
        let names = Lookup::default();
        let mut resolver = Resolver::new(ResolvedSyntax::default(), &names, source(), 100);
        resolver.output(&dsl::parse_expr(text).unwrap()).unwrap();
        resolver.finish()
    }
    fn test_lineage() -> crate::specialize::Lineage {
        crate::specialize::Lineage {
            declaration: source().declaration,
            instance: SemanticId::NIL.into(),
            path: String::new(),
            demand: Vec::new(),
            default_owner: None,
            is_override: false,
            presets: Vec::new(),
        }
    }
    #[test]
    fn resolved_framing_is_alpha_and_span_independent_but_exact_about_literals_and_roles() {
        let first = syntax("x+x where x=1");
        let second = syntax("renamed + renamed where renamed = 1");
        assert_eq!(first.identity().unwrap(), second.identity().unwrap());
        assert_ne!(first.node_sources, second.node_sources);
        assert_ne!(
            syntax("0.0").identity().unwrap(),
            syntax("-0.0").identity().unwrap()
        );
        assert_ne!(
            syntax("9007199254740993").identity().unwrap(),
            syntax("9007199254740992").identity().unwrap()
        );
        assert_ne!(
            syntax("1 {m}").identity().unwrap(),
            syntax("1 {s}").identity().unwrap()
        );
        let mut names = Lookup::default();
        names
            .0
            .insert("x".into(), Selection::Symbol(SymbolIx::from(0)));
        let mut actual = ResolvedSyntax::default();
        actual.symbols.push(ResolvedSymbol {
            id: SemanticId::from_bytes([2; 16]),
            ty: Type::Integer,
            role: ModelingDeclarationKind::Variable,
            expression: None,
            initial: None,
            domain: ModelingVariableDomain::Continuous,
            definition: None,
            lineage: test_lineage(),
        });
        let mut resolver = Resolver::new(actual, &names, source(), 100);
        resolver.output(&dsl::parse_expr("x+1").unwrap()).unwrap();
        let mut actual = resolver.finish();
        let identity = actual.identity().unwrap();
        actual
            .symbols
            .rebind(SymbolIx::from(0), SemanticId::from_bytes([3; 16]))
            .unwrap();
        actual.node_sources.clear();
        assert_eq!(identity, actual.identity().unwrap());
        actual.symbols[SymbolIx::from(0)].role = ModelingDeclarationKind::Parameter;
        assert_ne!(identity, actual.identity().unwrap());
    }
    #[test]
    fn resolved_framing_preserves_typed_carrier_order_and_explicit_absence() {
        let mut actual = ResolvedSyntax::default();
        actual.carriers.push(Carrier {
            kind: CarrierKind::OrderedSet,
            element: Type::Integer,
            members: vec![
                crate::specialize::Value::Integer(1),
                crate::specialize::Value::Integer(2),
            ]
            .into(),
        });
        actual.groups.push(IndexedGroup {
            axes: vec![CarrierIx::from(0)],
            members: vec![(vec![crate::specialize::Value::Integer(1)], None)],
        });
        let names = Lookup(BTreeMap::from([(
            "x".into(),
            Selection::Group(GroupIx::from(0)),
        )]));
        let mut resolver = Resolver::new(actual, &names, source(), 100);
        resolver.output(&dsl::parse_expr("x[1]").unwrap()).unwrap();
        let mut actual = resolver.finish();
        let identity = actual.identity().unwrap();
        actual.carriers[CarrierIx::from(0)].members = vec![
            crate::specialize::Value::Integer(2),
            crate::specialize::Value::Integer(1),
        ]
        .into();
        assert_ne!(identity, actual.identity().unwrap());
        actual.groups[GroupIx::from(0)]
            .members
            .push((vec![crate::specialize::Value::Integer(1)], None));
        assert!(actual.identity().is_err());
    }
    #[test]
    fn resolved_lookup_refuses_unknown_members_and_keeps_lexical_scope_local() {
        let names = Lookup::default();
        let mut resolver = Resolver::new(ResolvedSyntax::default(), &names, source(), 100);
        resolver
            .output(&dsl::parse_expr("x where x=1").unwrap())
            .unwrap();
        assert!(resolver.output(&dsl::parse_expr("x").unwrap()).is_err());
        let names = BTreeMap::from([("known".into(), SymbolIx::from(0))]);
        let actual = resolver.finish();
        let borrowed = SymbolTypes {
            symbols: &actual.symbols,
            names: &names,
        };
        assert!(borrowed.get("unknown").is_none());
        assert!(borrowed.get("known").is_none());
    }
    #[test]
    fn resolved_aliases_cannot_erase_an_actual_bound_transfer_owner() {
        let mut actual = ResolvedSyntax::default();
        let quantity = pse_quantity::scheme::Scheme::Concrete(
            pse_quantity::QuantityTypeId::from_id(SemanticId::from_bytes([4; 16])),
        );
        let boundary = |byte| crate::BoundaryRef::Bound {
            instance: SemanticId::from_bytes([byte; 16]).into(),
            declaration: source().declaration,
            coordinates: vec![crate::specialize::Value::Integer(1)],
        };
        actual.symbols.push(ResolvedSymbol {
            id: SemanticId::from_bytes([2; 16]),
            ty: Type::RefinedQuantity {
                quantity,
                refinement: crate::PhysicalRefinement::Transfer {
                    boundary: boundary(5),
                    direction: crate::TransferDirection::Into,
                },
            },
            role: ModelingDeclarationKind::Variable,
            expression: None,
            initial: None,
            domain: ModelingVariableDomain::Continuous,
            definition: None,
            lineage: test_lineage(),
        });
        let names = Lookup(BTreeMap::from([(
            "x".into(),
            Selection::Symbol(SymbolIx::from(0)),
        )]));
        let mut resolver = Resolver::new(actual, &names, source(), 100);
        resolver.output(&dsl::parse_expr("x").unwrap()).unwrap();
        let mut actual = resolver.finish();
        let identity = actual.identity().unwrap();
        actual
            .symbols
            .rebind(SymbolIx::from(0), SemanticId::from_bytes([3; 16]))
            .unwrap();
        assert_eq!(identity, actual.identity().unwrap());
        let Type::RefinedQuantity {
            refinement:
                crate::PhysicalRefinement::Transfer {
                    boundary: owner, ..
                },
            ..
        } = &mut actual.symbols[SymbolIx::from(0)].ty
        else {
            panic!("transfer fixture");
        };
        *owner = boundary(6);
        assert_ne!(identity, actual.identity().unwrap());
    }
    #[test]
    fn canonical_layout_issues_the_exact_first_use_role_correspondence() {
        let mut actual = ResolvedSyntax::default();
        for (byte, role) in [(2, LocalRole::Coordinate), (3, LocalRole::Parameter)] {
            actual.symbols.push(ResolvedSymbol {
                id: SemanticId::from_bytes([byte; 16]),
                ty: Type::Integer,
                role: if role == LocalRole::Parameter {
                    ModelingDeclarationKind::Parameter
                } else {
                    ModelingDeclarationKind::Variable
                },
                expression: None,
                initial: None,
                domain: ModelingVariableDomain::Continuous,
                definition: None,
                lineage: test_lineage(),
            });
        }
        let names = Lookup(BTreeMap::from([
            ("x".into(), Selection::Symbol(SymbolIx::from(0))),
            ("y".into(), Selection::Symbol(SymbolIx::from(1))),
        ]));
        let mut resolver = Resolver::new(actual, &names, source(), 100);
        resolver.output(&dsl::parse_expr("y+x+y").unwrap()).unwrap();
        let actual = resolver.finish();
        let layout = actual.canonical_layout().unwrap();
        assert_eq!(layout.identity, actual.identity().unwrap());
        assert_eq!(
            layout.symbols,
            [
                CanonicalSymbol {
                    symbol: SymbolIx::from(1),
                    role: LocalRole::Parameter,
                    ordinal: 0
                },
                CanonicalSymbol {
                    symbol: SymbolIx::from(0),
                    role: LocalRole::Coordinate,
                    ordinal: 0
                },
            ]
        );
        assert_eq!(layout.nodes[0], actual.outputs[0]);
        assert_eq!(layout.nodes.len(), actual.nodes.len());
        assert!(layout.retained_bytes() >= size_of::<CanonicalLayout>());
    }
}

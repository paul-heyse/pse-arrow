// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Query-owned local typed function construction, separate from caller argument evaluation.
use super::*;
use crate::resolved::{FunctionProduct, Reference, functions::ArgumentLayout};
use std::sync::Arc;

/// Complete local construction input. No caller root arena or mutable evaluator escapes.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionFragmentInput {
    pub(super) package: Arc<CheckedPackage>,
    pub(super) recipe: Arc<FunctionFragmentRecipe>,
}
/// Complete local fields excluding the separately owned shared static package.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionFragmentRecipe {
    pub(super) function: DeclarationId,
    pub(super) instance: InstanceId,
    pub(super) contract: crate::Function,
    pub(super) formals: Vec<(String, Type)>,
    pub(super) lexical: BTreeMap<String, Expr>,
    pub(super) indexed: BTreeMap<String, functions::IndexedSelection>,
    pub(super) lexical_references: BTreeMap<String, Reference>,
    pub(super) substitution: pse_quantity::scheme::Substitution,
    pub(super) statics: Environment,
    pub(super) argument_layouts: Vec<ArgumentLayout>,
    pub(super) selections: crate::scientific_selection::Selections,
    pub(super) root: DeclarationId,
    pub(super) fixture: bool,
    pub(super) limits: Limits,
    pub(super) function_stack: Vec<DeclarationId>,
    pub(super) permissions: Vec<pse_model::applicability::Permission>,
}
impl Eq for FunctionFragmentRecipe {}
impl std::hash::Hash for FunctionFragmentRecipe {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Complete PartialEq, not this finite bucket hint, owns memo identity.
        self.function.hash(state);
        self.instance.hash(state);
        self.statics.hash(state);
    }
}
impl FunctionFragmentRecipe {
    pub(super) fn inspection_context(&self) -> ContentHash {
        let mut context = FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
        context.str("inspection-context").id(&self.function.as_id());
        for (name, value) in &self.statics {
            context.str(name);
            value.frame(&mut context);
        }
        context.finish_hash()
    }
    /// Conservative known extent; the static package has its own unique owner.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.contract.retained_bytes()
            + self
                .formals
                .iter()
                .map(|(name, ty)| name.capacity() + crate::extent::ty(ty) + 128)
                .sum::<usize>()
            + self
                .lexical
                .iter()
                .map(|(name, expression)| {
                    name.capacity() + crate::extent::expression(expression) + 128
                })
                .sum::<usize>()
            + self
                .statics
                .iter()
                .map(|(name, value)| name.capacity() + value.retained_bytes() + 128)
                .sum::<usize>()
            + crate::extent::selections(&self.selections)
            + self.function_stack.capacity() * size_of::<DeclarationId>()
            + self.permissions.capacity() * size_of::<pse_model::applicability::Permission>()
            + self
                .permissions
                .iter()
                .map(|permission| permission.targets.capacity() * size_of::<SemanticId>())
                .sum::<usize>()
            + self
                .indexed
                .iter()
                .map(|(name, group)| {
                    name.capacity()
                        + 128
                        + group
                            .members
                            .iter()
                            .map(|(tuple, value)| {
                                values(tuple)
                                    + crate::extent::expression(&value.inspection)
                                    + reference(&value.reference)
                                    + 128
                            })
                            .sum::<usize>()
                        + domains(&group.domains)
                })
                .sum::<usize>()
            + self
                .lexical_references
                .iter()
                .map(|(name, value)| name.capacity() + reference(value) + 128)
                .sum::<usize>()
            + self.argument_layouts.capacity() * size_of::<ArgumentLayout>()
            + self.argument_layouts.iter().map(layout).sum::<usize>()
            + self
                .substitution
                .iter()
                .map(|(name, contract)| name.capacity() + contract.heap_bytes() + 128)
                .sum::<usize>()
    }
}
impl FunctionFragmentInput {
    /// Preserve shared package ownership separately from local derivation fields.
    pub fn parts(&self) -> (Arc<CheckedPackage>, Arc<FunctionFragmentRecipe>) {
        (self.package.clone(), self.recipe.clone())
    }
    /// Reassemble immutable local input after reading its tracked package owner.
    pub fn from_parts(package: Arc<CheckedPackage>, recipe: Arc<FunctionFragmentRecipe>) -> Self {
        Self { package, recipe }
    }
}
/// Immutable local producer output; invocation edges bind actual caller arguments separately.
#[derive(Clone, Debug)]
pub struct FunctionFragment {
    pub(super) product: Arc<FunctionProduct>,
    pub(super) name: String,
    pub(super) inspections: BTreeMap<String, crate::Function>,
    pub(super) derivations: crate::derived::Facts,
    pub(super) selections: crate::scientific_selection::Selections,
    pub(super) finite: finite::FiniteFacts,
    pub(super) construction_items: usize,
    pub(super) original_guards: Vec<OriginalGuard>,
}
impl PartialEq for FunctionFragment {
    fn eq(&self, other: &Self) -> bool {
        self.product.identity == other.product.identity
            && self.product.syntax == other.product.syntax
            && self.name == other.name
            && self.inspections == other.inspections
            && self.derivations == other.derivations
            && self.selections == other.selections
            && self.finite == other.finite
            && self.construction_items == other.construction_items
            && self.original_guards == other.original_guards
    }
}
impl FunctionFragment {
    /// Conservative known result extent, including source/finite/selection receipts.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + product_extent(&self.product)
            + self.name.capacity()
            + self
                .inspections
                .iter()
                .map(|(name, function)| name.capacity() + function.retained_bytes() + 128)
                .sum::<usize>()
            + self.derivations.retained_bytes()
            + crate::extent::selections(&self.selections)
            + self.finite.retained_bytes()
            + self.original_guards.capacity() * size_of::<OriginalGuard>()
            + self
                .original_guards
                .iter()
                .map(|guard| guard.source.field.role.capacity() + guard.declarations.len() * 128)
                .sum::<usize>()
    }
}
/// Construct the same immutable typed function for direct and tracked specialization.
/// # Errors
/// Original source, physical, finite selection, recursion or resource refusals.
pub fn derive(
    input: &FunctionFragmentInput,
    fragments: Option<&dyn prelude::Fragments>,
    cancel: &dyn Fn() -> bool,
    discretizer: &dyn crate::continuous::Discretizer,
) -> Result<FunctionFragment> {
    let context = input.package.context();
    let mut engine = Engine::new(
        &input.package,
        &context,
        input.recipe.root,
        input.recipe.limits,
        cancel,
        discretizer,
        fragments,
    );
    engine.reader = if input.recipe.fixture {
        crate::provenance::Reader::Fixture
    } else {
        crate::provenance::Reader::Production(input.recipe.root)
    };
    engine.function_stack = input.recipe.function_stack.clone();
    engine.inspection_context = Some(input.recipe.inspection_context());
    engine.function_permissions = Some(Arc::new(input.recipe.permissions.clone()));
    engine.states.insert(
        input.recipe.instance,
        State {
            selections: BTreeMap::new(),
            presets: Vec::new(),
            definition: input.recipe.function,
            parent: None,
            env: Environment::new(),
            scope: Environment::new(),
            path: String::new(),
            members: BTreeMap::new(),
            member_kinds: BTreeMap::new(),
            children: BTreeMap::new(),
            symbols: BTreeMap::new(),
            stack: Vec::new(),
        },
    );
    let (product, name) = engine.build_function_fragment(input)?;
    if !engine.model.symbols.is_empty() || !engine.model.equations.is_empty() {
        return Err(invalid(
            input.recipe.function,
            "local function construction escaped its explicit typed function scope",
        ));
    }
    Ok(FunctionFragment {
        product,
        name,
        inspections: engine.model.functions,
        derivations: engine.model.derivations,
        selections: engine.selection_collector.into_inner(),
        finite: engine.finite_collector.finish(),
        construction_items: engine.count,
        original_guards: engine.original_guards,
    })
}

fn values(values: &[Value]) -> usize {
    values.iter().map(Value::retained_bytes).sum()
}
fn reference(value: &Reference) -> usize {
    match value {
        Reference::Indexed { indices, .. } => {
            indices.capacity() * size_of::<crate::resolved::NodeIx>()
        }
        _ => 0,
    }
}
fn domains(domains: &[(usize, Vec<Value>, crate::resolved::Carrier)]) -> usize {
    size_of_val(domains)
        + domains
            .iter()
            .map(|(_, prefix, carrier)| values(prefix) + carrier_extent(carrier))
            .sum::<usize>()
}
fn carrier_extent(carrier: &crate::resolved::Carrier) -> usize {
    crate::extent::ty(&carrier.element) + values(&carrier.members)
}
fn layout(layout: &ArgumentLayout) -> usize {
    crate::extent::ty(&layout.ty)
        + layout.tuples.capacity() * size_of::<Vec<Value>>()
        + layout
            .tuples
            .iter()
            .map(|tuple| values(tuple))
            .sum::<usize>()
        + domains(&layout.domains)
        + layout
            .static_value
            .as_ref()
            .map_or(0, Value::retained_bytes)
}
fn reads(reads: &crate::envelope::Reads) -> usize {
    reads.sets.capacity() * size_of::<SemanticId>() + reads.variables.capacity() * size_of::<u32>()
}
fn boundary(boundary: &pse_quantity::AdmittedOutputBoundary) -> usize {
    boundary.source().heap_bytes() + boundary.target().heap_bytes()
}
fn product_extent(product: &Arc<FunctionProduct>) -> usize {
    product_extent_once(product, &mut BTreeSet::new())
}
fn product_extent_once(product: &Arc<FunctionProduct>, seen: &mut BTreeSet<usize>) -> usize {
    if !seen.insert(Arc::as_ptr(product) as usize) {
        return 0;
    }
    size_of::<FunctionProduct>()
        + product.syntax.retained_bytes()
        + product.physical.retained_bytes()
        + product
            .syntax
            .functions
            .iter()
            .map(|function| product_extent_once(&function.product, seen))
            .sum::<usize>()
        + product.formals.capacity() * size_of::<crate::resolved::SymbolIx>()
        + product.signature.capacity() * size_of::<Type>()
        + product
            .signature
            .iter()
            .map(crate::extent::ty)
            .sum::<usize>()
        + product.arguments.capacity() * size_of::<ArgumentLayout>()
        + product.arguments.iter().map(layout).sum::<usize>()
        + product
            .selections
            .iter()
            .map(|meaning| {
                values(&meaning.closure.roots)
                    + meaning.closure.context.retained_bytes()
                    + values(&meaning.closure.records)
                    + meaning
                        .closure
                        .edges
                        .iter()
                        .map(|(record, edges)| {
                            record.retained_bytes()
                                + edges
                                    .iter()
                                    .map(|value| value.retained_bytes() + 128)
                                    .sum::<usize>()
                                + 128
                        })
                        .sum::<usize>()
                    + meaning
                        .records
                        .iter()
                        .map(|record| {
                            record
                                .values
                                .iter()
                                .map(|(name, value)| name.capacity() + value.retained_bytes() + 128)
                                .sum::<usize>()
                                + record
                                    .uncertainties
                                    .keys()
                                    .map(|name| name.capacity() + 128)
                                    .sum::<usize>()
                                + size_of_val(record)
                        })
                        .sum::<usize>()
                    + 256
            })
            .sum::<usize>()
        + product
            .literals
            .values()
            .map(|ty| crate::extent::ty(ty) + 128)
            .sum::<usize>()
        + crate::extent::ty(&product.result)
        + product.envelopes.capacity() * size_of::<crate::resolved::functions::FunctionEnvelope>()
        + product
            .envelopes
            .iter()
            .map(|envelope| crate::extent::ty(&envelope.ty) + reads(&envelope.reads))
            .sum::<usize>()
        + product.applicability.capacity() * size_of::<crate::resolved::functions::EvidenceUse>()
        + product
            .applicability
            .iter()
            .map(|usage| {
                usage.node.retained_bytes()
                    + usage.inputs.capacity() * size_of::<crate::resolved::NodeIx>()
                    + usage.predicates.capacity() * size_of::<crate::resolved::PredicateIx>()
            })
            .sum::<usize>()
        + product.physical_operation.as_ref().map_or(
            0,
            crate::resolved::operation::OperationMeaning::retained_bytes,
        )
        + product.prerequisites.capacity() * size_of::<usize>()
        + product
            .admissions
            .values()
            .map(|admission| admission.retained_bytes() + 128)
            .sum::<usize>()
        + product.reduction_admission.as_ref().map_or(
            0,
            crate::expression::admission::PhysicalAdmission::retained_bytes,
        )
        + product
            .reduction
            .as_ref()
            .map_or(0, |reduction| reduction.prototype.heap_bytes())
        + product.reduction_selection.as_ref().map_or(0, |selection| {
            carrier_extent(&selection.carrier) + values(&selection.selected)
        })
        + product.output_boundary.as_ref().map_or(0, boundary)
        + product.body_inputs.capacity() * size_of::<Option<crate::resolved::BodyInputSupport>>()
        + product
            .body_inputs
            .iter()
            .flatten()
            .map(|input| boundary(&input.boundary))
            .sum::<usize>()
        + reads(&product.validity_reads)
        + product.external.as_ref().map_or(0, |external| {
            external.implementation.capacity()
                + external.shapes.capacity() * size_of::<crate::external::ArgumentShape>()
                + external
                    .shapes
                    .iter()
                    .map(|shape| {
                        shape.axes.capacity() * size_of::<SemanticId>()
                            + shape.coordinates.capacity() * size_of::<Vec<SemanticId>>()
                            + shape
                                .coordinates
                                .iter()
                                .map(|tuple| tuple.capacity() * size_of::<SemanticId>())
                                .sum::<usize>()
                    })
                    .sum::<usize>()
        })
}

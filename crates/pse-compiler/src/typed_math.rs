// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Direct authored syntax to physically admitted library bodies.
//! Source occurrences and formal bindings remain separate from Symbolica arithmetic.
#[path = "typed_math_witness.rs"]
mod witness;
use pse_authoring::dsl::{self, BinaryOp, CompareOp, Expr, ExprKind, PredicateKind};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError,
    binding::BodySpec,
    guarded::Comparison,
    typed::{Binary, BodyBuilder, BodyLimits, Guard, TypedValue},
};
#[cfg(test)]
use pse_math::{guarded::CompiledBody, library::Optimization};
use pse_modeling::DeclarationId;
use pse_quantity::{
    IndexSet, QuantityRegistry, QuantityTypeId, Ratio, infer::InvariantChecker,
    literal::LiteralContext,
};
use std::sync::Arc;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
};
use witness::WitnessPass;

// Synthetic indicator arms are attributed by the enclosing domain/applicability owner.
fn indicator_arms() -> (Expr, Expr) {
    let literal = |value| Expr {
        kind: ExprKind::Number(dsl::Number {
            value,
            exact_integer: None,
            unit: None,
        }),
        span: dsl::Span::default(),
    };
    (literal(1.0), literal(0.0))
}

/// One formal scalar declaration, independent of global instance values.
#[derive(Clone, Debug, PartialEq)]
pub struct Formal {
    /// Resolved authored path.
    pub path: String,
    /// Complete scalar physical contract.
    pub quantity: QuantityTypeId,
}

/// An admitted finite axis, shared by any number of indexed groups.
#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    /// Actual finite membership in semantic order.
    pub members: pse_math::binding::FiniteDomain,
    /// Physical domain role.
    pub kind: pse_quantity::EntityKindId,
}
/// One finite or ragged group, preserving declared axis order and actual tuples.
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    /// Physical quantity including the declared shape.
    pub quantity: QuantityTypeId,
    /// Domain names in declared axis order.
    pub axes: Vec<String>,
    /// Actual member tuple to reusable local input slot. Missing tuples are errors.
    pub slots: BTreeMap<Vec<SemanticId>, usize>,
}
#[derive(Clone, Copy)]
struct Coordinate {
    bound: pse_quantity::BoundIndexRef,
    member: Option<SemanticId>,
}

/// Exact source occurrence retained for diagnostics, excluded from body reuse identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    /// Stable source occurrence identity.
    pub id: SemanticId,
    /// Authored definition owning this local occurrence.
    pub definition: SemanticId,
    /// Byte range in the authored definition.
    pub span: dsl::Span,
}

/// A scalar output selection from an executable, potentially multi-output provider.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderCall {
    /// Immutable physically admitted provider descriptor.
    pub descriptor: pse_kernels::AdmittedProvider,
    /// Selected output ordinal.
    pub output: usize,
}

/// Physical validity attached to a resolved local path, before library normalization.
#[derive(Clone, Debug, PartialEq)]
pub struct Validity {
    /// Lower endpoint of the validity range.
    pub lower: Expr,
    /// Upper endpoint of the validity range.
    pub upper: Expr,
    /// Declaration that authored the range.
    pub source: SemanticId,
    /// The member the range bounds.
    pub target: SemanticId,
}
/// One authored local body and the exact physical/specialization context it consumes.
#[derive(Debug)]
pub struct Request<'a> {
    /// Definition identity for diagnostics, not arithmetic interning.
    pub definition: SemanticId,
    /// Source-bearing syntax from the authoring parser.
    pub expressions: &'a [Expr],
    /// Formal input layout in declared order.
    pub formals: &'a [Formal],
    /// Finite domains by resolved authored name.
    pub domains: &'a BTreeMap<String, Domain>,
    /// Indexed group declarations and actual tuple membership.
    pub groups: &'a BTreeMap<String, Group>,
    /// Executable provider output selections by authored kernel path.
    pub providers: &'a BTreeMap<String, ProviderCall>,
    /// Contextual literal contracts keyed by exact source range.
    pub literals: &'a BTreeMap<(u32, u32), QuantityTypeId>,
    /// Complete physical registry content identity.
    pub physical: ContentHash,
    /// Structural specialization content identity.
    pub structure: ContentHash,
    /// Bounded local construction profile.
    pub limits: BodyLimits,
}

/// Immutable physically admitted definition, independent of compilation options.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedBody {
    semantic_identity: Option<pse_ids::roles::SemanticBodyHash>,
    /// Durable semantic specification.
    pub(crate) spec: BodySpec,
    /// Result's complete physical contract.
    pub(crate) quantities: Vec<QuantityTypeId>,
    /// Authored occurrence provenance.
    pub(crate) occurrences: Vec<Occurrence>,
    /// Symbolica regions and complete structural support, without mutable workspaces.
    pub(crate) math: Arc<pse_math::guarded::PreparedBody>,
}
impl AdmittedBody {
    /// Compiler-issued complete dependency identity, sealed against external mutation.
    pub fn semantic_identity(&self) -> Option<pse_ids::roles::SemanticBodyHash> {
        self.semantic_identity
    }
    pub(crate) fn for_semantic_identity(
        mut self,
        identity: pse_ids::roles::SemanticBodyHash,
    ) -> Self {
        self.semantic_identity = Some(identity);
        self
    }
    /// Borrow its admitted specification without exposing mutation of its witness.
    pub fn spec(&self) -> &BodySpec {
        &self.spec
    }
    /// Ordered admitted output quantities.
    pub fn quantities(&self) -> &[QuantityTypeId] {
        &self.quantities
    }
    /// Immutable admitted mathematics; body clones retain original accounting.
    pub fn math(&self) -> &Arc<pse_math::guarded::PreparedBody> {
        &self.math
    }
    /// Known descriptor/vector allocation extent, excluding shared mathematics.
    pub fn descriptor_bytes(&self) -> usize {
        size_of::<Self>()
            + self.spec.providers.capacity() * size_of::<ContentHash>()
            + self.quantities.capacity() * size_of::<QuantityTypeId>()
            + self.occurrences.capacity() * size_of::<Occurrence>()
    }
    /// Additional owned descriptor and math-handle wrappers when attaching an owner;
    /// immutable library mathematics stays shared.
    pub(crate) fn owner_attachment_bytes(&self) -> usize {
        size_of::<Self>()
            + size_of::<pse_math::guarded::PreparedBody>()
            + 256
            + self.spec.providers.len() * size_of::<ContentHash>()
            + self.quantities.len() * size_of::<QuantityTypeId>()
            + self.occurrences.len() * size_of::<Occurrence>()
    }
    /// Attach accounting without changing sealed scientific meaning.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.math = Arc::new(self.math.as_ref().clone().with_owner(owner));
        self
    }
}
/// An admitted definition and its immutable demand artifact (unit fixture only).
#[cfg(test)]
#[derive(Clone, Debug)]
pub struct Body {
    /// Single semantic authority, reusable across compilation profiles.
    pub prepared: Arc<AdmittedBody>,
    /// Immutable compiled library programs.
    pub artifact: Arc<CompiledBody>,
}
#[cfg(test)]
impl std::ops::Deref for Body {
    type Target = AdmittedBody;
    fn deref(&self) -> &Self::Target {
        &self.prepared
    }
}

#[cfg(test)]
impl Body {
    /// A worker receives its own mutable library scratch.
    pub fn worker(&self) -> pse_math::guarded::Worker {
        self.artifact.worker()
    }
}

impl Request<'_> {
    /// Compile only this specialization; no model-wide graph or relation transport.
    /// # Errors
    /// Physical/name/profile failure, bounded resource refusal or library error.
    #[cfg(test)]
    fn compile(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        order: DerivativeOrder,
        options: Optimization,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<Body, MathError> {
        pse_math::initialize()?;
        let prepared = Arc::new(self.admit(registry, checker, cancelled)?);
        let artifact = Arc::new(prepared.math.compile(
            &(0..prepared.math.output_count()).collect::<Vec<_>>(),
            &(0..self.formals.len()).collect::<Vec<_>>(),
            order,
            options,
            pse_math::jets::EvaluationLimits::default(),
            cancelled,
        )?);
        Ok(Body { prepared, artifact })
    }
    /// Admit ordered outputs and consumed contracts without constructing numeric artifacts.
    pub fn admit(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<AdmittedBody, MathError> {
        self.admit_functions(registry, checker, cancelled, &BTreeMap::new())
    }
    /// Admit a finite body with checked package function declarations.
    pub fn admit_functions(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        cancelled: &Arc<AtomicBool>,
        functions: &BTreeMap<String, pse_modeling::Function>,
    ) -> Result<AdmittedBody, MathError> {
        self.admit_function_outputs(
            registry,
            checker,
            cancelled,
            functions,
            &[],
            &BTreeMap::new(),
        )
    }
    /// Admit output contracts without repurposing source spans as synthetic type keys.
    pub fn admit_function_outputs(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        cancelled: &Arc<AtomicBool>,
        functions: &BTreeMap<String, pse_modeling::Function>,
        outputs: &[QuantityTypeId],
        local_quantities: &BTreeMap<String, QuantityTypeId>,
    ) -> Result<AdmittedBody, MathError> {
        self.admit_modeling_outputs(
            registry,
            checker,
            cancelled,
            functions,
            outputs,
            local_quantities,
            &BTreeMap::new(),
        )
    }
    /// Admit definition-owned range obligations with the same expression and library authority.
    #[expect(
        clippy::too_many_arguments,
        reason = "the physical registry, invariant checker and cancellation accompany the model's separately owned function, output, quantity and validity tables"
    )]
    pub fn admit_modeling_outputs(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        cancelled: &Arc<AtomicBool>,
        functions: &BTreeMap<String, pse_modeling::Function>,
        outputs: &[QuantityTypeId],
        local_quantities: &BTreeMap<String, QuantityTypeId>,
        validity: &BTreeMap<String, Validity>,
    ) -> Result<AdmittedBody, MathError> {
        if !outputs.is_empty() && outputs.len() != self.expressions.len() {
            return Err(MathError::Contract("output physical contract arity".into()));
        }
        let entries = self
            .domains
            .values()
            .map(|v| v.members.members().len())
            .chain(self.groups.values().map(|v| v.slots.len()))
            .try_fold(self.formals.len(), usize::checked_add)
            .ok_or(MathError::Limit("preparation cardinality"))?;
        let names = self
            .formals
            .iter()
            .map(|v| v.path.len())
            .chain(self.domains.keys().map(String::len))
            .chain(self.groups.keys().map(String::len))
            .try_fold(0usize, usize::checked_add)
            .ok_or(MathError::Limit("preparation text"))?;
        if entries > self.limits.occurrences
            || names > self.limits.occurrences.saturating_mul(256)
            || self.providers.len() > self.limits.slots
        {
            return Err(MathError::Limit("preparation inputs"));
        }
        let mut builder = BodyBuilder::new(
            pse_math::context()?,
            registry,
            checker,
            self.formals.len(),
            self.limits,
        )?;
        let mut paths = BTreeMap::new();
        let mut hash = FramedHasher::new(pse_ids::Frame::MathTypedDefinitionV10);
        hash.u64(self.formals.len() as u64);
        for (slot, formal) in self.formals.iter().enumerate() {
            if paths.insert(formal.path.clone(), slot).is_some() {
                return Err(MathError::Contract("duplicate formal path".into()));
            }
            registry.quantity_type(formal.quantity)?;
            hash.str(&formal.path).id(&formal.quantity.as_id());
        }
        let mut lower = Lower {
            request: self,
            registry,
            checker,
            paths,
            locals: BTreeMap::new(),
            occurrences: vec![],
            hash,
            cancelled,
            coordinates: BTreeMap::new(),
            physical_only: false,
            functions,
            local_quantities,
            calls: Vec::new(),
            validity,
            validating: Vec::new(),
            operation_scope: None,
            witness_pass: None,
        };
        if self.expressions.is_empty() {
            return Err(MathError::Contract("empty authored output group".into()));
        }
        lower.hash.u64(self.expressions.len() as u64);
        let values = self
            .expressions
            .iter()
            .enumerate()
            .map(|(i, expr)| {
                let value =
                    lower.expression_expected(expr, &mut builder, 0, outputs.get(i).copied())?;
                match outputs.get(i) {
                    Some(expected) => builder.named_boundary(value, *expected),
                    None => Ok(value),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let quantities = values
            .iter()
            .map(TypedValue::quantity)
            .collect::<Result<Vec<_>, _>>()?;
        builder.frame_admissions(&mut lower.hash);
        let admissions = builder.admission_identity();
        let math = Arc::new(builder.prepare(&values)?);
        let spec = BodySpec {
            definition: lower.hash.finish_hash(),
            structure: self.structure,
            physical: self.physical,
            admissions,
            providers: math
                .providers()
                .iter()
                .map(pse_kernels::ProviderSpec::identity)
                .collect(),
            policy: pse_math::binding::guarded_real_policy(),
        };
        Ok(AdmittedBody {
            semantic_identity: None,
            spec,
            quantities,
            occurrences: lower.occurrences,
            math,
        })
    }
}

#[derive(Clone)]
struct OperationScope {
    obligations: pse_modeling::expression::admission::ExpressionAdmissions,
    admissions: BTreeMap<
        pse_modeling::expression::admission::ExpressionOccurrence,
        pse_quantity::ResolvedInference,
    >,
    occurrences: BTreeMap<usize, pse_modeling::expression::admission::ExpressionOccurrence>,
}
/// A short-circuit continuation retains the checked AST references of every branch.
enum ConditionalBranch<'a> {
    Expression(&'a Expr),
    Predicate {
        predicate: &'a dsl::Predicate,
        then: &'a ConditionalBranch<'a>,
        otherwise: &'a ConditionalBranch<'a>,
    },
}
struct Lower<'a, 'b> {
    checker: &'a dyn InvariantChecker,
    request: &'a Request<'b>,
    registry: &'a QuantityRegistry,
    paths: BTreeMap<String, usize>,
    locals: BTreeMap<String, TypedValue>,
    occurrences: Vec<Occurrence>,
    hash: FramedHasher,
    cancelled: &'a Arc<AtomicBool>,
    coordinates: BTreeMap<String, Coordinate>,
    physical_only: bool,
    functions: &'a BTreeMap<String, pse_modeling::Function>,
    local_quantities: &'a BTreeMap<String, QuantityTypeId>,
    calls: Vec<DeclarationId>,
    validity: &'a BTreeMap<String, Validity>,
    validating: Vec<String>,
    operation_scope: Option<OperationScope>,
    witness_pass: Option<WitnessPass>,
}
impl Lower<'_, '_> {
    fn finite_reduction_admission(
        &self,
        function: &pse_modeling::Function,
    ) -> Result<pse_quantity::ResolvedInference, MathError> {
        self.operation_scope
            .as_ref()
            .and_then(|scope| {
                scope.admissions.get(
                    &pse_modeling::expression::admission::ExpressionOccurrence::finite_reduction(
                        function.id,
                    ),
                )
            })
            .cloned()
            .ok_or_else(|| {
                MathError::Contract(
                    "checked finite reduction occurrence is missing its admission".into(),
                )
            })
    }
    fn validated(
        &mut self,
        name: &str,
        value: TypedValue,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
    ) -> Result<TypedValue, MathError> {
        if !self.calls.is_empty() {
            return Ok(value);
        }
        let Some(range) = self.validity.get(name).cloned() else {
            return Ok(value);
        };
        if self.validating.iter().any(|n| n == name) {
            return Err(MathError::Contract(
                "cyclic validity range dependency".into(),
            ));
        }
        self.validating.push(name.into());
        // The closure layer's range names the member it bounds (Plan 23 H5).
        self.hash
            .str("validity")
            .id(&range.source)
            .id(&range.target);
        let lower =
            self.expression_expected(&range.lower, builder, depth + 1, Some(value.quantity()?))?;
        let upper =
            self.expression_expected(&range.upper, builder, depth + 1, Some(value.quantity()?))?;
        self.validating.pop();
        builder.within_range(
            value,
            lower,
            upper,
            Arc::new(pse_model::diagnostic::ValidityLineage {
                layer: pse_model::generated::enums::ModelingValidityLayer::Closure,
                source: range.source,
                form: None,
                sets: Vec::new(),
                variables: Vec::new(),
                members: vec![range.target],
            }),
        )
    }
    fn source(&mut self, expr: &Expr, depth: usize) -> Result<SemanticId, MathError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if depth > 128 {
            return Err(MathError::Limit("authored syntax depth"));
        }
        if self.occurrences.len() >= self.request.limits.occurrences {
            return Err(MathError::Limit("authored syntax occurrences"));
        }
        let mut h = FramedHasher::new(pse_ids::Frame::MathLocalOccurrenceV3);
        h.u64(self.occurrences.len() as u64);
        let id = h.finish_id();
        self.occurrences.push(Occurrence {
            id,
            definition: self.request.definition,
            span: expr.span,
        });
        Ok(id)
    }
    /// Instantiate an expression operation before numerical construction. A checked
    /// function body must contain its occurrence product; missing admission is a refusal.
    fn arithmetic_admission(
        &self,
        expression: &Expr,
        request: pse_quantity::infer::OpRequest<'_>,
        operands: &[&TypedValue],
        builder: &BodyBuilder<'_>,
    ) -> Result<pse_quantity::ResolvedInference, MathError> {
        if let Some(scope) = &self.operation_scope {
            let occurrence = scope
                .occurrences
                .get(&(std::ptr::from_ref(expression) as usize))
                .ok_or_else(|| {
                    MathError::Contract(
                        "operation is outside its checked function occurrence tree".into(),
                    )
                })?;
            let admission = scope.admissions.get(occurrence).cloned().ok_or_else(|| {
                MathError::Contract(
                    format!(
                        "checked function operation occurrence is missing its admission: function {:?}, occurrence {occurrence:?}, request {request:?}",
                        self.calls.last(),
                    ),
                )
            })?;
            if let pse_quantity::infer::OpRequest::Reduce { kind, bound } = request {
                let [prototype] = operands else {
                    return Err(MathError::Contract(
                        "bound reduction prototype arity".into(),
                    ));
                };
                return scope
                    .obligations
                    .get(occurrence)
                    .ok_or_else(|| {
                        MathError::Contract(
                            "checked reduction occurrence is missing its obligation".into(),
                        )
                    })?
                    .instantiate_reduction_binder(
                        &admission,
                        kind,
                        bound,
                        prototype.physical_contract(),
                        self.registry,
                        self.checker,
                        DeclarationId::from(self.request.definition),
                    )
                    .map_err(|error| MathError::Contract(error.to_string()));
            }
            return Ok(admission);
        }
        // A Request is the admission front door for a finite mathematical definition.
        // Its actual group binders and literal contracts are now resolved. The modeling
        // checker creates the product here; BodyBuilder only consumes it below.
        pse_modeling::expression::admission::admit_operation(
            &request,
            &operands
                .iter()
                .map(|value| value.physical_contract().clone())
                .collect::<Vec<_>>(),
            self.registry,
            self.checker,
            builder.physical_formula_authority(),
            DeclarationId::from(self.request.definition),
        )
        .map_err(|error| MathError::Contract(error.to_string()))
    }
    fn expression(
        &mut self,
        expr: &Expr,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
    ) -> Result<TypedValue, MathError> {
        self.expression_expected(expr, builder, depth, None)
    }
    fn expression_expected(
        &mut self,
        expr: &Expr,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        expected: Option<QuantityTypeId>,
    ) -> Result<TypedValue, MathError> {
        let source = self.source(expr, depth)?;
        match &expr.kind {
            ExprKind::Number(number) => {
                self.hash
                    .str("literal")
                    .u64(pse_ids::canonical_f64_bits(number.value));
                let unit = if let Some(product) = &number.unit {
                    // Composed from atomic factors; its identity is spelling-independent
                    // (ADR-0124).
                    self.registry.compose(product)?
                } else {
                    // A bare scalar uses the physical registry's neutral unit.
                    let quantity = self.registry.neutral_dimensionless().ok_or_else(|| {
                        MathError::Contract("no declared neutral literal type".into())
                    })?;
                    self.registry
                        .unit(self.registry.quantity_type(quantity)?.canonical_unit)?
                        .clone()
                };
                self.hash.id(&unit.id.as_id());
                let context = if let Some(&quantity) =
                    self.request.literals.get(&(expr.span.start, expr.span.end))
                {
                    self.hash.id(&quantity.as_id());
                    LiteralContext::Explicit {
                        quantity_type: quantity,
                    }
                } else if number.unit.is_none() {
                    let quantity = self.registry.neutral_dimensionless().ok_or_else(|| {
                        MathError::Contract("no declared neutral literal type".into())
                    })?;
                    LiteralContext::Explicit {
                        quantity_type: quantity,
                    }
                } else if let Some(quantity_type) = expected {
                    self.hash.id(&quantity_type.as_id());
                    LiteralContext::Explicit { quantity_type }
                } else {
                    LiteralContext::Free
                };
                builder.literal(number.value, &unit, context, source)
            }
            ExprKind::Path(path) => {
                let name = path
                    .segments
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join(".");
                self.hash.str("path").str(&name);
                let index_exprs: Vec<_> = path.segments.iter().flat_map(|s| &s.indices).collect();
                if !index_exprs.is_empty() {
                    return self.group(&name, &index_exprs, builder, source);
                }
                if let Some(value) = self.locals.get(&name) {
                    let value = value.clone();
                    return self.validated(&name, value, builder, depth);
                }
                let slot = *self
                    .paths
                    .get(&name)
                    .ok_or_else(|| MathError::Contract(format!("unresolved path {name}")))?;
                let value = builder.input(
                    slot,
                    self.request.formals[slot].quantity,
                    IndexSet::new(),
                    source,
                )?;
                self.validated(&name, value, builder, depth)
            }
            ExprKind::Neg(value) => {
                self.hash.str("neg");
                let value = self.expression_expected(value, builder, depth + 1, expected)?;
                let admission = self.arithmetic_admission(
                    expr,
                    pse_quantity::infer::OpRequest::Neg,
                    &[&value],
                    builder,
                )?;
                builder.negate_admitted(value, admission, source)
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.hash.str(op.as_str());
                let additive = matches!(op, BinaryOp::Add | BinaryOp::Sub);
                let left = self.expression_expected(
                    lhs,
                    builder,
                    depth + 1,
                    if additive { expected } else { None },
                )?;
                let right = self.expression_expected(
                    rhs,
                    builder,
                    depth + 1,
                    if additive {
                        left.physical_contract().named_id()
                    } else {
                        None
                    },
                )?;
                let exponent = if *op == BinaryOp::Pow {
                    literal_exponent(rhs)
                } else {
                    None
                };
                let operation = match op {
                    BinaryOp::Add => Binary::Add,
                    BinaryOp::Sub => Binary::Sub,
                    BinaryOp::Mul => Binary::Mul,
                    BinaryOp::Div => Binary::Div,
                    BinaryOp::Pow => Binary::Pow,
                };
                let request = match op {
                    BinaryOp::Add => pse_quantity::infer::OpRequest::Add,
                    BinaryOp::Sub => pse_quantity::infer::OpRequest::Sub,
                    BinaryOp::Mul => pse_quantity::infer::OpRequest::Mul,
                    BinaryOp::Div => pse_quantity::infer::OpRequest::Div,
                    BinaryOp::Pow => pse_quantity::infer::OpRequest::Pow {
                        exponent: exponent.map_or(
                            pse_quantity::infer::Exponent::Symbolic,
                            pse_quantity::infer::Exponent::Rational,
                        ),
                    },
                };
                let admission =
                    self.arithmetic_admission(expr, request, &[&left, &right], builder)?;
                builder.binary_admitted(operation, left, right, exponent, admission, source)
            }
            ExprKind::Call { function, args } => {
                self.hash.str("function").str(function.as_str());
                if function.implementation() == pse_math::Implementation::Unavailable {
                    return Err(MathError::Contract(format!(
                        "{} is unavailable in the scalar profile",
                        function.as_str()
                    )));
                }
                if let pse_math::Implementation::Extremum { minimum } = function.implementation()
                    && args.len() == 2
                {
                    let left = self.expression(&args[0], builder, depth + 1)?;
                    let right = self.expression(&args[1], builder, depth + 1)?;
                    return builder.extremum(minimum, left, right, source);
                }
                if args.len() != 1 {
                    return Err(MathError::Contract(
                        "function requires its distinct constructor and argument contract".into(),
                    ));
                }
                let value = self.expression(&args[0], builder, depth + 1)?;
                let request = match function.as_str() {
                    "sqrt" => pse_quantity::infer::OpRequest::Sqrt,
                    "abs" => pse_quantity::infer::OpRequest::Abs,
                    "exp" => {
                        pse_quantity::infer::OpRequest::Transcendental(pse_quantity::Opcode::Exp)
                    }
                    "log" => {
                        pse_quantity::infer::OpRequest::Transcendental(pse_quantity::Opcode::Log)
                    }
                    "sin" => {
                        pse_quantity::infer::OpRequest::Transcendental(pse_quantity::Opcode::Sin)
                    }
                    "cos" => {
                        pse_quantity::infer::OpRequest::Transcendental(pse_quantity::Opcode::Cos)
                    }
                    _ => {
                        return Err(MathError::Contract(
                            "primitive operation has no checked physical request".into(),
                        ));
                    }
                };
                let admission = self.arithmetic_admission(expr, request, &[&value], builder)?;
                builder.unary_admitted(*function, value, admission, source)
            }
            ExprKind::Conditional {
                guard,
                then,
                otherwise,
            } => self.conditional(guard, then, otherwise, builder, depth + 1, source),
            ExprKind::Let { bindings, body } => {
                self.hash.str("let").u64(bindings.len() as u64);
                let saved = self.locals.clone();
                let mut names = std::collections::BTreeSet::new();
                for (name, expr) in bindings {
                    if !names.insert(name) {
                        return Err(MathError::Contract("duplicate local binding".into()));
                    }
                    self.hash.str(name);
                    let value = self.expression_expected(
                        expr,
                        builder,
                        depth + 1,
                        self.local_quantities.get(name).copied(),
                    )?;
                    self.locals.insert(name.clone(), builder.bind(value)?);
                }
                let result = self.expression_expected(body, builder, depth + 1, expected);
                self.locals = saved;
                result
            }
            ExprKind::Reduce { kind, binder, body } => {
                self.reduce(expr, *kind, binder, body, builder, depth + 1, source)
            }
            ExprKind::NamedCall { name, args } => self.function(
                &dsl::render_path(name),
                args,
                &[],
                builder,
                depth + 1,
                source,
            ),
            ExprKind::Partial {
                function,
                wrt,
                args,
            } => self.function(
                &dsl::render_path(function),
                args,
                wrt,
                builder,
                depth + 1,
                source,
            ),
            ExprKind::Kernel { name, args } => {
                let call = self.request.providers.get(name).ok_or_else(|| {
                    MathError::Contract(
                        "provider call requires an executable physical registration".into(),
                    )
                })?;
                let registration = call.descriptor.clone();
                let output = call.output;
                self.hash
                    .str("provider")
                    .hash(&registration.spec().identity())
                    .u64(output as u64);
                let inputs = args
                    .iter()
                    .map(|arg| self.expression(arg, builder, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?;
                builder
                    .provider(&registration, &inputs, source)?
                    .get(output)
                    .cloned()
                    .ok_or_else(|| MathError::Contract("provider output ordinal".into()))
            }
            ExprKind::Derivative { .. } => Err(MathError::Contract(
                "dynamic derivative is outside the algebraic value profile".into(),
            )),
            ExprKind::Fold { .. } => Err(MathError::Contract(
                "finite fold must be specialized before mathematical admission".into(),
            )),
        }
    }
    fn function(
        &mut self,
        name: &str,
        args: &[Expr],
        wrt: &[dsl::Path],
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        use pse_quantity::scheme::Substitution;
        let f =
            self.functions.get(name).cloned().ok_or_else(|| {
                MathError::Contract(format!("unresolved package function {name}"))
            })?;
        if self.calls.contains(&f.id) {
            return Err(MathError::Contract("recursive package function".into()));
        }
        if args.len() != f.arguments.len() {
            return Err(MathError::Contract(
                "package function argument count".into(),
            ));
        }
        self.hash
            .str("numerical-prerequisites")
            .u64(f.prerequisites.len() as u64);
        for index in &f.prerequisites {
            if *index >= args.len() {
                return Err(MathError::Contract(
                    "numerical prerequisite argument index".into(),
                ));
            }
            self.hash.u64(*index as u64);
        }
        self.verify_scientific_response(&f, name, args, builder, depth, source)?;
        let mut substitutions = Substitution::new();
        let mut arguments = Vec::new();
        for (expr, (_, ty)) in args.iter().zip(&f.arguments) {
            let Some(scheme) = ty.quantity_scheme() else {
                return Err(MathError::Contract(
                    "runtime function argument requires a scalar physical type".into(),
                ));
            };
            let expected = if scheme.is_bound(&substitutions) {
                Some(
                    scheme
                        .resolve_contract_with_evidence(self.registry, &substitutions, self.checker)
                        .map_err(|error| MathError::Contract(error.to_string()))?,
                )
            } else {
                None
            };
            let value = self.expression_expected(
                expr,
                builder,
                depth,
                expected.as_ref().and_then(|contract| contract.named_id()),
            )?;
            let value = if let Some(expected) = &expected {
                builder.contract_boundary(value, expected)?
            } else {
                value
            };
            scheme
                .bind_contract_with_evidence(
                    value.physical_contract(),
                    self.registry,
                    &mut substitutions,
                    self.checker,
                )
                .map_err(|error| MathError::Contract(error.to_string()))?;
            // A specialized finite reduction is an operation, not a source
            // function argument boundary. Keep its terms in the surrounding
            // symbolic graph unless an explicit partial needs independent slots.
            arguments.push(if f.reduction.is_some() && wrt.is_empty() {
                value
            } else if !wrt.is_empty() || f.continuity.is_some() {
                builder.independent(value)?
            } else {
                builder.bind(value)?
            });
        }
        if let Some(operation) = &f.physical_operation {
            operation.frame(&mut self.hash);
            let body_arguments = operation.body_arguments(&f.arguments);
            for ((value, (_, original)), (_, consumed)) in
                arguments.iter_mut().zip(&f.arguments).zip(body_arguments)
            {
                if original != &consumed {
                    let target = consumed
                        .quantity_scheme()
                        .ok_or_else(|| {
                            MathError::Contract(
                                "physical operation argument payload must be numerical".into(),
                            )
                        })?
                        .resolve_with_evidence(self.registry, &substitutions, self.checker)
                        .map_err(|error| MathError::Contract(error.to_string()))?;
                    let mut identity =
                        FramedHasher::new(pse_ids::Frame::ModelingPhysicalOperationV1);
                    identity.id(&f.id.as_id());
                    operation.frame(&mut identity);
                    let authorization = pse_quantity::AdmittedOutputBoundary::declared(
                        identity.finish_id(),
                        value.physical_contract().clone(),
                        target,
                        self.registry,
                    )?;
                    *value = builder.authorized_boundary(value.clone(), &authorization)?;
                }
            }
        }
        if let Some(value) = self.abstract_scientific_call(
            &f,
            name,
            args,
            &mut arguments,
            &substitutions,
            wrt,
            builder,
            depth,
            source,
        )? {
            return Ok(value);
        }
        let concrete_admissions = f
            .physical_admissions
            .iter()
            .map(|(occurrence, obligation)| {
                obligation
                    .specialize(
                        self.registry,
                        &substitutions,
                        self.checker,
                        builder.physical_formula_authority(),
                        f.id,
                    )
                    .map(|admission| (occurrence.clone(), admission))
                    .map_err(|error| MathError::Contract(error.to_string()))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let mut occurrences = f
            .body
            .as_ref()
            .map(pse_modeling::expression::admission::ExpressionOccurrence::in_body)
            .unwrap_or_default();
        for predicate in f
            .validity
            .iter()
            .chain(f.envelopes.iter().map(|guard| &guard.predicate))
        {
            predicate_occurrences(predicate, &mut occurrences);
        }
        for usage in &f.applicability_uses {
            for predicate in &usage.predicates {
                predicate_occurrences(predicate, &mut occurrences);
            }
            for input in &usage.inputs {
                occurrences.extend(
                    pse_modeling::expression::admission::ExpressionOccurrence::in_body(input),
                );
            }
        }
        let prior_operations = self.operation_scope.replace(OperationScope {
            obligations: f.physical_admissions.clone(),
            admissions: concrete_admissions,
            occurrences,
        });
        if let Some(reduction) = &f.reduction
            && wrt.is_empty()
        {
            self.hash.str("finite-reduction").id(&f.id.as_id());
            let value = {
                let admission = self.finite_reduction_admission(&f)?;
                builder.finite_reduce_admitted(
                    reduction.kind,
                    &reduction.prototype,
                    &arguments,
                    admission,
                    source,
                )
            }?;
            let Some(result) = f.result.quantity_scheme() else {
                return Err(MathError::Contract(
                    "finite reduction result must be physical".into(),
                ));
            };
            let expected = result
                .resolve_contract_with_evidence(self.registry, &substitutions, self.checker)
                .map_err(|e| MathError::Contract(e.to_string()))?;
            self.operation_scope = prior_operations;
            let value = builder.contract_boundary(value, &expected)?;
            let value = f.prerequisites.iter().fold(value, |value, index| {
                builder.with_prerequisite(value, &arguments[*index])
            });
            return Ok(value);
        }
        // ADR-0123 Outcome 4: the form layer's domain and each rejecting data-layer guard are
        // domain predicates, each attributed to its own source: the function, or the
        // declaration of the guarded envelope. An extrapolating guard is observed instead. A
        // rejection names its lineage: its layer, the form, the parameter sets bounding it
        // and the arguments it constrains (Plan 23 H5).
        use pse_model::generated::enums::ModelingValidityLayer as Layer;
        let lineage = |layer: Layer, source: SemanticId, reads: &pse_modeling::envelope::Reads| {
            Arc::new(pse_model::diagnostic::ValidityLineage {
                layer,
                source,
                form: Some(f.id.as_id()),
                sets: reads.sets.clone(),
                variables: reads.variables.clone(),
                members: Vec::new(),
            })
        };
        let domains = f
            .validity
            .iter()
            .map(|validity| {
                (
                    "function-validity",
                    lineage(Layer::Form, f.id.as_id(), &f.validity_reads),
                    validity,
                )
            })
            .chain(f.envelopes.iter().map(|guard| {
                (
                    "envelope-guard",
                    lineage(Layer::Data, guard.envelope.owner.as_id(), &guard.reads),
                    &guard.predicate,
                )
            }))
            .collect::<Vec<_>>();
        let mut assumptions = Vec::with_capacity(domains.len());
        for (label, lineage, validity) in domains {
            let saved = std::mem::replace(
                &mut self.locals,
                f.arguments
                    .iter()
                    .zip(&arguments)
                    .map(|((name, _), value)| (name.clone(), value.clone()))
                    .collect(),
            );
            self.calls.push(f.id);
            let domain_source = lineage.source;
            self.hash
                .str(label)
                .id(&domain_source)
                .id(&f.id.as_id())
                .u64(lineage.sets.len() as u64);
            for set in &lineage.sets {
                self.hash.id(set);
            }
            self.hash.u64(lineage.variables.len() as u64);
            for variable in &lineage.variables {
                self.hash.u64(u64::from(*variable));
            }
            self.hash.str(&dsl::render_predicate(validity));
            let (yes, no) = indicator_arms();
            let predicate = builder.domain(lineage, |builder| {
                self.conditional(validity, &yes, &no, builder, depth + 1, domain_source)
            });
            self.calls.pop();
            self.locals = saved;
            assumptions.push(predicate?);
        }
        for usage in &f.applicability_uses {
            usage.frame(&mut self.hash);
            let saved = std::mem::replace(
                &mut self.locals,
                f.arguments
                    .iter()
                    .zip(&arguments)
                    .map(|((name, _), value)| (name.clone(), value.clone()))
                    .collect(),
            );
            self.calls.push(f.id);
            let source = usage.node.claim.form;
            let plan = Arc::new(usage.node.clone());
            let (yes, no) = indicator_arms();
            let assumption = builder.applicability(plan, |builder| {
                let predicates = usage
                    .predicates
                    .iter()
                    .map(|p| self.conditional(p, &yes, &no, builder, depth + 1, source))
                    .collect::<Result<Vec<_>, _>>()?;
                let inputs = usage
                    .inputs
                    .iter()
                    .map(|e| self.expression(e, builder, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((predicates, inputs))
            });
            self.calls.pop();
            self.locals = saved;
            assumptions.push(assumption?);
        }
        if let Some(external) = &f.external {
            let call = self
                .request
                .providers
                .get(&external.implementation)
                .ok_or_else(|| {
                    MathError::Contract(format!(
                        "unregistered external implementation {}",
                        external.implementation
                    ))
                })?;
            let spec = call.descriptor.spec();
            if spec.revision != external.revision
                || spec.data != external.data
                || spec.derivative_source != external.derivative_source
                || spec.derivatives as u8 != external.derivatives
                || spec.smoothness as u8 != external.smoothness
            {
                return Err(MathError::Contract("external implementation differs from its authored revision, data or derivative contract".into()));
            }
            if external.shapes.len() != spec.shapes.inputs.len() {
                return Err(MathError::Contract(
                    "external logical input shape count".into(),
                ));
            }
            for shape in &external.shapes {
                let logical = spec
                    .shapes
                    .inputs
                    .iter()
                    .find(|s| s.id == shape.argument)
                    .ok_or_else(|| {
                        MathError::Contract("external logical argument identity".into())
                    })?;
                let end = shape
                    .start
                    .checked_add(shape.coordinates.len())
                    .ok_or(MathError::Limit("external logical extent"))?;
                let ports = spec
                    .inputs
                    .get(shape.start..end)
                    .ok_or_else(|| MathError::Contract("external logical scalar range".into()))?;
                if logical.axes != shape.axes
                    || logical.coordinates != shape.coordinates
                    || logical.cells != ports.iter().map(|p| p.id).collect::<Vec<_>>()
                {
                    return Err(MathError::Contract(
                        "external coordinate order differs from registration".into(),
                    ));
                }
            }
            let ExprKind::Number(output) = &external.output.kind else {
                return Err(MathError::Contract(
                    "external selected output is not static".into(),
                ));
            };
            let output = output
                .integer()
                .and_then(|v| usize::try_from(v).ok())
                .ok_or_else(|| MathError::Contract("external output ordinal".into()))?;
            let partial = wrt
                .iter()
                .map(|path| {
                    if path.segments.len() != 1 || !path.segments[0].indices.is_empty() {
                        return Err(MathError::Contract(
                            "external partial must name a specialized scalar argument".into(),
                        ));
                    }
                    f.arguments
                        .iter()
                        .position(|(name, _)| *name == path.segments[0].name)
                        .ok_or_else(|| {
                            MathError::Contract("external partial argument absent".into())
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let outputs =
                builder.provider_partial(&call.descriptor, &arguments, &partial, source)?;
            let value = outputs.get(output).cloned().ok_or_else(|| {
                MathError::Contract("external output ordinal outside registration".into())
            })?;
            let Some(result) = f.result.quantity_scheme() else {
                return Err(MathError::Contract(
                    "external selected cell must have a scalar physical type".into(),
                ));
            };
            let mut result = result.clone();
            for i in &partial {
                result = pse_quantity::scheme::Scheme::Quotient(
                    Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(result))),
                    Box::new(pse_quantity::scheme::Scheme::Delta(Box::new(
                        pse_quantity::scheme::Scheme::Concrete(arguments[*i].quantity()?),
                    ))),
                );
            }
            let expected = result
                .resolve_with_evidence(self.registry, &substitutions, self.checker)
                .map_err(|e| MathError::Contract(e.to_string()))?;
            let value = builder.named_boundary(value, expected)?;
            self.hash
                .str("external-function")
                .hash(&spec.identity())
                .u64(output as u64)
                .u64(partial.len() as u64);
            for i in partial {
                self.hash.u64(i as u64);
            }
            let value = assumptions.iter().fold(value, |value, assumption| {
                builder.with_assumption(value, assumption)
            });
            let value = f.prerequisites.iter().fold(value, |value, index| {
                builder.with_prerequisite(value, &arguments[*index])
            });
            self.operation_scope = prior_operations;
            return builder.bind(value);
        }
        let scope = builder.function_scope();
        let saved = std::mem::replace(
            &mut self.locals,
            f.arguments
                .iter()
                .zip(&arguments)
                .map(|((name, _), v)| (name.clone(), v.clone()))
                .collect(),
        );
        self.calls.push(f.id);
        self.hash
            .str("package-function")
            .id(&f.id.as_id())
            .u64(wrt.len() as u64);
        let Some(result) = f.result.quantity_scheme() else {
            return Err(MathError::Contract(
                "runtime function result requires a scalar physical type".into(),
            ));
        };
        let expected = result
            .resolve_contract_with_evidence(self.registry, &substitutions, self.checker)
            .map_err(|error| MathError::Contract(error.to_string()))?;
        let prior_formula = builder.physical_formula_scope(match &f.physical_operation {
            Some(pse_modeling::PhysicalOperation::Response {
                potential: Some(potential),
                ..
            }) => {
                let mut identity = FramedHasher::new(pse_ids::Frame::ModelingPhysicalOperationV1);
                identity.id(&f.id.as_id()).id(&potential.as_id());
                Some(pse_quantity::PhysicalFormulaAuthority::response(
                    identity.finish_id(),
                ))
            }
            Some(pse_modeling::PhysicalOperation::Response {
                potential: None, ..
            }) => {
                return Err(MathError::Contract(
                    "response formula requires its actual reconstructed potential witness".into(),
                ));
            }
            _ => None,
        });
        let value = if let Some(reduction) = &f.reduction {
            self.hash
                .str("finite-reduction")
                .str(reduction.kind.as_str())
                .bool(reduction.domain.is_some());
            reduction.prototype.frame(&mut self.hash);
            if let Some(domain) = reduction.domain {
                self.hash.id(&domain.as_id());
            }
            {
                let admission = self.finite_reduction_admission(&f)?;
                builder.finite_reduce_admitted(
                    reduction.kind,
                    &reduction.prototype,
                    &arguments,
                    admission,
                    source,
                )
            }
        } else {
            let body = f
                .body
                .as_ref()
                .ok_or_else(|| MathError::Contract("function has no selected body".into()))?;
            self.expression_expected(
                body,
                builder,
                depth,
                if f.physical_operation.is_some() {
                    None
                } else {
                    expected.named_id()
                },
            )
        };
        self.operation_scope = prior_operations;
        builder.physical_formula_scope(prior_formula);
        self.locals = saved;
        self.calls.pop();
        let mut value = value?;
        if let Some(order) = f.continuity {
            let order = match order {
                0 => DerivativeOrder::Value,
                1 => DerivativeOrder::First,
                2 => DerivativeOrder::Second,
                _ => return Err(MathError::Contract("piecewise derivative order".into())),
            };
            self.hash.str("verified-piecewise").u64(order as u64);
            builder.verify_piecewise(scope, &arguments, order)?;
        }
        if let Some(operation) = &f.physical_operation {
            let authorization = operation
                .admit_numeric_result(
                    value.physical_contract(),
                    &f.result,
                    self.registry,
                    self.checker,
                    f.id,
                )
                .map_err(|error| MathError::Contract(error.to_string()))?;
            value = builder.authorized_boundary(value, &authorization)?;
        } else {
            value = builder.contract_boundary(value, &expected)?;
        }
        if !wrt.is_empty() {
            let variables = wrt
                .iter()
                .map(|path| {
                    if path.segments.len() != 1 || !path.segments[0].indices.is_empty() {
                        return Err(MathError::Contract(
                            "partial must select an explicit scalar argument".into(),
                        ));
                    }
                    let index = f
                        .arguments
                        .iter()
                        .position(|(n, _)| n == &path.segments[0].name)
                        .ok_or_else(|| MathError::Contract("partial argument absent".into()))?;
                    self.hash.u64(index as u64);
                    Ok(arguments[index].clone())
                })
                .collect::<Result<Vec<_>, MathError>>()?;
            let admission = pse_modeling::expression::admission::admit_partial(
                value.physical_contract(),
                &variables
                    .iter()
                    .map(|value| value.physical_contract().clone())
                    .collect::<Vec<_>>(),
                self.registry,
                self.checker,
                f.id,
            )
            .map_err(|error| MathError::Contract(error.to_string()))?;
            value = builder.partial_admitted(scope, value, &variables, admission, source)?;
        }
        let value = assumptions.iter().fold(value, |value, assumption| {
            builder.with_assumption(value, assumption)
        });
        let value = f.prerequisites.iter().fold(value, |value, index| {
            builder.with_prerequisite(value, &arguments[*index])
        });
        builder.bind(value)
    }
    fn group(
        &mut self,
        name: &str,
        indices: &[&Expr],
        builder: &mut BodyBuilder<'_>,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let group = self
            .request
            .groups
            .get(name)
            .ok_or_else(|| MathError::Contract(format!("missing indexed group {name}")))?;
        if indices.len() != group.axes.len() {
            return Err(MathError::Contract("indexed group axis arity".into()));
        }
        let ty = self.registry.quantity_type(group.quantity)?;
        let mut tuple = vec![];
        let mut coordinates = vec![];
        let mut shape = vec![];
        for (index, axis) in indices.iter().zip(&group.axes) {
            let ExprKind::Path(path) = &index.kind else {
                return Err(MathError::Contract(
                    "index needs a bound semantic member".into(),
                ));
            };
            let name = plain_path(path)?;
            let coordinate = self
                .coordinates
                .get(&name)
                .ok_or_else(|| MathError::Contract("unbound index".into()))?;
            let domain = self
                .request
                .domains
                .get(axis)
                .ok_or_else(|| MathError::Contract("group axis domain is absent".into()))?;
            if coordinate.bound.domain.as_id() != domain.members.id {
                return Err(MathError::Contract(
                    "index domain differs from declared axis".into(),
                ));
            }
            coordinates.push(coordinate.bound);
            shape.push(domain.kind);
            if let Some(member) = coordinate.member {
                tuple.push(member);
            }
        }
        if ty.key.shape != shape {
            return Err(MathError::Contract(
                "group physical shape differs from declared axes".into(),
            ));
        }
        let indices = IndexSet::try_from_iter(coordinates)
            .map_err(|_| MathError::Contract("duplicate incompatible binders".into()))?;
        if indices.len() != group.axes.len() {
            return Err(MathError::Contract(
                "one binder cannot stand for multiple group axes".into(),
            ));
        }
        let slot = if self.physical_only {
            0
        } else {
            *group
                .slots
                .get(&tuple)
                .ok_or_else(|| MathError::Contract("missing ragged group member".into()))?
        };
        if !self.physical_only {
            let formal =
                self.request.formals.get(slot).ok_or_else(|| {
                    MathError::Contract("group slot is outside formal layout".into())
                })?;
            let mut scalar = ty.key.clone();
            scalar.shape.clear();
            let scalar_type = self.registry.resolve_key(&scalar)?;
            pse_quantity::admission::require_same_contract(
                scalar_type,
                formal.quantity,
                self.registry,
            )?;
            self.hash.u64(slot as u64);
        }
        builder.input(slot, group.quantity, indices, source)
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the reduction's expression, kind, binder and body accompany the shared body builder, recursion depth and source identity"
    )]
    fn reduce(
        &mut self,
        expression: &Expr,
        kind: dsl::ReduceKind,
        binder: &dsl::Binder,
        body: &Expr,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let kind = match kind {
            dsl::ReduceKind::Sum => pse_quantity::ReductionKind::Sum,
            dsl::ReduceKind::Prod => pse_quantity::ReductionKind::Prod,
            dsl::ReduceKind::Integral => {
                return Err(MathError::Contract(
                    "continuous integral is outside the finite algebraic profile".into(),
                ));
            }
        };
        let domain_name = plain_path(&binder.domain)?;
        let domain = self
            .request
            .domains
            .get(&domain_name)
            .ok_or_else(|| MathError::Contract("reduction domain is absent".into()))?;
        let bound = pse_quantity::BoundIndexRef::new(
            pse_quantity::BoundIndexId::from_id(source),
            pse_quantity::DomainId::from_id(domain.members.id),
            domain.kind,
        );
        self.hash.str("reduce").str(kind.as_str()).str(&domain_name);
        let prior = self.coordinates.insert(
            binder.var.clone(),
            Coordinate {
                bound,
                member: None,
            },
        );
        // A separate physical pass checks body declarations even for an empty or fully
        // filtered domain. It neither evaluates nor normalizes that body's arithmetic.
        let mut prototype_lower = Lower {
            request: self.request,
            registry: self.registry,
            checker: self.checker,
            paths: self.paths.clone(),
            locals: self.locals.clone(),
            occurrences: vec![],
            hash: FramedHasher::new(pse_ids::Frame::MathPhysicalPassV1),
            cancelled: self.cancelled,
            coordinates: self.coordinates.clone(),
            physical_only: true,
            functions: self.functions,
            local_quantities: self.local_quantities,
            calls: self.calls.clone(),
            validity: self.validity,
            validating: self.validating.clone(),
            operation_scope: self.operation_scope.clone(),
            witness_pass: self.witness_pass.clone(),
        };
        let mut physical = builder.physical_pass()?;
        let prototype = prototype_lower.expression(body, &mut physical, depth)?;
        self.hash.hash(&prototype_lower.hash.finish_hash());
        if let Some(filter) = binder.filter.as_deref() {
            self.filter_definition(filter, 0)?;
        }
        let mut terms = vec![];
        if !self.physical_only {
            for &member in domain.members.members() {
                if self.cancelled.load(Ordering::Relaxed) {
                    return Err(MathError::Cancelled);
                }
                self.coordinates.insert(
                    binder.var.clone(),
                    Coordinate {
                        bound,
                        member: Some(member),
                    },
                );
                let selected = binder
                    .filter
                    .as_deref()
                    .map_or(Ok(true), |filter| self.filter(filter));
                let selected = selected?;
                self.hash.bool(selected);
                if selected {
                    terms.push(self.expression(body, builder, depth)?);
                }
            }
        }
        match prior {
            Some(value) => {
                self.coordinates.insert(binder.var.clone(), value);
            }
            None => {
                self.coordinates.remove(&binder.var);
            }
        }
        let admission = self.arithmetic_admission(
            expression,
            pse_quantity::infer::OpRequest::Reduce { kind, bound },
            &[&prototype],
            builder,
        )?;
        builder.reduce_admitted(kind, &prototype, &terms, admission, source)
    }
    fn filter(&self, predicate: &dsl::Predicate) -> Result<bool, MathError> {
        match &predicate.kind {
            PredicateKind::Bool(value) => Ok(*value),
            PredicateKind::And(a, b) => Ok(self.filter(a)? && self.filter(b)?),
            PredicateKind::Or(a, b) => Ok(self.filter(a)? || self.filter(b)?),
            PredicateKind::Not(value) => Ok(!self.filter(value)?),
            PredicateKind::In { expr, domain } => {
                let ExprKind::Path(path) = &expr.kind else {
                    return Err(MathError::Contract(
                        "membership filter requires a lexical member".into(),
                    ));
                };
                let member = self
                    .coordinates
                    .get(&plain_path(path)?)
                    .and_then(|c| c.member)
                    .ok_or_else(|| MathError::Contract("filter member is unbound".into()))?;
                let domain = self
                    .request
                    .domains
                    .get(&plain_path(domain)?)
                    .ok_or_else(|| MathError::Contract("filter domain is absent".into()))?;
                Ok(domain.members.contains(member))
            }
            PredicateKind::Compare { op, lhs, rhs }
                if matches!(op, CompareOp::Eq | CompareOp::NotEq) =>
            {
                let member = |expr: &Expr| -> Result<SemanticId, MathError> {
                    let ExprKind::Path(path) = &expr.kind else {
                        return Err(MathError::Contract(
                            "finite comparison requires lexical members".into(),
                        ));
                    };
                    self.coordinates
                        .get(&plain_path(path)?)
                        .and_then(|c| c.member)
                        .ok_or_else(|| MathError::Contract("unbound filter member".into()))
                };
                let equal = member(lhs)? == member(rhs)?;
                Ok(if *op == CompareOp::Eq { equal } else { !equal })
            }
            _ => Err(MathError::Contract(
                "finite filter requires declared membership predicates".into(),
            )),
        }
    }
    fn filter_definition(
        &mut self,
        predicate: &dsl::Predicate,
        depth: usize,
    ) -> Result<(), MathError> {
        if depth > 128 {
            return Err(MathError::Limit("finite filter depth"));
        }
        match &predicate.kind {
            PredicateKind::Bool(value) => {
                self.hash.str("bool").bool(*value);
            }
            PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
                self.hash
                    .str(if matches!(predicate.kind, PredicateKind::And(..)) {
                        "and"
                    } else {
                        "or"
                    });
                self.filter_definition(a, depth + 1)?;
                self.filter_definition(b, depth + 1)?;
            }
            PredicateKind::Not(value) => {
                self.hash.str("not");
                self.filter_definition(value, depth + 1)?;
            }
            PredicateKind::In { expr, domain } => {
                let ExprKind::Path(path) = &expr.kind else {
                    return Err(MathError::Contract(
                        "membership filter requires a lexical member".into(),
                    ));
                };
                let name = plain_path(path)?;
                let domain_name = plain_path(domain)?;
                let coordinate = self
                    .coordinates
                    .get(&name)
                    .ok_or_else(|| MathError::Contract("filter member is unbound".into()))?;
                let domain = self
                    .request
                    .domains
                    .get(&domain_name)
                    .ok_or_else(|| MathError::Contract("filter domain is absent".into()))?;
                if coordinate.bound.kind != domain.kind {
                    return Err(MathError::Contract(
                        "filter domain kind differs from lexical member".into(),
                    ));
                }
                self.hash.str("in").str(&name).str(&domain_name);
            }
            PredicateKind::Compare { op, lhs, rhs }
                if matches!(op, CompareOp::Eq | CompareOp::NotEq) =>
            {
                let coordinate =
                    |expr: &Expr| -> Result<(String, pse_quantity::EntityKindId), MathError> {
                        let ExprKind::Path(path) = &expr.kind else {
                            return Err(MathError::Contract(
                                "finite comparison requires lexical members".into(),
                            ));
                        };
                        let name = plain_path(path)?;
                        let c = self.coordinates.get(&name).ok_or_else(|| {
                            MathError::Contract("unbound filter declaration".into())
                        })?;
                        Ok((name, c.bound.kind))
                    };
                let (a, ak) = coordinate(lhs)?;
                let (b, bk) = coordinate(rhs)?;
                if ak != bk {
                    return Err(MathError::Contract("filter member kinds differ".into()));
                }
                self.hash
                    .str("member_compare")
                    .str(op.as_str())
                    .str(&a)
                    .str(&b);
            }
            _ => {
                return Err(MathError::Contract(
                    "finite filter requires declared membership predicates".into(),
                ));
            }
        }
        Ok(())
    }
    fn conditional(
        &mut self,
        predicate: &dsl::Predicate,
        then: &Expr,
        otherwise: &Expr,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        self.conditional_branches(
            predicate,
            &ConditionalBranch::Expression(then),
            &ConditionalBranch::Expression(otherwise),
            builder,
            depth,
            source,
        )
    }
    fn conditional_branch(
        &mut self,
        branch: &ConditionalBranch<'_>,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        match branch {
            ConditionalBranch::Expression(expression) => {
                self.expression(expression, builder, depth)
            }
            ConditionalBranch::Predicate {
                predicate,
                then,
                otherwise,
            } => self.conditional_branches(predicate, then, otherwise, builder, depth, source),
        }
    }
    fn conditional_branches(
        &mut self,
        predicate: &dsl::Predicate,
        then: &ConditionalBranch<'_>,
        otherwise: &ConditionalBranch<'_>,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        if depth > 128 {
            return Err(MathError::Limit("conditional depth"));
        }
        self.hash.str("conditional");
        match &predicate.kind {
            PredicateKind::Not(inner) => {
                self.hash.str("not");
                self.conditional_branches(inner, otherwise, then, builder, depth + 1, source)
            }
            PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
                let conjunction = matches!(predicate.kind, PredicateKind::And(..));
                self.hash.str(if conjunction { "and" } else { "or" });
                let nested = ConditionalBranch::Predicate {
                    predicate: b,
                    then,
                    otherwise,
                };
                if conjunction {
                    self.conditional_branches(a, &nested, otherwise, builder, depth + 1, source)
                } else {
                    self.conditional_branches(a, then, &nested, builder, depth + 1, source)
                }
            }
            _ => {
                let guard = if let PredicateKind::Bool(value) = predicate.kind {
                    self.hash.str("bool").u64(u64::from(value));
                    builder.fixed_guard(value, source)?
                } else {
                    self.guard(predicate, builder, depth + 1, source)?
                };
                let cell = std::cell::RefCell::new(self);
                builder.conditional(
                    guard,
                    |b| {
                        cell.borrow_mut()
                            .conditional_branch(then, b, depth + 1, source)
                    },
                    |b| {
                        cell.borrow_mut()
                            .conditional_branch(otherwise, b, depth + 1, source)
                    },
                    source,
                )
            }
        }
    }
    fn guard(
        &mut self,
        predicate: &dsl::Predicate,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
        source: SemanticId,
    ) -> Result<Guard, MathError> {
        let PredicateKind::Compare { op, lhs, rhs } = &predicate.kind else {
            return Err(MathError::Contract(
                "guard requires an admitted scalar comparison".into(),
            ));
        };
        self.hash.str(op.as_str());
        let left = self.expression(lhs, builder, depth + 1)?;
        let right = self.expression_expected(rhs, builder, depth + 1, Some(left.quantity()?))?;
        match op {
            CompareOp::Eq => builder.compare(Comparison::Eq, &left, &right, source),
            CompareOp::NotEq => builder.compare(Comparison::Ne, &left, &right, source),
            CompareOp::Lt => builder.compare(Comparison::Lt, &left, &right, source),
            CompareOp::Le => builder.compare(Comparison::Le, &left, &right, source),
            CompareOp::Gt => builder.compare(Comparison::Lt, &right, &left, source),
            CompareOp::Ge => builder.compare(Comparison::Le, &right, &left, source),
        }
    }
}
/// A product, a quotient or an exact literal power belongs to a multiplicative chain.
fn predicate_occurrences(
    predicate: &dsl::Predicate,
    occurrences: &mut BTreeMap<usize, pse_modeling::expression::admission::ExpressionOccurrence>,
) {
    use pse_modeling::expression::admission::ExpressionOccurrence;
    match &predicate.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            occurrences.extend(ExpressionOccurrence::in_body(lhs));
            occurrences.extend(ExpressionOccurrence::in_body(rhs));
        }
        PredicateKind::Atom(expression)
        | PredicateKind::In {
            expr: expression, ..
        } => {
            occurrences.extend(ExpressionOccurrence::in_body(expression));
        }
        PredicateKind::And(left, right) | PredicateKind::Or(left, right) => {
            predicate_occurrences(left, occurrences);
            predicate_occurrences(right, occurrences);
        }
        PredicateKind::Not(inner) => predicate_occurrences(inner, occurrences),
        _ => {}
    }
}
fn literal_exponent(expression: &Expr) -> Option<Ratio> {
    match &expression.kind {
        ExprKind::Number(number)
            if number.unit.is_none()
                && number.value.fract() == 0.0
                && number.value >= f64::from(i16::MIN)
                && number.value <= f64::from(i16::MAX) =>
        {
            Ratio::new(number.value as i32, 1).ok()
        }
        ExprKind::Neg(inner) => {
            let positive = literal_exponent(inner)?;
            Ratio::new(-i32::from(positive.num()), i32::from(positive.den())).ok()
        }
        ExprKind::Binary {
            op: BinaryOp::Div,
            lhs,
            rhs,
        } => {
            let numerator = literal_exponent(lhs)?;
            let denominator = literal_exponent(rhs)?;
            // Only explicit signed ratios of integers are exact authored facts.
            if numerator.den() != 1 || denominator.den() != 1 {
                return None;
            }
            Ratio::new(i32::from(numerator.num()), i32::from(denominator.num())).ok()
        }
        _ => None,
    }
}
fn plain_path(path: &dsl::Path) -> Result<String, MathError> {
    if path.segments.is_empty() || path.segments.iter().any(|s| !s.indices.is_empty()) {
        return Err(MathError::Contract(
            "expected an unindexed semantic name".into(),
        ));
    }
    Ok(path
        .segments
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_quantity::standard::{StandardInvariantChecker, ids, standard_registry};
    fn compile(text: &str, formals: &[Formal], order: DerivativeOrder) -> Result<Body, MathError> {
        compile_literals(text, formals, order, &BTreeMap::new())
    }
    fn compile_literals(
        text: &str,
        formals: &[Formal],
        order: DerivativeOrder,
        literals: &BTreeMap<(u32, u32), QuantityTypeId>,
    ) -> Result<Body, MathError> {
        let expression = dsl::parse_expr(text).unwrap();
        let registry = standard_registry().unwrap();
        let h = ContentHash::from_bytes([1; 32]);
        Request {
            definition: SemanticId::from_bytes([1; 16]),
            expressions: std::slice::from_ref(&expression),
            formals,
            domains: &BTreeMap::new(),
            providers: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            literals,
            physical: h,
            structure: h,

            limits: BodyLimits::default(),
        }
        .compile(
            &registry,
            &StandardInvariantChecker,
            order,
            Optimization::default(),
            &Arc::new(AtomicBool::new(false)),
        )
    }
    fn checked_physical_product(registry: &QuantityRegistry) -> pse_modeling::Function {
        use pse_modeling::expression::admission::{ExpressionOccurrence, PhysicalAdmission};
        let named = |name| match registry.physical_name(name).unwrap() {
            pse_quantity::PhysicalName::QuantityType(id) => id,
            other => panic!("{other:?}"),
        };
        let length = named("Length");
        let body = dsl::parse_expr("a*b").unwrap();
        let contract =
            pse_quantity::ResolvedPhysicalContract::named(length, IndexSet::new(), registry)
                .unwrap();
        let admission = pse_modeling::expression::admission::admit_operation(
            &pse_quantity::infer::OpRequest::Mul,
            &[contract.clone(), contract],
            registry,
            &StandardInvariantChecker,
            None,
            DeclarationId::from(SemanticId::from_bytes([81; 16])),
        )
        .unwrap();
        assert_eq!(admission.result.named_id(), None);
        let product_result = admission.result.clone();
        let occurrence = ExpressionOccurrence::in_body(&body)
            .remove(&(std::ptr::from_ref(&body) as usize))
            .unwrap();
        pse_modeling::Function {
            applicability: Vec::new(),
            applicability_uses: Vec::new(),
            prerequisites: Vec::new(),
            physical_admissions: BTreeMap::from([(
                occurrence,
                PhysicalAdmission::checked(
                    &pse_quantity::infer::OpRequest::Mul,
                    vec![pse_quantity::scheme::Scheme::Concrete(length); 2],
                    Some(admission),
                    DeclarationId::from(SemanticId::from_bytes([81; 16])),
                )
                .unwrap(),
            )]),
            physical_operation: None,
            reduction: None,
            validity: None,
            envelopes: Vec::new(),
            validity_reads: pse_modeling::envelope::Reads::default(),
            external: None,
            continuity: None,
            id: DeclarationId::from(SemanticId::from_bytes([81; 16])),
            variables: Default::default(),
            arguments: ["a", "b"]
                .map(|name| {
                    (
                        name.to_owned(),
                        pse_modeling::Type::Quantity(pse_quantity::scheme::Scheme::Concrete(
                            length,
                        )),
                    )
                })
                .to_vec(),
            result: pse_modeling::Type::Quantity(pse_quantity::scheme::Scheme::from_contract(
                product_result,
            )),
            body: Some(body),
        }
    }
    #[test]
    fn checked_occurrence_handoff_refuses_missing_admission_and_preserves_coincident_physical_partials()
     {
        pse_math::initialize().unwrap();
        let registry = standard_registry().unwrap();
        let function = checked_physical_product(&registry);
        let length = function.arguments[0]
            .1
            .quantity_scheme()
            .unwrap()
            .resolve(&registry, &BTreeMap::new())
            .unwrap();
        let expression = dsl::parse_expr("partial(physical_product,a)(x,x)").unwrap();
        let formals = [Formal {
            path: "x".into(),
            quantity: length,
        }];
        let h = ContentHash::from_bytes([1; 32]);
        let empty = BTreeMap::new();
        let cancelled = Arc::new(AtomicBool::new(false));
        let request = Request {
            definition: SemanticId::from_bytes([82; 16]),
            expressions: std::slice::from_ref(&expression),
            formals: &formals,
            domains: &empty,
            groups: &BTreeMap::new(),
            providers: &BTreeMap::new(),
            literals: &BTreeMap::new(),
            physical: h,
            structure: h,
            limits: BodyLimits::default(),
        };
        let mut functions = BTreeMap::from([("physical_product".into(), function)]);
        let admitted = request
            .admit_function_outputs(
                &registry,
                &StandardInvariantChecker,
                &cancelled,
                &functions,
                &[length],
                &BTreeMap::new(),
            )
            .unwrap();
        assert_eq!(admitted.quantities, vec![length]);
        let compiled = admitted
            .math
            .compile(
                &[0],
                &[0],
                DerivativeOrder::Second,
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancelled,
            )
            .unwrap();
        let values = compiled
            .worker()
            .evaluate(
                &[3.0],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &cancelled,
            )
            .unwrap();
        assert_eq!(values.values, vec![3.0]);
        assert_eq!(values.jacobian, vec![1.0]);
        functions
            .get_mut("physical_product")
            .unwrap()
            .physical_admissions
            .clear();
        let error = request
            .admit_function_outputs(
                &registry,
                &StandardInvariantChecker,
                &cancelled,
                &functions,
                &[length],
                &BTreeMap::new(),
            )
            .unwrap_err();
        assert!(
            error.to_string().contains("missing its admission"),
            "{error}"
        );
    }
    fn checked_scalar_function(registry: &QuantityRegistry, text: &str) -> pse_modeling::Function {
        use pse_modeling::expression::admission::AdmissionRecorder;
        let scalar = ids::quantity("neutral");
        let ty = pse_modeling::Type::Quantity(pse_quantity::scheme::Scheme::Concrete(scalar));
        let recorder = AdmissionRecorder::default();
        let preconditions = pse_quantity::PhysicalPreconditions::new(
            pse_quantity::generated::standard_preconditions(),
        )
        .unwrap();
        let scope = pse_modeling::PhysicalScope::default();
        let context = pse_modeling::TypeContext {
            admissions: Some(&recorder),
            formula_authority: None,
            quantities: registry,
            preconditions: &preconditions,
            scope: &scope,
        };
        let package = pse_modeling::check(&[], &context).unwrap();
        let mut function = checked_physical_product(registry);
        let body = dsl::parse_expr(text).unwrap();
        let arguments = BTreeMap::from([("x".into(), ty.clone())]);
        pse_modeling::expression::infer(
            &body,
            &arguments,
            &package,
            &context,
            function.id,
            Some(&ty),
        )
        .unwrap();
        function.physical_admissions = recorder.into_inner().remove(&function.id).unwrap();
        function.arguments = arguments.into_iter().collect();
        function.result = ty;
        function.body = Some(body);
        function
    }
    #[test]
    fn checked_compound_guards_preserve_original_arithmetic_occurrences() {
        pse_math::initialize().unwrap();
        let registry = standard_registry().unwrap();
        let scalar = ids::quantity("neutral");
        let formals = [Formal {
            path: "x".into(),
            quantity: scalar,
        }];
        let expression = dsl::parse_expr("guarded(x)").unwrap();
        let h = ContentHash::from_bytes([84; 32]);
        let cancelled = Arc::new(AtomicBool::new(false));
        let request = Request {
            definition: SemanticId::from_bytes([84; 16]),
            expressions: std::slice::from_ref(&expression),
            formals: &formals,
            domains: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            providers: &BTreeMap::new(),
            literals: &BTreeMap::new(),
            physical: h,
            structure: h,
            limits: BodyLimits::default(),
        };
        for (text, cases) in [
            (
                "if x != 0 and 1/x > 0 then x*x else x+x",
                [(0.0, 0.0), (-2.0, -4.0), (2.0, 4.0)],
            ),
            (
                "if x == 0 or 1/x > 0 then x+x else x*x",
                [(0.0, 0.0), (-2.0, 4.0), (2.0, 4.0)],
            ),
        ] {
            let functions =
                BTreeMap::from([("guarded".into(), checked_scalar_function(&registry, text))]);
            let admitted = request
                .admit_function_outputs(
                    &registry,
                    &StandardInvariantChecker,
                    &cancelled,
                    &functions,
                    &[scalar],
                    &BTreeMap::new(),
                )
                .unwrap();
            let compiled = admitted
                .math
                .compile(
                    &[0],
                    &[0],
                    DerivativeOrder::Value,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancelled,
                )
                .unwrap();
            let mut worker = compiled.worker();
            for (x, expected) in cases {
                assert_eq!(
                    worker
                        .evaluate(
                            &[x],
                            DerivativeOrder::Value,
                            &mut BTreeMap::new(),
                            &cancelled
                        )
                        .unwrap()
                        .values,
                    vec![expected]
                );
            }
        }
    }
    #[test]
    fn checked_finite_reduction_refuses_missing_retained_admission() {
        use pse_modeling::expression::admission::{ExpressionOccurrence, PhysicalAdmission};
        pse_math::initialize().unwrap();
        let registry = standard_registry().unwrap();
        let scalar = ids::quantity("neutral");
        let prototype =
            pse_quantity::ResolvedPhysicalContract::named(scalar, IndexSet::new(), &registry)
                .unwrap();
        let request = pse_quantity::infer::OpRequest::FiniteReduce {
            kind: pse_quantity::ReductionKind::Sum,
            domain: None,
        };
        let admission = pse_modeling::expression::admission::admit_operation(
            &request,
            std::slice::from_ref(&prototype),
            &registry,
            &StandardInvariantChecker,
            None,
            DeclarationId::from(SemanticId::from_bytes([85; 16])),
        )
        .unwrap();
        let mut function = checked_physical_product(&registry);
        function.body = None;
        function.reduction = Some(pse_modeling::FiniteReduction {
            kind: pse_quantity::ReductionKind::Sum,
            domain: None,
            prototype,
        });
        function.arguments = ["term_0", "term_1"]
            .map(|name| {
                (
                    name.into(),
                    pse_modeling::Type::Quantity(pse_quantity::scheme::Scheme::Concrete(scalar)),
                )
            })
            .to_vec();
        function.result =
            pse_modeling::Type::Quantity(pse_quantity::scheme::Scheme::Concrete(scalar));
        function.physical_admissions = BTreeMap::from([(
            ExpressionOccurrence::finite_reduction(function.id),
            PhysicalAdmission::checked(
                &request,
                vec![pse_quantity::scheme::Scheme::Concrete(scalar)],
                Some(admission),
                function.id,
            )
            .unwrap(),
        )]);
        let formals = [Formal {
            path: "x".into(),
            quantity: scalar,
        }];
        let h = ContentHash::from_bytes([85; 32]);
        let cancelled = Arc::new(AtomicBool::new(false));
        for (text, expected) in [
            ("finite_sum(x,x)", 6.0),
            ("partial(finite_sum,term_0)(x,x)", 1.0),
        ] {
            let expression = dsl::parse_expr(text).unwrap();
            let request = Request {
                definition: SemanticId::from_bytes([85; 16]),
                expressions: std::slice::from_ref(&expression),
                formals: &formals,
                domains: &BTreeMap::new(),
                groups: &BTreeMap::new(),
                providers: &BTreeMap::new(),
                literals: &BTreeMap::new(),
                physical: h,
                structure: h,
                limits: BodyLimits::default(),
            };
            let mut functions = BTreeMap::from([("finite_sum".into(), function.clone())]);
            let admitted = request
                .admit_function_outputs(
                    &registry,
                    &StandardInvariantChecker,
                    &cancelled,
                    &functions,
                    &[scalar],
                    &BTreeMap::new(),
                )
                .unwrap();
            let compiled = admitted
                .math
                .compile(
                    &[0],
                    &[0],
                    DerivativeOrder::Value,
                    Optimization::default(),
                    pse_math::jets::EvaluationLimits::default(),
                    &cancelled,
                )
                .unwrap();
            assert_eq!(
                compiled
                    .worker()
                    .evaluate(
                        &[3.0],
                        DerivativeOrder::Value,
                        &mut BTreeMap::new(),
                        &cancelled
                    )
                    .unwrap()
                    .values,
                vec![expected]
            );
            functions
                .get_mut("finite_sum")
                .unwrap()
                .physical_admissions
                .clear();
            let error = request
                .admit_function_outputs(
                    &registry,
                    &StandardInvariantChecker,
                    &cancelled,
                    &functions,
                    &[scalar],
                    &BTreeMap::new(),
                )
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("finite reduction occurrence is missing its admission"),
                "{error}"
            );
        }
    }
    #[test]
    fn direct_powers_preserve_integral_degree_bound() {
        pse_math::initialize().unwrap();
        let formals = [Formal {
            path: "x".into(),
            quantity: ids::quantity("neutral"),
        }];
        for text in ["x^1025", "x^(-1025)"] {
            let error = compile(text, &formals, DerivativeOrder::Value).unwrap_err();
            assert!(
                error.to_string().contains("integral power degree"),
                "{error}"
            );
        }
        assert!(compile("x^1024", &formals, DerivativeOrder::Value).is_ok());
    }
    /// A maximal product lowers as one chain typed by its canonical monomial (ADR-0124):
    /// c3·t³/3 is a declared enthalpy increment although t³ alone names no kind.
    #[test]
    fn multiplicative_chains_lower_by_monomial() {
        use pse_quantity::scheme::{Scheme, Substitution};
        let registry = standard_registry().unwrap();
        let quantity = |hex| QuantityTypeId::from_id(SemanticId::parse_hex(hex).unwrap());
        let temperature = quantity("c64b96975a4a59755f8711d3bf628bc9");
        let coefficient = Scheme::Quotient(
            Box::new(Scheme::Concrete(quantity(
                "cd653ba98fa94d16b5d66b363f21c3d6",
            ))),
            Box::new(Scheme::Power(
                Box::new(Scheme::Concrete(temperature)),
                Ratio::new(2, 1).unwrap(),
            )),
        )
        .resolve_with_evidence(&registry, &Substitution::new(), &StandardInvariantChecker)
        .unwrap();
        let formals = [
            Formal {
                path: "c".into(),
                quantity: coefficient,
            },
            Formal {
                path: "t".into(),
                quantity: temperature,
            },
        ];
        let body = compile("c*t^3/3", &formals, DerivativeOrder::First).unwrap();
        assert_eq!(
            body.prepared.quantities,
            [quantity("d5bb3d48b9804f2f8d5a6f0a7cadaee8")]
        );
        let jet = body
            .worker()
            .evaluate(
                &[3.0, 10.0],
                DerivativeOrder::First,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert!((jet.values[0] - 1000.0).abs() < 1e-9, "{}", jet.values[0]);
        let error = compile("t^3", &formals, DerivativeOrder::First)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("anonymous intermediate requires an admitted named result boundary"),
            "{error}"
        );
    }
    /// The entropy increment lowers through the declared reinterpretation: ln(T/T0) and
    /// (T − T0)/T are logarithmic temperature increments, and a molar heat capacity over
    /// one is a molar entropy difference (ADR-0124).
    #[test]
    fn entropy_increment_lowers_to_its_closed_form() {
        use pse_quantity::scheme::{Scheme, Substitution};
        let registry = standard_registry().unwrap();
        let quantity = |hex| QuantityTypeId::from_id(SemanticId::parse_hex(hex).unwrap());
        let temperature = quantity("c64b96975a4a59755f8711d3bf628bc9");
        let cp = quantity("cd653ba98fa94d16b5d66b363f21c3d6");
        let per_temperature = Scheme::Quotient(
            Box::new(Scheme::Concrete(cp)),
            Box::new(Scheme::Concrete(temperature)),
        )
        .resolve_with_evidence(&registry, &Substitution::new(), &StandardInvariantChecker)
        .unwrap();
        let formals = [
            ("T0", temperature),
            ("T", temperature),
            ("c1", cp),
            ("c2", per_temperature),
        ]
        .map(|(path, quantity)| Formal {
            path: path.into(),
            quantity,
        });
        let body = compile(
            "c1*log(T/T0) + c2*T*r where r = (T - T0)/T",
            &formals,
            DerivativeOrder::First,
        )
        .unwrap();
        assert_eq!(
            body.prepared.quantities,
            [quantity("a9c45d0c2b764f4e87d1b95d5bc15da6")]
        );
        let jet = body
            .worker()
            .evaluate(
                &[300.0, 400.0, 10.0, 0.5],
                DerivativeOrder::First,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        // 10·ln(4/3) + 0.5·100, and ∂/∂T = c1/T + c2.
        assert!(
            (jet.values[0] - 52.876_820_724_517_81).abs() < 1e-9,
            "{}",
            jet.values[0]
        );
        assert!(
            (jet.jacobian[1] - 0.525).abs() < 1e-12,
            "{}",
            jet.jacobian[1]
        );
        // A heat-capacity increment is not an entropy difference.
        let heat = compile("c2*(T - T0)", &formals, DerivativeOrder::First).unwrap();
        assert_eq!(heat.prepared.quantities, [cp]);
    }
    #[test]
    fn authored_power_literals_preserve_exact_facts_and_analytic_jets() {
        let formals = [Formal {
            path: "x".into(),
            quantity: ids::quantity("neutral"),
        }];
        let cancel = Arc::new(AtomicBool::new(false));
        for (text, x, value, first, second) in [
            ("x^2", 3.0, 9.0, 6.0, 2.0),
            ("x^-1", 2.0, 0.5, -0.25, 0.25),
            ("x^(1/2)", 4.0, 2.0, 0.25, -0.03125),
            ("x^(-1/2)", 4.0, 0.5, -0.0625, 0.0234375),
            ("x^(2/4)", 4.0, 2.0, 0.25, -0.03125),
        ] {
            let body = compile(text, &formals, DerivativeOrder::Second).unwrap();
            let jet = body
                .worker()
                .evaluate(&[x], DerivativeOrder::Second, &mut BTreeMap::new(), &cancel)
                .unwrap();
            for (actual, expected) in [
                (jet.values[0], value),
                (jet.jacobian[0], first),
                (jet.hessians[0], second),
            ] {
                assert!(
                    (actual - expected).abs() < 1e-12,
                    "{text}: {actual} != {expected}"
                );
            }
        }
        assert!(literal_exponent(&dsl::parse_expr("0.5").unwrap()).is_none());
        for text in ["x^-1", "x^(1/2)"] {
            let body = compile(text, &formals, DerivativeOrder::Second).unwrap();
            assert!(
                body.worker()
                    .evaluate(
                        &[0.0],
                        DerivativeOrder::Second,
                        &mut BTreeMap::new(),
                        &cancel
                    )
                    .is_err()
            );
        }
        assert!(compile("x^1025", &formals, DerivativeOrder::Value).is_err());
    }

    #[test]
    fn normalized_temperature_cubic_has_analytic_jets() {
        let formals = [Formal {
            path: "T".into(),
            quantity: ids::quantity("temperature.point"),
        }];
        let text = "(T/1000{K})^3";
        let start = text.find("1000").unwrap() as u32;
        let registry = standard_registry().unwrap();
        let scale = registry
            .quantity_types()
            .find(|q| q.key.kind == ids::kind("temperature_scale") && q.key.shape.is_empty())
            .unwrap()
            .id;
        let literals = BTreeMap::from([((start, start + 7), scale)]);
        let body = compile_literals(text, &formals, DerivativeOrder::Second, &literals).unwrap();
        let jet = body
            .worker()
            .evaluate(
                &[500.0],
                DerivativeOrder::Second,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        assert!((jet.values[0] - 0.125).abs() < 1e-12);
        assert!((jet.jacobian[0] - 0.00075).abs() < 1e-12);
        assert!((jet.hessians[0] - 0.000003).abs() < 1e-12);
    }

    #[test]
    fn source_compilation_preserves_obligations_erased_by_cas() {
        pse_math::initialize().unwrap();
        let formals = [Formal {
            path: "x".into(),
            quantity: ids::quantity("neutral"),
        }];
        for text in ["x/x", "exp(log(x))", "log(1/x)"] {
            let body = compile(text, &formals, DerivativeOrder::Second).unwrap();
            assert!(
                body.worker()
                    .evaluate(
                        &[0.0],
                        DerivativeOrder::Value,
                        &mut BTreeMap::new(),
                        &Arc::new(AtomicBool::new(false))
                    )
                    .is_err()
            );
            assert!(
                body.worker()
                    .evaluate(
                        &[2.0],
                        DerivativeOrder::Value,
                        &mut BTreeMap::new(),
                        &Arc::new(AtomicBool::new(false))
                    )
                    .is_ok()
            );
        }
    }
    #[test]
    fn typed_source_identity_excludes_whitespace_and_occurrence_spans() {
        pse_math::initialize().unwrap();
        let formals = [Formal {
            path: "x".into(),
            quantity: ids::quantity("neutral"),
        }];
        let a = compile("x + 1", &formals, DerivativeOrder::Value).unwrap();
        let b = compile("  x+1  ", &formals, DerivativeOrder::Value).unwrap();
        assert_eq!(a.spec.key(), b.spec.key());
        assert_ne!(a.occurrences, b.occurrences);
    }
    #[test]
    fn physically_invalid_point_addition_fails_before_execution() {
        pse_math::initialize().unwrap();
        let formals = [Formal {
            path: "t".into(),
            quantity: ids::quantity("temperature.point"),
        }];
        assert!(compile("t+t", &formals, DerivativeOrder::Value).is_err());
        let body = compile("t-t", &formals, DerivativeOrder::Value).unwrap();
        assert_eq!(body.quantities[0], ids::quantity("temperature.difference"));
    }
    fn indexed(
        text: &str,
        members: &[u8],
        present: &[u8],
        physical_name: &str,
    ) -> Result<Body, MathError> {
        let original = standard_registry().unwrap();
        let scalar = ids::quantity(physical_name);
        let mut shaped = original.quantity_type(scalar).unwrap().clone();
        shaped.id = QuantityTypeId::from_id(SemanticId::from_bytes([91; 16]));
        // A physical name is declared once; the shaped variant is unnamed.
        shaped.name = None;
        shaped.key.shape = vec![
            standard_registry()
                .unwrap()
                .entity_kind_named("species")
                .unwrap(),
        ];
        let shaped_id = original.resolve_key(&shaped.key).unwrap_or(shaped.id);
        let mut registry = original.to_builder();
        if shaped_id == shaped.id {
            registry.quantity_type(shaped);
        }
        let registry = registry.build().unwrap();
        let member = |n: u8| SemanticId::from_bytes([n; 16]);
        let domain = Domain {
            members: pse_math::binding::FiniteDomain::new(
                member(80),
                members.iter().map(|&n| member(n)).collect(),
                100,
            )
            .unwrap(),
            kind: standard_registry()
                .unwrap()
                .entity_kind_named("species")
                .unwrap(),
        };
        let selected = Domain {
            members: pse_math::binding::FiniteDomain::new(member(81), vec![member(2)], 100)
                .unwrap(),
            kind: standard_registry()
                .unwrap()
                .entity_kind_named("species")
                .unwrap(),
        };
        let domains = BTreeMap::from([("species".into(), domain), ("selected".into(), selected)]);
        let groups = BTreeMap::from([(
            "flow".into(),
            Group {
                quantity: shaped_id,
                axes: vec!["species".into()],
                slots: present
                    .iter()
                    .enumerate()
                    .map(|(slot, &n)| (vec![member(n)], slot))
                    .collect(),
            },
        )]);
        let formals: Vec<_> = present
            .iter()
            .enumerate()
            .map(|(slot, _)| Formal {
                path: format!("flow_{slot}"),
                quantity: scalar,
            })
            .collect();
        let expression = dsl::parse_expr(text).unwrap();
        let h = ContentHash::from_bytes([1; 32]);
        Request {
            definition: member(90),
            expressions: std::slice::from_ref(&expression),
            formals: &formals,
            domains: &domains,
            providers: &BTreeMap::new(),
            groups: &groups,
            literals: &BTreeMap::new(),
            physical: h,
            structure: h,

            limits: BodyLimits::default(),
        }
        .compile(
            &registry,
            &StandardInvariantChecker,
            DerivativeOrder::Value,
            Optimization::default(),
            &Arc::new(AtomicBool::new(false)),
        )
    }
    fn evaluate(body: &mut Body, values: &[f64]) -> Vec<f64> {
        body.worker()
            .evaluate(
                values,
                DerivativeOrder::Value,
                &mut BTreeMap::new(),
                &Arc::new(AtomicBool::new(false)),
            )
            .unwrap()
            .values
    }
    #[test]
    fn finite_filtered_and_empty_reductions_preserve_multiplicity() {
        pse_math::initialize().unwrap();
        let mut sum = indexed(
            "sum(i in species | flow[i] + flow[i])",
            &[2, 1],
            &[1, 2],
            "neutral",
        )
        .unwrap();
        assert_eq!(evaluate(&mut sum, &[3.0, 4.0]), vec![14.0]);
        let mut filtered = indexed(
            "sum(i in species where i in selected | flow[i] + flow[i])",
            &[1, 2],
            &[1, 2],
            "neutral",
        )
        .unwrap();
        assert_eq!(evaluate(&mut filtered, &[3.0, 4.0]), vec![8.0]);
        let mut empty = indexed("sum(i in species | flow[i])", &[], &[], "neutral").unwrap();
        assert_eq!(evaluate(&mut empty, &[]), vec![0.0]);
        let mut product = indexed("prod(i in species | flow[i])", &[], &[], "neutral").unwrap();
        assert_eq!(evaluate(&mut product, &[]), vec![1.0]);
    }
    #[test]
    fn empty_reductions_still_validate_physics_and_filter_declarations() {
        pse_math::initialize().unwrap();
        assert!(indexed("sum(i in species | flow[i])", &[], &[], "temperature.point").is_err());
        assert!(
            indexed(
                "sum(i in species where i in missing | flow[i])",
                &[],
                &[],
                "neutral"
            )
            .is_err()
        );
        assert!(indexed("sum(i in species | missing[i])", &[], &[], "neutral").is_err());
    }
    #[test]
    fn ragged_members_are_explicit_and_never_defaulted() {
        pse_math::initialize().unwrap();
        assert!(indexed("sum(i in species | flow[i])", &[1, 2], &[2], "neutral").is_err());
        let mut filtered = indexed(
            "sum(i in species where i in selected | flow[i])",
            &[1, 2],
            &[2],
            "neutral",
        )
        .unwrap();
        assert_eq!(evaluate(&mut filtered, &[7.0]), vec![7.0]);
    }
    #[test]
    fn short_circuit_guards_do_not_touch_unsafe_predicates_or_values() {
        pse_math::initialize().unwrap();
        let formals = [Formal {
            path: "x".into(),
            quantity: ids::quantity("neutral"),
        }];
        let body = compile(
            "if x != 0 and 1/x > 0 then log(x) else 0",
            &formals,
            DerivativeOrder::Value,
        )
        .unwrap();
        for (x, expected) in [(0.0, 0.0), (-1.0, 0.0), (2.0, 2.0f64.ln())] {
            let value = body
                .worker()
                .evaluate(
                    &[x],
                    DerivativeOrder::Value,
                    &mut BTreeMap::new(),
                    &Arc::new(AtomicBool::new(false)),
                )
                .unwrap();
            assert!((value.values[0] - expected).abs() < 1e-14);
        }
        assert!(compile("if x > 0 then x else 0", &formals, DerivativeOrder::First).is_err());
        let fixed = compile(
            "if true then x else log(-1)",
            &formals,
            DerivativeOrder::Second,
        )
        .unwrap();
        assert_eq!(
            fixed
                .worker()
                .evaluate(
                    &[2.0],
                    DerivativeOrder::Second,
                    &mut BTreeMap::new(),
                    &Arc::new(AtomicBool::new(false))
                )
                .unwrap()
                .values,
            vec![2.0]
        );
        assert!(compile("tanh(x)", &formals, DerivativeOrder::Value).is_err());
    }
    #[test]
    fn ordered_outputs_share_semantics_across_artifact_profiles() {
        pse_math::initialize().unwrap();
        let registry = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let expressions = [
            dsl::parse_expr("x*x").unwrap(),
            dsl::parse_expr("x+1").unwrap(),
        ];
        let formals = [Formal {
            path: "x".into(),
            quantity,
        }];
        let request = Request {
            definition: SemanticId::from_bytes([1; 16]),
            expressions: &expressions,
            formals: &formals,
            domains: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            providers: &BTreeMap::new(),
            literals: &BTreeMap::new(),
            physical: ContentHash::from_bytes([1; 32]),
            structure: ContentHash::from_bytes([2; 32]),

            limits: BodyLimits::default(),
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let a = request
            .compile(
                &registry,
                &StandardInvariantChecker,
                DerivativeOrder::First,
                Optimization::default(),
                &cancel,
            )
            .unwrap();
        let b = request
            .compile(
                &registry,
                &StandardInvariantChecker,
                DerivativeOrder::Second,
                Optimization {
                    cores: 2,
                    ..Optimization::default()
                },
                &cancel,
            )
            .unwrap();
        assert_eq!(a.spec, b.spec);
        assert!(!Arc::ptr_eq(&a.artifact, &b.artifact));
        assert_eq!(
            b.worker()
                .evaluate(
                    &[2.0],
                    DerivativeOrder::Second,
                    &mut BTreeMap::new(),
                    &cancel
                )
                .unwrap()
                .hessians,
            vec![2.0, 0.0]
        );
        let reversed = [expressions[1].clone(), expressions[0].clone()];
        let reversed = Request {
            expressions: &reversed,
            ..request
        };
        assert_ne!(
            a.spec.key(),
            reversed
                .admit(&registry, &StandardInvariantChecker, &cancel)
                .unwrap()
                .spec
                .key()
        );
    }

    #[test]
    fn executable_provider_admission_preserves_physics_phase_and_recoverable_errors() {
        pse_math::initialize().unwrap();
        use pse_kernels::{
            Port, Provider, ProviderError, ProviderFactory, ProviderSpec, ProviderValues,
            Registration,
        };
        #[derive(Debug, Clone)]
        struct Square(ProviderSpec);
        impl Provider for Square {
            fn spec(&self) -> &ProviderSpec {
                &self.0
            }
            fn evaluate(
                &mut self,
                x: &[f64],
                request: &pse_kernels::ProviderRequest,
                context: &pse_kernels::EvaluationContext<'_>,
            ) -> Result<ProviderValues, ProviderError> {
                request.validate(&self.0, context)?;
                let order = request.order;
                if x.len() != 1 || x[0] < 0.0 {
                    return Err(ProviderError::Trial("selected physical branch".into()));
                }
                if order != DerivativeOrder::Value {
                    return Err(ProviderError::Contract("value-only implementation".into()));
                }
                Ok(ProviderValues {
                    values: vec![x[0] * x[0]],
                    jacobian: vec![],
                    hessians: vec![],
                })
            }
        }
        impl ProviderFactory for Square {
            fn spec(&self) -> &ProviderSpec {
                &self.0
            }
            fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
                Ok(Box::new(self.clone()))
            }
        }
        let registry = standard_registry().unwrap();
        let id = SemanticId::from_bytes([3; 16]);
        let hash = ContentHash::from_bytes([3; 32]);
        let quantity = ids::quantity("neutral");
        let unit = registry.quantity_type(quantity).unwrap().canonical_unit;
        let port = Port { id, quantity, unit };
        let registration = Registration::new(
            Arc::new(Square(ProviderSpec {
                shapes: pse_kernels::ProviderShapes::default(),
                derivative_source: pse_kernels::DerivativeSource::Analytic,
                id,
                revision: hash,
                data: hash,

                inputs: vec![port.clone()],
                outputs: vec![port],
                derivatives: DerivativeOrder::Value,
                smoothness: DerivativeOrder::Second,
            })),
            &registry,
        )
        .unwrap();
        let providers = BTreeMap::from([(
            "square".into(),
            ProviderCall {
                descriptor: registration.descriptor(),
                output: 0,
            },
        )]);
        let formals = [Formal {
            path: "x".into(),
            quantity,
        }];
        let expr = dsl::parse_expr("kernel.square(x)+1").unwrap();
        let request = Request {
            definition: id,
            expressions: std::slice::from_ref(&expr),
            formals: &formals,
            domains: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            providers: &providers,
            literals: &BTreeMap::new(),
            physical: hash,
            structure: hash,

            limits: BodyLimits::default(),
        };
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(
            request
                .compile(
                    &registry,
                    &StandardInvariantChecker,
                    DerivativeOrder::First,
                    Optimization::default(),
                    &cancel
                )
                .is_err()
        );
        let body = request
            .compile(
                &registry,
                &StandardInvariantChecker,
                DerivativeOrder::Value,
                Optimization::default(),
                &cancel,
            )
            .unwrap();
        let mut extra = providers.clone();
        extra.insert(
            "unused".into(),
            ProviderCall {
                descriptor: registration.descriptor(),
                output: 0,
            },
        );
        let admitted = Request {
            providers: &extra,
            ..request
        }
        .admit(&registry, &StandardInvariantChecker, &cancel)
        .unwrap();
        assert_eq!(admitted.spec, body.spec);
        assert_eq!(body.spec.providers.len(), 1);
        let mut worker = body.worker();
        let mut workers =
            BTreeMap::from([(registration.spec().key(), registration.worker().unwrap())]);
        assert!(
            matches!(worker.evaluate(&[-1.0],DerivativeOrder::Value,&mut workers,&cancel),Err(MathError::Provider{source_id,..}) if body.occurrences.iter().any(|o|o.id==source_id))
        );
        assert_eq!(
            worker
                .evaluate(&[3.0], DerivativeOrder::Value, &mut workers, &cancel)
                .unwrap()
                .values,
            vec![10.0]
        );
        let other = Square(ProviderSpec {
            revision: ContentHash::from_bytes([4; 32]),
            ..registration.spec().clone()
        });
        workers.insert(registration.spec().key(), Box::new(other));
        assert!(
            worker
                .evaluate(&[3.0], DerivativeOrder::Value, &mut workers, &cancel)
                .is_err()
        );
    }
}

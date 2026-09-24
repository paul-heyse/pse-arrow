// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Direct authored syntax to physically admitted library bodies.
//! Source occurrences and formal bindings remain separate from Symbolica arithmetic.
use pse_authoring::dsl::{self, BinaryOp, CompareOp, Expr, ExprKind, PredicateKind};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
#[cfg(test)]
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError,
    binding::BodySpec,
    guarded::Comparison,
    typed::{Binary, BodyBuilder, BodyLimits, Guard, TypedValue},
};
#[cfg(test)]
use pse_math::{guarded::CompiledBody, library::Optimization};
use pse_quantity::{
    IndexSet, QuantityRegistry, QuantityTypeId, Ratio, UnitId, infer::InvariantChecker,
    literal::LiteralContext,
};
use std::sync::Arc;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
};

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
    pub kind: pse_quantity::DomainKind,
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

/// One authored local body and the exact physical/specialization context it consumes.
#[derive(Debug)]
pub(crate) struct Request<'a> {
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
    /// Resolved unit spellings, including the unitless spelling `1`.
    pub units: &'a BTreeMap<String, UnitId>,
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
    /// Durable semantic specification.
    pub spec: BodySpec,
    /// Result's complete physical contract.
    pub quantities: Vec<QuantityTypeId>,
    /// Authored occurrence provenance.
    pub occurrences: Vec<Occurrence>,
    /// Symbolica regions and complete structural support, without mutable workspaces.
    pub math: Arc<pse_math::guarded::PreparedBody>,
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
    pub(crate) fn admit(
        &self,
        registry: &QuantityRegistry,
        checker: &dyn InvariantChecker,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<AdmittedBody, MathError> {
        let entries = self
            .domains
            .values()
            .map(|v| v.members.members().len())
            .chain(self.groups.values().map(|v| v.slots.len()))
            .try_fold(self.formals.len(), |n, m| n.checked_add(m))
            .ok_or(MathError::Limit("preparation cardinality"))?;
        let names = self
            .formals
            .iter()
            .map(|v| v.path.len())
            .chain(self.domains.keys().map(String::len))
            .chain(self.groups.keys().map(String::len))
            .chain(self.units.keys().map(String::len))
            .try_fold(0usize, |n, m| n.checked_add(m))
            .ok_or(MathError::Limit("preparation text"))?;
        if entries > self.limits.occurrences
            || names > self.limits.occurrences.saturating_mul(256)
            || self.providers.len() > self.limits.slots
        {
            return Err(MathError::Limit("preparation inputs"));
        }
        let mut builder = BodyBuilder::new(registry, checker, self.formals.len(), self.limits)?;
        let mut paths = BTreeMap::new();
        let mut hash = FramedHasher::new("pse.math.typed-definition.v1");
        hash.id(&self.definition).u64(self.formals.len() as u64);
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
            paths,
            locals: BTreeMap::new(),
            occurrences: vec![],
            hash,
            cancelled,
            coordinates: BTreeMap::new(),
            physical_only: false,
        };
        if self.expressions.is_empty() {
            return Err(MathError::Contract("empty authored output group".into()));
        }
        lower.hash.u64(self.expressions.len() as u64);
        let values = self
            .expressions
            .iter()
            .map(|expr| lower.expression(expr, &mut builder, 0))
            .collect::<Result<Vec<_>, _>>()?;
        let quantities = values.iter().map(TypedValue::quantity).collect();
        let math = Arc::new(builder.prepare(&values)?);
        let spec = BodySpec {
            definition: lower.hash.finish_hash(),
            structure: self.structure,
            physical: self.physical,
            providers: math.providers().iter().map(|p| p.identity()).collect(),
            policy: pse_math::binding::guarded_real_policy(),
        };
        Ok(AdmittedBody {
            spec,
            quantities,
            occurrences: lower.occurrences,
            math,
        })
    }
}

struct Lower<'a, 'b> {
    request: &'a Request<'b>,
    registry: &'a QuantityRegistry,
    paths: BTreeMap<String, usize>,
    locals: BTreeMap<String, TypedValue>,
    occurrences: Vec<Occurrence>,
    hash: FramedHasher,
    cancelled: &'a Arc<AtomicBool>,
    coordinates: BTreeMap<String, Coordinate>,
    physical_only: bool,
}
impl Lower<'_, '_> {
    fn source(&mut self, expr: &Expr, depth: usize) -> Result<SemanticId, MathError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        if depth > 128 || self.occurrences.len() >= self.request.limits.occurrences {
            return Err(MathError::Limit("authored syntax depth or occurrences"));
        }
        let mut h = FramedHasher::new("pse.math.source-occurrence.v1");
        h.id(&self.request.definition)
            .u64(self.occurrences.len() as u64);
        let id = h.finish_id();
        self.occurrences.push(Occurrence {
            id,
            span: expr.span,
        });
        Ok(id)
    }
    fn expression(
        &mut self,
        expr: &Expr,
        builder: &mut BodyBuilder<'_>,
        depth: usize,
    ) -> Result<TypedValue, MathError> {
        let source = self.source(expr, depth)?;
        match &expr.kind {
            ExprKind::Number(number) => {
                self.hash
                    .str("literal")
                    .u64(pse_ids::canonical_f64_bits(number.value));
                let spelling = number.unit.as_deref().unwrap_or("1");
                let unit =
                    *self.request.units.get(spelling).ok_or_else(|| {
                        MathError::Contract(format!("unresolved unit {spelling}"))
                    })?;
                self.hash.id(&unit.as_id());
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
                } else {
                    LiteralContext::Free
                };
                builder.literal(number.value, unit, context, source)
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
                    return Ok(value.clone());
                }
                let slot = *self
                    .paths
                    .get(&name)
                    .ok_or_else(|| MathError::Contract(format!("unresolved path {name}")))?;
                builder.input(
                    slot,
                    self.request.formals[slot].quantity,
                    IndexSet::new(),
                    source,
                )
            }
            ExprKind::Neg(value) => {
                self.hash.str("neg");
                let value = self.expression(value, builder, depth + 1)?;
                builder.negate(value, source)
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.hash.str(op.as_str());
                let left = self.expression(lhs, builder, depth + 1)?;
                let right = self.expression(rhs, builder, depth + 1)?;
                let exponent = if *op == BinaryOp::Pow {
                    integer_exponent(rhs)
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
                builder.binary(operation, left, right, exponent, source)
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
                builder.unary(*function, value, source)
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
                    let value = self.expression(expr, builder, depth + 1)?;
                    self.locals.insert(name.clone(), value);
                }
                let result = self.expression(body, builder, depth + 1);
                self.locals = saved;
                result
            }
            ExprKind::Reduce { kind, binder, body } => {
                self.reduce(*kind, binder, body, builder, depth + 1, source)
            }
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
        }
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
    fn reduce(
        &mut self,
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
            paths: self.paths.clone(),
            locals: self.locals.clone(),
            occurrences: vec![],
            hash: FramedHasher::new("pse.math.physical-pass.v1"),
            cancelled: self.cancelled,
            coordinates: self.coordinates.clone(),
            physical_only: true,
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
        builder.reduce(kind, bound, &prototype, &terms, source)
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
                    |expr: &Expr| -> Result<(String, pse_quantity::DomainKind), MathError> {
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
        if depth > 128 {
            return Err(MathError::Limit("conditional depth"));
        }
        self.hash.str("conditional");
        match &predicate.kind {
            PredicateKind::Not(inner) => {
                self.hash.str("not");
                self.conditional(inner, otherwise, then, builder, depth + 1, source)
            }
            PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
                let conjunction = matches!(predicate.kind, PredicateKind::And(..));
                self.hash.str(if conjunction { "and" } else { "or" });
                let nested = Expr {
                    span: predicate.span,
                    kind: ExprKind::Conditional {
                        guard: Box::new((**b).clone()),
                        then: Box::new(then.clone()),
                        otherwise: Box::new(otherwise.clone()),
                    },
                };
                if conjunction {
                    self.conditional(a, &nested, otherwise, builder, depth + 1, source)
                } else {
                    self.conditional(a, then, &nested, builder, depth + 1, source)
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
                    |b| cell.borrow_mut().expression(then, b, depth + 1),
                    |b| cell.borrow_mut().expression(otherwise, b, depth + 1),
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
        let right = self.expression(rhs, builder, depth + 1)?;
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
fn integer_exponent(expression: &Expr) -> Option<Ratio> {
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
            let positive = integer_exponent(inner)?;
            Ratio::new(-i32::from(positive.num()), 1).ok()
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
        let expression = dsl::parse_expr(text).unwrap();
        let registry = standard_registry().unwrap();
        let units = BTreeMap::from([(
            "1".into(),
            registry
                .quantity_type(ids::quantity("neutral"))
                .unwrap()
                .canonical_unit,
        )]);
        let h = ContentHash::from_bytes([1; 32]);
        Request {
            definition: SemanticId::from_bytes([1; 16]),
            expressions: std::slice::from_ref(&expression),
            formals,
            domains: &BTreeMap::new(),
            providers: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            units: &units,
            literals: &BTreeMap::new(),
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
    #[test]
    fn source_compilation_preserves_obligations_erased_by_cas() {
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
        shaped.key.shape = vec![pse_quantity::DomainKind::Species];
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
            kind: pse_quantity::DomainKind::Species,
        };
        let selected = Domain {
            members: pse_math::binding::FiniteDomain::new(member(81), vec![member(2)], 100)
                .unwrap(),
            kind: pse_quantity::DomainKind::Species,
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
        let units = BTreeMap::from([(
            "1".into(),
            registry
                .quantity_type(ids::quantity("neutral"))
                .unwrap()
                .canonical_unit,
        )]);
        Request {
            definition: member(90),
            expressions: std::slice::from_ref(&expression),
            formals: &formals,
            domains: &domains,
            providers: &BTreeMap::new(),
            groups: &groups,
            units: &units,
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
        assert!(dsl::parse_expr("tanh(x)").is_err());
    }
    #[test]
    fn ordered_outputs_share_semantics_across_artifact_profiles() {
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
        let units = BTreeMap::from([(
            "1".into(),
            registry.quantity_type(quantity).unwrap().canonical_unit,
        )]);
        let request = Request {
            definition: SemanticId::from_bytes([1; 16]),
            expressions: &expressions,
            formals: &formals,
            domains: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            providers: &BTreeMap::new(),
            units: &units,
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
        use pse_kernels::{
            Phase, Port, Provider, ProviderError, ProviderFactory, ProviderSpec, ProviderValues,
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
                id,
                revision: hash,
                data: hash,
                envelope: None,
                components: vec![],
                phase: Phase { id, revision: hash },
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
        let units = BTreeMap::from([("1".into(), unit)]);
        let expr = dsl::parse_expr("kernel.square(x)+1").unwrap();
        let request = Request {
            definition: id,
            expressions: std::slice::from_ref(&expr),
            formals: &formals,
            domains: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            providers: &providers,
            units: &units,
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
            phase: Phase {
                id,
                revision: ContentHash::from_bytes([4; 32]),
            },
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

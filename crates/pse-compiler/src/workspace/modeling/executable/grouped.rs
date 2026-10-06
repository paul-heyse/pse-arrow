// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Normalize each consuming expression once, then bind semantic coordinates into CasePlan.
use super::*;

/// Complete immutable body dependencies; the digest excludes revision attribution.
#[derive(Clone, Debug)]
struct BodyInput {
    identity: pse_ids::roles::SemanticBodyHash,
    expression: Expr,
    formals: Vec<Formal>,
    functions: BTreeMap<String, pse_modeling::Function>,
    providers: BTreeMap<String, ProviderCall>,
    hints: BTreeMap<String, QuantityTypeId>,
    validity: BTreeMap<String, crate::typed_math::Validity>,
    quantity: QuantityTypeId,
    physical: ContentHash,
    limits: BodyLimits,
}
impl PartialEq for BodyInput {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}
impl Eq for BodyInput {}
impl std::hash::Hash for BodyInput {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        std::hash::Hash::hash(&self.identity, h);
    }
}
#[salsa::interned(heap_size = body_key_heap)]
struct BodyKey<'db> {
    input: Arc<BodyInput>,
}
fn body_key_heap((input,): &(Arc<BodyInput>,)) -> usize {
    size_of::<BodyInput>()
        + pse_modeling::expression::retained_bytes(&input.expression)
        + input
            .formals
            .iter()
            .map(|f| size_of::<Formal>() + f.path.capacity())
            .sum::<usize>()
        + input
            .functions
            .iter()
            .map(|(n, f)| n.capacity() + f.retained_bytes() + 64)
            .sum::<usize>()
        + input.providers.len() * 512
        + input.hints.len() * 128
        + input.validity.len() * 512
}
#[salsa::tracked(returns(clone), lru = 64, heap_size = body_heap)]
fn semantic_body(
    db: &dyn CompilerDb,
    inventory: Inventory,
    key: BodyKey<'_>,
) -> Result<Arc<AdmittedBody>> {
    checkpoint(db);
    let input = key.input(db);
    let retained = match db.body_retention() {
        Some(cache) => cache.get(input.identity)?,
        None => None,
    };
    if let Some(body) = retained {
        let _reuse = tracing::info_span!(
            "pse.case.semantic_body_reuse",
            product = "body",
            derivative_order = "value",
            reused = true
        )
        .entered();
        if body.semantic_identity() != Some(input.identity) || body.spec.physical != input.physical
        {
            body_refused(
                db,
                MathError::Contract(
                    "retained body has a different immutable dependency closure".into(),
                ),
            );
        }
        return Ok(body);
    }
    // Enter only after retained lookup misses: a query validation/cache hit is not
    // a fresh arithmetic admission. The benchmark subscriber observes this owner.
    let admission = tracing::info_span!(
        "pse.case.semantic_body_admission",
        product = "body",
        derivative_order = "value",
        success = false
    );
    let _admission = admission.enter();
    let generation = db.body_retention().map(|cache| cache.generation());
    let mut body = crate::typed_math::Request {
        definition: SemanticId::NIL,
        expressions: std::slice::from_ref(&input.expression),
        formals: &input.formals,
        domains: &BTreeMap::new(),
        groups: &BTreeMap::new(),
        providers: &input.providers,
        literals: &BTreeMap::new(),
        physical: input.physical,
        structure: input.identity.as_id(),
        limits: input.limits,
    }
    .admit_modeling_outputs(
        inventory.quantities(db),
        inventory.preconditions(db).as_ref(),
        db.cancel(),
        &input.functions,
        &[input.quantity],
        &input.hints,
        &input.validity,
    )?;
    body.math = Arc::new(
        Arc::unwrap_or_clone(body.math)
            .with_checked_members(input.validity.values().map(|range| range.target).collect())?,
    );
    checkpoint(db);
    let body = Arc::new(body.for_semantic_identity(input.identity));
    admission.record("success", true);
    if let Some(cache) = db.body_retention() {
        return match cache.retain(generation.unwrap_or(u64::MAX), input.identity, body.clone()) {
            Ok(retained) if retained.as_ref() == body.as_ref() => Ok(retained),
            Ok(_) => body_refused(
                db,
                MathError::Contract("retention changed admitted mathematical meaning".into()),
            ),
            Err(error) => body_refused(db, error),
        };
    }
    Ok(body)
}
fn body_heap(value: &Result<Arc<AdmittedBody>>) -> usize {
    value
        .as_ref()
        .map_or_else(CompileError::retained_bytes, |b| {
            b.descriptor_bytes() + b.math.retained_bytes()
        })
}

pub(super) fn configure(db: &mut CompilerDatabase, n: usize) {
    semantic_body::set_lru_capacity(db, n);
}

fn predicate_expressions(predicate: &dsl::Predicate, pending: &mut Vec<Expr>) {
    use dsl::PredicateKind;
    match &predicate.kind {
        PredicateKind::Compare { lhs, rhs, .. } => {
            pending.push(lhs.as_ref().clone());
            pending.push(rhs.as_ref().clone());
        }
        PredicateKind::In { expr, .. } | PredicateKind::Atom(expr) => {
            pending.push(expr.as_ref().clone())
        }
        PredicateKind::And(a, b) | PredicateKind::Or(a, b) => {
            predicate_expressions(a, pending);
            predicate_expressions(b, pending);
        }
        PredicateKind::Not(p) => predicate_expressions(p, pending),
        PredicateKind::Bool(_) | PredicateKind::Null => {}
    }
}
/// Follow only functions and provider calls the body can actually consume.
fn dependencies(
    expression: &Expr,
    validity: &BTreeMap<String, crate::typed_math::Validity>,
    available: &BTreeMap<String, pse_modeling::Function>,
) -> (BTreeMap<String, pse_modeling::Function>, BTreeSet<String>) {
    let mut pending = vec![expression.clone()];
    // Member-range checks are consumed by Lower::validated independently of the
    // arithmetic expression. normalize has already trimmed these to demanded members.
    for range in validity.values() {
        pending.push(range.lower.clone());
        pending.push(range.upper.clone());
    }
    let mut functions = BTreeMap::new();
    let mut providers = BTreeSet::new();
    while let Some(mut expression) = pending.pop() {
        let _: std::result::Result<(), std::convert::Infallible> =
            expression.try_walk_mut(|node| {
                if let ExprKind::Kernel { name, .. } = &node.kind {
                    providers.insert(name.clone());
                }
                let name = match &node.kind {
                    ExprKind::NamedCall { name, .. } => Some(dsl::render_path(name)),
                    ExprKind::Partial { function, .. } => Some(dsl::render_path(function)),
                    _ => None,
                };
                if let Some(name) = name
                    && !functions.contains_key(&name)
                    && let Some(function) = available.get(&name)
                {
                    if let Some(body) = &function.body {
                        pending.push(body.clone());
                    }
                    // Validity and envelope predicates may themselves call functions.
                    if let Some(validity) = &function.validity {
                        predicate_expressions(validity, &mut pending);
                    }
                    for guard in &function.envelopes {
                        predicate_expressions(&guard.predicate, &mut pending);
                    }
                    for usage in &function.applicability_uses {
                        for predicate in &usage.predicates {
                            predicate_expressions(predicate, &mut pending);
                        }
                        pending.extend(usage.inputs.iter().cloned());
                    }
                    // Contextual admission retains the actual common-state anchor
                    // expressions too, independently of the numerical wrapper body.
                    let translation = match &function.physical_operation {
                        Some(pse_modeling::PhysicalOperation::ReferenceTranslation(
                            translation,
                        ))
                        | Some(pse_modeling::PhysicalOperation::ReferenceAnchorMean {
                            translation,
                            ..
                        }) => Some(translation),
                        _ => None,
                    };
                    if let Some(translation) = translation {
                        pending.push(translation.temperature.clone());
                        pending.push(translation.pressure.clone());
                        for anchor in &translation.anchors {
                            pending.extend(anchor.values.iter().cloned());
                        }
                    }
                    if let Some(external) = &function.external {
                        providers.insert(external.implementation.clone());
                    }
                    functions.insert(name.clone(), function.clone());
                }
                Ok(())
            });
    }
    (functions, providers)
}
use pse_kernels::Port;
use pse_math::binding::{
    CaseLimits, CaseStructure, Contribution, InstanceBinding, Row, SlotBinding, Target, Variable,
};

impl ModelingOutput {
    /// Stable output-row identity; presentation order is independent of assembly order.
    pub fn row_id(&self) -> SemanticId {
        match self {
            Self::ConditionalBoundary(id) => {
                pse_ids::named_id(*id, "conditional-boundary-residual")
            }
            Self::Inventory(id) => pse_ids::named_id(*id, "conserved-inventory-output"),
            Self::InventoryTransfer { balance, event } => pse_ids::named_id(
                pse_ids::named_id(*balance, &format!("event-{event}")),
                "conserved-transfer-output",
            ),
            Self::InitialState { state, equation } => {
                pse_ids::named_id(*state, &format!("initial-state-{equation}"))
            }
            Self::DynamicRate { state, .. } => pse_ids::named_id(*state, "dynamic-rate"),
            Self::Test { id, component } => pse_ids::named_id(*id, &format!("test-{component:?}")),
            Self::Hint {
                target,
                declaration,
                kind,
            } => pse_ids::named_id(
                pse_ids::named_id(*target, &declaration.to_string()),
                &format!("hint-{kind:?}"),
            ),
            Self::OriginalEquation(id) => pse_ids::named_id(*id, "original-residual"),
            Self::Penalty(id) => pse_ids::named_id(*id, "elastic-penalty"),
            Self::Term {
                equation, ordinal, ..
            } => pse_ids::named_id(*equation, &format!("original-term-{ordinal}")),
            Self::Equation { id, .. } => *id,
            Self::LevelBound { row, .. } => *row,
            Self::Member(id) => pse_ids::named_id(*id, "modeling-member-output"),
            Self::Contribution { contribution, .. } => {
                pse_ids::named_id(*contribution, "modeling-contribution-output")
            }
        }
    }
}
impl ModelingOutput {
    /// The selected constraint row this output is, with its sense: an equation residual,
    /// or a generated objective bound. Every other output is an observation.
    pub fn constraint(&self) -> Option<(SemanticId, EquationSense)> {
        match self {
            Self::ConditionalBoundary(_) => Some((self.row_id(), EquationSense::Eq)),
            Self::Equation { id, sense } => Some((*id, *sense)),
            Self::LevelBound { row, sense, .. } => Some((*row, *sense)),
            _ => None,
        }
    }
}
impl AdmittedModeling {
    /// Derive selected equation scales from the original term outputs at an explicit
    /// nominal point. These values never come from a simplified residual.
    /// # Errors
    /// Absent rows, invalid observations or an unrepresentable scale.
    pub fn derived_scales(
        &self,
        schemes: &BTreeMap<SemanticId, pse_model::generated::enums::ConstraintScalingScheme>,
        values: &[f64],
    ) -> Result<BTreeMap<SemanticId, f64>> {
        if values.len() != self.case.rows().len() {
            return Err(CompileError::Missing("nominal observation extent".into()));
        }
        let rows = self
            .case
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect::<BTreeMap<_, _>>();
        schemes
            .iter()
            .map(|(id, scheme)| {
                let sources = self.term_outputs.get(id).ok_or_else(|| {
                    CompileError::Missing("original terms for scaling target".into())
                })?;
                let terms = sources
                    .iter()
                    .map(|(row, sign)| {
                        rows.get(row)
                            .map(|i| sign * values[*i])
                            .ok_or_else(|| CompileError::Missing("original term row".into()))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok((*id, pse_math::numerics::term_scale(*scheme, &terms)?))
            })
            .collect()
    }
    /// Use the existing sparse assembly and derivative-demand owner.
    /// # Errors
    /// Physical mismatch, unavailable derivatives, cancellation or resource refusal.
    pub fn plan(
        &self,
        registry: &QuantityRegistry,
        order: DerivativeOrder,
        limits: AssemblyLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<CasePlan>> {
        Ok(Arc::new(CasePlan::prepare(
            self.case.clone(),
            self.bodies
                .iter()
                .map(|(k, b)| (*k, b.math.clone()))
                .collect(),
            registry,
            order,
            limits,
            cancel,
        )?))
    }
    /// Restore public semantic output order after the assembly's deterministic row ordering.
    /// # Errors
    /// A result does not match this immutable structure.
    pub fn ordered_values(&self, values: &[f64]) -> Result<Vec<f64>> {
        if values.len() != self.case.rows().len() {
            return Err(CompileError::Missing("modeling result extent".into()));
        }
        let rows = self
            .case
            .rows()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.id, i))
            .collect::<BTreeMap<_, _>>();
        self.outputs
            .iter()
            .map(|o| {
                rows.get(&o.row_id())
                    .map(|i| values[*i])
                    .ok_or_else(|| CompileError::Missing("modeling output row".into()))
            })
            .collect()
    }
}

/// A trimmed consumer expression with its formals, gathered inputs, local quantities
/// and validity ranges.
type Normalized = (
    Expr,
    Vec<Formal>,
    Vec<SemanticId>,
    BTreeMap<String, QuantityTypeId>,
    BTreeMap<String, crate::typed_math::Validity>,
    BTreeMap<SemanticId, SemanticId>,
);
/// Trim expression members to the consumer's transitive dependencies and rename local
/// symbols by dependency order. Global IDs survive only in the instance gather.
fn normalize(
    expression: &Expr,
    formals: &[Formal],
    inputs: &[SemanticId],
    quantities: &BTreeMap<String, QuantityTypeId>,
    validity: &BTreeMap<String, crate::typed_math::Validity>,
) -> Result<Normalized> {
    let (bindings, body) = match &expression.kind {
        ExprKind::Let { bindings, body } => (bindings.as_slice(), body.as_ref()),
        _ => ([].as_slice(), expression),
    };
    let locals = bindings
        .iter()
        .map(|(n, e)| (n.clone(), e))
        .collect::<BTreeMap<_, _>>();
    let runtime = formals
        .iter()
        .zip(inputs)
        .map(|(f, id)| (f.path.clone(), (f, *id)))
        .collect::<BTreeMap<_, _>>();
    struct Rename<'a> {
        locals: &'a BTreeMap<String, &'a Expr>,
        runtime: &'a BTreeMap<String, (&'a Formal, SemanticId)>,
        names: BTreeMap<String, String>,
        bindings: Vec<(String, Expr)>,
        formals: Vec<Formal>,
        inputs: Vec<SemanticId>,
        quantities: &'a BTreeMap<String, QuantityTypeId>,
        hints: BTreeMap<String, QuantityTypeId>,
        active: BTreeSet<String>,
        validity: &'a BTreeMap<String, crate::typed_math::Validity>,
        normalized_validity: BTreeMap<String, crate::typed_math::Validity>,
        checked_members: BTreeMap<SemanticId, SemanticId>,
    }
    impl Rename<'_> {
        fn expression(&mut self, source: &Expr) -> Result<Expr> {
            let mut result = source.clone();
            result.try_walk_mut(|node| -> Result<()> {
                let ExprKind::Path(path) = &mut node.kind else {
                    return Ok(());
                };
                if path.segments.len() != 1 || !path.segments[0].indices.is_empty() {
                    return Ok(());
                }
                let name = path.segments[0].name.clone();
                if let Some(fresh) = self.names.get(&name) {
                    path.segments[0].name = fresh.clone();
                    return Ok(());
                }
                if let Some((formal, id)) = self.runtime.get(&name) {
                    let fresh = format!("input_{}", self.inputs.len());
                    self.inputs.push(*id);
                    self.formals.push(Formal {
                        path: fresh.clone(),
                        quantity: formal.quantity,
                    });
                    self.names.insert(name.clone(), fresh.clone());
                    path.segments[0].name = fresh;
                } else if let Some(expression) = self.locals.get(&name) {
                    if !self.active.insert(name.clone()) {
                        return Err(CompileError::Missing("cyclic consumer dependency".into()));
                    }
                    let expression = self.expression(expression)?;
                    self.active.remove(&name);
                    let fresh = format!("member_{}", self.bindings.len());
                    self.bindings.push((fresh.clone(), expression));
                    if let Some(q) = self.quantities.get(&name) {
                        self.hints.insert(fresh.clone(), *q);
                    }
                    self.names.insert(name.clone(), fresh.clone());
                    path.segments[0].name = fresh;
                }
                if let Some(range) = self.validity.get(&name).cloned() {
                    let lower = self.expression(&range.lower)?;
                    let upper = self.expression(&range.upper)?;
                    let fresh = self
                        .names
                        .get(&name)
                        .ok_or_else(|| {
                            CompileError::Missing("validity target was not normalized".into())
                        })?
                        .clone();
                    let mut token = FramedHasher::new(pse_ids::Frame::ModelingCheckedMemberV1);
                    token.str(&fresh);
                    let token = token.finish_id();
                    if self
                        .checked_members
                        .insert(token, range.target)
                        .is_some_and(|old| old != range.target)
                    {
                        return Err(CompileError::Missing(
                            "ambiguous checked member binding".into(),
                        ));
                    }
                    self.normalized_validity.insert(
                        fresh,
                        crate::typed_math::Validity {
                            lower,
                            upper,
                            source: range.source,
                            target: token,
                        },
                    );
                }
                Ok(())
            })?;
            Ok(result)
        }
    }
    let mut rename = Rename {
        locals: &locals,
        runtime: &runtime,
        names: BTreeMap::new(),
        bindings: vec![],
        formals: vec![],
        inputs: vec![],
        quantities,
        hints: BTreeMap::new(),
        active: BTreeSet::new(),
        validity,
        normalized_validity: BTreeMap::new(),
        checked_members: BTreeMap::new(),
    };
    let body = rename.expression(body)?;
    let expression = if rename.bindings.is_empty() {
        body
    } else {
        Expr {
            kind: ExprKind::Let {
                bindings: rename.bindings,
                body: Box::new(body),
            },
            span: Span::default(),
        }
    };
    Ok((
        expression,
        rename.formals,
        rename.inputs,
        rename.hints,
        rename.normalized_validity,
        rename.checked_members,
    ))
}

pub(super) fn admit(
    db: &dyn CompilerDb,
    inventory: Inventory,
    p: &Projection,
) -> Result<Arc<AdmittedModeling>> {
    let (implicit, mut providers) = implicit::admit(db, inventory, p)?;
    for function in p.functions.values() {
        if let Some(external) = &function.external {
            let call =
                provider(db, inventory, external.implementation.clone()).ok_or_else(|| {
                    CompileError::Missing(format!(
                        "external implementation {}",
                        external.implementation
                    ))
                })?;
            providers.insert(external.implementation.clone(), call);
        }
    }
    let registry = inventory.quantities(db);
    let checker = inventory.preconditions(db);
    let mut ports = BTreeMap::new();
    let mut variables = vec![];
    let mut parameters = vec![];
    for (id, formal) in p.inputs.iter().zip(&p.formals) {
        let port = Port {
            id: *id,
            quantity: formal.quantity,
            unit: registry
                .quantity_type(formal.quantity)
                .map_err(MathError::from)?
                .canonical_unit,
        };
        ports.insert(*id, port.clone());
        if let Some(domain) = p.free.get(id).copied() {
            // A binary decision or a lowering's convex weight lives in the unit box;
            // cases may only narrow it.
            let unit = domain == pse_model::generated::enums::ModelingVariableDomain::Binary
                || p.unit_interval.contains(id);
            variables.push(Variable {
                port,
                fixed: false,
                domain,
                lower: (unit || p.nonnegative.contains(id)).then_some(0.0),
                upper: unit.then_some(1.0),
            });
        } else {
            parameters.push(port);
        }
    }
    let penalty = p
        .outputs
        .iter()
        .any(|o| matches!(o, ModelingOutput::Penalty(_)));
    // The solved level's members contribute their scales to the objective, and every
    // bounded level's members to its bound row (ADR-0111). Several levels without a
    // selected one are optimized together by a native lexicographic route: each level's
    // members contribute to that level's objective.
    let lexicographic = p.objectives.levels.len() > 1 && p.objectives.selected.is_none();
    let mut objective_scales = BTreeMap::<SemanticId, Vec<(Target, f64)>>::new();
    let solved = p.objectives.solved();
    for (position, level) in p.objectives.levels.iter().enumerate() {
        let target = if lexicographic {
            Some(Target::Objective(position))
        } else {
            solved
                .is_some_and(|solved| std::ptr::eq(solved, level))
                .then_some(Target::PRIMARY)
        };
        for member in p.objectives.members_of(level) {
            let targets = objective_scales.entry(member.term).or_default();
            if let Some(target) = target {
                targets.push((target, member.scale));
            }
            if let Some(bound) = &level.bound {
                targets.push((Target::Row(bound.row), member.scale));
            }
        }
    }
    let levels = if lexicographic {
        if penalty {
            return Err(CompileError::Missing(
                "elastic penalties need one objective; select a lexicographic level".into(),
            ));
        }
        let last = p.objectives.levels.len() - 1;
        p.objectives
            .levels
            .iter()
            .enumerate()
            .map(|(position, level)| {
                let degradation = match (
                    position < last,
                    level.absolute_tolerance,
                    level.relative_tolerance,
                ) {
                    (false, _, _) => None,
                    (true, Some(absolute), Some(relative)) => {
                        Some(pse_math::binding::Degradation { absolute, relative })
                    }
                    (true, _, _) => {
                        return Err(CompileError::Missing(
                            "an earlier lexicographic level declares both tolerances".into(),
                        ));
                    }
                };
                Ok((
                    pse_math::binding::Objective {
                        quantity: level.quantity,
                        sense: objective_sense(level.sense),
                    },
                    degradation,
                ))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let objective = if let Some(level) = solved {
        let quantity = level.quantity;
        if penalty && Some(quantity) != registry.neutral_dimensionless() {
            return Err(CompileError::Missing(
                "elastic penalties require an explicitly normalized dimensionless objective".into(),
            ));
        }
        Some(pse_math::binding::Objective {
            quantity,
            sense: objective_sense(level.sense),
        })
    } else if penalty {
        Some(pse_math::binding::Objective {
            quantity: registry
                .neutral_dimensionless()
                .ok_or_else(|| CompileError::Missing("dimensionless elastic penalty".into()))?,
            sense: pse_math::binding::ObjectiveSense::Minimize,
        })
    } else {
        None
    };
    let mut bodies = BTreeMap::new();
    let mut instances = vec![];
    let mut rows = vec![];
    for (index, (expression, output)) in p.expressions.iter().zip(&p.outputs).enumerate() {
        checkpoint(db);
        if let ModelingOutput::Equation {
            id,
            sense: EquationSense::Eq,
        } = output
            && p.conservation.contains_key(id)
        {
            rows.push(Row {
                id: *id,
                quantity: p.quantities[index],
                lower: 0.0,
                upper: 0.0,
            });
            continue;
        }
        let (expression, formals, inputs, hints, validity, checked_members) = normalize(
            expression,
            &p.formals,
            &p.inputs,
            &p.local_quantities,
            &p.validity,
        )?;
        let (functions, provider_names) = dependencies(&expression, &validity, &p.functions);
        let selected_providers: BTreeMap<_, _> = provider_names
            .iter()
            .filter_map(|n| providers.get(n).map(|v| (n.clone(), v.clone())))
            .collect();
        let physical = physical_identity(registry, checker);
        let mut h = FramedHasher::new(pse_ids::Frame::ModelingConsumerBodyV4);
        h.str(&dsl::render_expr(&expression))
            .id(&p.quantities[index].as_id())
            .hash(&physical)
            .u64(p.body_limits.slots as u64)
            .u64(p.body_limits.occurrences as u64);
        h.u64(formals.len() as u64);
        for f in &formals {
            h.str(&f.path).id(&f.quantity.as_id());
        }
        h.u64(hints.len() as u64);
        for (n, q) in &hints {
            h.str(n).id(&q.as_id());
        }
        h.u64(validity.len() as u64);
        for (name, range) in &validity {
            h.str(name)
                .id(&range.source)
                .id(&range.target)
                .str(&dsl::render_expr(&range.lower))
                .str(&dsl::render_expr(&range.upper));
        }
        h.u64(functions.len() as u64);
        for (name, function) in &functions {
            // Specialized function IDs already frame body, signature, validity,
            // imported data, applicability and physical operation admission.
            h.str(name).id(&function.id.as_id());
        }
        h.u64(selected_providers.len() as u64);
        for (name, provider) in &selected_providers {
            h.str(name)
                .hash(&provider.descriptor.spec().identity())
                .u64(provider.output as u64);
        }
        let identity = pse_ids::roles::SemanticBodyHash::from_id(h.finish_hash());
        let key = BodyKey::new(
            db,
            Arc::new(BodyInput {
                identity,
                expression,
                formals: formals.clone(),
                functions,
                providers: selected_providers,
                hints,
                validity,
                quantity: p.quantities[index],
                physical,
                limits: p.body_limits,
            }),
        );
        let body = semantic_body(db, inventory, key).map_err(|cause| MathError::Instance {
            instance: output.row_id(),
            checked_members: checked_members.clone(),
            cause: Box::new(MathError::Typed {
                retained: cause.retained_bytes(),
                cause: pse_model::diagnostic::DiagnosticCause::new(cause),
            }),
        })?;
        let key = body.spec.key();
        bodies.insert(key, body);
        let slots = inputs
            .iter()
            .zip(&formals)
            .map(|(id, f)| {
                SlotBinding::new(
                    &ports[id],
                    &Port {
                        id: *id,
                        quantity: f.quantity,
                        unit: registry.quantity_type(f.quantity)?.canonical_unit,
                    },
                    registry,
                )
            })
            .collect::<std::result::Result<Vec<_>, MathError>>()?;
        let (lower, upper) = match output.constraint() {
            Some((_, EquationSense::Eq)) => (0.0, 0.0),
            Some((_, EquationSense::Le)) => (f64::NEG_INFINITY, 0.0),
            Some((_, EquationSense::Ge)) => (0.0, f64::INFINITY),
            None => (f64::NEG_INFINITY, f64::INFINITY),
        };
        let id = output.row_id();
        rows.push(Row {
            id,
            quantity: p.quantities[index],
            lower,
            upper,
        });
        let mut contributions = vec![Contribution {
            output: 0,
            target: Target::Row(id),
            // β enters its bound row negatively: `level value − β`.
            scale: if matches!(output, ModelingOutput::LevelBound { .. }) {
                -1.0
            } else {
                1.0
            },
        }];
        if let ModelingOutput::Contribution { contribution, .. } = output {
            for (row, terms) in &p.conservation {
                if let Some((_, sign)) = terms.iter().find(|(term, _)| term == contribution) {
                    contributions.push(Contribution {
                        output: 0,
                        target: Target::Row(*row),
                        scale: *sign,
                    });
                }
            }
        }
        if matches!(output, ModelingOutput::Penalty(_)) {
            contributions.push(Contribution {
                output: 0,
                target: Target::PRIMARY,
                scale: objective.as_ref().map_or(1.0, |o| o.sense.sign()),
            });
        }
        if let ModelingOutput::Member(member) = output {
            for (target, scale) in objective_scales.get(member).into_iter().flatten() {
                contributions.push(Contribution {
                    output: 0,
                    target: *target,
                    scale: *scale,
                });
            }
        }
        instances.push(InstanceBinding {
            instance: id,
            body: key,
            checked_members,
            slots,
            contributions,
        });
    }
    let mut term_outputs = BTreeMap::<SemanticId, Vec<(SemanticId, f64)>>::new();
    for output in &p.outputs {
        if let ModelingOutput::Term {
            equation, negative, ..
        } = output
        {
            term_outputs
                .entry(*equation)
                .or_default()
                .push((output.row_id(), if *negative { -1.0 } else { 1.0 }));
        }
    }
    let case_limits = CaseLimits::default();
    let case = if levels.is_empty() {
        CaseStructure::new(
            variables,
            parameters,
            instances,
            rows,
            objective,
            case_limits,
        )?
    } else {
        CaseStructure::lexicographic(variables, parameters, instances, rows, levels, case_limits)?
    };
    let case = Arc::new(
        case.with_native(p.native.clone())?
            .with_requirements(p.requirements.iter().copied()),
    );
    Ok(Arc::new(AdmittedModeling {
        inputs: p.inputs.clone(),
        outputs: p
            .outputs
            .iter()
            .filter(|o| !matches!(o, ModelingOutput::Term { .. }))
            .cloned()
            .collect(),
        term_outputs,
        implicit,
        bodies,
        case,
    }))
}

#[cfg(test)]
mod dependency_tests {
    use super::*;
    use crate::workspace::{CompilerWorkspace, WorkspaceLimits};
    use pse_modeling::{Bindings, Limits, specialize::root_instance};

    #[test]
    fn reusable_checked_members_bind_actual_inputs_and_computed_locals() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::DiagnosticProjection;
        let source = "package p { def Cell { var x:Scalar; let z:Scalar=2*x; annotation valid x(1,3); annotation valid z(3,6); eq e:z-z==0; } def Root { child a:Cell=Cell(); child b:Cell=Cell(); } }";
        let admit_source = |source: &str| {
            let rows = crate::authored_transfer_tests::rows(source);
            let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
            let mut workspace = CompilerWorkspace::new(
                crate::authored_transfer_tests::context(),
                WorkspaceLimits::default(),
            )
            .unwrap();
            workspace
                .publish_modeling(rows, PhysicalScope::default())
                .unwrap();
            let admitted = workspace
                .admit_modeling(
                    root,
                    root_instance(root),
                    Bindings::default(),
                    Limits::default(),
                )
                .unwrap();
            (workspace, admitted)
        };
        let (workspace, admitted) = admit_source(source);
        let equations = admitted
            .outputs
            .iter()
            .filter_map(|output| match output {
                ModelingOutput::Equation { id, .. } => Some(*id),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let instances = admitted
            .case
            .instances()
            .iter()
            .filter(|binding| equations.contains(&binding.instance))
            .collect::<Vec<_>>();
        assert_eq!(instances.len(), 2);
        assert_eq!(
            instances[0].body, instances[1].body,
            "actual member attribution must not split shared mathematics"
        );
        assert!(Arc::ptr_eq(
            &admitted.bodies[&instances[0].body],
            &admitted.bodies[&instances[1].body]
        ));
        assert_eq!(instances[0].checked_members.len(), 2);
        assert_eq!(
            instances[0].checked_members.keys().collect::<Vec<_>>(),
            instances[1].checked_members.keys().collect::<Vec<_>>()
        );
        assert!(instances[0].checked_members.values().all(|actual| {
            !instances[1]
                .checked_members
                .values()
                .any(|other| other == actual)
        }));
        let token = |path: &str| {
            let mut hash = FramedHasher::new(pse_ids::Frame::ModelingCheckedMemberV1);
            hash.str(path);
            hash.finish_id()
        };
        let cancellation = Arc::new(AtomicBool::new(false));
        let plan = admitted
            .plan(
                &workspace.inputs.quantities,
                DerivativeOrder::Second,
                AssemblyLimits::default(),
                &cancellation,
            )
            .unwrap();
        // Missing or invented attribution is rejected before evaluator construction.
        for extra in [false, true] {
            let mut invalid = admitted.case.instances().to_vec();
            let invalid_binding = invalid
                .iter_mut()
                .find(|binding| binding.instance == instances[0].instance)
                .unwrap();
            if extra {
                invalid_binding.checked_members.insert(
                    SemanticId::from_bytes([123; 16]),
                    SemanticId::from_bytes([124; 16]),
                );
            } else {
                invalid_binding.checked_members.pop_first();
            }
            let structure = Arc::new(
                CaseStructure::new(
                    admitted.case.variables().to_vec(),
                    admitted.case.parameters().to_vec(),
                    invalid,
                    admitted.case.rows().to_vec(),
                    admitted.case.objective().cloned(),
                    CaseLimits::default(),
                )
                .unwrap(),
            );
            let refused = CasePlan::prepare(
                structure,
                admitted
                    .bodies
                    .iter()
                    .map(|(key, body)| (*key, body.math.clone()))
                    .collect(),
                &workspace.inputs.quantities,
                DerivativeOrder::Second,
                AssemblyLimits::default(),
                &cancellation,
            );
            assert!(
                matches!(refused, Err(MathError::Contract(message)) if message == "incomplete or invalid checked-member attribution")
            );
        }
        let mut rebound = admitted.case.instances().to_vec();
        let first = rebound
            .iter_mut()
            .find(|binding| binding.instance == instances[0].instance)
            .unwrap();
        first
            .checked_members
            .insert(token("member_0"), SemanticId::from_bytes([124; 16]));
        let rebound = CaseStructure::new(
            admitted.case.variables().to_vec(),
            admitted.case.parameters().to_vec(),
            rebound,
            admitted.case.rows().to_vec(),
            admitted.case.objective().cloned(),
            CaseLimits::default(),
        )
        .unwrap();
        assert_ne!(
            admitted.case.key(),
            rebound.key(),
            "actual attribution belongs to the current bound view identity"
        );
        let formal_slots = admitted
            .case
            .instances()
            .iter()
            .map(|binding| binding.slots.len())
            .sum();
        let refused = CaseStructure::new(
            admitted.case.variables().to_vec(),
            admitted.case.parameters().to_vec(),
            admitted.case.instances().to_vec(),
            admitted.case.rows().to_vec(),
            admitted.case.objective().cloned(),
            CaseLimits {
                slots: formal_slots,
                ..CaseLimits::default()
            },
        );
        assert!(
            matches!(
                refused,
                Err(MathError::Limit("case slots or empty output layout"))
            ),
            "actual checked-member bindings consume the existing binding extent allowance"
        );
        let assembly = Arc::new(
            plan.compile(Default::default(), Default::default(), &cancellation)
                .unwrap(),
        );
        let mut worker = assembly.worker(BTreeMap::new(), cancellation);
        let mut values = CaseValues {
            scalars: admitted.inputs.iter().map(|id| (*id, 2.0)).collect(),
        };
        assert!(worker.constraints(&values).is_ok());
        for binding in &instances {
            // Input is within its range but the computed local is outside its own range.
            values.scalars.insert(binding.slots[0].source(), 1.0);
            let diagnostic = worker
                .constraints(&values)
                .unwrap_err()
                .boundary_diagnostic(DiagnosticStage::Evaluation);
            assert_eq!(
                diagnostic.validity.as_ref().unwrap().members,
                vec![binding.checked_members[&token("member_0")]]
            );
            assert_eq!(diagnostic.causes[0].validity, diagnostic.validity);
            assert!(
                diagnostic
                    .sources
                    .contains(&binding.checked_members[&token("member_0")])
            );
            assert!(
                binding
                    .checked_members
                    .keys()
                    .all(|local| !diagnostic.sources.contains(local))
            );
            values.scalars.insert(binding.slots[0].source(), 4.0);
            let diagnostic = worker
                .constraints(&values)
                .unwrap_err()
                .boundary_diagnostic(DiagnosticStage::Evaluation);
            assert_eq!(
                diagnostic.validity.as_ref().unwrap().members,
                vec![binding.checked_members[&token("input_0")]]
            );
            assert!(
                diagnostic
                    .sources
                    .contains(&binding.checked_members[&token("input_0")])
            );
            assert!(
                binding
                    .checked_members
                    .keys()
                    .all(|local| !diagnostic.sources.contains(local))
            );
            values.scalars.insert(binding.slots[0].source(), 2.0);
        }
        assert!(worker.constraints(&values).is_ok());
        let (_, changed_range) = admit_source(&source.replace("valid z(3,6)", "valid z(3,7)"));
        let changed = changed_range
            .case
            .instances()
            .iter()
            .find(|binding| binding.instance == instances[0].instance)
            .unwrap();
        assert_ne!(instances[0].body, changed.body);
        let (_, changed_type) = admit_source(
            &source
                .replace("Scalar", "Time")
                .replace("valid x(1,3)", "valid x(1{s},3{s})")
                .replace("valid z(3,6)", "valid z(3{s},6{s})")
                .replace("z-z==0", "z-z==0{s}"),
        );
        let changed = changed_type
            .case
            .instances()
            .iter()
            .find(|binding| binding.instance == instances[0].instance)
            .unwrap();
        assert_ne!(instances[0].body, changed.body);
        assert_ne!(admitted.case.key(), changed_range.case.key());
    }

    #[test]
    fn checked_member_context_crosses_compiler_error_adapter() {
        use pse_diagnostics::DiagnosticStage;
        use pse_model::diagnostic::{DiagnosticCause, DiagnosticProjection, ValidityLineage};
        use pse_model::generated::enums::ModelingValidityLayer;
        let token = SemanticId::from_bytes([1; 16]);
        let actual = SemanticId::from_bytes([2; 16]);
        let source = SemanticId::from_bytes([3; 16]);
        let inner = MathError::Validity(Box::new(ValidityLineage {
            layer: ModelingValidityLayer::Closure,
            source,
            form: None,
            sets: vec![],
            variables: vec![],
            members: vec![token],
        }));
        let native = MathError::Native {
            source_id: token,
            retained: inner.retained_bytes(),
            cause: DiagnosticCause::new(inner),
        };
        let compiler = CompileError::Math(Arc::new(native));
        let expected_class = compiler
            .boundary_diagnostic(DiagnosticStage::ModelingAdmission)
            .class;
        let retained = compiler.retained_bytes();
        let error = MathError::Instance {
            instance: source,
            checked_members: BTreeMap::from([(token, actual)]),
            cause: Box::new(MathError::Typed {
                retained,
                cause: DiagnosticCause::new(compiler),
            }),
        };
        let projected = error.boundary_diagnostic(DiagnosticStage::ModelingAdmission);
        assert_eq!(projected.class, expected_class);
        assert_eq!(projected.validity.as_ref().unwrap().members, vec![actual]);
        assert!(projected.sources.contains(&token));
        let native = &projected.causes[0];
        assert_eq!(native.validity.as_ref().unwrap().members, vec![actual]);
        assert!(native.sources.contains(&token));
        let member_only = &native.causes[0];
        assert_eq!(member_only.validity.as_ref().unwrap().members, vec![actual]);
        assert!(member_only.sources.contains(&actual));
        assert!(!member_only.sources.contains(&token));
        assert!(member_only.sources.contains(&source));
    }

    #[test]
    fn demanded_member_ranges_retain_only_their_transitive_functions() {
        let source = "package p { fn bottom(x:Scalar)->Scalar=1+x; fn lower(x:Scalar)->Scalar=bottom(x); fn upper(x:Scalar)->Scalar=3+x; def Root { var x:Scalar; annotation valid x(lower(0),upper(0)); eq e:x==2; } }";
        let rows = crate::authored_transfer_tests::rows(source);
        let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
        let mut workspace = CompilerWorkspace::new(
            crate::authored_transfer_tests::context(),
            WorkspaceLimits::default(),
        )
        .unwrap();
        workspace
            .publish_modeling(rows, PhysicalScope::default())
            .unwrap();
        let (catalog, request) = workspace
            .modeling_request(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
            )
            .unwrap();
        let projected = projection(&workspace.db, workspace.inventory, catalog, request).unwrap();
        let ordinal = projected
            .outputs
            .iter()
            .position(|output| matches!(output, ModelingOutput::Equation { .. }))
            .unwrap();
        let (expression, _, _, _, validity, _) = normalize(
            &projected.expressions[ordinal],
            &projected.formals,
            &projected.inputs,
            &projected.local_quantities,
            &projected.validity,
        )
        .unwrap();
        assert_eq!(validity.len(), 1);
        let mut available = projected.functions.clone();
        available.insert(
            "unrelated".into(),
            available.values().next().unwrap().clone(),
        );
        let (functions, providers) = dependencies(&expression, &validity, &available);
        assert_eq!(
            functions.len(),
            3,
            "lower, its bottom dependency, and upper"
        );
        assert!(!functions.contains_key("unrelated"));
        assert!(providers.is_empty());
        // Contextual wrappers also retain concrete anchor expressions outside their
        // arithmetic body. Their calls use the same transitive worklist.
        let scalar = workspace.inputs.quantities.neutral_dimensionless().unwrap();
        let magnitude = pse_quantity::CanonicalConversionPlan::registered(
            &workspace.inputs.quantities,
            scalar,
            workspace
                .inputs
                .quantities
                .quantity_type(scalar)
                .unwrap()
                .canonical_unit,
        )
        .unwrap()
        .apply(1.0)
        .unwrap();
        let mut wrapper = available.values().next().unwrap().clone();
        wrapper.body = Some(dsl::parse_expr("0").unwrap());
        wrapper.validity = None;
        wrapper.envelopes.clear();
        wrapper.applicability_uses.clear();
        wrapper.physical_operation = Some(pse_modeling::PhysicalOperation::ReferenceTranslation(
            pse_modeling::contextual::ReferenceTranslation {
                source: scalar,
                target: scalar,
                source_anchor: wrapper.id,
                target_anchor: wrapper.id,
                temperature: dsl::parse_expr("0").unwrap(),
                pressure: dsl::parse_expr("0").unwrap(),
                component_kind: wrapper.id,
                provenance: vec![],
                anchors: vec![pse_modeling::contextual::ReferenceAnchorPair {
                    member: SemanticId::NIL,
                    values: [
                        validity.values().next().unwrap().lower.clone(),
                        validity.values().next().unwrap().upper.clone(),
                    ],
                    temperature: magnitude,
                    pressure: magnitude,
                }],
            },
        ));
        available.insert("contextual_wrapper".into(), wrapper);
        let (anchored, _) = dependencies(
            &dsl::parse_expr("contextual_wrapper()").unwrap(),
            &BTreeMap::new(),
            &available,
        );
        assert_eq!(anchored.len(), functions.len() + 1);
        assert!(functions.keys().all(|name| anchored.contains_key(name)));
        assert!(!anchored.contains_key("unrelated"));
        // Exercise admission too: a scalar equation's range functions must reach the
        // mathematical consumer, even though no function occurs in its arithmetic.
        workspace
            .prepare_modeling_cancellable(
                root,
                root_instance(root),
                Bindings::default(),
                Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
    }
}

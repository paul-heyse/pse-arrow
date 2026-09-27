// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Normalize each consuming expression once, then bind semantic coordinates into CasePlan.
use super::*;
use pse_kernels::Port;
use pse_math::binding::{
    CaseLimits, CaseStructure, Contribution, InstanceBinding, Row, SlotBinding, Target, Variable,
};

impl ModelingOutput {
    /// Stable output-row identity; presentation order is independent of assembly order.
    pub fn row_id(&self) -> SemanticId {
        match self {
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
            Self::Member(id) => pse_ids::named_id(*id, "modeling-member-output"),
            Self::Contribution { contribution, .. } => {
                pse_ids::named_id(*contribution, "modeling-contribution-output")
            }
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
        limits: pse_math::assembly::AssemblyLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<pse_math::assembly::CasePlan>> {
        Ok(Arc::new(pse_math::assembly::CasePlan::prepare(
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

/// Trim expression members to the consumer's transitive dependencies and rename local
/// symbols by dependency order. Global IDs survive only in the instance gather.
fn normalize(
    expression: &Expr,
    formals: &[Formal],
    inputs: &[SemanticId],
    quantities: &BTreeMap<String, QuantityTypeId>,
    validity: &BTreeMap<String, crate::typed_math::Validity>,
) -> Result<(
    Expr,
    Vec<Formal>,
    Vec<SemanticId>,
    BTreeMap<String, QuantityTypeId>,
    BTreeMap<String, crate::typed_math::Validity>,
)> {
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
                    self.normalized_validity.insert(
                        fresh,
                        crate::typed_math::Validity {
                            lower,
                            upper,
                            source: range.source,
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
    ))
}

pub(super) fn admit(
    db: &dyn CompilerDb,
    inventory: Inventory,
    p: &Projection,
) -> Result<Arc<AdmittedModeling>> {
    let (implicit, mut providers) = super::implicit::admit(db, inventory, p)?;
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
    let units = registry.units().map(|u| (u.symbol.clone(), u.id)).collect();
    let mut ports = BTreeMap::new();
    let mut variables = vec![];
    let mut parameters = vec![];
    for (id, formal) in p.inputs.iter().zip(&p.formals) {
        let port = Port {
            id: *id,
            quantity: formal.quantity,
            unit: registry
                .quantity_type(formal.quantity)
                .map_err(pse_math::MathError::from)?
                .canonical_unit,
        };
        ports.insert(*id, port.clone());
        if let Some(domain) = p.free.get(id).copied() {
            // A binary decision implies the unit box; cases may only narrow it.
            let binary = domain == pse_model::generated::enums::ModelingVariableDomain::Binary;
            variables.push(Variable {
                port,
                fixed: false,
                domain,
                lower: (binary || p.nonnegative.contains(id)).then_some(0.0),
                upper: binary.then_some(1.0),
            });
        } else {
            parameters.push(port);
        }
    }
    let penalty = p.outputs.iter().any(|o| matches!(o, ModelingOutput::Penalty(_)));
    let objective = if let Some((target, sense)) = p.objective {
        let index = p.outputs.iter().position(|o| *o == ModelingOutput::Member(target))
            .ok_or_else(|| CompileError::Missing("objective scalar output".into()))?;
        let quantity = p.quantities[index];
        if penalty && Some(quantity) != registry.neutral_dimensionless() {
            return Err(CompileError::Missing("elastic penalties require an explicitly normalized dimensionless objective".into()));
        }
        Some(pse_math::binding::Objective { quantity, sense })
    } else if penalty {
        Some(pse_math::binding::Objective {
            quantity: registry.neutral_dimensionless().ok_or_else(|| CompileError::Missing("dimensionless elastic penalty".into()))?,
            sense: pse_math::binding::ObjectiveSense::Minimize,
        })
    } else { None };
    let mut bodies = BTreeMap::new();
    let mut normalized = BTreeMap::<ContentHash, Arc<AdmittedBody>>::new();
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
        let (expression, formals, inputs, hints, validity) = normalize(
            expression,
            &p.formals,
            &p.inputs,
            &p.local_quantities,
            &p.validity,
        )?;
        let declaration = p.declarations[index];
        let mut h = FramedHasher::new("pse.modeling.consumer-body.v1");
        h.id(&declaration)
            .str(&dsl::render_expr(&expression))
            .id(&p.quantities[index].as_id());
        for f in &formals {
            h.str(&f.path).id(&f.quantity.as_id());
        }
        for (n, q) in &hints {
            h.str(n).id(&q.as_id());
        }
        for (name, range) in &validity {
            h.str(name)
                .id(&range.source)
                .str(&dsl::render_expr(&range.lower))
                .str(&dsl::render_expr(&range.upper));
        }
        let identity = h.finish_hash();
        let body = if let Some(body) = normalized.get(&identity) {
            body.clone()
        } else {
            let body = Arc::new(
                crate::typed_math::Request {
                    definition: declaration,
                    expressions: &[expression],
                    formals: &formals,
                    domains: &BTreeMap::new(),
                    groups: &BTreeMap::new(),
                    providers: &providers,
                    units: &units,
                    literals: &BTreeMap::new(),
                    physical: physical_identity(&registry, &checker),
                    structure: identity,
                    limits: p.body_limits,
                }
                .admit_modeling_outputs(
                    &registry,
                    checker.as_ref(),
                    db.cancel(),
                    &p.functions,
                    &[p.quantities[index]],
                    &hints,
                    &validity,
                )
                .map_err(|cause| pse_math::MathError::Instance {
                    instance: output.row_id(),
                    cause: Box::new(cause),
                })?,
            );
            normalized.insert(identity, body.clone());
            body
        };
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
                    &registry,
                )
            })
            .collect::<std::result::Result<Vec<_>, pse_math::MathError>>()?;
        let (lower, upper) = match output {
            ModelingOutput::Equation {
                sense: EquationSense::Eq,
                ..
            } => (0.0, 0.0),
            ModelingOutput::Equation {
                sense: EquationSense::Le,
                ..
            } => (f64::NEG_INFINITY, 0.0),
            ModelingOutput::Equation {
                sense: EquationSense::Ge,
                ..
            } => (0.0, f64::INFINITY),
            _ => (f64::NEG_INFINITY, f64::INFINITY),
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
            scale: 1.0,
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
                target: Target::Objective,
                scale: objective.as_ref().map_or(1.0, |o| o.sense.sign()),
            });
        }
        if p.objective.is_some_and(|(target, _)| *output == ModelingOutput::Member(target)) {
            contributions.push(Contribution {output: 0, target: Target::Objective, scale: 1.0});
        }
        instances.push(InstanceBinding {
            instance: id,
            body: key,
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
    let case = Arc::new(CaseStructure::new(
        variables,
        parameters,
        instances,
        rows,
        objective,
        CaseLimits::default(),
    )?);
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

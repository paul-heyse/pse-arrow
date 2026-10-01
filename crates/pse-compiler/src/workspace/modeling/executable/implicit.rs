// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use pse_kernels::{AdmittedProvider, Port, ProviderSpec};
pub use pse_math::implicit::SelectionEquivalence;
/// Authored/admitted mathematical meaning, independent of a numerical initial guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplicitMeaning {
    /// Original residual relation; expression use needs selection or uniqueness evidence.
    Relation,
    /// Library-established unique graph, or the separately proven affine-rate operation.
    Unique,
    /// Explicitly restricted mathematical branch.
    Branch,
    /// Semantic anchor and algorithm/settings reference.
    Operational(String),
    /// Authored minimum-score selection among alternative regimes.
    MinimumScore,
}
/// One compiler-issued selector, consumed by evaluation, native realization and export.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitSelection {
    /// Selected scientific operation identity; excludes incidental numerical hints.
    pub identity: ContentHash,
    /// Relation or selected function contract.
    pub meaning: ImplicitMeaning,
    /// Semantic anchors in unknown output order, independent of numerical starts.
    pub anchors: Option<Arc<AdmittedBody>>,
    /// Physical branch/neighborhood predicate encoded as a scalar indicator.
    pub restriction: Option<Arc<AdmittedBody>>,
    /// Supported scalar half-line restriction; true is positive.
    pub sign: Option<bool>,
    /// Whether the recognized half-line excludes its zero endpoint.
    pub strict: bool,
    /// Checked equivalence of residual plus restrictions to the selected graph.
    pub equivalence: SelectionEquivalence,
    /// Justified selector-neighborhood derivative capability.
    pub neighborhood: DerivativeOrder,
}
#[derive(Clone, Debug, PartialEq)]
struct SelectionProjection {
    identity: ContentHash,
    meaning: ImplicitMeaning,
    anchors: Vec<Expr>,
    restriction: Option<Expr>,
    sign: Option<bool>,
    strict: bool,
}
fn sign_restriction(predicate: &dsl::Predicate, unknowns: &[SemanticId]) -> Option<bool> {
    use dsl::{CompareOp, PredicateKind};
    let PredicateKind::Compare { op, lhs, rhs } = &predicate.kind else {
        return None;
    };
    if unknowns.len() != 1 {
        return None;
    }
    let is_unknown = |e: &Expr| matches!(&e.kind, ExprKind::Path(p) if p.segments.len()==1 && p.segments[0].name==symbol_name(unknowns[0]));
    let is_zero =
        |e: &Expr| matches!(&e.kind, ExprKind::Number(n) if n.value==0. && n.unit.is_none());
    match (
        is_unknown(lhs),
        is_zero(rhs),
        is_zero(lhs),
        is_unknown(rhs),
        op,
    ) {
        (true, true, _, _, CompareOp::Gt | CompareOp::Ge)
        | (_, _, true, true, CompareOp::Lt | CompareOp::Le) => Some(true),
        (true, true, _, _, CompareOp::Lt | CompareOp::Le)
        | (_, _, true, true, CompareOp::Gt | CompareOp::Ge) => Some(false),
        _ => None,
    }
}
fn indicator(predicate: dsl::Predicate) -> Result<Expr> {
    Ok(Expr {
        kind: ExprKind::Conditional {
            guard: Box::new(predicate),
            then: Box::new(dsl::parse_expr("1").map_err(|e| CompileError::Missing(e.to_string()))?),
            otherwise: Box::new(
                dsl::parse_expr("0").map_err(|e| CompileError::Missing(e.to_string()))?,
            ),
        },
        span: Span::default(),
    })
}

/// Algorithm selected by source realization or by the compiler's affine rate proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplicitAlgorithm {
    /// Nested solve by the shared native inner solver.
    Native,
    /// Registered accelerator named by the source realization.
    Accelerator(String),
    /// Closed-form solve of rates proven affine by the compiler.
    AffineRates,
}
impl ImplicitAlgorithm {
    fn retained_bytes(&self) -> usize {
        if let Self::Accelerator(id) = self {
            id.capacity()
        } else {
            0
        }
    }
    fn key(&self) -> String {
        match self {
            Self::Native => "nested".into(),
            Self::Accelerator(id) => format!("accelerator:{id}"),
            Self::AffineRates => "affine-rates".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct AssessmentProjection {
    eligibility: Expr,
    criterion: Vec<Expr>,
    quantities: Vec<QuantityTypeId>,
}
/// Selected scaling declaration and its original term range in the nominal program.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitScale {
    /// Equation row the scaling applies to.
    pub row: SemanticId,
    /// Declaration that selected the scaling.
    pub source: DeclarationId,
    /// Declared scaling scheme.
    pub scheme: pse_model::generated::enums::ConstraintScalingScheme,
    /// The row's original terms within the residual's nominal-term body.
    pub terms: std::ops::Range<usize>,
}
#[derive(Clone, Debug, PartialEq)]
struct ResidualProjection {
    id: SemanticId,
    rows: Vec<SemanticId>,
    expressions: Vec<Expr>,
    quantities: Vec<QuantityTypeId>,
    assessment: Option<AssessmentProjection>,
    hints: Vec<(
        SemanticId,
        DeclarationId,
        ModelingHint,
        Expr,
        QuantityTypeId,
    )>,
    terms: Vec<(Expr, QuantityTypeId)>,
    scales: Vec<ImplicitScale>,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Projection {
    selection: SelectionProjection,
    algorithm: ImplicitAlgorithm,
    id: SemanticId,
    unknowns: Vec<SemanticId>,
    formals: Vec<Formal>,
    residuals: Vec<ResidualProjection>,
    local_quantities: BTreeMap<String, QuantityTypeId>,
    spec: ProviderSpec,
    validity: BTreeMap<String, crate::typed_math::Validity>,
}
impl Projection {
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.algorithm.retained_bytes()
            + self
                .selection
                .anchors
                .iter()
                .chain(self.selection.restriction.iter())
                .map(pse_modeling::expression::retained_bytes)
                .sum::<usize>()
            + match &self.selection.meaning {
                ImplicitMeaning::Operational(settings) => settings.capacity(),
                _ => 0,
            }
            + self
                .validity
                .iter()
                .map(|(n, v)| {
                    n.capacity()
                        + 128
                        + pse_modeling::expression::retained_bytes(&v.lower)
                        + pse_modeling::expression::retained_bytes(&v.upper)
                })
                .sum::<usize>()
            + self
                .residuals
                .iter()
                .map(|r| {
                    size_of::<ResidualProjection>()
                        + size_of_val(r.rows.as_slice())
                        + size_of_val(r.quantities.as_slice())
                        + r.expressions
                            .iter()
                            .map(pse_modeling::expression::retained_bytes)
                            .sum::<usize>()
                        + r.assessment.as_ref().map_or(0, |a| {
                            pse_modeling::expression::retained_bytes(&a.eligibility)
                                + a.criterion
                                    .iter()
                                    .map(pse_modeling::expression::retained_bytes)
                                    .sum::<usize>()
                        })
                        + size_of_val(r.scales.as_slice())
                        + r.terms
                            .iter()
                            .map(|(e, _)| {
                                size_of::<(Expr, QuantityTypeId)>()
                                    + pse_modeling::expression::retained_bytes(e)
                            })
                            .sum::<usize>()
                        + r.hints
                            .iter()
                            .map(|h| {
                                size_of_val(h) + pse_modeling::expression::retained_bytes(&h.3)
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self
                .formals
                .iter()
                .map(|f| size_of::<Formal>() + f.path.capacity())
                .sum::<usize>()
            + size_of_val(self.unknowns.as_slice())
            + self
                .local_quantities
                .keys()
                .map(|n| n.capacity() + 128)
                .sum::<usize>()
    }
}
/// The criterion and eligibility are separately demanded: an ineligible branch has no score.
#[derive(Clone, Debug, PartialEq)]
pub struct RegimeAssessment {
    pub eligibility: Arc<AdmittedBody>,
    pub criterion: Arc<AdmittedBody>,
}
/// A root problem, with independent equation identities and optional regime assessment.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedResidual {
    pub id: SemanticId,
    pub body: Arc<AdmittedBody>,
    pub rows: Vec<SemanticId>,
    pub assessment: Option<RegimeAssessment>,
    /// Selected common/local hints in the residual's lexical scope and formal order.
    pub hints: Option<Arc<AdmittedBody>>,
    pub terms: Option<Arc<AdmittedBody>>,
    pub scales: Vec<ImplicitScale>,
    pub hint_targets: Vec<(SemanticId, DeclarationId, ModelingHint)>,
}
/// Checked implicit systems over a single ordered unknown set.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedImplicit {
    /// Compiler-owned mathematical selection shared across realizations.
    pub selection: ImplicitSelection,
    /// Algorithm that solves the system.
    pub algorithm: ImplicitAlgorithm,
    /// Provider descriptor through which the solved unknowns are called.
    pub descriptor: AdmittedProvider,
    /// Unknowns in the provider's output order.
    pub unknowns: Vec<SemanticId>,
    /// Alternative root problems over the same unknowns.
    pub residuals: Vec<AdmittedResidual>,
}
impl AdmittedImplicit {
    fn residual_requirements(
        &self,
        residual: &AdmittedResidual,
        requested_output: DerivativeOrder,
        inner_minimum: DerivativeOrder,
    ) -> Result<pse_kernels::DerivativeRequirements> {
        let available = residual
            .body
            .math
            .available_order_for(&(0..residual.body.math.input_count()).collect::<Vec<_>>());
        pse_kernels::DerivativeRequirements::new(
            available,
            available,
            self.selection.neighborhood,
            inner_minimum,
            requested_output,
        )
        .map_err(|e| CompileError::Missing(e.to_string()))
    }
    /// Resolve the selected algorithms' minimum and the shared residual/output capability.
    /// Called under the library allocation owner before dependencies receive their demands.
    pub fn requirements(
        &self,
        requested_output: DerivativeOrder,
        native_minimum: DerivativeOrder,
        accelerators: &pse_math::implicit::accelerators::Accelerators,
        cancel: &Arc<AtomicBool>,
        limits: EvaluationLimits,
    ) -> Result<pse_kernels::DerivativeRequirements> {
        let mut available = None;
        let mut minimum = DerivativeOrder::Value;
        for residual in &self.residuals {
            let order = residual
                .body
                .math
                .available_order_for(&(0..residual.body.math.input_count()).collect::<Vec<_>>());
            available = Some(available.map_or(order, |a: DerivativeOrder| a.min(order)));
            let inner = match &self.algorithm {
                ImplicitAlgorithm::Native => native_minimum,
                ImplicitAlgorithm::AffineRates => {
                    use pse_math::implicit::InnerSolver;
                    pse_math::implicit::Affine::new(&residual.body.math, self.unknowns.len())?
                        .minimum_order()
                }
                ImplicitAlgorithm::Accelerator(id) => accelerators
                    .admit(id, &residual.body.math, self.unknowns.len(), limits, cancel)?
                    .minimum_order(),
            };
            minimum = minimum.max(inner);
        }
        let available =
            available.ok_or_else(|| CompileError::Missing("empty implicit residuals".into()))?;
        pse_kernels::DerivativeRequirements::new(
            available,
            available,
            self.selection.neighborhood,
            minimum,
            requested_output,
        )
        .map_err(|e| CompileError::Missing(e.to_string()))
    }
    /// Actual dependency demands for residual, numerical and selector programs.
    pub fn provider_demands(
        &self,
        requirements: pse_kernels::DerivativeRequirements,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, DerivativeOrder>> {
        let mut demands = BTreeMap::new();
        let mut add = |body: &Arc<AdmittedBody>, order| -> Result<()> {
            for (key, required) in body.math.provider_demands(order)? {
                demands
                    .entry(key)
                    .and_modify(|o: &mut DerivativeOrder| *o = (*o).max(required))
                    .or_insert(required);
            }
            Ok(())
        };
        let local = if requirements.requested_output > DerivativeOrder::Value {
            DerivativeOrder::First
        } else {
            DerivativeOrder::Value
        };
        for residual in &self.residuals {
            add(&residual.body, requirements.residual_compilation)?;
            for body in residual.hints.iter().chain(&residual.terms) {
                add(body, DerivativeOrder::Value)?;
            }
            if let Some(assessment) = &residual.assessment {
                add(&assessment.eligibility, local)?;
                add(&assessment.criterion, local)?;
            }
        }
        if let Some(body) = &self.selection.anchors {
            add(body, DerivativeOrder::Value)?;
        }
        if let Some(body) = &self.selection.restriction {
            add(body, local)?;
        }
        Ok(demands)
    }
    /// Every admitted body: residuals, hints, nominal terms and regime assessments.
    pub fn bodies(&self) -> impl Iterator<Item = &Arc<AdmittedBody>> {
        self.residuals
            .iter()
            .flat_map(|r| {
                std::iter::once(&r.body)
                    .chain(r.hints.iter())
                    .chain(r.terms.iter())
                    .chain(
                        r.assessment
                            .iter()
                            .flat_map(|a| [&a.eligibility, &a.criterion]),
                    )
            })
            .chain(self.selection.anchors.iter())
            .chain(self.selection.restriction.iter())
    }
    /// Approximate heap bytes retained by the system, for cache accounting.
    pub fn retained_bytes(&self) -> usize {
        size_of_val(self.unknowns.as_slice())
            + self.algorithm.retained_bytes()
            + match &self.selection.meaning {
                ImplicitMeaning::Operational(settings) => settings.capacity(),
                _ => 0,
            }
            + self
                .residuals
                .iter()
                .map(|r| {
                    size_of_val(r.rows.as_slice())
                        + size_of_val(r.hint_targets.as_slice())
                        + size_of_val(r.scales.as_slice())
                        + size_of::<AdmittedResidual>()
                })
                .sum::<usize>()
            + self
                .bodies()
                .map(|b| {
                    b.math.retained_bytes()
                        + size_of_val(b.quantities.as_slice())
                        + size_of_val(b.occurrences.as_slice())
                })
                .sum::<usize>()
    }
    /// Every branch receives its own resolved physical hints. The selector has one total deadline.
    pub fn factory(
        &self,
        mut configurations: BTreeMap<SemanticId, pse_math::implicit::Configuration>,
        solver: Arc<dyn pse_math::implicit::InnerSolver>,
        requested_output: DerivativeOrder,
        native_minimum: DerivativeOrder,
        accelerators: &pse_math::implicit::accelerators::Accelerators,
        cancel: Arc<AtomicBool>,
        limits: EvaluationLimits,
    ) -> Result<pse_math::implicit::ImplicitFactory> {
        use pse_math::implicit::{Factory, ImplicitFactory, RegimeFactory, RegimeFactoryBranch};
        if self.algorithm == ImplicitAlgorithm::Native && native_minimum != solver.minimum_order() {
            return Err(CompileError::Missing(
                "native implicit derivative minimum disagrees with selected adapter".into(),
            ));
        }
        if configurations.len() != self.residuals.len() || self.residuals.is_empty() {
            return Err(CompileError::Missing(
                "implicit branch configuration extent".into(),
            ));
        }
        let mut branches = Vec::new();
        let mut total_time = None;
        for residual in &self.residuals {
            let configuration = configurations
                .remove(&residual.id)
                .ok_or_else(|| CompileError::Missing("implicit branch configuration".into()))?;
            let unknowns = match &configuration {
                pse_math::implicit::Configuration::Fixed(unknowns, _) => unknowns.clone(),
                pse_math::implicit::Configuration::Hints(_) => self
                    .unknowns
                    .iter()
                    .map(|id| pse_math::implicit::Unknown {
                        id: *id,
                        lower: f64::NEG_INFINITY,
                        upper: f64::INFINITY,
                    })
                    .collect(),
            };
            if unknowns
                .iter()
                .map(|u| u.id)
                .ne(self.unknowns.iter().copied())
            {
                return Err(CompileError::Missing(
                    "implicit unknown binding order".into(),
                ));
            }
            total_time = Some(
                total_time.map_or(configuration.time_limit(), |t: std::time::Duration| {
                    t.min(configuration.time_limit())
                }),
            );
            let compile = |body: &AdmittedBody, order| -> Result<Arc<CompiledBody>> {
                Ok(Arc::new(body.math.compile(
                    &(0..body.quantities.len()).collect::<Vec<_>>(),
                    &(0..body.math.input_count()).collect::<Vec<_>>(),
                    order,
                    Optimization::default(),
                    limits,
                    &cancel,
                )?))
            };
            let branch_solver: Arc<dyn pse_math::implicit::InnerSolver> = match &self.algorithm {
                ImplicitAlgorithm::Accelerator(id) => accelerators.admit(
                    id,
                    &residual.body.math,
                    self.unknowns.len(),
                    limits,
                    &cancel,
                )?,
                ImplicitAlgorithm::AffineRates => Arc::new(pse_math::implicit::Affine::new(
                    &residual.body.math,
                    self.unknowns.len(),
                )?),
                ImplicitAlgorithm::Native => solver.clone(),
            };
            let requirements = self.residual_requirements(
                residual,
                requested_output,
                branch_solver.minimum_order(),
            )?;
            let mut spec = self
                .descriptor
                .restrict_order(requirements.requested_output)
                .map_err(|e| CompileError::Missing(e.to_string()))?
                .spec()
                .clone();
            spec.id = residual.id;
            let restriction = self
                .selection
                .restriction
                .as_ref()
                .map(|body| {
                    body.math
                        .compile_branch_local(
                            &(0..body.math.output_count()).collect::<Vec<_>>(),
                            &(0..body.math.input_count()).collect::<Vec<_>>(),
                            if requested_output > DerivativeOrder::Value {
                                DerivativeOrder::First
                            } else {
                                DerivativeOrder::Value
                            },
                            Optimization::default(),
                            limits,
                            &cancel,
                        )
                        .map(Arc::new)
                        .map_err(CompileError::from)
                })
                .transpose()?;
            let factory = Factory {
                selection: pse_math::implicit::Selection {
                    anchor: self
                        .selection
                        .anchors
                        .as_ref()
                        .map(|b| compile(b, DerivativeOrder::Value))
                        .transpose()?,
                    settings: match &self.selection.meaning {
                        ImplicitMeaning::Operational(s) => Some(s.clone()),
                        _ => None,
                    },
                    restriction,
                    sign: self.selection.sign,
                },
                requirements,
                spec,
                body: compile(&residual.body, requirements.residual_compilation)?,
                unknowns,
                rows: residual.rows.clone(),
                hints: if matches!(configuration, pse_math::implicit::Configuration::Hints(_)) {
                    residual
                        .hints
                        .as_ref()
                        .map(|body| compile(body, DerivativeOrder::Value))
                        .transpose()?
                } else {
                    None
                },
                terms: if matches!(configuration, pse_math::implicit::Configuration::Hints(_)) {
                    residual
                        .terms
                        .as_ref()
                        .map(|body| compile(body, DerivativeOrder::Value))
                        .transpose()?
                } else {
                    None
                },
                configuration,
                solver: branch_solver,
                cancel: cancel.clone(),
                max_entries: limits.derivative_components,
                providers: BTreeMap::new(),
            };
            let local = |body: &Arc<AdmittedBody>| -> Result<Arc<CompiledBody>> {
                Ok(Arc::new(body.math.compile_branch_local(
                    &(0..body.math.output_count()).collect::<Vec<_>>(),
                    &(0..body.math.input_count()).collect::<Vec<_>>(),
                    if requested_output > DerivativeOrder::Value {
                        DerivativeOrder::First
                    } else {
                        DerivativeOrder::Value
                    },
                    Optimization::default(),
                    limits,
                    &cancel,
                )?))
            };
            match &residual.assessment {
                Some(a) => branches.push(RegimeFactoryBranch {
                    residual: factory,
                    eligibility: local(&a.eligibility)?,
                    criterion: local(&a.criterion)?,
                }),
                None if self.residuals.len() == 1 => return Ok(ImplicitFactory::Root(factory)),
                None => {
                    return Err(CompileError::Missing(
                        "implicit selection requires every assessment".into(),
                    ));
                }
            }
        }
        Ok(ImplicitFactory::Regimes(RegimeFactory {
            spec: self
                .descriptor
                .restrict_order(requested_output)
                .map_err(|e| CompileError::Missing(e.to_string()))?
                .spec()
                .clone(),
            maximum_regimes: self.residuals.len(),
            alternatives: branches,
            time_limit: total_time
                .ok_or_else(|| CompileError::Missing("empty implicit selector".into()))?,
            cancel,
        }))
    }
}
pub(super) fn project(
    model: &SpecializedModel,
    registry: &QuantityRegistry,
    p: &mut super::Projection,
    bindings: &mut Vec<(String, Expr)>,
) -> Result<()> {
    use pse_model::generated::enums::ModelingDeclarationKind as Kind;
    use pse_modeling::specialize::Realization as Policy;
    let mut hidden = BTreeSet::new();
    let mut unknown_ids = BTreeSet::new();
    let mut additions = vec![];
    let ports = p
        .inputs
        .iter()
        .zip(&p.formals)
        .map(|(id, f)| {
            Ok((
                *id,
                Port {
                    id: *id,
                    quantity: f.quantity,
                    unit: registry
                        .quantity_type(f.quantity)
                        .map_err(MathError::from)?
                        .canonical_unit,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    // Descendants are projected before their owners so a parent residual can call
    // a child stage with its current unknowns as inputs on the same worker.
    let mut hierarchy = DiGraph::<InstanceId, ()>::new();
    let nodes = model
        .instances
        .keys()
        .map(|id| (*id, hierarchy.add_node(*id)))
        .collect::<BTreeMap<_, _>>();
    for (id, item) in &model.instances {
        if let Some(parent) = item.parent {
            let owner = nodes
                .get(&parent)
                .ok_or_else(|| CompileError::Missing("implicit parent instance".into()))?;
            hierarchy.add_edge(nodes[id], *owner, ());
        }
    }
    let order = toposort(&hierarchy, None)
        .map_err(|_| CompileError::Missing("cyclic instance hierarchy".into()))?;
    let mut stages = order
        .into_iter()
        .filter_map(|node| {
            // A stage is identified by its implicit block's instance, or by the generated
            // rate system's own identity below.
            let id = hierarchy[node];
            match model.implicit.get(&id) {
                Some(Policy::Nested) => Some((id.as_id(), ImplicitAlgorithm::Native)),
                Some(Policy::Accelerated(reference)) => Some((
                    id.as_id(),
                    ImplicitAlgorithm::Accelerator(reference.clone()),
                )),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    if !model.derivatives.is_empty() {
        let axis = model
            .integrated
            .keys()
            .next()
            .ok_or_else(|| CompileError::Missing("rate system axis".into()))?;
        stages.push((
            pse_ids::named_id(*axis, "affine-rate-system"),
            ImplicitAlgorithm::AffineRates,
        ));
    }
    for (stage, algorithm) in &stages {
        let generated = *algorithm == ImplicitAlgorithm::AffineRates;
        let instance = &InstanceId::from_id(*stage);
        let unknowns = if generated {
            model
                .derivatives
                .values()
                .map(|d| d.rate)
                .collect::<Vec<_>>()
        } else {
            model
                .symbols
                .values()
                .filter(|s| {
                    s.lineage.instance == *instance
                        && s.role == Kind::Variable
                        && s.expression.is_none()
                })
                .map(|s| s.id)
                .collect::<Vec<_>>()
        };
        let selection = model.regimes.get(instance);
        let mut operation = SelectionProjection {
            identity: ContentHash::from_bytes([0; 32]),
            meaning: if selection.is_some() {
                ImplicitMeaning::MinimumScore
            } else if generated {
                ImplicitMeaning::Unique
            } else {
                ImplicitMeaning::Relation
            },
            anchors: vec![],
            restriction: None,
            sign: None,
            strict: false,
        };
        if let Some(authored) = model.root_selections.get(instance) {
            use pse_modeling::specialize::RootSelection;
            let predicate = match authored {
                RootSelection::Branch(p) => {
                    operation.meaning = ImplicitMeaning::Branch;
                    Some(p)
                }
                RootSelection::Operational {
                    anchors,
                    settings,
                    neighborhood,
                } => {
                    operation.meaning = ImplicitMeaning::Operational(settings.clone());
                    if anchors.len() != unknowns.len()
                        || unknowns.iter().any(|u| !anchors.contains_key(u))
                    {
                        return Err(CompileError::Missing(
                            "operational selector requires one anchor per unknown".into(),
                        ));
                    }
                    operation.anchors = unknowns.iter().map(|u| anchors[u].clone()).collect();
                    neighborhood.as_ref()
                }
            };
            if let Some(predicate) = predicate {
                operation.sign = sign_restriction(predicate, &unknowns);
                operation.strict = matches!(
                    predicate.kind,
                    dsl::PredicateKind::Compare {
                        op: dsl::CompareOp::Gt | dsl::CompareOp::Lt,
                        ..
                    }
                );
                operation.restriction = Some(indicator(predicate.clone())?);
            }
        }
        let branches = if generated {
            let rows = p
                .outputs
                .iter()
                .find_map(|o| match o {
                    ModelingOutput::DynamicRate { equations, .. } => Some(equations.clone()),
                    _ => None,
                })
                .ok_or_else(|| CompileError::Missing("rate system equations".into()))?;
            if rows.iter().any(|r| hidden.contains(r)) {
                return Err(CompileError::Missing(
                    "time derivatives cannot be owned by a nested algebraic block".into(),
                ));
            }
            vec![(*stage, rows, None)]
        } else if let Some(selection) = selection {
            selection
                .alternatives
                .iter()
                .map(|r| {
                    (
                        r.id,
                        r.equations.iter().map(|r| r.id).collect::<Vec<_>>(),
                        Some(r),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            vec![(
                *stage,
                model
                    .equations
                    .iter()
                    .filter(|r| r.lineage.instance == *instance)
                    .map(|r| r.id)
                    .collect(),
                None,
            )]
        };
        let mut residuals = Vec::new();
        let branch_declarations = model
            .regimes
            .values()
            .flat_map(|r| {
                r.alternatives
                    .iter()
                    .flat_map(|a| a.annotations.iter().map(|v| v.lineage.declaration))
            })
            .collect::<BTreeSet<_>>();
        for (id, rows, alternative) in branches {
            if unknowns.is_empty() || rows.len() != unknowns.len() {
                return Err(CompileError::Missing(
                    "nested implicit block must be a nonempty square residual system".into(),
                ));
            }
            let indices=p.outputs.iter().enumerate().filter_map(|(i,o)|matches!(o,ModelingOutput::Equation{id,sense:EquationSense::Eq} if rows.contains(id)).then_some(i)).collect::<Vec<_>>();
            if indices.len() != rows.len() {
                return Err(CompileError::Missing(
                    "implicit block requires equality residuals".into(),
                ));
            }
            let assessment = if let Some((alternative, selection)) = alternative.zip(selection) {
                let delta = pse_quantity::scheme::Scheme::Delta(Box::new(
                    pse_quantity::scheme::Scheme::Concrete(selection.quantity),
                ))
                .resolve(registry, &BTreeMap::new())
                .map_err(|e| CompileError::Missing(e.to_string()))?;
                Some(AssessmentProjection {
                    eligibility: Expr {
                        kind: ExprKind::Conditional {
                            guard: Box::new(alternative.eligibility.clone()),
                            then: Box::new(
                                dsl::parse_expr("1")
                                    .map_err(|e| CompileError::Missing(e.to_string()))?,
                            ),
                            otherwise: Box::new(
                                dsl::parse_expr("0")
                                    .map_err(|e| CompileError::Missing(e.to_string()))?,
                            ),
                        },
                        span: Span::default(),
                    },
                    criterion: vec![selection.criterion.clone(), selection.tolerance.clone()],
                    quantities: vec![selection.quantity, delta],
                })
            } else {
                None
            };
            let local_declarations = alternative
                .into_iter()
                .flat_map(|a| a.annotations.iter().map(|v| v.lineage.declaration))
                .collect::<BTreeSet<_>>();
            let mut hints = BTreeMap::new();
            for local in [false, true] {
                let mut seen = BTreeSet::new();
                for (i, output) in p.outputs.iter().enumerate() {
                    if let ModelingOutput::Hint {
                        target,
                        declaration,
                        kind,
                    } = output
                        && (unknowns.contains(target) || rows.contains(target))
                        && matches!(
                            kind,
                            ModelingHint::Start
                                | ModelingHint::Lower
                                | ModelingHint::Upper
                                | ModelingHint::Nominal
                        )
                        && if local {
                            local_declarations.contains(declaration)
                        } else {
                            !branch_declarations.contains(declaration)
                        }
                    {
                        if !seen.insert((*target, *kind)) {
                            return Err(CompileError::Missing(
                                "competing implicit numerical hints in one regime".into(),
                            ));
                        }
                        hints.insert(
                            (*target, *kind),
                            (
                                *target,
                                *declaration,
                                *kind,
                                p.expressions[i].clone(),
                                p.quantities[i],
                            ),
                        );
                    }
                }
            }
            let mut selected_scales = BTreeMap::new();
            for annotations in [
                model.annotations.as_slice(),
                alternative.map_or(&[][..], |a| a.annotations.as_slice()),
            ] {
                let mut seen = BTreeSet::new();
                for annotation in annotations {
                    if rows.contains(&annotation.target)
                        && let pse_modeling::annotation::AnnotationValue::Scale(scheme) =
                            annotation.value
                    {
                        if !seen.insert(annotation.target) {
                            return Err(CompileError::Missing(
                                "competing implicit scaling schemes in one regime".into(),
                            ));
                        }
                        selected_scales
                            .insert(annotation.target, (scheme, annotation.lineage.declaration));
                    }
                }
            }
            let mut terms = Vec::new();
            let mut scales = Vec::new();
            for (row, (scheme, source)) in selected_scales {
                let begin = terms.len();
                for (i, output) in p.outputs.iter().enumerate() {
                    if matches!(output,ModelingOutput::Term{equation,..} if *equation==row) {
                        terms.push((p.expressions[i].clone(), p.quantities[i]));
                    }
                }
                if begin == terms.len() {
                    return Err(CompileError::Missing(
                        "implicit original scaling terms are absent".into(),
                    ));
                }
                scales.push(ImplicitScale {
                    row,
                    source,
                    scheme,
                    terms: begin..terms.len(),
                });
            }
            residuals.push(ResidualProjection {
                id,
                rows: indices.iter().map(|i| p.outputs[*i].row_id()).collect(),
                expressions: indices.iter().map(|i| p.expressions[*i].clone()).collect(),
                quantities: indices.iter().map(|i| p.quantities[*i]).collect(),
                assessment,
                hints: hints.into_values().collect(),
                terms,
                scales,
            });
            hidden.extend(rows);
        }
        let descendants = p
            .implicit
            .iter()
            .filter(|child| {
                generated
                    || petgraph::algo::has_path_connecting(
                        &hierarchy,
                        nodes[&InstanceId::from_id(child.id)],
                        nodes[instance],
                        None,
                    )
            })
            .flat_map(|child| child.unknowns.iter().map(|id| symbol_name(*id)))
            .collect::<BTreeSet<_>>();
        let mut scope_bindings = bindings.clone();
        scope_bindings.extend(
            additions
                .iter()
                .filter(|(name, _)| descendants.contains(name))
                .cloned(),
        );
        let scope_bindings = ordered_bindings(scope_bindings)?;
        // Configuration cannot depend on the unknown it initializes, including through
        // derived locals, nested providers, or validity guards. Enclosing unknowns are inputs.
        for expression in residuals
            .iter()
            .flat_map(|r| r.hints.iter().map(|h| &h.3))
            .chain(&operation.anchors)
        {
            let mut dependencies = expression
                .free_paths()
                .into_iter()
                .filter(|&p| p.segments.len() == 1)
                .map(|p| p.segments[0].name.clone())
                .collect::<BTreeSet<_>>();
            loop {
                let before = dependencies.len();
                for (name, expression) in &scope_bindings {
                    if dependencies.contains(name) {
                        dependencies.extend(
                            expression
                                .free_paths()
                                .into_iter()
                                .filter(|&p| p.segments.len() == 1)
                                .map(|p| p.segments[0].name.clone()),
                        );
                    }
                }
                for (name, guard) in &p.validity {
                    if dependencies.contains(name) {
                        dependencies.extend(
                            guard
                                .lower
                                .free_paths()
                                .into_iter()
                                .chain(guard.upper.free_paths())
                                .filter(|&p| p.segments.len() == 1)
                                .map(|p| p.segments[0].name.clone()),
                        );
                    }
                }
                if before == dependencies.len() {
                    break;
                }
            }
            if unknowns
                .iter()
                .any(|id| dependencies.contains(&symbol_name(*id)))
            {
                return Err(CompileError::Missing(
                    "implicit numerical hint depends on its own unknowns".into(),
                ));
            }
        }
        let local_names = scope_bindings
            .iter()
            .map(|(n, _)| n.clone())
            .collect::<BTreeSet<_>>();
        let mut needed = BTreeSet::new();
        for expression in residuals
            .iter()
            .flat_map(|r| {
                r.expressions
                    .iter()
                    .chain(r.hints.iter().map(|h| &h.3))
                    .chain(r.terms.iter().map(|(e, _)| e))
                    .chain(
                        r.assessment
                            .iter()
                            .flat_map(|a| std::iter::once(&a.eligibility).chain(&a.criterion)),
                    )
            })
            .chain(&operation.anchors)
            .chain(operation.restriction.iter())
        {
            for path in expression.paths() {
                if path.segments.len() == 1 {
                    needed.insert(path.segments[0].name.clone());
                }
            }
        }
        loop {
            let old = needed.len();
            for (name, guard) in &p.validity {
                if needed.contains(name) {
                    for path in guard.lower.paths().into_iter().chain(guard.upper.paths()) {
                        if path.segments.len() == 1 {
                            needed.insert(path.segments[0].name.clone());
                        }
                    }
                }
            }
            for (name, e) in &scope_bindings {
                if needed.contains(name) {
                    for path in e.paths() {
                        if path.segments.len() == 1 {
                            needed.insert(path.segments[0].name.clone());
                        }
                    }
                }
            }
            if old == needed.len() {
                break;
            }
        }
        let inputs = p
            .inputs
            .iter()
            .copied()
            .filter(|id| {
                !unknowns.contains(id)
                    && !local_names.contains(&symbol_name(*id))
                    && needed.contains(&symbol_name(*id))
            })
            .collect::<Vec<_>>();
        let local_bindings = scope_bindings
            .iter()
            .filter(|(n, _)| needed.contains(n))
            .cloned()
            .collect::<Vec<_>>();
        let wrap = |expression: &mut Expr| {
            if !local_bindings.is_empty() {
                *expression = Expr {
                    kind: ExprKind::Let {
                        bindings: local_bindings.clone(),
                        body: Box::new(expression.clone()),
                    },
                    span: Span::default(),
                };
            }
            expression.strip_spans();
        };
        for residual in &mut residuals {
            for hint in &mut residual.hints {
                wrap(&mut hint.3);
            }
            for (e, _) in &mut residual.terms {
                wrap(e);
            }
            for e in &mut residual.expressions {
                wrap(e);
            }
            if let Some(a) = &mut residual.assessment {
                wrap(&mut a.eligibility);
                for e in &mut a.criterion {
                    wrap(e);
                }
            }
        }
        for anchor in &mut operation.anchors {
            wrap(anchor);
        }
        if let Some(restriction) = &mut operation.restriction {
            wrap(restriction);
        }
        let formals = unknowns
            .iter()
            .chain(&inputs)
            .map(|id| Formal {
                path: symbol_name(*id),
                quantity: ports[id].quantity,
            })
            .collect::<Vec<_>>();
        let mut h = FramedHasher::new(pse_ids::Frame::ModelingImplicitResidualV5);
        let mut semantic = FramedHasher::new(pse_ids::Frame::ModelingImplicitOperationV1);
        h.id(stage).str(&algorithm.key());
        semantic.id(stage);
        for hasher in [&mut h, &mut semantic] {
            hasher.str(match &operation.meaning {
                ImplicitMeaning::Relation => "relation",
                ImplicitMeaning::Unique => "unique",
                ImplicitMeaning::Branch => "branch",
                ImplicitMeaning::MinimumScore => "minimum-score",
                ImplicitMeaning::Operational(s) => s,
            });
            hasher.u64(operation.anchors.len() as u64);
            for anchor in &operation.anchors {
                hasher.str(&dsl::render_expr(anchor));
            }
            hasher.bool(operation.restriction.is_some());
            if let Some(restriction) = &operation.restriction {
                hasher.str(&dsl::render_expr(restriction));
            }
        }
        for residual in &residuals {
            h.id(&residual.id);
            semantic
                .id(&residual.id)
                .u64(residual.expressions.len() as u64);
            for scale in &residual.scales {
                h.id(&scale.row)
                    .id(&scale.source.as_id())
                    .str(scale.scheme.as_str())
                    .u64(scale.terms.start as u64)
                    .u64(scale.terms.end as u64);
            }
            for (expression, quantity) in &residual.terms {
                h.id(&quantity.as_id()).str(&dsl::render_expr(expression));
            }
            for (target, source, kind, expression, quantity) in &residual.hints {
                h.id(target)
                    .id(&source.as_id())
                    .u64(kind.code())
                    .id(&quantity.as_id())
                    .str(&dsl::render_expr(expression));
            }
            for e in &residual.expressions {
                h.str(&dsl::render_expr(e));
                semantic.str(&dsl::render_expr(e));
            }
            for quantity in &residual.quantities {
                semantic.id(&quantity.as_id());
            }
            if let Some(a) = &residual.assessment {
                h.str(&dsl::render_expr(&a.eligibility));
                semantic.str(&dsl::render_expr(&a.eligibility));
                for e in &a.criterion {
                    h.str(&dsl::render_expr(e));
                    semantic.str(&dsl::render_expr(e));
                }
            }
        }
        // Function bodies and guards affect the executable meaning, including assessor-only calls.
        for h in [&mut h, &mut semantic] {
            for function in p.functions.values() {
                h.id(&function.id.as_id());
                // Plan 23 H5: each validity predicate with the sets and arguments it reads.
                let reads = |h: &mut FramedHasher, reads: &pse_modeling::envelope::Reads| {
                    h.u64(reads.sets.len() as u64);
                    for set in &reads.sets {
                        h.id(set);
                    }
                    h.u64(reads.variables.len() as u64);
                    for variable in &reads.variables {
                        h.u64(u64::from(*variable));
                    }
                };
                if let Some(validity) = &function.validity {
                    h.str(&dsl::render_predicate(validity));
                    reads(h, &function.validity_reads);
                }
                // ADR-0123 Outcome 4: the data-layer guards, their envelopes and policies.
                h.u64(function.envelopes.len() as u64);
                for guard in &function.envelopes {
                    h.id(&guard.envelope.owner.as_id())
                        .str(&dsl::render_predicate(&guard.predicate));
                    reads(h, &guard.reads);
                }
                if let Some(external) = &function.external {
                    h.hash(&external.revision)
                        .hash(&external.data)
                        .str(&external.implementation);
                }
                if let Some(body) = &function.body {
                    h.str(&dsl::render_expr(body));
                }
            }
            for (name, guard) in &p.validity {
                h.str(name)
                    .str(&dsl::render_expr(&guard.lower))
                    .str(&dsl::render_expr(&guard.upper));
            }
            for f in &formals {
                h.id(&f.quantity.as_id());
            }
        }
        operation.identity = semantic.finish_hash();
        let revision = h.finish_hash();
        let spec = ProviderSpec {
            shapes: pse_kernels::ProviderShapes::default(),
            derivative_source: pse_kernels::DerivativeSource::Implicit,
            id: *stage,
            revision,
            data: revision,

            inputs: inputs.iter().map(|id| ports[id].clone()).collect(),
            outputs: unknowns.iter().map(|id| ports[id].clone()).collect(),
            // Body admission issues actual capabilities before this descriptor is consumed.
            derivatives: DerivativeOrder::Value,
            smoothness: DerivativeOrder::Value,
        };
        let input_expressions = inputs
            .iter()
            .map(|id| {
                dsl::parse_expr(&symbol_name(*id)).map_err(|e| CompileError::Missing(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        for (i, id) in unknowns.iter().enumerate() {
            additions.push((
                symbol_name(*id),
                Expr {
                    kind: ExprKind::Kernel {
                        name: call_name(*stage, i),
                        args: input_expressions.clone(),
                    },
                    span: Span::default(),
                },
            ));
            p.local_quantities
                .insert(symbol_name(*id), ports[id].quantity);
            if !p
                .outputs
                .iter()
                .any(|o| matches!(o,ModelingOutput::Member(existing) if existing==id))
            {
                p.outputs.push(ModelingOutput::Member(*id));
                p.expressions.push(
                    dsl::parse_expr(&symbol_name(*id))
                        .map_err(|e| CompileError::Missing(e.to_string()))?,
                );
                p.quantities.push(ports[id].quantity);
                p.declarations.push(model.symbols[id].lineage.declaration);
            }
        }
        p.implicit.push(Projection {
            selection: operation,
            algorithm: algorithm.clone(),
            id: *stage,
            unknowns: unknowns.clone(),
            formals,
            residuals,
            local_quantities: p.local_quantities.clone(),
            spec,
            validity: p.validity.clone(),
        });
        unknown_ids.extend(unknowns);
    }
    let mut keep = Vec::new();
    for (i, output) in p.outputs.iter().enumerate() {
        if !matches!(output,ModelingOutput::Equation{id,..}|ModelingOutput::Term{equation:id,..} if hidden.contains(id))
        {
            keep.push(i);
        }
    }
    p.outputs = keep.iter().map(|i| p.outputs[*i].clone()).collect();
    p.expressions = keep.iter().map(|i| p.expressions[*i].clone()).collect();
    p.quantities = keep.iter().map(|i| p.quantities[*i]).collect();
    p.declarations = keep.iter().map(|i| p.declarations[*i]).collect();
    let keep = p
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(i, id)| (!unknown_ids.contains(id)).then_some(i))
        .collect::<Vec<_>>();
    p.inputs = keep.iter().map(|i| p.inputs[*i]).collect();
    p.formals = keep.iter().map(|i| p.formals[*i].clone()).collect();
    // A nested root cannot decide a discrete unknown (ADR-0103 item 6).
    model.require_fixed_discrete(
        unknown_ids.iter().copied(),
        pse_modeling::DomainAnalysis::Root,
    )?;
    p.free.retain(|id, _| !unknown_ids.contains(id));
    additions.append(bindings);
    *bindings = ordered_bindings(additions)?;
    Ok(())
}
/// Ordinary members and nested outputs use the same dependency ordering.
fn ordered_bindings(bindings: Vec<(String, Expr)>) -> Result<Vec<(String, Expr)>> {
    let mut graph = DiGraph::<usize, ()>::new();
    let nodes = (0..bindings.len())
        .map(|i| graph.add_node(i))
        .collect::<Vec<_>>();
    let names = bindings
        .iter()
        .enumerate()
        .map(|(i, (name, _))| (name.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    for (i, (_, expression)) in bindings.iter().enumerate() {
        for path in expression.free_paths() {
            if path.segments.len() == 1
                && let Some(j) = names.get(path.segments[0].name.as_str())
            {
                graph.add_edge(nodes[*j], nodes[i], ());
            }
        }
    }
    let order = toposort(&graph, None)
        .map_err(|_| CompileError::Missing("cyclic implicit/output dependency".into()))?;
    Ok(order
        .into_iter()
        .map(|node| bindings[graph[node]].clone())
        .collect())
}
impl AdmittedModeling {
    /// Provider demand of selected original observations before implicit dependency propagation.
    pub fn provider_demands_for(
        &self,
        rows: Option<&BTreeSet<SemanticId>>,
        order: DerivativeOrder,
    ) -> Result<BTreeMap<pse_kernels::ProviderKey, DerivativeOrder>> {
        let mut demands = BTreeMap::new();
        for instance in self
            .case
            .instances()
            .iter()
            .filter(|i| rows.is_none_or(|r| r.contains(&i.instance)))
        {
            let body = self
                .bodies
                .get(&instance.body)
                .ok_or_else(|| CompileError::Missing("observation body".into()))?;
            for (key, required) in body.math.provider_demands(order)? {
                demands
                    .entry(key)
                    .and_modify(|o: &mut DerivativeOrder| *o = (*o).max(required))
                    .or_insert(required);
            }
        }
        Ok(demands)
    }
    /// Child residual providers before their consumers, using admitted library dependencies.
    pub fn implicit_order(&self) -> Result<Vec<Arc<AdmittedImplicit>>> {
        self.implicit_order_for(None)
    }
    /// Admit only providers reachable from selected observations and their hint/residual dependencies.
    pub fn implicit_order_for(
        &self,
        rows: Option<&BTreeSet<SemanticId>>,
    ) -> Result<Vec<Arc<AdmittedImplicit>>> {
        let mut graph = DiGraph::<SemanticId, ()>::new();
        let nodes = self
            .implicit
            .keys()
            .map(|id| (*id, graph.add_node(*id)))
            .collect::<BTreeMap<_, _>>();
        for (id, inner) in &self.implicit {
            for provider in inner.bodies().flat_map(|b| b.math.providers()) {
                if let Some(dependency) = nodes.get(&provider.id) {
                    graph.add_edge(*dependency, nodes[id], ());
                }
            }
        }
        if let Some(rows) = rows {
            let mut pending = Vec::new();
            for instance in self
                .case
                .instances()
                .iter()
                .filter(|i| rows.contains(&i.instance))
            {
                let body = self
                    .bodies
                    .get(&instance.body)
                    .ok_or_else(|| CompileError::Missing("observation body".into()))?;
                pending.extend(
                    body.math
                        .providers()
                        .iter()
                        .filter_map(|p| nodes.get(&p.id).copied()),
                );
            }
            let mut required = BTreeSet::new();
            let reversed = petgraph::visit::Reversed(&graph);
            let mut traversal = petgraph::visit::Dfs::empty(reversed);
            for root in pending {
                traversal.move_to(root);
                while let Some(node) = traversal.next(reversed) {
                    required.insert(graph[node]);
                }
            }
            graph.retain_nodes(|graph, node| required.contains(&graph[node]));
        }
        let order = toposort(&graph, None)
            .map_err(|_| CompileError::Missing("cyclic implicit provider dependency".into()))?;
        Ok(order
            .into_iter()
            .map(|node| self.implicit[&graph[node]].clone())
            .collect())
    }
}

/// Admitted implicit systems by identity, and the provider calls that expose them.
type Admitted = (
    BTreeMap<SemanticId, Arc<AdmittedImplicit>>,
    BTreeMap<String, ProviderCall>,
);
pub(super) fn admit(
    db: &dyn CompilerDb,
    inventory: Inventory,
    p: &super::Projection,
) -> Result<Admitted> {
    let registry = inventory.quantities(db);
    let checker = inventory.preconditions(db);
    let mut definitions = BTreeMap::new();
    let mut calls = BTreeMap::new();
    let mut external_calls = BTreeMap::new();
    for function in p.functions.values() {
        if let Some(external) = &function.external {
            external_calls.insert(
                external.implementation.clone(),
                provider(db, inventory, external.implementation.clone()).ok_or_else(|| {
                    CompileError::Missing(format!(
                        "external implementation {}",
                        external.implementation
                    ))
                })?,
            );
        }
    }
    for implicit in &p.implicit {
        checkpoint(db);
        let mut available = external_calls.clone();
        available.extend(calls.clone());
        let admit_body = |definition,
                          expressions: &[Expr],
                          quantities: &[QuantityTypeId]|
         -> Result<Arc<AdmittedBody>> {
            Ok(Arc::new(
                crate::typed_math::Request {
                    definition,
                    expressions,
                    formals: &implicit.formals,
                    domains: &BTreeMap::new(),
                    groups: &BTreeMap::new(),
                    providers: &available,
                    literals: &BTreeMap::new(),
                    physical: physical_identity(registry, checker),
                    structure: implicit.spec.revision,
                    limits: p.body_limits,
                }
                .admit_modeling_outputs(
                    registry,
                    checker.as_ref(),
                    db.cancel(),
                    &p.functions,
                    quantities,
                    &implicit.local_quantities,
                    &implicit.validity,
                )?,
            ))
        };
        let mut residuals = Vec::new();
        for r in &implicit.residuals {
            let body = admit_body(r.id, &r.expressions, &r.quantities)?;
            if implicit.algorithm == ImplicitAlgorithm::AffineRates {
                pse_math::implicit::Affine::new(&body.math, implicit.unknowns.len())?;
            }
            let assessment = r
                .assessment
                .as_ref()
                .map(|a| -> Result<RegimeAssessment> {
                    Ok(RegimeAssessment {
                        eligibility: admit_body(
                            pse_ids::named_id(r.id, "eligibility"),
                            std::slice::from_ref(&a.eligibility),
                            &[registry.neutral_dimensionless().ok_or_else(|| {
                                CompileError::Missing("dimensionless eligibility".into())
                            })?],
                        )?,
                        criterion: admit_body(
                            pse_ids::named_id(r.id, "criterion"),
                            &a.criterion,
                            &a.quantities,
                        )?,
                    })
                })
                .transpose()?;
            residuals.push(AdmittedResidual {
                id: r.id,
                body,
                rows: r.rows.clone(),
                assessment,
                hints: if r.hints.is_empty() {
                    None
                } else {
                    Some(admit_body(
                        pse_ids::named_id(r.id, "numerical-hints"),
                        &r.hints.iter().map(|h| h.3.clone()).collect::<Vec<_>>(),
                        &r.hints.iter().map(|h| h.4).collect::<Vec<_>>(),
                    )?)
                },
                terms: if r.terms.is_empty() {
                    None
                } else {
                    Some(admit_body(
                        pse_ids::named_id(r.id, "nominal-terms"),
                        &r.terms.iter().map(|(e, _)| e.clone()).collect::<Vec<_>>(),
                        &r.terms.iter().map(|(_, q)| *q).collect::<Vec<_>>(),
                    )?)
                },
                scales: r.scales.clone(),
                hint_targets: r.hints.iter().map(|h| (h.0, h.1, h.2)).collect(),
            });
        }
        let available_order = residuals
            .iter()
            .map(|r| {
                r.body
                    .math
                    .available_order_for(&(0..r.body.math.input_count()).collect::<Vec<_>>())
            })
            .min()
            .ok_or_else(|| CompileError::Missing("empty implicit residuals".into()))?;
        let mut meaning = implicit.selection.meaning.clone();
        let equivalence = if residuals.len() == 1 {
            pse_math::implicit::graph_equivalence(
                &residuals[0].body.math,
                implicit.unknowns.len(),
                implicit.selection.sign,
            )?
        } else {
            SelectionEquivalence::Unestablished
        };
        if meaning == ImplicitMeaning::Relation {
            if equivalence == SelectionEquivalence::NondegenerateAffine {
                meaning = ImplicitMeaning::Unique;
            } else {
                return Err(CompileError::Missing("implicit expression requires an explicit function selector; numerical starts and bounds do not select mathematical meaning".into()));
            }
        }
        // Regime margin and assessor separation establish a stable winner only after
        // each alternative has a checked root selection of its own. One native iterate
        // per regime cannot establish which of multiple roots within that regime is selected.
        let unique_regimes = if matches!(meaning, ImplicitMeaning::MinimumScore) {
            let mut unique = true;
            for residual in &residuals {
                if pse_math::implicit::graph_equivalence(
                    &residual.body.math,
                    implicit.unknowns.len(),
                    None,
                )? != SelectionEquivalence::NondegenerateAffine
                {
                    unique = false;
                    break;
                }
            }
            unique
        } else {
            false
        };
        let mut neighborhood = if unique_regimes
            || implicit.algorithm == ImplicitAlgorithm::AffineRates
            || equivalence != SelectionEquivalence::Unestablished
        {
            available_order
        } else {
            DerivativeOrder::Value
        };
        if residuals
            .iter()
            .filter_map(|r| r.assessment.as_ref())
            .any(|a| {
                a.eligibility.math.branch_local_order() < DerivativeOrder::First
                    || a.criterion.math.branch_local_order() < DerivativeOrder::First
            })
        {
            neighborhood = DerivativeOrder::Value;
        }
        let mut selection = ImplicitSelection {
            identity: implicit.selection.identity,
            meaning,
            anchors: if implicit.selection.anchors.is_empty() {
                None
            } else {
                Some(admit_body(
                    pse_ids::named_id(implicit.id, "selection-anchors"),
                    &implicit.selection.anchors,
                    &implicit
                        .unknowns
                        .iter()
                        .map(|u| {
                            implicit
                                .spec
                                .outputs
                                .iter()
                                .find(|p| p.id == *u)
                                .map(|p| p.quantity)
                                .ok_or_else(|| CompileError::Missing("anchor quantity".into()))
                        })
                        .collect::<Result<Vec<_>>>()?,
                )?)
            },
            restriction: implicit
                .selection
                .restriction
                .as_ref()
                .map(|e| {
                    admit_body(
                        pse_ids::named_id(implicit.id, "selection-neighborhood"),
                        std::slice::from_ref(e),
                        &[registry.neutral_dimensionless().ok_or_else(|| {
                            CompileError::Missing("selection indicator quantity".into())
                        })?],
                    )
                })
                .transpose()?,
            sign: implicit.selection.sign,
            strict: implicit.selection.strict,
            equivalence,
            neighborhood,
        };
        if selection
            .restriction
            .as_ref()
            .is_some_and(|b| b.math.branch_local_order() < DerivativeOrder::First)
        {
            selection.neighborhood = DerivativeOrder::Value;
        }
        let mut spec = implicit.spec.clone();
        spec.derivatives = available_order.min(selection.neighborhood);
        spec.smoothness = spec.derivatives;
        let descriptor = AdmittedProvider::new(spec, registry)
            .map_err(|e| CompileError::Missing(e.to_string()))?;
        for output in 0..implicit.unknowns.len() {
            calls.insert(
                call_name(implicit.id, output),
                ProviderCall {
                    descriptor: descriptor.clone(),
                    output,
                },
            );
        }
        definitions.insert(
            implicit.id,
            Arc::new(AdmittedImplicit {
                selection,
                algorithm: implicit.algorithm.clone(),
                descriptor,
                unknowns: implicit.unknowns.clone(),
                residuals,
            }),
        );
    }
    Ok((definitions, calls))
}
fn call_name(id: SemanticId, output: usize) -> String {
    format!("implicit_{}_{}", id.to_hex(), output)
}

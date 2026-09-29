// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored source interpretation feeds the common sparse fitting engine.
use super::*;
use crate::math::modeling::ModelingPreparation;
use crate::workflow::modeling::{ModelingPackage, ModelingSimulation, results as checks};
use pse_compiler::workspace::{ModelingOutput, Profile};
use pse_model::generated::identities::RunId;
use pse_model::{HeapUsage, generated::enums::ModelingAnalysisRoute as Route};
use pse_modeling::Limits;
use pse_relations::{
    columnar::{FieldCheckedBatch, RelationRow},
    generated::authored::{datasets, fit_cases, observations},
};

/// Registry-owned fit and measurement relations accompanying modeling declarations.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitData {
    /// Declared fits with their parameters, experiments and observation selections.
    pub fits: Vec<fit_cases::Row>,
    /// Measured observations referenced by fits.
    pub observations: Vec<observations::Row>,
    /// Datasets the observations come from.
    pub datasets: Vec<datasets::Row>,
    #[serde(skip)]
    owner: Option<Arc<pse_columnar::AllocationLease>>,
}
impl FitData {
    pub(crate) fn from_batches(
        batches: &BTreeMap<SemanticId, FieldCheckedBatch>,
    ) -> Result<Self, WorkflowError> {
        Ok(Self {
            fits: batches
                .get(&fit_cases::RELATION_ID)
                .map(fit_cases::Row::rows)
                .transpose()
                .map_err(super::super::relation)?
                .unwrap_or_default(),
            observations: batches
                .get(&observations::RELATION_ID)
                .map(observations::Row::rows)
                .transpose()
                .map_err(super::super::relation)?
                .unwrap_or_default(),
            datasets: batches
                .get(&datasets::RELATION_ID)
                .map(datasets::Row::rows)
                .transpose()
                .map_err(super::super::relation)?
                .unwrap_or_default(),
            owner: None,
        })
    }
    pub(crate) fn tables(
        &self,
        registry: &pse_schema::Registry,
    ) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let mut result = BTreeMap::new();
        macro_rules! relation {
            ($name:ident, $rows:expr) => {{
                let mut builder = $name::Builder::with_registry(registry, $rows.len())
                    .map_err(super::super::relation)?;
                for row in $rows {
                    builder.push(row.clone()).map_err(super::super::relation)?;
                }
                result.insert(
                    $name::RELATION_ID,
                    builder.finish().map_err(super::super::relation)?,
                );
            }};
        }
        relation!(fit_cases, &self.fits);
        relation!(observations, &self.observations);
        relation!(datasets, &self.datasets);
        Ok(result)
    }
}
#[derive(Clone, Debug)]
pub(super) enum Assessment {
    Steady {
        model: ModelingPreparation,
        program: Option<Arc<ExecutableCase>>,
        numerics: Arc<ResolvedNumericalPolicy>,
    },
    Transient(Box<ModelingSimulation>),
}
fn member(model: &ModelingPreparation, path: &str) -> Result<SemanticId, WorkflowError> {
    model
        .compiled()
        .model
        .paths
        .get(path)
        .copied()
        .ok_or_else(|| contract(format!("unknown fit source path {path}")))
}
fn port(model: &ModelingPreparation, id: SemanticId) -> Result<Port, WorkflowError> {
    let source = &model.compiled().admitted.case;
    source
        .parameters()
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or_else(|| contract("a shared fit parameter must bind a numerical parameter member"))
}
impl ModelingPackage {
    /// Attach generated measurement relations to this immutable source revision.
    pub fn with_fit_data(mut self, mut data: FitData) -> Result<Self, WorkflowError> {
        let unique = |ids: Vec<SemanticId>| ids.iter().collect::<BTreeSet<_>>().len() == ids.len();
        if !unique(data.fits.iter().map(|r| r.fit_id.as_id()).collect())
            || !unique(data.observations.iter().map(|r| r.observation_id).collect())
            || !unique(data.datasets.iter().map(|r| r.dataset_id).collect())
            || data
                .observations
                .iter()
                .any(|o| !data.datasets.iter().any(|d| d.dataset_id == o.dataset_id))
        {
            return Err(contract(
                "duplicate fit data identity or absent observation dataset",
            ));
        }
        let bytes = data
            .fits
            .owned_bytes()
            .checked_add(data.observations.owned_bytes())
            .and_then(|n| n.checked_add(data.datasets.owned_bytes()))
            .ok_or_else(|| contract("fit source extent"))?;
        data.owner = Some(
            self.runtime
                .shared
                .math()
                .reserve("modeling:fit-data", bytes)?,
        );
        self.fit_data = Arc::new(data);
        Ok(self)
    }
    /// Compile authored case paths once; trial parameter values never enter source queries.
    pub async fn prepare_fit(
        &self,
        id: FitId,
        profile: FitProfile,
        compiler: Profile,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedFit, WorkflowError> {
        let (problem, assessments) = self
            .prepare_fit_problem(id, profile, compiler, limits, cancel)
            .await?;
        let facts = native::routing::oracle_facts(
            &problem.contract,
            true,
            problem.bounds.iter().all(|(a, b)| a.is_finite() && a == b),
        );
        let route = native::routing::Requirements {
            table: &native::execution::LINKED,
            facts: &facts,
            intent: problem.profile.solver.intent,
            convex: false,
            controls: &problem.profile.solver.controls,
        }
        .select(problem.profile.solver.selection)
        .map_err(crate::math::MathRuntimeError::from)?;
        crate::math::solves::admit_profile(&problem.profile.solver, route)
            .map_err(crate::math::MathRuntimeError::from)?;
        Ok(PreparedFit {
            source: self.clone(),
            problem: Arc::new(problem),
            route,
            assessments,
        })
    }
    pub(in crate::workflow::fitting) async fn prepare_fit_problem(
        &self,
        id: FitId,
        profile: FitProfile,
        compiler: Profile,
        limits: Limits,
        cancel: &crate::CancelSource,
    ) -> Result<(FitProblem, Vec<Assessment>), WorkflowError> {
        let policy = &self.runtime.shared.budget().math;
        let reservation = datafusion::execution::memory_pool::MemoryConsumer::new("fit:prepared")
            .register(&self.runtime.shared.pool());
        reservation
            .try_grow(policy.workspace_bytes)
            .map_err(crate::math::MathRuntimeError::from)?;
        let d = self
            .fit_data
            .fits
            .iter()
            .find(|d| d.fit_id == id)
            .cloned()
            .ok_or_else(|| contract("unknown fitting declaration"))?;
        profile
            .solver
            .controls
            .validate()
            .map_err(crate::math::MathRuntimeError::from)?;
        if profile.solver.controls.start != native::solve::StartPolicy::NoPriorStart
            || profile.solver.controls.reuse != native::solve::ReusePolicy::Fresh
        {
            return Err(contract(
                "fitting consumes declared guesses and fresh solver allocations",
            ));
        }
        if profile.solver.intent != SolveIntent::Optimize
            || !profile.rank_tolerance.is_finite()
            || !(0.0..1.0).contains(&profile.rank_tolerance)
            || profile.rank_tolerance == 0.0
            || profile.max_cells == 0
            || d.experiments.is_empty()
            || d.parameters.is_empty()
        {
            return Err(contract("fit intent, rank policy or inventories"));
        }
        if d.parameters
            .iter()
            .map(|p| p.symbol_id)
            .collect::<BTreeSet<_>>()
            .len()
            != d.parameters.len()
            || d.experiments
                .iter()
                .map(|e| e.experiment_id)
                .collect::<BTreeSet<_>>()
                .len()
                != d.experiments.len()
            || d.observations
                .iter()
                .map(|o| o.observation_id)
                .collect::<BTreeSet<_>>()
                .len()
                != d.observations.len()
        {
            return Err(contract(
                "duplicate fit parameter, experiment or observation binding",
            ));
        }
        let second = profile.solver.controls.hessian == HessianMode::Exact;
        if second && d.experiments.iter().any(|e| e.route == Route::Integrated) {
            return Err(contract(
                "transient fitting requires limited-memory Hessians",
            ));
        }
        let order = if second {
            DerivativeOrder::Second
        } else {
            DerivativeOrder::First
        };
        let q = &self.quantities;
        // Resolve paths and physical contracts before creating any global coordinates.
        let mut sources = Vec::new();
        let mut parameter_ports: Vec<Option<Port>> = vec![None; d.parameters.len()];
        for e in &d.experiments {
            let (mut bindings, mut case) = self
                .declared_case(e.case_id, e.route, limits, cancel)
                .await?;
            bindings
                .demand
                .extend(e.bindings.iter().map(|b| b.path.clone()));
            bindings.demand.extend(
                d.observations
                    .iter()
                    .filter(|o| o.experiment_id == e.experiment_id)
                    .map(|o| o.output_path.clone()),
            );
            bindings.demand.sort();
            bindings.demand.dedup();
            let model = self
                .prepare(e.case_id, e.experiment_id, bindings.clone(), limits, cancel)
                .await?;
            let mut local = BTreeMap::new();
            let mut local_ids = BTreeSet::new();
            for binding in &e.bindings {
                let parameter = d
                    .parameters
                    .iter()
                    .position(|p| p.symbol_id == binding.parameter_id)
                    .ok_or_else(|| contract("experiment binds an undeclared fit parameter"))?;
                let id = member(&model, &binding.path)?;
                let mut p = port(&model, id)?;
                p.id = binding.parameter_id;
                if parameter_ports[parameter]
                    .as_ref()
                    .is_some_and(|old| old != &p)
                {
                    return Err(contract("shared parameter physical contract mismatch"));
                }
                parameter_ports[parameter] = Some(p);
                if local.insert(parameter, id).is_some() || !local_ids.insert(id) {
                    return Err(contract("duplicate experiment parameter binding"));
                }
                case.values
                    .insert(binding.path.clone(), d.parameters[parameter].value);
            }
            sources.push((bindings, case, model, local));
        }
        let parameter_ports = parameter_ports
            .into_iter()
            .map(|p| p.ok_or_else(|| contract("fit parameter has no experiment binding")))
            .collect::<Result<Vec<_>, _>>()?;
        let lineage = pse_model::lineage::Fitted::new(
            d.fit_id,
            sources.iter().map(|(_, _, model, _)| model.solved()),
        );
        let named = lineage.lineage();
        let mut targets = Vec::new();
        let mut declarations = Vec::new();
        let mut vars = Vec::new();
        let mut initial = Vec::new();
        let mut parameter_columns = Vec::new();
        for (p, port) in d.parameters.iter().zip(&parameter_ports) {
            if !p.value.is_finite()
                || !p.scale.is_finite()
                || p.scale <= 0.0
                || p.lower.is_some_and(|v| !v.is_finite() || v > p.value)
                || p.upper.is_some_and(|v| !v.is_finite() || v < p.value)
                || (!p.fixed && p.lower.is_some() && p.lower == p.upper)
            {
                return Err(contract("parameter bounds, scale or fixed decision"));
            }
            targets.push(TargetSpec {
                id: p.symbol_id,
                kind: NumericalTarget::Variable,
                quantity: port.quantity,
                unit: port.unit,
                integer: false,
                declared_tolerance: None,
            });
            declarations.push(SourcedRequirement {
                source: NumericalSource::Model,
                declaration: NumericalRequirement {
                    requirement_id: pse_ids::named_id(
                        d.fit_id.as_id(),
                        &format!("nominal.{}", p.symbol_id),
                    ),
                    model_id: named.model_id,
                    case_id: named.case_id,
                    instance_id: named.instance_id,
                    fit_id: named.fit_id,
                    target_id: p.symbol_id,
                    target_kind: NumericalTarget::Variable,
                    nominal: Some(p.scale),
                    scaling_factor: None,
                    absolute_tolerance: None,
                    relative_tolerance: None,
                    unit_id: Some(port.unit.as_id()),
                    coordinates: NumericalCoordinates::Physical,
                    priority: 0,
                    required: true,
                    provenance: "authored fitting parameter nominal".into(),
                },
            });
            parameter_columns.push(if p.fixed {
                None
            } else {
                let col = OriginalCol::new(vars.len());
                vars.push(Variable {
                    id: p.symbol_id,
                    lower: p.lower.unwrap_or(f64::NEG_INFINITY),
                    upper: p.upper.unwrap_or(f64::INFINITY),
                });
                initial.push(p.value);
                Some(col)
            });
        }
        let mut experiments = Vec::new();
        let mut assessments = Vec::new();
        let mut bounds = Vec::new();
        let mut rows = Vec::new();
        let mut measurements = Vec::new();
        let mut bytes = profile
            .solver
            .controls
            .report_allowance()
            .map_err(crate::math::MathRuntimeError::from)?
            .checked_add(256 * 1024)
            .ok_or_else(|| contract("fit report extent"))?;
        let mut physical_cells = 0usize;
        let mut identity = FramedHasher::new(pse_ids::Frame::ModelingFitSourceV1);
        identity.hash(&self.physical.key);
        identity.hash(&self.revision.identity());
        let mut execution_identity = FramedHasher::new(pse_ids::Frame::ModelingFitExecutionV1);
        for row in &self.fit_data.observations {
            row.frame(&mut identity);
        }
        for row in &self.fit_data.datasets {
            row.frame(&mut identity);
        }
        for (ei, (e, (bindings, case, model, local))) in
            d.experiments.iter().zip(sources).enumerate()
        {
            let local_observations = d
                .observations
                .iter()
                .filter(|o| o.experiment_id == e.experiment_id)
                .collect::<Vec<_>>();
            let mut bound_times = BTreeMap::new();
            let experiment = if e.route == Route::Integrated {
                let mut integration = if let Some(profile) =
                    profile.simulations.get(&e.experiment_id)
                {
                    profile.clone()
                } else {
                    let data=model.compiled().model.fixtures.get(&e.experiment_id).ok_or_else(||contract("transient experiment needs explicit or authored integration controls"))?;
                    self.integration_profile(&model, data, &profile.solver.numerics)?
                };
                for o in &local_observations {
                    let raw = o
                        .time
                        .ok_or_else(|| contract("transient observation needs time"))?;
                    let scale = if let Some(id) = o.time_unit_id {
                        let unit = q.unit(UnitId::from_id(id)).map_err(math)?;
                        if unit.is_affine
                            || unit.dimension
                                != pse_quantity::DimensionVector::base(
                                    pse_quantity::BaseDimension::Time,
                                )
                        {
                            return Err(contract(
                                "observation time requires a non-affine time unit",
                            ));
                        }
                        unit.scale_to_canonical
                    } else {
                        1.
                    };
                    // Generated dynamics uses the authored canonical axis directly, without a second time origin.
                    let time = super::super::time::observation(
                        raw,
                        scale,
                        o.time_basis.unwrap_or(
                            pse_relations::generated::enums::ObservationTimeBasis::Elapsed,
                        ),
                        0.,
                        integration.start,
                    )?;
                    if !time.is_finite() || time < integration.start || time > integration.end {
                        return Err(contract("observation time outside the experiment domain"));
                    }
                    bound_times.insert(o.observation_id, time);
                }
                // Keep requested diagnostic/end samples as well as every included measurement.
                integration.samples.extend(
                    local_observations
                        .iter()
                        .filter(|o| o.included)
                        .map(|o| bound_times[&o.observation_id]),
                );
                integration.samples.sort_by(f64::total_cmp);
                integration.samples.dedup();
                integration.sensitivities = local.keys().any(|p| parameter_columns[*p].is_some());
                let simulation = if let Some(modes) = profile.modes.get(&e.experiment_id) {
                    self.prepare_simulation_modes(
                        e.case_id,
                        e.experiment_id,
                        bindings,
                        limits,
                        case,
                        compiler,
                        integration,
                        modes.clone(),
                        cancel,
                    )
                    .await?
                } else {
                    self.prepare_simulation(
                        e.case_id,
                        e.experiment_id,
                        bindings,
                        limits,
                        case,
                        compiler,
                        integration,
                        cancel,
                    )
                    .await?
                };
                execution_identity.hash(&simulation.identity());
                bytes = bytes
                    .checked_add(simulation.bytes)
                    .ok_or_else(|| contract("fit transient extent"))?;
                let bindings = local
                    .iter()
                    .map(|(parameter, id)| {
                        let position = simulation
                            .contract()
                            .parameters
                            .iter()
                            .position(|p| p == id)
                            .ok_or_else(|| {
                                contract("fitted dynamic member is not a sensitivity parameter")
                            })?;
                        Ok(ParameterBinding {
                            local: position,
                            parameter: *parameter,
                            conversion: pse_quantity::UnitConvertSpec {
                                from: parameter_ports[*parameter].unit,
                                to: parameter_ports[*parameter].unit,
                                scale: 1.,
                                offset: 0.,
                            },
                        })
                    })
                    .collect::<Result<Vec<_>, WorkflowError>>()?;
                let output_ports = simulation
                    .contract()
                    .outputs
                    .iter()
                    .map(|id| {
                        let row = simulation
                            .model()
                            .compiled()
                            .admitted
                            .case
                            .rows()
                            .iter()
                            .find(|r| r.id == *id)
                            .ok_or_else(|| contract("dynamic output contract absent"))?;
                        Ok(Port {
                            id: *id,
                            quantity: row.quantity,
                            unit: q.quantity_type(row.quantity).map_err(math)?.canonical_unit,
                        })
                    })
                    .collect::<Result<Vec<_>, WorkflowError>>()?;
                physical_cells = physical_cells
                    .checked_add(
                        simulation
                            .profile()
                            .samples
                            .len()
                            .checked_mul(simulation.model().compiled().admitted.outputs.len())
                            .ok_or_else(|| contract("fit check extent"))?,
                    )
                    .ok_or_else(|| contract("fit check extent"))?;
                let result = Experiment::Transient(Box::new(Transient {
                    program: simulation.program(),
                    profile: simulation.profile().clone(),
                    parameters: simulation.parameters.clone(),
                    output_ports,
                    bindings,
                }));
                assessments.push(Assessment::Transient(Box::new(simulation)));
                result
            } else {
                if profile.simulations.contains_key(&e.experiment_id)
                    || profile.modes.contains_key(&e.experiment_id)
                {
                    return Err(contract("algebraic experiment has a dynamic profile"));
                }
                let resolved = self
                    .resolve_case(
                        e.case_id,
                        e.experiment_id,
                        bindings,
                        limits,
                        case,
                        order,
                        compiler,
                        profile.solver.clone(),
                        Default::default(),
                        BTreeMap::new(),
                        BTreeMap::new(),
                        false,
                        cancel,
                    )
                    .await?;
                let model = resolved.model.model.clone();
                let source = resolved.model.case.compiled().plan.structure();
                if source.objective().is_some() {
                    return Err(contract("fit experiment has an authored objective"));
                }
                execution_identity.hash(&source.key());
                let mut values = resolved.model.values.clone();
                let mut coordinates = TiVec::<GlobalCol, _>::new();
                // ADR-0103 item 6: estimation refuses a discrete variable the case leaves free.
                model
                    .compiled()
                    .model
                    .require_fixed_discrete(
                        source
                            .variables()
                            .iter()
                            .filter(|v| !v.fixed)
                            .map(|v| v.port.id),
                        pse_modeling::DomainAnalysis::Fitting,
                    )
                    .map_err(crate::workflow::modeling_error)?;
                for v in source.variables().iter().filter(|v| !v.fixed) {
                    targets.push(TargetSpec {
                        id: alias(e.experiment_id, v.port.id),
                        kind: NumericalTarget::Variable,
                        quantity: v.port.quantity,
                        unit: v.port.unit,
                        integer: false,
                        declared_tolerance: None,
                    });
                    let col = vars.len();
                    vars.push(Variable {
                        id: alias(e.experiment_id, v.port.id),
                        lower: v.lower.unwrap_or(f64::NEG_INFINITY),
                        upper: v.upper.unwrap_or(f64::INFINITY),
                    });
                    initial.push(
                        *values
                            .scalars
                            .get(&v.port.id)
                            .ok_or_else(|| contract("missing experiment initial state"))?,
                    );
                    coordinates.push((v.port.id, OriginalCol::new(col)));
                }
                let local_states = coordinates.len();
                for (parameter, id) in &local {
                    values.scalars.insert(*id, d.parameters[*parameter].value);
                    if let Some(col) = parameter_columns[*parameter] {
                        coordinates.push((*id, col));
                    }
                }
                for (id, value) in &values.scalars {
                    execution_identity.id(id).u64(value.to_bits());
                }
                let mut outputs = source
                    .rows()
                    .iter()
                    .filter(|r| r.lower.is_finite() || r.upper.is_finite())
                    .map(|r| r.id)
                    .collect::<BTreeSet<_>>();
                for o in &local_observations {
                    if o.included {
                        outputs.insert(
                            ModelingOutput::Member(member(&model, &o.output_path)?).row_id(),
                        );
                    }
                }
                let case = self
                    .runtime
                    .shared
                    .math()
                    .prepare_modeling_functions(
                        self.workspace.clone(),
                        model.clone(),
                        outputs.into_iter().collect(),
                        coordinates.iter().map(|v| v.0).collect(),
                        order,
                        compiler,
                        cancel,
                    )
                    .await?;
                let mut constraints = Vec::new();
                // Function projection owns derivatives; the admitted case owns row bounds.
                for (i, r) in case.assembly.structure().rows().iter().enumerate() {
                    if let Some(original) = source
                        .rows()
                        .iter()
                        .find(|o| o.id == r.id && (o.lower.is_finite() || o.upper.is_finite()))
                    {
                        targets.push(TargetSpec {
                            id: alias(e.experiment_id, r.id),
                            kind: NumericalTarget::Row,
                            quantity: r.quantity,
                            unit: q.quantity_type(r.quantity).map_err(math)?.canonical_unit,
                            integer: false,
                            declared_tolerance: None,
                        });
                        constraints.push((GlobalRow::new(i), OriginalRow::new(rows.len())));
                        rows.push(alias(e.experiment_id, r.id));
                        bounds.push((original.lower, original.upper));
                    }
                }
                for mut r in resolved.numerical.declarations.clone() {
                    r.declaration.target_id = alias(e.experiment_id, r.declaration.target_id);
                    r.declaration.requirement_id =
                        alias(e.experiment_id, r.declaration.requirement_id);
                    if targets.iter().any(|t| {
                        t.id == r.declaration.target_id && t.kind == r.declaration.target_kind
                    }) {
                        declarations.push(r);
                    }
                }
                let check_rows = checks::observation_rows(model.compiled(), &values)?;
                let program = if check_rows.is_empty() {
                    None
                } else {
                    Some(
                        self.runtime
                            .shared
                            .math()
                            .prepare_modeling_functions(
                                self.workspace.clone(),
                                model.clone(),
                                check_rows.into_iter().collect(),
                                vec![],
                                DerivativeOrder::Value,
                                compiler,
                                cancel,
                            )
                            .await?,
                    )
                };
                let numerics = resolved.numerics;
                physical_cells = physical_cells
                    .checked_add(model.compiled().admitted.outputs.len())
                    .ok_or_else(|| contract("fit check extent"))?;
                bytes = bytes
                    .checked_add(case.assembly.numeric_worker_bytes())
                    .and_then(|n| {
                        n.checked_add(
                            program
                                .as_ref()
                                .map_or(0, |p| p.assembly.numeric_worker_bytes()),
                        )
                    })
                    .ok_or_else(|| contract("fit worker extent"))?;
                assessments.push(Assessment::Steady {
                    model,
                    program,
                    numerics,
                });
                Experiment::Steady(Steady {
                    variables: source.variables().to_vec(),
                    case,
                    values,
                    coordinates,
                    constraints,
                    local_states,
                    providers: resolved.providers,
                })
            };
            for binding in local_observations {
                let observation = self
                    .fit_data
                    .observations
                    .iter()
                    .find(|o| o.observation_id == binding.observation_id)
                    .ok_or_else(|| contract("missing authored observation"))?;
                let output = ModelingOutput::Member(member(&model, &binding.output_path)?).row_id();
                let (row, port) = match &experiment {
                    Experiment::Steady(s) => {
                        if binding.time.is_some()
                            || binding.time_basis.is_some()
                            || binding.time_unit_id.is_some()
                        {
                            return Err(contract("algebraic observation has a time coordinate"));
                        }
                        let r = model
                            .compiled()
                            .admitted
                            .case
                            .rows()
                            .iter()
                            .find(|r| r.id == output)
                            .ok_or_else(|| contract("unknown observed member"))?;
                        (
                            s.case
                                .assembly
                                .structure()
                                .rows()
                                .iter()
                                .position(|r| r.id == output)
                                .unwrap_or(usize::MAX),
                            Port {
                                id: output,
                                quantity: r.quantity,
                                unit: q.quantity_type(r.quantity).map_err(math)?.canonical_unit,
                            },
                        )
                    }
                    Experiment::Transient(s) => {
                        let i=s.output_ports.iter().position(|p|p.id==output).ok_or_else(||contract("transient observations require a state or an authored report member"))?;
                        (i, s.output_ports[i].clone())
                    }
                };
                let ty = q.quantity_type(port.quantity).map_err(math)?;
                let conversion = pse_quantity::convert_spec_for_type(
                    q.unit(UnitId::from_id(observation.unit_id)).map_err(math)?,
                    q.unit(port.unit).map_err(math)?,
                    &ty.key,
                )
                .map_err(math)?;
                let value = observation
                    .value
                    .map(|v| v * conversion.scale + conversion.offset);
                let sigma = observation.std_dev.map(|s| s * conversion.scale.abs());
                if !binding.importance.is_finite()
                    || binding.importance <= 0.
                    || value.is_some_and(|v| !v.is_finite())
                    || sigma.is_some_and(|s| !s.is_finite() || s <= 0.)
                    || sigma.is_some_and(|s| !(binding.importance.sqrt() / s).is_finite())
                    || (binding.included && (value.is_none() || sigma.is_none()))
                {
                    return Err(contract(
                        "included observations require finite values, positive difference-unit standard deviations and importance",
                    ));
                }
                measurements.push(Measurement {
                    id: binding.observation_id,
                    experiment: ei,
                    row,
                    time: bound_times.get(&binding.observation_id).copied(),
                    sample_index: match &experiment {
                        Experiment::Transient(s) if binding.included => Some(
                            s.profile
                                .samples
                                .binary_search_by(|t| {
                                    t.total_cmp(&bound_times[&binding.observation_id])
                                })
                                .map_err(|_| contract("unbound observation sample"))?,
                        ),
                        _ => None,
                    },
                    included: binding.included,
                    value,
                    sigma,
                    importance: binding.importance,
                    port,
                });
            }
            experiments.push(experiment);
        }
        if measurements.len() != d.observations.len()
            || !measurements.iter().any(|o| o.included)
            || profile
                .simulations
                .keys()
                .chain(profile.modes.keys())
                .any(|id| {
                    !d.experiments
                        .iter()
                        .any(|e| e.experiment_id == *id && e.route == Route::Integrated)
                })
        {
            return Err(contract("fit observation or dynamic profile ownership"));
        }
        let problem = PreparedExperiments {
            execution_identity: execution_identity.finish_hash(),
            declaration: d,
            lineage,
            profile,
            order,
            variables: vars,
            rows,
            bounds,
            initial,
            parameter_ports,
            parameter_columns,
            experiments,
            measurements,
            targets,
            declarations,
            bytes,
            physical_cells,
        }
        .finish(
            self.runtime.clone(),
            q.clone(),
            identity.finish_hash(),
            reservation,
        )?;
        Ok((problem, assessments))
    }
}
impl PreparedFit {
    pub(crate) fn execute(
        &self,
        run_id: RunId,
        flag: Arc<std::sync::atomic::AtomicBool>,
        progress: Arc<native::solve::Progress>,
    ) -> Result<FitReport, crate::math::MathRuntimeError> {
        let started = std::time::Instant::now();
        let mut report = self.problem.execute(self.route, flag.clone(), progress)?;
        let Some(candidate) = report.candidate.as_ref() else {
            return Ok(report);
        };
        let assess=|| -> Result<(Vec<super::super::ModelingCheck>,Vec<super::super::ModelingReport>),WorkflowError> {
            let mut rows=Vec::new(); let mut reports=Vec::new();
            for (ei,(source,experiment)) in self.assessments.iter().zip(&self.problem.experiments).enumerate() {
                match (source,experiment) {
                    (Assessment::Steady{model,program,numerics},Experiment::Steady(s))=>{
                        let mut values=s.values.clone();
                        for (id,col) in &s.coordinates {values.scalars.insert(*id,candidate[col.get()]);}
                        let observed=if let Some(program)=program {
                            let providers=s.providers.values().map(|p|p.worker_scoped(flag.clone()).map(|w|(p.spec().key(),w)).map_err(|e|WorkflowError::from(crate::math::MathRuntimeError::from(native::ProblemError::Provider(e))))).collect::<Result<_,_>>()?;
                            let mut worker=program.assembly.worker(providers,flag.clone());
                            let outputs=worker.constraints(&values).map_err(math)?;
                            program.assembly.structure().rows().iter().map(|r|r.id).zip(outputs).collect()
                        }else{BTreeMap::new()};
                        let (checks,observations)=checks::assess_observations(run_id,model.compiled(),&values,&observed,numerics,&self.problem.quantities,true,None,None)?;
                        rows.extend(checks); reports.extend(observations);
                    },
                    (Assessment::Transient(simulation),Experiment::Transient(s))=>{
                        let mut parameters=s.parameters.clone();
                        for binding in &s.bindings {
                            let value=self.problem.parameter_columns[binding.parameter].map_or(self.problem.declaration.parameters[binding.parameter].value,|col|candidate[col.get()]);
                            parameters[binding.local]=value*binding.conversion.scale+binding.conversion.offset;
                        }
                        let trajectory=report.trajectories.get(&self.problem.declaration.experiments[ei].experiment_id).ok_or_else(||contract("final original trajectory unavailable"))?;
                        let checks=simulation.check_samples(run_id,trajectory,&parameters,&flag,started);
                        if let Some(error)=checks.error {return Err(WorkflowError::Boundary(Box::new(error)));}
                        if !checks.complete{return Err(contract("original trajectory assessment incomplete"));}
                        rows.extend(checks.rows); reports.extend(checks.reports);
                    },
                    _=>return Err(contract("fit assessment source mismatch")),
                }
            }
            Ok((rows,reports))
        };
        match assess() {
            Ok((checks, reports)) => {
                report.checks = checks;
                report.reports = reports;
                report.checks_complete = true;
            }
            Err(error) => report.validation_error = Some(error.boundary_diagnostic()),
        }
        Ok(report)
    }
}

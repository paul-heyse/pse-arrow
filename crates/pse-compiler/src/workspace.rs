// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Salsa owns semantic dependencies. Native compilation is an explicit effect outside queries.
use crate::typed_math::{AdmittedBody, Formal, Occurrence, ProviderCall};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError,
    assembly::{AssemblyLimits, CasePlan, LocalDemand},
    binding::{CaseLimits, CaseStructure, CaseValues, Target},
    coefficients::Coefficients,
    guarded::{CompiledBody, PreparedBody},
    jets::EvaluationLimits,
    library::Optimization,
    typed::BodyLimits,
};
use pse_quantity::{PhysicalPreconditions, QuantityRegistry, QuantityTypeId};
use pse_structural::{
    incidence::{CaseIncidence, Constraint, Incidence, StructuralAnalysis},
    projection::{GraphLimits, Scope},
};
use salsa::Database;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Immutable physical and provider context for checked modeling declarations.
#[derive(Clone, Debug, PartialEq)]
pub struct CompilerContext {
    /// Actual admitted physical meanings.
    pub quantities: Arc<QuantityRegistry>,
    /// Immutable physical prerequisites.
    pub preconditions: Arc<PhysicalPreconditions>,
    /// Admitted provider descriptors; factories remain attempt-owned.
    pub providers: BTreeMap<String, ProviderCall>,
}
/// Shared complete identity of actual physical declarations and prerequisites.
pub fn physical_identity(
    quantities: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
) -> ContentHash {
    crate::physical_identity::identity(quantities, preconditions)
}
/// Consumed compilation controls, separate from semantic preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Profile {
    /// Optimization controls.
    pub optimization: Optimization,
    /// Local numeric limits affecting evaluator admission.
    pub evaluation: EvaluationLimits,
    /// Per-case sparse assembly and complete numerical-worker admission limits.
    pub assembly: AssemblyLimits,
    /// Shared finite conservative construction work for domain facts and demanded
    /// class proof, including expression storage/substitution bounds and derivative
    /// calls. These units are not measured library-operation counters and do not
    /// admit byte allocations or numerical evaluator work.
    pub class_proof_work: usize,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            optimization: Optimization::default(),
            evaluation: EvaluationLimits::default(),
            assembly: AssemblyLimits::default(),
            class_proof_work: 1_000_000,
        }
    }
}
/// Finite workspace metadata policy. Generations never escape as Salsa handles.
#[derive(Clone, Copy, Debug)]
pub struct WorkspaceLimits {
    /// Maximum input inventory entries.
    pub entries: usize,
    /// Maximum admitted input extent (including conservative owned metadata allowance).
    pub input_bytes: usize,
    /// Maximum retained Salsa ingredient entries, including owned query keys.
    pub retained_entries: usize,
    /// Maximum known retained Salsa bytes; foreign library heaps are not claimed here.
    pub retained_bytes: usize,
    /// Retained values per expensive query.
    pub query_values: usize,
}
impl Default for WorkspaceLimits {
    fn default() -> Self {
        Self {
            entries: 4096,
            input_bytes: 2 << 30,
            retained_entries: 16_384,
            retained_bytes: 256 << 20,
            query_values: 64,
        }
    }
}
/// Typed deterministic diagnostic; original errors are retained for downcasting and attribution.
#[derive(Clone, Debug, thiserror::Error)]
pub enum CompileError {
    /// Missing admitted input.
    #[error("missing compiler input: {0}")]
    Missing(String),
    /// A valid original case does not establish the selected conditional realization.
    #[error("conditional realization unavailable: {0}")]
    ConditionalUnavailable(String),
    /// Source parser failure.
    #[error("definition {definition} source {source_index}: {error}")]
    Syntax {
        /// Authored definition, independent of its changing byte offsets.
        definition: SemanticId,
        /// Ordered expression within the definition.
        source_index: usize,
        /// Exact parser failure and byte range.
        #[source]
        error: Arc<pse_authoring::dsl::DslError>,
    },
    /// Physical/math rejection.
    #[error(transparent)]
    Math(Arc<MathError>),
    /// Generic modeling rejection.
    #[error(transparent)]
    Modeling(#[from] pse_modeling::ModelingError),
    /// Structural rejection.
    #[error(transparent)]
    Structure(#[from] pse_structural::projection::ProjectionError),
    /// Cancellation is transient and never retained as a query value.
    #[error("compiler cancelled")]
    Cancelled,
    /// Admission is checked outside tracked queries.
    #[error("compiler resource limit: {0}")]
    Limit(&'static str),
}
pse_diagnostics::impl_diagnostic! {
    CompileError,
    code(this) { match this { Self::Cancelled=>Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),Self::Limit(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),Self::Missing(_)=>Some(pse_diagnostics::DiagnosticCode::CompilerMissing),Self::ConditionalUnavailable(_)=>Some(pse_diagnostics::DiagnosticCode::ModelingConditionalUnitAdmissionUnsupported),Self::Math(_) | Self::Modeling(_) | Self::Syntax{..} | Self::Structure(_)=>None } },
    forward(this) { match this {Self::Math(e)=>Some(e.as_ref()),Self::Modeling(e)=>Some(e),Self::Syntax{error,..}=>Some(error.as_ref()),Self::Structure(e)=>Some(e),_=>None} },
    help(_this) { None },related(_this) { None },source(_this) { None },
    facts(this) {
        let rule = match this { Self::Missing(_) => Some(pse_diagnostics::DiagnosticRule::CompilerMissing), Self::ConditionalUnavailable(_) => Some(pse_diagnostics::DiagnosticRule::ModelingConditionalUnitAdmissionUnsupported), Self::Cancelled => Some(pse_diagnostics::DiagnosticRule::CompilerCancelled), Self::Limit(_) => Some(pse_diagnostics::DiagnosticRule::CompilerLimit), Self::Math(_) | Self::Modeling(_) | Self::Syntax{..} | Self::Structure(_) => None };
        pse_diagnostics::DiagnosticFacts{rule,..Default::default()}
    }
}
impl PartialEq for CompileError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Missing(a), Self::Missing(b)) => a == b,
            (Self::ConditionalUnavailable(a), Self::ConditionalUnavailable(b)) => a == b,
            (Self::Modeling(a), Self::Modeling(b)) => a == b,
            (
                Self::Syntax {
                    definition: a,
                    source_index: ai,
                    error: ae,
                },
                Self::Syntax {
                    definition: b,
                    source_index: bi,
                    error: be,
                },
            ) => a == b && ai == bi && Arc::ptr_eq(ae, be),
            (Self::Math(a), Self::Math(b)) => Arc::ptr_eq(a, b),
            (Self::Structure(a), Self::Structure(b)) => a == b,
            (Self::Cancelled, Self::Cancelled) => true,
            (Self::Limit(a), Self::Limit(b)) => a == b,
            _ => false,
        }
    }
}
impl From<MathError> for CompileError {
    fn from(e: MathError) -> Self {
        Self::Math(Arc::new(e))
    }
}
type Result<T> = std::result::Result<T, CompileError>;
mod modeling;
pub use modeling::{
    AdmittedImplicit, AdmittedModeling, AutomaticCausalUnit, BoundStructure,
    ConditionalUnitInventory, Derivation, Derived, FlowConnectionDocument, FlowSelectionDocument,
    ImplicitAlgorithm, ImplicitCapabilities, ImplicitMeaning, ImplicitScale, ImplicitSelection,
    ModelingBodyRetention, ModelingCaseBindings, ModelingExpectationResult, ModelingFlowSelection,
    ModelingHint, ModelingOutput, ModelingPointChecks, ModelingRevision, ModelingTestValue,
    ModelingValidityResult, ModelingVariableState, ObjectiveBound, PreparedModeling,
    SelectionEquivalence, SelectionNeighborhood, SemanticModeling,
};
#[salsa::db]
trait CompilerDb: Database {
    fn cancel(&self) -> &Arc<AtomicBool>;
    fn body_retention(&self) -> Option<&Arc<dyn ModelingBodyRetention>>;
    fn body_refusal(&self) -> &std::sync::Mutex<Option<MathError>>;
}
#[salsa::db]
#[derive(Clone, Default)]
struct CompilerDatabase {
    storage: salsa::Storage<Self>,
    cancel: Arc<AtomicBool>,
    body_retention: Option<Arc<dyn ModelingBodyRetention>>,
    body_refusal: Arc<std::sync::Mutex<Option<MathError>>>,
}
#[salsa::db]
impl Database for CompilerDatabase {}
#[salsa::db]
impl CompilerDb for CompilerDatabase {
    fn cancel(&self) -> &Arc<AtomicBool> {
        &self.cancel
    }
    fn body_retention(&self) -> Option<&Arc<dyn ModelingBodyRetention>> {
        self.body_retention.as_ref()
    }
    fn body_refusal(&self) -> &std::sync::Mutex<Option<MathError>> {
        &self.body_refusal
    }
}
// Resource admission is an effect. Abort tracked evaluation so a transient refusal
// never becomes a retained semantic result, then recover its typed cause at the front door.
fn body_refused(db: &dyn CompilerDb, error: MathError) -> ! {
    if let Ok(mut refusal) = db.body_refusal().lock() {
        *refusal = Some(error);
    }
    std::panic::resume_unwind(Box::new(salsa::Cancelled::Local))
}
fn compiler_cancelled(db: &dyn CompilerDb) -> CompileError {
    db.body_refusal()
        .lock()
        .ok()
        .and_then(|mut error| error.take())
        .map_or(CompileError::Cancelled, CompileError::from)
}
fn checkpoint(db: &dyn CompilerDb) {
    if db.cancel().load(Ordering::Acquire) {
        db.cancellation_token().cancel();
    }
    db.unwind_if_revision_cancelled();
}
#[salsa::interned(heap_size = name_key_heap)]
struct NameKey<'db> {
    text: String,
}
fn name_key_heap((text,): &(String,)) -> usize {
    text.capacity()
}
#[salsa::interned(heap_size = selection_key_heap)]
struct SelectionKey<'db> {
    ids: Vec<SemanticId>,
}
fn selection_key_heap((ids,): &(Vec<SemanticId>,)) -> usize {
    ids.capacity().saturating_mul(size_of::<SemanticId>())
}
#[salsa::input]
struct Inventory {
    environment: ContentHash,
    quantities: Arc<QuantityRegistry>,
    preconditions: Arc<PhysicalPreconditions>,
    providers: BTreeMap<String, ProviderCall>,
}
fn provider(db: &dyn CompilerDb, i: Inventory, name: String) -> Option<ProviderCall> {
    provider_query(db, i, NameKey::new(db, name))
}
#[salsa::tracked(returns(clone), lru = 64)]
fn provider_query(db: &dyn CompilerDb, i: Inventory, name: NameKey<'_>) -> Option<ProviderCall> {
    i.providers(db).get(name.text(db)).cloned()
}
fn structural_plan(
    id: SemanticId,
    p: &CasePlan,
    cancel: &Arc<AtomicBool>,
) -> Result<Arc<StructuralAnalysis>> {
    let _span = tracing::info_span!("pse.case.structural_analysis").entered();
    let rows = p
        .structure()
        .rows()
        .iter()
        .map(|r| Constraint {
            id: r.id,
            lower: r.lower.is_finite().then_some(r.lower),
            upper: r.upper.is_finite().then_some(r.upper),
        })
        .collect();
    let columns = p.columns().to_vec();
    let mut edges = vec![];
    let incidence = p.incidence(cancel)?;
    for (instance, b) in p.structure().instances().iter().enumerate() {
        for c in &b.contributions {
            if let Target::Row(row) = c.target {
                for &slot in incidence[instance]
                    .first_for_output(c.output)
                    .ok_or_else(|| CompileError::Missing("selected structural incidence".into()))?
                {
                    let column = b.slots[slot].source();
                    if columns.binary_search(&column).is_ok() {
                        edges.push(Incidence {
                            row,
                            column,
                            instance: b.instance,
                            output: c.output,
                        });
                    }
                }
            }
        }
    }
    let objective = p
        .dependencies(cancel)?
        .into_iter()
        .filter(|dependency| matches!(dependency.target, Target::Objective(_)))
        .flat_map(|dependency| dependency.execution)
        .filter(|id| columns.binary_search(id).is_ok())
        .collect();
    let inc = CaseIncidence::new(
        Scope::Whole(id),
        rows,
        columns,
        edges,
        objective,
        GraphLimits {
            nodes: 200_000,
            edges: 1_000_000,
        },
    )?;
    Ok(Arc::new(inc.analyze(cancel)?))
}

fn analyze_partition(
    id: SemanticId,
    p: &CasePlan,
    rows: &[SemanticId],
    columns: &[SemanticId],
    cancel: &Arc<AtomicBool>,
) -> Result<Arc<StructuralAnalysis>> {
    let matrix = p.jacobian_pattern();
    let mut edges = Vec::new();
    for (col, column) in p.columns().iter().enumerate() {
        for k in matrix.col_range(col) {
            let row = p.structure().rows()[matrix.row_idx()[k]].id;
            edges.push(Incidence {
                row,
                column: *column,
                instance: row,
                output: 0,
            });
        }
    }
    let graph = CaseIncidence::new(
        Scope::Conditional {
            model: id,
            rows: rows.iter().copied().collect(),
            columns: columns.iter().copied().collect(),
            inputs: p
                .structure()
                .variables()
                .iter()
                .filter(|v| v.fixed)
                .map(|v| v.port.id)
                .collect(),
        },
        rows.iter()
            .map(|id| Constraint {
                id: *id,
                lower: Some(0.0),
                upper: Some(0.0),
            })
            .collect(),
        columns.to_vec(),
        edges,
        Default::default(),
        GraphLimits {
            nodes: 200_000,
            edges: 1_000_000,
        },
    )?;
    Ok(Arc::new(graph.analyze(cancel)?))
}
/// Compiler-issued artifact specification. Its private fields prevent independent runtime keys.
#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactRequest {
    key: ContentHash,
    environment: ContentHash,
    demand: LocalDemand,
    body: Arc<PreparedBody>,
    support: Arc<pse_math::guarded::PreparedSupport>,
    optimization: Optimization,
    evaluation: EvaluationLimits,
}
impl ArtifactRequest {
    /// Preserve product accounting if the request outlives its preparation.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.body = Arc::new(self.body.as_ref().clone().with_owner(owner.clone()));
        self.support = Arc::new(self.support.as_ref().clone().with_owner(owner));
        self
    }
    /// Owned request descriptor storage, excluding shared body mathematics.
    pub fn descriptor_bytes(&self) -> usize {
        size_of::<Self>()
            + (self.demand.outputs.capacity() + self.demand.coordinates.capacity())
                * size_of::<usize>()
            + 64
    }
    /// Complete source/build/numerical identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Requested native optimizer concurrency.
    pub fn cores(&self) -> usize {
        self.optimization.cores
    }
    /// Maximum known numeric worker storage; foreign storage has a separate runtime allowance.
    pub fn scratch_limit(&self) -> usize {
        self.evaluation.scratch_bytes
    }
    /// Construct native programs only at the runtime effect boundary.
    pub fn build(&self, cancel: &Arc<AtomicBool>) -> std::result::Result<CompiledBody, MathError> {
        let _span = tracing::info_span!("pse.case.program_optimization").entered();
        let compiled = if self.demand.directional {
            self.support
                .compile_directional(self.optimization, self.evaluation, cancel)
        } else {
            self.support
                .compile(self.optimization, self.evaluation, cancel)
        };
        compiled.map_err(|error| error.with_derivative_body(None, self.demand.body))
    }
}
fn artifact_requests(
    p: &CasePlan,
    profile: Profile,
    environment: &ContentHash,
) -> Arc<Vec<ArtifactRequest>> {
    Arc::new(p.demands().iter().enumerate().map(|(index, d)|{
        let mut h=FramedHasher::new(if d.directional { pse_ids::Frame::MathDirectionalArtifactV1 } else { pse_ids::Frame::MathArtifactV5 });
        h.hash(&d.body).hash(environment).hash(&pse_buildinfo::SOURCE_IDENTITY).hash(&pse_buildinfo::BUILD_IDENTITY)
            .str("pse-math-evaluator-abi-v4;demanded-support;interpreted-f64;numerica-jets;real-algebra;no-jit;no-simd")
            .u64(d.order as u64).u64(d.outputs.len() as u64);
        for &x in &d.outputs{h.u64(x as u64);}h.u64(d.coordinates.len() as u64);for &x in &d.coordinates{h.u64(x as u64);}
        if d.directional { h.str("directional-first;one-taylor-axis;runtime-formal-seeds-v1"); }
        h.u64(p.supports()[index].remaining_occurrences() as u64);
        for x in [profile.optimization.cores,profile.optimization.horner_iterations,profile.optimization.cpe_iterations,profile.evaluation.derivative_components,profile.evaluation.operations,profile.evaluation.scratch_bytes,profile.evaluation.provider_calls]{h.u64(x as u64);}
        ArtifactRequest{key:h.finish_hash(),environment:*environment,demand:d.clone(),body:p.bodies()[&d.body].clone(),support:p.supports()[index].clone(),optimization:profile.optimization,evaluation:profile.evaluation}
    }).collect())
}
/// Pure general function projection; roles are supplied by the consuming physical workflow.
#[derive(Clone, Debug)]
pub struct PreparedFunctions {
    /// Shared assembly with explicit ordered derivative coordinates.
    pub plan: Arc<CasePlan>,
    /// Compiler-owned artifact identities, identical to ordinary algebraic compilation.
    pub artifacts: Arc<Vec<ArtifactRequest>>,
}
/// Owned result: neither Salsa handles nor native mutable state escape.
#[derive(Clone, Debug)]
pub struct PreparedCase {
    /// Consumed finite domain/class construction policy, continued by each class demand.
    pub class_proof_work: usize,
    /// Physical registry used by this immutable compilation and numerical resolution.
    pub quantities: pse_math::SharedAllocation<QuantityRegistry>,
    /// Library presolve projection with complete expression/value invalidation.
    pub presolve: pse_math::SharedAllocation<pse_math::presolve::Facts>,
    /// Exact consumed fixed/parameter values for the optional coefficient snapshot.
    pub coefficient_values: pse_math::SharedAllocation<Vec<(SemanticId, u64)>>,
    /// Pure class facts, tracked by the same compiler database as the case plan.
    pub facts: pse_math::facts::ProblemFacts,
    /// Complete physical/sparse case plan.
    pub plan: Arc<CasePlan>,
    /// Complete selected-case structural analysis.
    pub structure: pse_math::SharedAllocation<StructuralAnalysis>,
    /// Ordered compiler-issued requests.
    pub artifacts: pse_math::SharedAllocation<Vec<ArtifactRequest>>,
    /// Fresh source spans, separate from reusable arithmetic.
    pub occurrences: pse_math::SharedAllocation<BTreeMap<SemanticId, Vec<Occurrence>>>,
    /// Explicit optional coefficient projection; failure is not guessed as another class.
    pub coefficients: Option<pse_math::SharedAllocation<Coefficients>>,
    /// Rules of the parameters the bound structure determines (ADR-0104), prepared with the
    /// structure and shared by every value rebind.
    pub derivation: pse_math::SharedAllocation<Derivation>,
    /// Their values under the bound values, in canonical units; consumers evaluate with
    /// [`Self::complete`] values.
    pub derived: pse_math::SharedAllocation<Derived>,
}
impl PreparedCase {
    /// Prepare the discovered block alternative with the exact retained compiler
    /// environment and evaluator controls. No workspace or new profile is inferred.
    pub fn automatic_blocks(
        &self,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Alternative<Arc<Vec<PreparedBlock>>>> {
        let schedule = match self.automatic_alternatives(cancel)?.initialization {
            Alternative::Available(schedule) => schedule,
            Alternative::Unavailable(reason) => return Ok(Alternative::Unavailable(reason)),
        };
        let Some(request) = self.artifacts.first() else {
            return Ok(Alternative::Unavailable(
                "original compiler environment has no retained evaluator request".into(),
            ));
        };
        let profile = Profile {
            optimization: request.optimization,
            evaluation: request.evaluation,
            class_proof_work: self.class_proof_work,
            assembly: AssemblyLimits::default(),
        };
        Ok(Alternative::Available(conditional_blocks(
            &self.plan,
            &schedule,
            &self.quantities,
            profile,
            &request.environment,
            cancel,
        )?))
    }
    /// Conservative original-coordinate separator for class-native decomposition.
    /// This is a performance projection, never a numerical rank or elimination proof.
    pub fn automatic_separator(
        &self,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Option<pse_structural::incidence::Part>> {
        let dependencies = self.plan.dependencies(cancel)?;
        let free: BTreeSet<_> = self.plan.columns().iter().copied().collect();
        let mut owners: BTreeMap<SemanticId, BTreeSet<SemanticId>> = BTreeMap::new();
        for instance in self.plan.structure().instances() {
            for slot in &instance.slots {
                let id = slot.source();
                if free.contains(&id) {
                    owners.entry(id).or_default().insert(instance.instance);
                }
            }
        }
        let mut separator: BTreeSet<_> = owners
            .iter()
            .filter(|(_, owners)| owners.len() > 1)
            .map(|(id, _)| *id)
            .collect();
        let columns: BTreeMap<_, _> = self
            .structure
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(i, b)| b.members.columns.iter().map(move |c| (*c, i)))
            .collect();
        let rows: BTreeMap<_, _> = self
            .structure
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(i, b)| b.members.rows.iter().map(move |r| (*r, i)))
            .collect();
        for dependency in &dependencies {
            let used: BTreeSet<_> = dependency.execution.intersection(&free).copied().collect();
            match dependency.target {
                Target::Row(row) if rows.contains_key(&row) => {
                    for column in &used {
                        if columns.get(column) != rows.get(&row) {
                            separator.insert(*column);
                        }
                    }
                }
                _ => {
                    let blocks: BTreeSet<_> = used.iter().map(|c| columns.get(c)).collect();
                    if blocks.len() > 1 {
                        separator.extend(used);
                    }
                }
            }
        }
        if separator.is_empty() {
            return Ok(None);
        }
        let rows = dependencies
            .iter()
            .filter_map(|d| match d.target {
                Target::Row(row) if !d.execution.is_disjoint(&separator) => Some(row),
                _ => None,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(Some(pse_structural::incidence::Part {
            rows,
            columns: separator.into_iter().collect(),
        }))
    }
    /// Discover supported mathematical alternatives from the complete original
    /// projections. This does no native iteration or selected-sheet proof work.
    pub fn automatic_alternatives(
        &self,
        cancel: &Arc<AtomicBool>,
    ) -> Result<AutomaticAlternatives> {
        let dependencies = self.plan.dependencies(cancel)?;
        let execution: Vec<_> = dependencies
            .iter()
            .filter_map(|dependency| match dependency.target {
                Target::Row(row) => Some(
                    dependency
                        .execution
                        .iter()
                        .map(move |column| (row, *column)),
                ),
                Target::Objective(_) => None,
            })
            .flatten()
            .collect();
        let initialization = if !self.plan.structure().native().is_empty() {
            Alternative::Unavailable("native constraint locality is unavailable".into())
        } else if !self.plan.structure().objectives().is_empty()
            || self
                .plan
                .structure()
                .rows()
                .iter()
                .any(|row| !row.lower.is_finite() || row.lower != row.upper)
        {
            Alternative::Unavailable(
                "block initialization requires complete equalities without an objective".into(),
            )
        } else {
            match pse_structural::initialization::Plan::with_execution_dependencies(
                &self.structure,
                &execution,
                cancel,
            ) {
                Ok(plan) => Alternative::Available(plan),
                Err(pse_structural::projection::ProjectionError::Cancelled) => {
                    return Err(CompileError::Cancelled);
                }
                Err(cause) => Alternative::Unavailable(cause.to_string()),
            }
        };
        Ok(AutomaticAlternatives {
            initialization,
            dependencies,
        })
    }
    /// Resolve demanded coefficient-class evidence without preparing solver derivatives.
    /// Missing proof remains explicitly unresolved, and resource failure remains an error.
    pub fn prepare_class(&self, values: &CaseValues, cancel: &Arc<AtomicBool>) -> Result<Self> {
        use pse_math::presolve::{ClassEvidence, ClassRequest};
        let evidence = self.plan.class_evidence(
            values,
            &self.presolve,
            ClassRequest::Coefficients,
            self.presolve.proof_remaining,
            cancel,
        )?;
        let (facts, coefficients) = match evidence {
            ClassEvidence::Established {
                facts,
                coefficients,
            } => (facts, Some(Arc::new(coefficients))),
            ClassEvidence::RuledOut { facts, .. }
            | ClassEvidence::Pending { facts, .. }
            | ClassEvidence::RepresentationLimited { facts, .. } => (facts, None),
        };
        let mut result = self.clone();
        result.facts = pse_math::facts::ProblemFacts::from_plan(
            &self.plan,
            coefficients.as_deref(),
            &facts,
            cancel,
        )?;
        result.presolve = Arc::new(facts).into();
        result.coefficients = coefficients.map(Into::into);
        Ok(result)
    }
    /// Prepare first directional actions only after the consuming route requests them.
    /// Normal artifact keys and ordinals remain unchanged in the appended demand list.
    pub fn prepare_directional_actions(&self, cancel: &Arc<AtomicBool>) -> Result<Self> {
        let plan = Arc::new(self.plan.with_directional_actions(cancel)?);
        let profile = self.artifacts.first().map_or(
            Profile {
                assembly: self.plan.limits(),
                class_proof_work: self.class_proof_work,
                ..Profile::default()
            },
            |request| Profile {
                optimization: request.optimization,
                evaluation: request.evaluation,
                assembly: self.plan.limits(),
                class_proof_work: self.class_proof_work,
            },
        );
        let environment = self
            .artifacts
            .first()
            .map_or(ContentHash::from_bytes([0; 32]), |request| {
                request.environment
            });
        let mut result = self.clone();
        result.artifacts = artifact_requests(&plan, profile, &environment).into();
        result.plan = plan;
        Ok(result)
    }
    /// Prepare a stronger immutable kernel demand after contextual route selection.
    /// Scientific/value facts and original structural witnesses retain their owners.
    pub fn prepare_order(&self, order: DerivativeOrder, cancel: &Arc<AtomicBool>) -> Result<Self> {
        if order <= self.plan.order() {
            return Ok(self.clone());
        }
        let plan = Arc::new(self.plan.prepare_order(order, cancel)?);
        let profile = self.artifacts.first().map_or(
            Profile {
                assembly: self.plan.limits(),
                class_proof_work: self.class_proof_work,
                ..Profile::default()
            },
            |request| Profile {
                optimization: request.optimization,
                evaluation: request.evaluation,
                assembly: self.plan.limits(),
                class_proof_work: self.class_proof_work,
            },
        );
        // Existing artifact keys contain the physical environment consumed by admission.
        // Preserve it directly instead of reconstructing it from a public receipt.
        let environment = self
            .artifacts
            .first()
            .map_or(ContentHash::from_bytes([0; 32]), |request| {
                request.environment
            });
        let artifacts = artifact_requests(&plan, profile, &environment);
        let mut result = self.clone();
        result.facts.prepared_derivatives = order;
        result.plan = plan;
        result.artifacts = artifacts.into();
        Ok(result)
    }
    /// Known escaping payload, excluding opaque library/container overhead. This
    /// observation is separate from the bounded live Salsa generation allowance.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + 2048
            + self.plan.retained_bytes()
            + self.plan.owner_wrapper_bytes()
            + self.presolve.bytes()
            + self.quantities.allocation_extent()
            + self.structural_bytes()
            + self.binding_bytes()
            + self.provenance_bytes()
            + 2 * self.artifact_descriptor_bytes()
            + self
                .coefficients
                .as_ref()
                .map_or(0, |c| c.retained_bytes() + 64)
            + self.derivation.retained_bytes()
            + 64
            + self.derived.retained_bytes()
            + 64
    }
    /// Retained structural witness allocation.
    pub fn structural_bytes(&self) -> usize {
        self.structure.retained_bytes() + 64
    }
    /// Independently retained binding snapshot.
    pub fn binding_bytes(&self) -> usize {
        self.coefficient_values.capacity() * size_of::<(SemanticId, u64)>() + 64
    }
    /// Independently retained revision attribution.
    pub fn provenance_bytes(&self) -> usize {
        self.occurrences
            .values()
            .map(|v| 96 + v.capacity() * size_of::<Occurrence>())
            .sum::<usize>()
            + 64
    }
    /// Independently retained artifact descriptor vector; programs have separate owners.
    pub fn artifact_descriptor_bytes(&self) -> usize {
        self.artifacts
            .iter()
            .map(ArtifactRequest::descriptor_bytes)
            .sum::<usize>()
            + 64
    }
}
/// Candidate availability retains the actual mathematical reason for refusal.
#[derive(Clone, Debug)]
pub enum Alternative<T> {
    /// A supported lossless projection or initialization proposal, awaiting execution.
    Available(T),
    /// Current contracts do not establish this alternative; simultaneous remains valid.
    Unavailable(String),
}
/// Compiler-owned candidate inventory, distinct from native capability selection.
#[derive(Clone, Debug)]
pub struct AutomaticAlternatives {
    /// Complete block proposal; matching alone establishes no regular reduced sheet.
    pub initialization: Alternative<pse_structural::initialization::Plan>,
    /// Separate original numerical, execution/validity and objective/inequality targets.
    pub dependencies: Vec<pse_math::assembly::ContributionDependencies>,
}
/// The value-dependent products of a prepared plan: the library presolve projection, the
/// optional coefficient snapshot, the problem facts derived from both, and the fixed and
/// parameter values the snapshot assumes. Everything else in a [`PreparedCase`] depends on
/// structure only (A6).
struct ValueProducts {
    presolve: Arc<pse_math::presolve::Facts>,
    coefficients: Option<Arc<Coefficients>>,
    facts: pse_math::facts::ProblemFacts,
    assumptions: Vec<(SemanticId, u64)>,
}
impl ValueProducts {
    fn bind(
        plan: &CasePlan,
        values: &CaseValues,
        limit: usize,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Self> {
        let presolve = Arc::new(plan.presolve_domain_facts(values, limit, cancel)?);
        let coefficients: Option<Arc<Coefficients>> = None;
        // The convexity fact is established with the other value-dependent products, so a
        // rebind re-establishes it under the new values (ADR-0121, A6).
        let facts = pse_math::facts::ProblemFacts::from_plan(
            plan,
            coefficients.as_deref(),
            &presolve,
            cancel,
        )?;
        Ok(Self {
            presolve,
            coefficients,
            facts,
            assumptions: fixed_values(plan, values)?,
        })
    }
}
/// The fixed and parameter values of `plan`, bitwise.
fn fixed_values(plan: &CasePlan, values: &CaseValues) -> Result<Vec<(SemanticId, u64)>> {
    let structure = plan.structure();
    structure
        .parameters()
        .iter()
        .map(|p| p.id)
        .chain(
            structure
                .variables()
                .iter()
                .filter(|v| v.fixed)
                .map(|v| v.port.id),
        )
        .map(|id| {
            values
                .scalars
                .get(&id)
                .map(|v| (id, v.to_bits()))
                .ok_or_else(|| CompileError::Missing(format!("fixed or parameter value {id}")))
        })
        .collect()
}
/// Pure conditional block products; runtime attaches boundary values and evaluator owners.
#[derive(Clone, Debug)]
pub struct PreparedBlock {
    /// Actual finite domain/class construction policy of the declaring compiler profile.
    pub class_proof_work: usize,
    /// Conditional source rows/columns and explicit predecessor inputs.
    pub boundary: pse_structural::initialization::Block,
    /// Immutable plan containing only selected row demands.
    pub plan: Arc<CasePlan>,
    /// Structural analysis of the block's square system.
    pub structure: Arc<StructuralAnalysis>,
    /// Compiler-issued evaluator requests in plan order.
    pub artifacts: Arc<Vec<ArtifactRequest>>,
}
impl PreparedBlock {
    /// The block's solver view bound to `values` (A6): the plan, structural analysis and
    /// artifact requests are the block's own and shared; only the value-dependent products
    /// are built. Later values rebind it ([`PreparedCase::rebind`]).
    ///
    /// # Errors
    /// Values that do not bind the block, a failed projection, or cancellation.
    pub fn bind(
        &self,
        quantities: pse_math::SharedAllocation<QuantityRegistry>,
        values: &CaseValues,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PreparedCase> {
        self.plan.structure().validate_frozen_values(values)?;
        let bound = ValueProducts::bind(&self.plan, values, self.class_proof_work, cancel)?;
        Ok(PreparedCase {
            class_proof_work: self.class_proof_work,
            quantities,
            presolve: bound.presolve.into(),
            coefficient_values: Arc::new(bound.assumptions).into(),
            facts: bound.facts,
            plan: self.plan.clone(),
            structure: self.structure.clone().into(),
            artifacts: self.artifacts.clone().into(),
            occurrences: Arc::new(BTreeMap::new()).into(),
            coefficients: bound.coefficients.map(Into::into),
            derivation: Arc::new(Derivation::default()).into(),
            derived: Arc::new(Derived::default()).into(),
        })
    }
}
fn conditional_blocks(
    source: &CasePlan,
    schedule: &pse_structural::initialization::Plan,
    quantities: &QuantityRegistry,
    profile: Profile,
    environment: &ContentHash,
    cancel: &Arc<AtomicBool>,
) -> Result<Arc<Vec<PreparedBlock>>> {
    if source.structure().objective().is_some()
        || source
            .structure()
            .rows()
            .iter()
            .any(|r| !r.lower.is_finite() || r.lower != r.upper)
    {
        return Err(CompileError::Missing(
            "block initialization requires a complete equality selection without an objective"
                .into(),
        ));
    }
    let mut blocks = Vec::new();
    for b in &schedule.blocks {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        let plan = Arc::new(source.conditional(
            &b.members.rows.iter().copied().collect(),
            &b.members.columns.iter().copied().collect(),
            quantities,
            cancel,
        )?);
        let artifacts = artifact_requests(&plan, profile, environment);
        let structure = structural_plan(SemanticId::NIL, &plan, cancel)?;
        blocks.push(PreparedBlock {
            class_proof_work: profile.class_proof_work,
            boundary: b.clone(),
            plan,
            structure,
            artifacts,
        });
    }
    Ok(Arc::new(blocks))
}

/// Single-writer workspace. Callers serialize access; no database clones or partial batches escape.
pub struct CompilerWorkspace {
    db: CompilerDatabase,
    inventory: Inventory,
    inputs: CompilerContext,
    limits: WorkspaceLimits,

    generation: usize,
    modeling: Option<modeling::State>,
    /// Package data document inputs by identity (ADR-0125), each set only when its bytes
    /// change.
    documents: BTreeMap<SemanticId, modeling::DataDocumentInput>,
}
impl std::fmt::Debug for CompilerWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompilerWorkspace")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}
impl CompilerWorkspace {
    /// Freeze service retention before any revision enters this generation. Retention and
    /// accounting may change infrastructure availability, never admitted mathematical meaning.
    /// # Errors
    /// A revision was already published or an attachment was already selected.
    pub fn attach_body_retention(
        &mut self,
        retention: Arc<dyn ModelingBodyRetention>,
    ) -> Result<()> {
        if self.modeling.is_some() || self.db.body_retention.is_some() {
            return Err(CompileError::Math(Arc::new(MathError::Contract(
                "body retention must be selected once before revision publication".into(),
            ))));
        }
        self.db.body_retention = Some(retention);
        Ok(())
    }
    /// Admit finite inputs before allocating Salsa storage.
    pub fn new(inputs: CompilerContext, limits: WorkspaceLimits) -> Result<Self> {
        Self::with_events(inputs, limits, None)
    }
    fn with_events(
        inputs: CompilerContext,
        limits: WorkspaceLimits,
        event: Option<Box<dyn Fn(salsa::Event) + Send + Sync>>,
    ) -> Result<Self> {
        pse_math::initialize()?;
        validate(&inputs, limits)?;
        let mut db = CompilerDatabase {
            storage: salsa::Storage::new(event),
            cancel: Arc::default(),
            body_retention: None,
            body_refusal: Default::default(),
        };
        let inventory = inventory(&db, &inputs, pse_math::context()?.environment.identity());
        configure(&mut db, limits.query_values);
        Ok(Self {
            db,
            inventory,
            inputs,
            limits,

            generation: 0,
            modeling: None,
            documents: BTreeMap::new(),
        })
    }
    fn rebuild(&mut self, inputs: CompilerContext) -> Result<()> {
        let cancelled = self.db.cancellation_token().is_cancelled();
        let worker_cancel = self.db.cancel.clone();
        let generation = self.generation + 1;
        let mut replacement = Self::new(inputs, self.limits)?;
        replacement.db.body_retention = self.db.body_retention.clone();
        if let Some(state) = &self.modeling {
            replacement.publish_modeling_revision(state.revision.clone())?;
        }
        *self = replacement;
        self.generation = generation;
        self.db.cancel = worker_cancel;
        if cancelled {
            self.db.cancellation_token().cancel();
        }
        Ok(())
    }
    /// Cancellation token for this generation; an active caller owns its cancellation scope.
    pub fn cancellation_token(&self) -> salsa::CancellationToken {
        self.db.cancellation_token()
    }
    /// Derive conditional blocks from an immutable prepared case, including its fixed/free selection.
    pub fn prepare_bound_initialization(
        &self,
        case: &PreparedCase,
        profile: Profile,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<Vec<PreparedBlock>>> {
        let dependencies = case
            .plan
            .dependencies(cancel)?
            .into_iter()
            .filter_map(|dependency| match dependency.target {
                Target::Row(row) => Some(
                    dependency
                        .execution
                        .into_iter()
                        .map(move |column| (row, column)),
                ),
                Target::Objective(_) => None,
            })
            .flatten()
            .collect::<Vec<_>>();
        let schedule = pse_structural::initialization::Plan::with_execution_dependencies(
            &case.structure,
            &dependencies,
            cancel,
        )?;
        conditional_blocks(
            &case.plan,
            &schedule,
            &case.quantities,
            profile,
            self.inventory.environment(&self.db),
            cancel,
        )
    }
    fn trim_queries(&mut self) -> Result<()> {
        self.db.trigger_lru_eviction();
        // A single oversized result may escape in its returned Arc, but is not
        // retained as a memo for the next request. Input admission remains separate.
        if self.retention_exceeded() {
            self.rebuild(self.inputs.clone())?;
        }
        Ok(())
    }
    fn retention_exceeded(&self) -> bool {
        let (entries, bytes) = self.retention_usage();
        entries > self.limits.retained_entries || bytes > self.limits.retained_bytes
    }
    /// Actual retained ingredients and known bytes, including every owned query-key payload.
    /// Library heaps without Salsa heap callbacks remain outside this observation.
    pub fn retention_usage(&self) -> (usize, usize) {
        let report = <dyn Database>::memory_usage(&self.db);
        report.structs.iter().chain(report.queries.values()).fold(
            (0usize, 0usize),
            |(entries, bytes), info| {
                (
                    entries.saturating_add(info.count()),
                    bytes
                        .saturating_add(info.size_of_metadata())
                        .saturating_add(info.size_of_fields())
                        .saturating_add(info.heap_size_of_fields().unwrap_or(0)),
                )
            },
        )
    }
    /// Salsa-reported metadata/inline bytes, excluding unreported foreign allocations.
    pub fn metadata_bytes(&self) -> usize {
        let report = <dyn Database>::memory_usage(&self.db);
        report
            .structs
            .iter()
            .chain(report.queries.values())
            .fold(0usize, |n, i| {
                n.saturating_add(i.size_of_metadata())
                    .saturating_add(i.size_of_fields())
            })
    }
    /// Current generation number for resource diagnostics, never semantic identity.
    pub fn generation(&self) -> usize {
        self.generation
    }
}
fn inventory(db: &dyn CompilerDb, i: &CompilerContext, environment: ContentHash) -> Inventory {
    Inventory::builder(
        environment,
        i.quantities.clone(),
        i.preconditions.clone(),
        i.providers.clone(),
    )
    .environment_durability(salsa::Durability::HIGH)
    .quantities_durability(salsa::Durability::HIGH)
    .preconditions_durability(salsa::Durability::HIGH)
    .providers_durability(salsa::Durability::MEDIUM)
    .new(db)
}
fn configure(db: &mut CompilerDatabase, n: usize) {
    provider_query::set_lru_capacity(db, n);
}
fn validate(i: &CompilerContext, l: WorkspaceLimits) -> Result<()> {
    if [
        l.input_bytes,
        l.retained_entries,
        l.retained_bytes,
        l.query_values,
    ]
    .contains(&0)
    {
        return Err(CompileError::Limit("zero workspace allowance"));
    }
    let bytes = i
        .providers
        .iter()
        .fold(
            i.quantities.allocation_extent(),
            |bytes, (name, provider)| {
                let spec = provider.descriptor.spec();
                bytes
                    .saturating_add(name.len())
                    .saturating_add(spec.shapes.retained_bytes())
                    .saturating_add(
                        (spec.inputs.len() + spec.outputs.len())
                            .saturating_mul(size_of::<pse_kernels::Port>()),
                    )
            },
        )
        .saturating_add(
            i.preconditions
                .declarations()
                .iter()
                .fold(0usize, |bytes, p| {
                    bytes.saturating_add(128 + p.operand_positions.len() * 2)
                }),
        );
    if bytes > l.input_bytes {
        return Err(CompileError::Limit("context extent"));
    }
    for p in i.preconditions.declarations() {
        p.validate(&i.quantities).map_err(MathError::from)?;
    }
    for p in i.providers.values() {
        p.descriptor
            .spec()
            .validate(&i.quantities)
            .map_err(|e| CompileError::Math(Arc::new(MathError::Contract(e.to_string()))))?;
    }
    Ok(())
}

impl pse_model::diagnostic::DiagnosticProjection for CompileError {
    fn boundary_diagnostic(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        use pse_diagnostics::TypedDiagnostic;
        use pse_model::diagnostic::{SourceLocation, project_facts};
        match self {
            Self::Math(e) => e.boundary_diagnostic(stage),
            Self::Modeling(e) => e.boundary_diagnostic(),
            Self::Syntax {
                definition,
                source_index,
                error,
            } => {
                let mut d = project_facts(error.diagnostic_code(), error.diagnostic_facts(), stage);
                let (start, end) = match error.as_ref() {
                    pse_authoring::dsl::DslError::Syntax { span, .. } => {
                        (Some(span.start), Some(span.end))
                    }
                    pse_authoring::dsl::DslError::AmbiguousUnaryPower { offset }
                    | pse_authoring::dsl::DslError::NonFiniteNumber { offset } => {
                        (Some(*offset), Some(*offset))
                    }
                    pse_authoring::dsl::DslError::Budget { .. } => (None, None),
                };
                d.sources.push(*definition);
                d.rule = pse_diagnostics::DiagnosticRule::CompilerSyntax;
                d.locations.push(SourceLocation {
                    source: *definition,
                    revision: None,
                    path: format!("definition/{definition}/source/{source_index}"),
                    name: None,
                    start,
                    end,
                });
                d
            }
            Self::Structure(e) => project_facts(e.diagnostic_code(), e.diagnostic_facts(), stage),
            Self::Missing(_)
            | Self::ConditionalUnavailable(_)
            | Self::Cancelled
            | Self::Limit(_) => {
                project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage)
            }
        }
    }
    fn boundary_diagnostic_with_members(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
        bindings: &BTreeMap<SemanticId, SemanticId>,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        match self {
            Self::Math(error) => error.boundary_diagnostic_with_members(stage, bindings),
            _ => self.boundary_diagnostic(stage),
        }
    }
}

impl CompileError {
    /// Retained error extent, including original scientific/compiler causes.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(match self {
            Self::Missing(value) | Self::ConditionalUnavailable(value) => value.capacity(),
            Self::Math(error) => error.retained_bytes(),
            Self::Syntax { error, .. } => size_of_val(error.as_ref()) + error.to_string().len(),
            Self::Modeling(error) => size_of_val(error) + error.to_string().len(),
            Self::Structure(error) => size_of_val(error) + error.to_string().len(),
            Self::Cancelled | Self::Limit(_) => 0,
        })
    }
}

#[cfg(test)]
mod directional_identity_tests {
    use super::*;
    use pse_math::{
        binding::{Contribution, InstanceBinding, Row, SlotBinding, Variable},
        typed::{Binary, BodyBuilder, BodyLimits},
    };
    use pse_quantity::{
        IndexSet,
        standard::{StandardInvariantChecker, ids, standard_registry},
    };
    #[test]
    fn directional_requests_append_distinct_keys_and_build_one_axis_with_full_formal_support() {
        let registry = standard_registry().unwrap();
        let quantity = ids::quantity("neutral");
        let id = |n| SemanticId::from_bytes([n; 16]);
        let port = |n| pse_kernels::Port {
            id: id(n),
            quantity,
            unit: registry.quantity_type(quantity).unwrap().canonical_unit,
        };
        let mut builder = BodyBuilder::new(
            pse_math::initialize().unwrap(),
            &registry,
            &StandardInvariantChecker,
            3,
            BodyLimits::default(),
        )
        .unwrap();
        let a = builder.input(0, quantity, IndexSet::new(), id(10)).unwrap();
        let b = builder.input(1, quantity, IndexSet::new(), id(11)).unwrap();
        let c = builder.input(2, quantity, IndexSet::new(), id(12)).unwrap();
        let ab = builder.binary(Binary::Mul, a, b, None, id(13)).unwrap();
        let result = builder.binary(Binary::Add, ab, c, None, id(14)).unwrap();
        let body = Arc::new(builder.prepare(&[result]).unwrap());
        let key = ContentHash::from_bytes([17; 32]);
        let structure = Arc::new(
            CaseStructure::new(
                (1..=3)
                    .map(|n| Variable {
                        port: port(n),
                        fixed: false,
                        domain: pse_model::generated::enums::ModelingVariableDomain::Continuous,
                        lower: None,
                        upper: None,
                    })
                    .collect(),
                vec![],
                vec![InstanceBinding {
                    checked_members: Default::default(),
                    instance: id(7),
                    body: key,
                    slots: (1..=3)
                        .map(|n| SlotBinding::new(&port(n), &port(n), &registry).unwrap())
                        .collect(),
                    contributions: vec![Contribution {
                        output: 0,
                        target: Target::Row(id(8)),
                        scale: 1.0,
                    }],
                }],
                vec![Row {
                    id: id(8),
                    quantity,
                    lower: 0.0,
                    upper: 0.0,
                }],
                None,
                CaseLimits::default(),
            )
            .unwrap(),
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let plan = CasePlan::prepare(
            structure,
            BTreeMap::from([(key, body)]),
            &registry,
            DerivativeOrder::Value,
            AssemblyLimits::default(),
            &cancel,
        )
        .unwrap();
        let profile = Profile {
            evaluation: EvaluationLimits {
                derivative_components: 2,
                ..EvaluationLimits::default()
            },
            ..Profile::default()
        };
        let environment = ContentHash::from_bytes([19; 32]);
        let normal = artifact_requests(&plan, profile, &environment);
        let directional = plan.with_directional_actions(&cancel).unwrap();
        let requests = artifact_requests(&directional, profile, &environment);
        assert_eq!(normal.as_slice(), &requests[..normal.len()]);
        assert!(requests.len() > normal.len());
        assert_ne!(requests[normal.len()].key(), normal[0].key());
        let programs = requests
            .iter()
            .map(|r| Arc::new(r.build(&cancel).unwrap()))
            .collect();
        let assembly = Arc::new(Arc::new(directional).assemble(programs).unwrap());
        let inputs = CaseValues {
            scalars: BTreeMap::from([(id(1), 2.0), (id(2), 3.0), (id(3), 4.0)]),
        };
        let mut output = [0.0];
        assembly
            .worker(BTreeMap::new(), cancel)
            .jacobian_product(&inputs, &[5.0, 7.0, 11.0], &mut output)
            .unwrap();
        assert_eq!(output, [40.0]);
        assert_eq!(requests[normal.len()].support.coordinates(), [0, 1, 2]);
    }
}

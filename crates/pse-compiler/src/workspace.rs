// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Salsa owns semantic dependencies. Native compilation is an explicit effect outside queries.
use crate::typed_math::{AdmittedBody, Domain, Formal, Group, Occurrence, ProviderCall, Request};
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
use pse_quantity::{PhysicalPreconditions, QuantityRegistry, QuantityTypeId, UnitId};
use pse_structural::{
    incidence::{CaseIncidence, Constraint, Incidence, StructuralAnalysis},
    projection::{GraphLimits, Scope},
};
use salsa::{Database, Setter};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Owned definition source and selected lexical bindings. Values are not compiler input.
#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    /// Ordered source outputs.
    pub sources: Vec<String>,
    /// Ordered local formals.
    pub formals: Vec<Formal>,
    /// Selected domain names (missing differs from empty).
    pub domains: Vec<String>,
    /// Selected indexed group names.
    pub groups: Vec<String>,
    /// Selected immutable provider descriptors.
    pub providers: Vec<String>,
    /// Resolved unit spellings.
    pub units: BTreeMap<String, UnitId>,
    /// Contextual source literal contracts.
    pub literals: BTreeMap<(u32, u32), QuantityTypeId>,
    /// Finite construction limits.
    pub limits: BodyLimits,
}
/// Case declarations plus definition references. Instance body hashes are replaced by compiler output.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    /// Admitted complete selected case.
    pub structure: Arc<CaseStructure>,
    /// Every instance maps to a definition.
    pub definitions: BTreeMap<SemanticId, SemanticId>,
}
/// Atomic application-visible input batch. No executable factory is retained here.
#[derive(Clone, Debug, PartialEq)]
pub struct Inputs {
    /// Complete physical flowsheet declarations; absence is a tracked dependency.
    pub flows: BTreeMap<SemanticId, pse_structural::flowsheet::Declaration>,
    /// Actual admitted physical meanings.
    pub quantities: Arc<QuantityRegistry>,
    /// Actual immutable prerequisites.
    pub preconditions: Arc<PhysicalPreconditions>,
    /// Definitions including source provenance.
    pub definitions: BTreeMap<SemanticId, Definition>,
    /// Actual domain membership.
    pub domains: BTreeMap<String, Domain>,
    /// Actual finite/ragged membership and binding.
    pub groups: BTreeMap<String, Group>,
    /// Admitted descriptors; factories belong to runtime.
    pub providers: BTreeMap<String, ProviderCall>,
    /// Selected case inventory.
    pub cases: BTreeMap<SemanticId, Case>,
    /// Fixed and parameter values; free trial values are ignored by coefficient queries.
    pub values: BTreeMap<SemanticId, f64>,
}
/// Shared complete identity of actual physical declarations and prerequisites.
pub fn physical_identity(
    quantities: &QuantityRegistry,
    preconditions: &PhysicalPreconditions,
) -> ContentHash {
    crate::physical_identity::identity(quantities, preconditions)
}
/// Evaluator-affecting profile, separate from semantic preparation and resource admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Profile {
    /// Optimization controls.
    pub optimization: Optimization,
    /// Local numeric limits affecting evaluator admission.
    pub evaluation: EvaluationLimits,
}
/// Finite workspace metadata policy. Generations never escape as Salsa handles.
#[derive(Clone, Copy, Debug)]
pub struct WorkspaceLimits {
    /// Maximum input inventory entries.
    pub entries: usize,
    /// Maximum admitted input extent (including conservative owned metadata allowance).
    pub input_bytes: usize,
    /// Publish count before rebuilding a fresh generation from current inputs.
    pub revisions: usize,
    /// Prepare calls before generation reconstruction, bounding query keys too.
    pub preparations: usize,
    /// Retained values per expensive query.
    pub query_values: usize,
}
impl Default for WorkspaceLimits {
    fn default() -> Self {
        Self {
            entries: 4096,
            input_bytes: 2 << 30,
            revisions: 64,
            preparations: 256,
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
    /// Source parser failure.
    #[error("source syntax: {0}")]
    Syntax(Arc<pse_authoring::dsl::DslError>),
    /// Physical/math rejection.
    #[error(transparent)]
    Math(Arc<MathError>),
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
    code(this) { Some(match this { Self::Cancelled=>pse_diagnostics::DiagnosticCode::RuntimeCancelled,Self::Limit(_)=>pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,_=>pse_diagnostics::DiagnosticCode::CompileMath }) },
    forward(this) { match this {Self::Math(e)=>Some(e.as_ref()),Self::Structure(e)=>Some(e),_=>None} },
    help(_this) { None },related(_this) { None },source(_this) { None }
}
impl PartialEq for CompileError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Missing(a), Self::Missing(b)) => a == b,
            (Self::Syntax(a), Self::Syntax(b)) => Arc::ptr_eq(a, b),
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
#[salsa::db]
trait CompilerDb: Database {
    fn cancel(&self) -> &Arc<AtomicBool>;
}
#[salsa::db]
#[derive(Clone, Default)]
struct CompilerDatabase {
    storage: salsa::Storage<Self>,
    cancel: Arc<AtomicBool>,
}
#[salsa::db]
impl Database for CompilerDatabase {}
#[salsa::db]
impl CompilerDb for CompilerDatabase {
    fn cancel(&self) -> &Arc<AtomicBool> {
        &self.cancel
    }
}
fn checkpoint(db: &dyn CompilerDb) {
    if db.cancel().load(Ordering::Acquire) {
        db.cancellation_token().cancel();
    }
    db.unwind_if_revision_cancelled();
}
fn math_result<T>(db: &dyn CompilerDb, result: std::result::Result<T, MathError>) -> Result<T> {
    checkpoint(db);
    result.map_err(CompileError::from)
}
#[salsa::input]
struct Inventory {
    flows: BTreeMap<SemanticId, pse_structural::flowsheet::Declaration>,
    quantities: Arc<QuantityRegistry>,
    preconditions: Arc<PhysicalPreconditions>,
    definitions: BTreeMap<SemanticId, Definition>,
    domains: BTreeMap<String, Domain>,
    groups: BTreeMap<String, Group>,
    providers: BTreeMap<String, ProviderCall>,
    cases: BTreeMap<SemanticId, Case>,
    values: BTreeMap<SemanticId, f64>,
}
#[salsa::tracked(returns(clone), lru = 64)]
fn definition(db: &dyn CompilerDb, i: Inventory, id: SemanticId) -> Option<Definition> {
    i.definitions(db).get(&id).cloned()
}
#[salsa::tracked(returns(clone), lru = 64)]
fn domain(db: &dyn CompilerDb, i: Inventory, name: String) -> Option<Domain> {
    i.domains(db).get(&name).cloned()
}
#[salsa::tracked(returns(clone), lru = 64)]
fn group(db: &dyn CompilerDb, i: Inventory, name: String) -> Option<Group> {
    i.groups(db).get(&name).cloned()
}
#[salsa::tracked(returns(clone), lru = 64)]
fn provider(db: &dyn CompilerDb, i: Inventory, name: String) -> Option<ProviderCall> {
    i.providers(db).get(&name).cloned()
}
#[salsa::tracked(returns(clone), lru = 64)]
fn case(db: &dyn CompilerDb, i: Inventory, id: SemanticId) -> Option<Case> {
    i.cases(db).get(&id).cloned()
}
#[salsa::tracked(returns(clone), lru = 64)]
fn flow_declaration(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
) -> Option<pse_structural::flowsheet::Declaration> {
    i.flows(db).get(&id).cloned()
}
#[salsa::tracked(returns(clone), lru = 64)]
fn flow_graph(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
) -> Result<Arc<pse_structural::flowsheet::FlowGraph>> {
    checkpoint(db);
    let d = flow_declaration(db, i, id)
        .ok_or_else(|| CompileError::Missing(format!("flowsheet {id}")))?;
    let g = pse_structural::flowsheet::FlowGraph::admit(
        d,
        i.quantities(db),
        GraphLimits {
            nodes: 100_000,
            edges: 1_000_000,
        },
    )?;
    checkpoint(db);
    Ok(Arc::new(g))
}
#[salsa::tracked(returns(clone), lru = 64)]
fn initialization_plan(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
) -> Result<Arc<pse_structural::initialization::Plan>> {
    Ok(Arc::new(
        pse_structural::initialization::Plan::from_analysis(structure(db, i, id)?.as_ref())?,
    ))
}
#[salsa::tracked(returns(copy), lru = 64)]
fn physical_key(db: &dyn CompilerDb, i: Inventory) -> ContentHash {
    crate::physical_identity::identity(i.quantities(db), i.preconditions(db))
}
#[salsa::tracked(returns(clone), lru = 64)]
fn admitted(db: &dyn CompilerDb, i: Inventory, id: SemanticId) -> Result<Arc<AdmittedBody>> {
    let _span = tracing::info_span!("pse.case.definition_admission").entered();
    checkpoint(db);
    let d =
        definition(db, i, id).ok_or_else(|| CompileError::Missing(format!("definition {id}")))?;
    let expressions = d
        .sources
        .iter()
        .map(|s| pse_authoring::dsl::parse_expr(s).map_err(|e| CompileError::Syntax(Arc::new(e))))
        .collect::<Result<Vec<_>>>()?;
    let domains = d
        .domains
        .iter()
        .map(|n| {
            domain(db, i, n.clone())
                .map(|v| (n.clone(), v))
                .ok_or_else(|| CompileError::Missing(format!("domain {n}")))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let groups = d
        .groups
        .iter()
        .map(|n| {
            group(db, i, n.clone())
                .map(|v| (n.clone(), v))
                .ok_or_else(|| CompileError::Missing(format!("group {n}")))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let providers = d
        .providers
        .iter()
        .map(|n| {
            provider(db, i, n.clone())
                .map(|v| (n.clone(), v))
                .ok_or_else(|| CompileError::Missing(format!("provider {n}")))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let request = Request {
        definition: id,
        expressions: &expressions,
        formals: &d.formals,
        domains: &domains,
        groups: &groups,
        providers: &providers,
        units: &d.units,
        literals: &d.literals,
        physical: physical_key(db, i),
        structure: ContentHash::from_bytes([0; 32]),

        limits: d.limits,
    };
    let result = math_result(
        db,
        request.admit(i.quantities(db), i.preconditions(db).as_ref(), db.cancel()),
    )?;
    checkpoint(db);
    Ok(Arc::new(result))
}
/// Arithmetic product deliberately excludes diagnostic byte spans from equality.
#[derive(Clone, Debug, PartialEq)]
struct SemanticBody {
    key: ContentHash,
    body: Arc<PreparedBody>,
}
#[salsa::tracked(returns(clone), lru = 64)]
fn semantic_body(db: &dyn CompilerDb, i: Inventory, id: SemanticId) -> Result<Arc<SemanticBody>> {
    let a = admitted(db, i, id)?;
    Ok(Arc::new(SemanticBody {
        key: a.spec.key(),
        body: a.math.clone(),
    }))
}
#[derive(Clone, Debug)]
struct Planned(Arc<CasePlan>);
impl PartialEq for Planned {
    fn eq(&self, other: &Self) -> bool {
        self.0.structure() == other.0.structure()
            && self.0.bodies() == other.0.bodies()
            && self.0.columns() == other.0.columns()
            && self.0.demands() == other.0.demands()
    }
}
#[salsa::tracked(returns(clone), lru = 64)]
fn plan(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
    order: DerivativeOrder,
) -> Result<Planned> {
    let c = case(db, i, id).ok_or_else(|| CompileError::Missing(format!("case {id}")))?;
    let mut bodies = BTreeMap::new();
    let mut instances = c.structure.instances().to_vec();
    for instance in &mut instances {
        checkpoint(db);
        let def = c.definitions.get(&instance.instance).ok_or_else(|| {
            CompileError::Missing(format!("instance definition {}", instance.instance))
        })?;
        let b = semantic_body(db, i, *def)?;
        instance.body = b.key;
        if instance.slots.len() != b.body.input_count() {
            return Err(MathError::Contract("instance arity".into()).into());
        }
        for (slot, quantity) in instance.slots.iter_mut().zip(b.body.input_quantities()) {
            if let Some(q) = quantity {
                *slot = slot.readmit(*q, i.quantities(db))?;
            }
        }
        bodies.insert(b.key, b.body.clone());
    }
    let structure = Arc::new(CaseStructure::new(
        c.structure.variables().to_vec(),
        c.structure.parameters().to_vec(),
        instances,
        c.structure.rows().to_vec(),
        c.structure.objective().cloned(),
        CaseLimits::default(),
    )?);
    let _span = tracing::info_span!("pse.case.sparse_plan").entered();
    Ok(Planned(Arc::new(math_result(
        db,
        CasePlan::prepare(
            structure,
            bodies,
            i.quantities(db),
            order,
            AssemblyLimits::default(),
            db.cancel(),
        ),
    )?)))
}
#[salsa::tracked(returns(clone), lru=64, heap_size=structure_heap)]
fn structure(db: &dyn CompilerDb, i: Inventory, id: SemanticId) -> Result<Arc<StructuralAnalysis>> {
    let p = plan(db, i, id, DerivativeOrder::Value)?.0;
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
    for b in p.structure().instances() {
        for c in &b.contributions {
            if let Target::Row(row) = c.target {
                for &slot in &p.bodies()[&b.body].support().first[c.output] {
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
        .objective_support()
        .into_iter()
        .map(|c| columns[c])
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
    checkpoint(db);
    let result = inc.analyze(db.cancel());
    checkpoint(db);
    let result = result?;
    Ok(Arc::new(result))
}
#[salsa::tracked(returns(clone), lru = 64)]
fn algebraic_partition(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
    rows: Vec<SemanticId>,
    columns: Vec<SemanticId>,
) -> Result<Arc<StructuralAnalysis>> {
    let p = function_plan(
        db,
        i,
        id,
        rows.clone(),
        columns.clone(),
        DerivativeOrder::First,
    )?
    .0;
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
        Scope::Whole(id),
        rows.iter()
            .map(|id| Constraint {
                id: *id,
                lower: Some(0.0),
                upper: Some(0.0),
            })
            .collect(),
        columns.clone(),
        edges,
        Default::default(),
        GraphLimits {
            nodes: 200_000,
            edges: 1_000_000,
        },
    )?;
    checkpoint(db);
    let result = graph.analyze(db.cancel());
    checkpoint(db);
    Ok(Arc::new(result?))
}
fn structure_heap(result: &Result<Arc<StructuralAnalysis>>) -> usize {
    result.as_ref().map_or(0, |a| {
        let part = |p: &pse_structural::incidence::Part| {
            (p.rows.capacity() + p.columns.capacity()) * size_of::<SemanticId>()
        };
        size_of::<StructuralAnalysis>()
            + a.matching.capacity() * size_of::<(SemanticId, SemanticId)>()
            + part(&a.over)
            + part(&a.under)
            + part(&a.square)
            + a.contributions.capacity() * size_of::<Incidence>()
            + a.blocks.capacity() * size_of::<pse_structural::incidence::Block>()
            + a.blocks.iter().map(|b| part(&b.members)).sum::<usize>()
    })
}
/// Compiler-issued artifact specification. Its private fields prevent independent runtime keys.
#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactRequest {
    key: ContentHash,
    demand: LocalDemand,
    body: Arc<PreparedBody>,
    profile: Profile,
}
impl ArtifactRequest {
    /// Complete source/build/numerical identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Requested native optimizer concurrency.
    pub fn cores(&self) -> usize {
        self.profile.optimization.cores
    }
    /// Maximum known numeric worker storage; foreign storage has a separate runtime allowance.
    pub fn scratch_limit(&self) -> usize {
        self.profile.evaluation.scratch_bytes
    }
    /// Construct native programs only at the runtime effect boundary.
    pub fn build(&self, cancel: &Arc<AtomicBool>) -> std::result::Result<CompiledBody, MathError> {
        let _span = tracing::info_span!("pse.case.program_optimization").entered();
        self.body.compile(
            &self.demand.outputs,
            &self.demand.coordinates,
            self.demand.order,
            self.profile.optimization,
            self.profile.evaluation,
            cancel,
        )
    }
}
#[salsa::tracked(returns(clone), lru = 64)]
fn artifacts(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
    order: DerivativeOrder,
    profile: Profile,
) -> Result<Arc<Vec<ArtifactRequest>>> {
    let p = plan(db, i, id, order)?.0;
    Ok(artifact_requests(&p, profile))
}
fn artifact_requests(p: &CasePlan, profile: Profile) -> Arc<Vec<ArtifactRequest>> {
    Arc::new(p.demands().iter().map(|d|{
        let mut h=FramedHasher::new("pse.math.artifact.v3");
        h.hash(&d.body).hash(&pse_buildinfo::SOURCE_IDENTITY).hash(&pse_buildinfo::BUILD_IDENTITY)
            .str("pse-math-evaluator-abi-v3;interpreted-f64;numerica-jets;real-algebra;no-jit;no-simd")
            .u64(d.order as u64).u64(d.outputs.len() as u64);
        for &x in &d.outputs{h.u64(x as u64);}h.u64(d.coordinates.len() as u64);for &x in &d.coordinates{h.u64(x as u64);}
        for x in [profile.optimization.cores,profile.optimization.horner_iterations,profile.optimization.cpe_iterations,profile.evaluation.derivative_components,profile.evaluation.operations,profile.evaluation.scratch_bytes,profile.evaluation.provider_calls]{h.u64(x as u64);}
        ArtifactRequest{key:h.finish_hash(),demand:d.clone(),body:p.bodies()[&d.body].clone(),profile}
    }).collect())
}
#[salsa::tracked(returns(clone), lru = 64)]
fn function_plan(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
    outputs: Vec<SemanticId>,
    coordinates: Vec<SemanticId>,
    order: DerivativeOrder,
) -> Result<Planned> {
    let source = plan(db, i, id, DerivativeOrder::Value)?.0;
    Ok(Planned(Arc::new(math_result(
        db,
        source.functions(&outputs, coordinates, i.quantities(db), order, db.cancel()),
    )?)))
}
/// Pure general function projection; roles are supplied by the consuming physical workflow.
#[derive(Clone, Debug)]
pub struct PreparedFunctions {
    /// Shared assembly with explicit ordered derivative coordinates.
    pub plan: Arc<CasePlan>,
    /// Compiler-owned artifact identities, identical to ordinary algebraic compilation.
    pub artifacts: Arc<Vec<ArtifactRequest>>,
}
#[salsa::tracked(returns(clone), lru = 64)]
fn assumptions(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
) -> Result<Vec<(SemanticId, u64)>> {
    let p = plan(db, i, id, DerivativeOrder::Value)?.0;
    let mut result = BTreeMap::new();
    for b in p.structure().instances() {
        for s in &b.slots {
            if p.columns().binary_search(&s.source()).is_err() {
                let v = i.values(db).get(&s.source()).copied().ok_or_else(|| {
                    CompileError::Missing(format!("coefficient parameter {}", s.source()))
                })?;
                result.insert(s.source(), v.to_bits());
            }
        }
    }
    Ok(result.into_iter().collect())
}
#[derive(Clone, Debug)]
struct CoefficientProduct(Arc<Coefficients>);
impl PartialEq for CoefficientProduct {
    fn eq(&self, other: &Self) -> bool {
        self.0.assumptions == other.0.assumptions
    }
}
#[salsa::tracked(returns(clone), lru = 64)]
fn coefficients(db: &dyn CompilerDb, i: Inventory, id: SemanticId) -> Result<CoefficientProduct> {
    let values = CaseValues {
        scalars: assumptions(db, i, id)?
            .into_iter()
            .map(|(k, v)| (k, f64::from_bits(v)))
            .collect(),
    };
    checkpoint(db);
    let result = math_result(
        db,
        plan(db, i, id, DerivativeOrder::Value)?.0.coefficients(
            &values,
            Optimization::default(),
            100_000,
            db.cancel(),
        ),
    )?;
    checkpoint(db);
    Ok(CoefficientProduct(Arc::new(result)))
}
#[salsa::tracked(returns(clone), lru = 64)]
fn problem_facts(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
    order: DerivativeOrder,
    with_coefficients: bool,
) -> Result<pse_math::facts::ProblemFacts> {
    let p = plan(db, i, id, order)?.0;
    let c = if with_coefficients {
        Some(coefficients(db, i, id)?.0)
    } else {
        None
    };
    math_result(
        db,
        pse_math::facts::ProblemFacts::from_plan(&p, c.as_deref()),
    )
}
#[derive(Clone, Debug)]
struct PresolveProduct(Arc<pse_math::presolve::Facts>);
impl PartialEq for PresolveProduct {
    fn eq(&self, other: &Self) -> bool {
        self.0.key == other.0.key
    }
}
#[salsa::tracked(returns(clone), lru = 64, heap_size=presolve_heap)]
fn presolve_facts(db: &dyn CompilerDb, i: Inventory, id: SemanticId) -> Result<PresolveProduct> {
    let values = CaseValues {
        scalars: assumptions(db, i, id)?
            .into_iter()
            .map(|(k, v)| (k, f64::from_bits(v)))
            .collect(),
    };
    let p = plan(db, i, id, DerivativeOrder::Value)?.0;
    checkpoint(db);
    let facts = math_result(db, p.presolve_facts(&values, 100_000, db.cancel()))?;
    checkpoint(db);
    Ok(PresolveProduct(Arc::new(facts)))
}
fn presolve_heap(value: &Result<PresolveProduct>) -> usize {
    value.as_ref().map_or(0, |p| p.0.bytes())
}
/// Owned result: neither Salsa handles nor native mutable state escape.
#[derive(Clone, Debug)]
pub struct PreparedCase {
    /// Library presolve projection with complete expression/value invalidation.
    pub presolve: Arc<pse_math::presolve::Facts>,
    /// Exact consumed fixed/parameter values for the optional coefficient snapshot.
    pub coefficient_values: Vec<(SemanticId, u64)>,
    /// Pure class facts, tracked by the same compiler database as the case plan.
    pub facts: pse_math::facts::ProblemFacts,
    /// Complete physical/sparse case plan.
    pub plan: Arc<CasePlan>,
    /// Complete selected-case structural analysis.
    pub structure: Arc<StructuralAnalysis>,
    /// Ordered compiler-issued requests.
    pub artifacts: Arc<Vec<ArtifactRequest>>,
    /// Fresh source spans, separate from reusable arithmetic.
    pub occurrences: BTreeMap<SemanticId, Vec<Occurrence>>,
    /// Explicit optional coefficient projection; failure is not guessed as another class.
    pub coefficients: Option<Arc<Coefficients>>,
}
/// Pure conditional block products; runtime attaches boundary values and evaluator owners.
#[derive(Clone, Debug)]
pub struct PreparedBlock {
    /// Conditional source rows/columns and explicit predecessor inputs.
    pub boundary: pse_structural::initialization::Block,
    /// Immutable plan containing only selected row demands.
    pub plan: Arc<CasePlan>,
    /// Compiler-issued evaluator requests in plan order.
    pub artifacts: Arc<Vec<ArtifactRequest>>,
}
#[derive(Clone, Debug)]
struct InitializationBlocks(Arc<Vec<PreparedBlock>>);
impl PartialEq for InitializationBlocks {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self.0.iter().zip(other.0.iter()).all(|(a, b)| {
                a.boundary == b.boundary
                    && a.plan.structure().key() == b.plan.structure().key()
                    && a.artifacts == b.artifacts
            })
    }
}
#[salsa::tracked(returns(clone), lru = 64)]
fn initialization_blocks(
    db: &dyn CompilerDb,
    i: Inventory,
    id: SemanticId,
    profile: Profile,
) -> Result<InitializationBlocks> {
    let source = plan(db, i, id, DerivativeOrder::First)?.0;
    if source
        .structure()
        .rows()
        .iter()
        .any(|r| !r.lower.is_finite() || r.lower != r.upper)
    {
        return Err(CompileError::Missing(
            "block initialization requires a complete equality selection".into(),
        ));
    }
    let schedule = initialization_plan(db, i, id)?;
    let mut blocks = Vec::new();
    for b in &schedule.blocks {
        checkpoint(db);
        let p = Arc::new(math_result(
            db,
            source.conditional(
                &b.members.rows.iter().copied().collect(),
                &b.members.columns.iter().copied().collect(),
                i.quantities(db),
                db.cancel(),
            ),
        )?);
        let requests = artifact_requests(&p, profile);
        blocks.push(PreparedBlock {
            boundary: b.clone(),
            plan: p,
            artifacts: requests,
        });
    }
    Ok(InitializationBlocks(Arc::new(blocks)))
}
/// Single-writer workspace. Callers serialize access; no database clones or partial batches escape.
pub struct CompilerWorkspace {
    db: CompilerDatabase,
    inventory: Inventory,
    inputs: Inputs,
    limits: WorkspaceLimits,
    revisions: usize,
    calls: usize,
    generation: usize,
}
impl std::fmt::Debug for CompilerWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompilerWorkspace")
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}
impl CompilerWorkspace {
    /// Admit finite inputs before allocating Salsa storage.
    pub fn new(inputs: Inputs, limits: WorkspaceLimits) -> Result<Self> {
        Self::with_events(inputs, limits, None)
    }
    fn with_events(
        inputs: Inputs,
        limits: WorkspaceLimits,
        event: Option<Box<dyn Fn(salsa::Event) + Send + Sync>>,
    ) -> Result<Self> {
        validate(&inputs, limits)?;
        let mut db = CompilerDatabase {
            storage: salsa::Storage::new(event),
            cancel: Arc::default(),
        };
        let inventory = inventory(&db, &inputs);
        configure(&mut db, limits.query_values);
        Ok(Self {
            db,
            inventory,
            inputs,
            limits,
            revisions: 0,
            calls: 0,
            generation: 0,
        })
    }
    /// Application-visible atomic publication: validation precedes all setters.
    pub fn publish(&mut self, next: Inputs) -> Result<()> {
        validate(&next, self.limits)?;
        if self.inputs == next {
            return Ok(());
        }
        if self.revisions >= self.limits.revisions
            || self.metadata_bytes() > self.limits.input_bytes / 4
        {
            self.rebuild(next)?;
            return Ok(());
        }
        macro_rules! field {
            ($name:ident,$setter:ident) => {
                if self.inputs.$name != next.$name {
                    self.inventory.$setter(&mut self.db).to(next.$name.clone());
                }
            };
        }
        field!(quantities, set_quantities);
        field!(preconditions, set_preconditions);
        field!(definitions, set_definitions);
        field!(domains, set_domains);
        field!(groups, set_groups);
        field!(providers, set_providers);
        field!(cases, set_cases);
        field!(values, set_values);
        field!(flows, set_flows);
        self.inputs = next;
        self.revisions += 1;
        self.db.trigger_lru_eviction();
        Ok(())
    }
    fn rebuild(&mut self, inputs: Inputs) -> Result<()> {
        let cancelled = self.db.cancellation_token().is_cancelled();
        let generation = self.generation + 1;
        let replacement = Self::new(inputs, self.limits)?;
        *self = replacement;
        self.generation = generation;
        if cancelled {
            self.db.cancellation_token().cancel();
        }
        Ok(())
    }
    /// Cancellation token for this generation; an active caller owns its cancellation scope.
    pub fn cancellation_token(&self) -> salsa::CancellationToken {
        self.db.cancellation_token()
    }
    /// Analyze a mass-zero partition using a bounded pure library-matching query.
    pub fn analyze_dynamic_partition(
        &mut self,
        id: SemanticId,
        rows: Vec<SemanticId>,
        columns: Vec<SemanticId>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Arc<StructuralAnalysis>> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        if rows.len() > self.limits.entries || columns.len() > self.limits.entries {
            return Err(CompileError::Limit("dynamic partition"));
        }
        if self.calls >= self.limits.preparations
            || self.metadata_bytes() > self.limits.input_bytes / 4
        {
            self.rebuild(self.inputs.clone())?;
        }
        self.db.cancel = cancel;
        self.calls += 1;
        let result = salsa::Cancelled::catch(|| {
            algebraic_partition(&self.db, self.inventory, id, rows, columns)
        })
        .map_err(|_| CompileError::Cancelled)?;
        self.db.trigger_lru_eviction();
        result
    }
    /// Prepare general functions through the same bounded pure Salsa database.
    pub fn prepare_functions(
        &mut self,
        id: SemanticId,
        outputs: Vec<SemanticId>,
        coordinates: Vec<SemanticId>,
        order: DerivativeOrder,
        profile: Profile,
        cancel: Arc<AtomicBool>,
    ) -> Result<PreparedFunctions> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        if self.calls >= self.limits.preparations
            || self.metadata_bytes() > self.limits.input_bytes / 4
        {
            self.rebuild(self.inputs.clone())?;
        }
        if coordinates.len() > self.limits.entries || outputs.len() > self.limits.entries {
            return Err(CompileError::Limit("derivative coordinates"));
        }
        self.db.cancel = cancel;
        self.calls += 1;
        let result = salsa::Cancelled::catch(|| {
            let plan = function_plan(&self.db, self.inventory, id, outputs, coordinates, order)?.0;
            Ok(PreparedFunctions {
                artifacts: artifact_requests(&plan, profile),
                plan,
            })
        })
        .map_err(|_| CompileError::Cancelled)?;
        self.db.trigger_lru_eviction();
        result
    }
    /// Prepare only each conditional block's demanded rows and derivative coordinates.
    pub fn prepare_initialization_blocks(
        &mut self,
        id: SemanticId,
        profile: Profile,
    ) -> Result<Arc<Vec<PreparedBlock>>> {
        if self.calls >= self.limits.preparations
            || self.metadata_bytes() > self.limits.input_bytes / 4
        {
            self.rebuild(self.inputs.clone())?;
        }
        self.db.cancel = Arc::new(AtomicBool::new(false));
        self.calls += 1;
        let result = salsa::Cancelled::catch(|| {
            initialization_blocks(&self.db, self.inventory, id, profile).map(|v| v.0)
        })
        .map_err(|_| CompileError::Cancelled)?;
        self.db.trigger_lru_eviction();
        result
    }
    /// Pure tracked flow projection. Runtime callers serialize this workspace lease.
    pub fn prepare_flow(
        &mut self,
        id: SemanticId,
    ) -> Result<Arc<pse_structural::flowsheet::FlowGraph>> {
        if self.calls >= self.limits.preparations
            || self.metadata_bytes() > self.limits.input_bytes / 4
        {
            self.rebuild(self.inputs.clone())?;
        }
        self.db.cancel = Arc::new(AtomicBool::new(false));
        self.calls += 1;
        let result = salsa::Cancelled::catch(|| flow_graph(&self.db, self.inventory, id))
            .map_err(|_| CompileError::Cancelled)?;
        self.db.trigger_lru_eviction();
        result
    }
    /// Pure tracked conditional block schedule; no solver state enters Salsa.
    pub fn prepare_initialization(
        &mut self,
        id: SemanticId,
    ) -> Result<Arc<pse_structural::initialization::Plan>> {
        if self.calls >= self.limits.preparations
            || self.metadata_bytes() > self.limits.input_bytes / 4
        {
            self.rebuild(self.inputs.clone())?;
        }
        self.db.cancel = Arc::new(AtomicBool::new(false));
        self.calls += 1;
        let result = salsa::Cancelled::catch(|| initialization_plan(&self.db, self.inventory, id))
            .map_err(|_| CompileError::Cancelled)?;
        self.db.trigger_lru_eviction();
        result
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
    /// Prepare the requested case. Graph preparation never builds native numeric programs.
    pub fn prepare(
        &mut self,
        id: SemanticId,
        order: DerivativeOrder,
        profile: Profile,
        project_coefficients: bool,
    ) -> Result<PreparedCase> {
        self.prepare_cancellable(
            id,
            order,
            profile,
            project_coefficients,
            Arc::new(AtomicBool::new(false)),
        )
    }
    /// Runtime cancellation is transient and cannot become a cached diagnostic.
    pub fn prepare_cancellable(
        &mut self,
        id: SemanticId,
        order: DerivativeOrder,
        profile: Profile,
        project_coefficients: bool,
        cancel: Arc<AtomicBool>,
    ) -> Result<PreparedCase> {
        if cancel.load(Ordering::Acquire) {
            return Err(CompileError::Cancelled);
        }
        if self.calls >= self.limits.preparations
            || self.metadata_bytes() > self.limits.input_bytes / 4
        {
            self.rebuild(self.inputs.clone())?;
        }
        self.db.cancel = cancel;
        self.calls += 1;
        let result = salsa::Cancelled::catch(|| {
            let p = plan(&self.db, self.inventory, id, order)?.0;
            let structural = structure(&self.db, self.inventory, id)?;
            let requests = artifacts(&self.db, self.inventory, id, order, profile)?;
            let c = case(&self.db, self.inventory, id)
                .ok_or_else(|| CompileError::Missing(format!("case {id}")))?;
            let mut occurrences = BTreeMap::new();
            for def in c.definitions.values() {
                occurrences.insert(
                    *def,
                    admitted(&self.db, self.inventory, *def)?
                        .occurrences
                        .clone(),
                );
            }
            let coefficients = if project_coefficients {
                Some(coefficients(&self.db, self.inventory, id)?.0)
            } else {
                None
            };
            Ok(PreparedCase {
                presolve: presolve_facts(&self.db, self.inventory, id)?.0,
                coefficient_values: if project_coefficients {
                    assumptions(&self.db, self.inventory, id)?
                } else {
                    Vec::new()
                },
                facts: problem_facts(&self.db, self.inventory, id, order, project_coefficients)?,
                plan: p,
                structure: structural,
                artifacts: requests,
                occurrences,
                coefficients,
            })
        })
        .map_err(|_| CompileError::Cancelled)?;
        self.db.trigger_lru_eviction();
        let report = <dyn Database>::memory_usage(&self.db);
        let known = report
            .structs
            .iter()
            .chain(report.queries.values())
            .fold(0usize, |n, i| {
                n.saturating_add(i.heap_size_of_fields().unwrap_or(0))
                    .saturating_add(i.size_of_metadata())
                    .saturating_add(i.size_of_fields())
            });
        if known > self.limits.input_bytes {
            self.rebuild(self.inputs.clone())?;
        }
        result
    }
}
fn inventory(db: &dyn CompilerDb, i: &Inputs) -> Inventory {
    Inventory::new(
        db,
        i.flows.clone(),
        i.quantities.clone(),
        i.preconditions.clone(),
        i.definitions.clone(),
        i.domains.clone(),
        i.groups.clone(),
        i.providers.clone(),
        i.cases.clone(),
        i.values.clone(),
    )
}
fn configure(db: &mut CompilerDatabase, n: usize) {
    initialization_blocks::set_lru_capacity(db, n);
    flow_declaration::set_lru_capacity(db, n);
    flow_graph::set_lru_capacity(db, n);
    initialization_plan::set_lru_capacity(db, n);
    problem_facts::set_lru_capacity(db, n);
    definition::set_lru_capacity(db, n);
    domain::set_lru_capacity(db, n);
    group::set_lru_capacity(db, n);
    provider::set_lru_capacity(db, n);
    case::set_lru_capacity(db, n);
    physical_key::set_lru_capacity(db, n);
    admitted::set_lru_capacity(db, n);
    semantic_body::set_lru_capacity(db, n);
    plan::set_lru_capacity(db, n);
    structure::set_lru_capacity(db, n);
    artifacts::set_lru_capacity(db, n);
    assumptions::set_lru_capacity(db, n);
    coefficients::set_lru_capacity(db, n);
    presolve_facts::set_lru_capacity(db, n);
}
fn validate(i: &Inputs, l: WorkspaceLimits) -> Result<()> {
    if [
        l.entries,
        l.input_bytes,
        l.revisions,
        l.preparations,
        l.query_values,
    ]
    .contains(&0)
    {
        return Err(CompileError::Limit("zero workspace allowance"));
    }
    let entries = i
        .definitions
        .len()
        .saturating_add(i.domains.len())
        .saturating_add(i.groups.len())
        .saturating_add(i.providers.len())
        .saturating_add(i.cases.len())
        .saturating_add(i.flows.len())
        .saturating_add(i.values.len());
    if entries > l.entries {
        return Err(CompileError::Limit("input inventory"));
    }
    let mut bytes = i
        .quantities
        .allocation_extent()
        .saturating_add(entries.saturating_mul(1024));
    for f in i.flows.values() {
        bytes = bytes
            .saturating_add(f.nodes.len().saturating_mul(128))
            .saturating_add(f.decisions.len().saturating_mul(64))
            .saturating_add(f.connections.len().saturating_mul(128));
        for n in &f.nodes {
            bytes = bytes.saturating_add(n.ports.len().saturating_mul(128));
        }
        for e in &f.connections {
            bytes = bytes.saturating_add(e.bindings.len().saturating_mul(64));
        }
    }
    for d in i.definitions.values() {
        bytes = bytes
            .saturating_add(d.sources.iter().map(String::len).sum::<usize>())
            .saturating_add(d.formals.iter().map(|f| f.path.len() + 128).sum::<usize>())
            .saturating_add(
                d.limits
                    .occurrences
                    .saturating_add(d.limits.slots)
                    .saturating_mul(16),
            );
    }
    for d in i.domains.values() {
        bytes = bytes.saturating_add(d.members.members().len().saturating_mul(32));
    }
    for g in i.groups.values() {
        bytes = bytes.saturating_add(g.slots.len().saturating_mul(128 + g.axes.len() * 32));
    }
    for c in i.cases.values() {
        bytes = bytes
            .saturating_add(
                c.structure
                    .instances()
                    .iter()
                    .map(|b| 256 + b.slots.len() * 128 + b.contributions.len() * 64)
                    .sum::<usize>(),
            )
            .saturating_add(
                (c.structure.variables().len()
                    + c.structure.rows().len()
                    + c.structure.parameters().len())
                .saturating_mul(256),
            );
    }
    // Reserve for retained query products as well as input inventories. These are
    // conservative project-container extents, not a claim about Symbolica's allocator.
    let mut products = 0usize;
    for d in i.definitions.values() {
        let text = d
            .sources
            .iter()
            .fold(0usize, |n, s| n.saturating_add(s.len()));
        let expansion = d
            .domains
            .iter()
            .filter_map(|n| i.domains.get(n))
            .fold(1usize, |n, d| {
                n.saturating_mul(d.members.members().len().max(1))
            });
        let operations = text.saturating_mul(expansion).min(d.limits.occurrences);
        products = products
            .saturating_add(operations.saturating_mul(1024))
            .saturating_add(
                d.formals
                    .len()
                    .saturating_mul(d.formals.len())
                    .saturating_mul(d.sources.len())
                    .saturating_mul(128),
            );
        bytes = bytes
            .saturating_add(
                d.domains
                    .iter()
                    .chain(&d.groups)
                    .chain(&d.providers)
                    .fold(0usize, |n, s| n.saturating_add(s.len() + 32)),
            )
            .saturating_add(
                d.units
                    .keys()
                    .fold(0usize, |n, s| n.saturating_add(s.len() + 96)),
            )
            .saturating_add(d.literals.len().saturating_mul(128));
    }
    for c in i.cases.values() {
        let s = &c.structure;
        products = products.saturating_add(
            s.rows()
                .len()
                .saturating_add(s.variables().len())
                .saturating_add(s.parameters().len())
                .saturating_mul(512),
        );
        for b in s.instances() {
            let width = b.slots.len();
            products = products.saturating_add(
                b.contributions
                    .len()
                    .saturating_mul(
                        width
                            .saturating_mul(width)
                            .saturating_add(width)
                            .saturating_add(1),
                    )
                    .saturating_mul(256),
            );
        }
    }
    for (name, p) in &i.providers {
        let p = p.descriptor.spec();
        bytes = bytes
            .saturating_add(name.len())
            .saturating_add(p.components.len().saturating_mul(32))
            .saturating_add(
                p.inputs
                    .len()
                    .saturating_add(p.outputs.len())
                    .saturating_mul(128),
            );
    }
    bytes = bytes
        .saturating_add(
            i.domains
                .keys()
                .chain(i.groups.keys())
                .fold(0usize, |n, s| n.saturating_add(s.len())),
        )
        .saturating_add(i.preconditions.declarations().iter().fold(0usize, |n, p| {
            n.saturating_add(128 + p.operand_positions.len() * 2)
        }));
    if bytes.saturating_add(products.saturating_mul(l.query_values)) > l.input_bytes {
        return Err(CompileError::Limit(
            "input and retained query capacity; reduce query_values or increase workspace allowance",
        ));
    }
    if bytes > l.input_bytes {
        return Err(CompileError::Limit("input extent"));
    }
    if i.values.values().any(|x| !x.is_finite()) {
        return Err(CompileError::Math(Arc::new(MathError::Contract(
            "nonfinite input value".into(),
        ))));
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

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;

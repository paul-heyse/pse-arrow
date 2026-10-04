// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit PETSc mathematical products over the existing compiled original source.
//! These products produce auxiliary numerical observations; original assessment remains
//! with the shared strategy driver.
use super::super::Workspace;
use super::*;
use pse_backend_native::petsc::{self as native_petsc, DomainOracle, FrozenMassRefresh};
use pse_backend_native::{NleOracle, OracleContract, RootOperations};
use pse_ids::ContentHash;
use pse_kernels::{DerivativeOrder, ExecutionScope};
use pse_math::{
    derived::{DerivedFamily, MassBinding, MassStructure, OriginalContract},
    index::{Entry, GlobalCol, GlobalRow},
};
use std::collections::BTreeSet;

/// Compiler-owned workspace and exact requested compilation profile for block products.
#[derive(Clone, Debug)]
pub struct PetscCompiler {
    /// Actual declaration/workspace authority.
    pub workspace: Workspace,
    /// Explicit compiler profile, independent of the auxiliary solver profile.
    pub profile: pse_compiler::workspace::Profile,
}
/// Source-issued physical mass, correspondence and finite artificial trajectory declaration.
#[derive(Clone, Debug)]
pub struct PetscFlowDefinition {
    /// Optional supplemental executable original guard supplier.
    pub domain: Option<Arc<dyn PetscDomainFactory>>,
    /// Actual M/DM actions or frozen-anchor mass producer.
    pub mass: Arc<dyn PetscMassFactory>,
    /// Mechanical original row/coordinate pairing.
    pub pairing: Vec<GlobalCol>,
    /// Declared artificial residual sign.
    pub sign: f64,
    /// Explicit finite trajectory work/time extent.
    pub limits: native_petsc::FlowLimits,
}

/// Complete inventory supplied by the original guard/domain producer. Arithmetic
/// incidence is not a source of this inventory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OriginalGuardInventory {
    /// Frozen original solve identity, including its parameters and providers.
    pub original: ContentHash,
    /// Actual immutable compiled preparation served by the producer.
    pub preparation: ContentHash,
    /// Complete selected guard-bearing compiled products and any explicitly supplied
    /// supplemental obligations, in canonical order.
    pub obligations: Vec<ContentHash>,
}
/// A source-bound domain supplier constructed on the admitted native worker.
pub trait PetscDomainFactory: std::fmt::Debug + Send + Sync {
    /// Actual complete inventory served by this supplier.
    fn inventory(&self) -> &OriginalGuardInventory;
    /// Immutable executable producer identity.
    fn source(&self) -> ContentHash;
    /// Escaping factory-owned storage, excluding shared source products.
    fn retained_bytes(&self) -> usize;
    /// Maximum actual attempt-owned storage, before construction.
    fn worker_bytes(&self) -> usize;
    /// Bind the same original guard obligation and scope; no numerical solve is run.
    fn bind(
        &self,
        original: &OriginalContract,
        scope: ExecutionScope,
    ) -> Result<Box<dyn DomainOracle>, ProblemError>;
}
/// Actual mass actions, distinct from solver settings and from residual derivatives.
pub trait PetscMassFactory: std::fmt::Debug + Send + Sync {
    /// Immutable physical mass and derivative support.
    fn structure(&self) -> &MassStructure;
    /// Frozen parameter/provider realization.
    fn realization(&self) -> ContentHash;
    /// Original solve and preparation served by these physical actions.
    fn binding(&self) -> (ContentHash, ContentHash);
    /// Factory-owned storage excluding shared source products.
    fn retained_bytes(&self) -> usize;
    /// Maximum actual attempt-owned storage, before construction.
    fn worker_bytes(&self) -> usize;
    /// Create actual frozen coefficients or state-dependent M/DM callbacks.
    fn bind(&self, scope: ExecutionScope) -> Result<MassBinding<ProblemError>, ProblemError>;
    /// Optional explicit accepted-anchor refresh. Rejected stages never call it.
    fn refresh(
        &self,
        _scope: ExecutionScope,
    ) -> Result<Option<Box<dyn FrozenMassRefresh>>, ProblemError> {
        Ok(None)
    }
}
/// A complete compiled root source. The physical authored equalities remain retained
/// separately from the numerical zero-residual view.
#[derive(Clone, Debug)]
pub struct PreparedPetscSource {
    original: Arc<PreparedSolve>,
    source: Arc<AlgebraicCase>,
    physical: Arc<OriginalContract>,
    zero: Arc<OriginalContract>,
    guards: Arc<OriginalGuardInventory>,
    scope: ExecutionScope,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedPetscSource {
    /// Full physical original contract for independent final assessment.
    pub fn physical(&self) -> &Arc<OriginalContract> {
        &self.physical
    }
    /// Explicit authored-RHS subtraction view; it grants no scientific permission.
    pub fn zero(&self) -> &Arc<OriginalContract> {
        &self.zero
    }
    /// Original task scope, retained without renewing its deadline.
    pub fn scope(&self) -> &ExecutionScope {
        &self.scope
    }
    /// Frozen original identity used by the shared strategy.
    pub fn original_identity(&self) -> ContentHash {
        self.guards.original
    }
    /// Actual frozen source preparation identity.
    pub fn preparation_identity(&self) -> ContentHash {
        self.guards.preparation
    }
    /// Complete inventory supplied by the original domain producer.
    pub fn guards(&self) -> &OriginalGuardInventory {
        &self.guards
    }
    /// Original profile used for correction and final assessment.
    pub fn original(&self) -> &PreparedSolve {
        &self.original
    }
    pub(crate) fn matches_case(&self, case: &Preparation, values: &CaseValues) -> bool {
        self.source.prepared.compiled().plan.structure().key()
            == case.compiled().plan.structure().key()
            && self.source.values.scalars.len() == values.scalars.len()
            && self.source.values.scalars.iter().all(|(id, value)| {
                values
                    .scalars
                    .get(id)
                    .is_some_and(|other| value.to_bits() == other.to_bits())
            })
    }
}
#[derive(Debug)]
enum Product {
    Flow {
        family: Arc<DerivedFamily>,
        mass: Arc<dyn PetscMassFactory>,
        limits: native_petsc::FlowLimits,
    },
    Blocks {
        blocks: Vec<CompiledBlock>,
        declaration: ContentHash,
    },
}
#[derive(Clone, Debug)]
struct CompiledBlock {
    family: Arc<DerivedFamily>,
    executable: Arc<ExecutableCase>,
    values: CaseValues,
    local_ids: Vec<pse_ids::SemanticId>,
    external: Vec<(GlobalCol, pse_ids::SemanticId)>,
    offsets: Vec<f64>,
    jacobian_slots: Vec<usize>,
}
fn block_binding_bytes(block: &CompiledBlock, retained: bool) -> Result<usize, MathRuntimeError> {
    let local = if retained {
        block.local_ids.capacity()
    } else {
        block.local_ids.len()
    };
    let external = if retained {
        block.external.capacity()
    } else {
        block.external.len()
    };
    let offsets = if retained {
        block.offsets.capacity()
    } else {
        block.offsets.len()
    };
    let slots = if retained {
        block.jacobian_slots.capacity()
    } else {
        block.jacobian_slots.len()
    };
    derived::case_values_bytes(block.values.scalars.len())?
        .checked_add(size_of::<CompiledBlock>())
        .and_then(|n| n.checked_add(local.checked_mul(size_of::<pse_ids::SemanticId>())?))
        .and_then(|n| {
            n.checked_add(external.checked_mul(size_of::<(GlobalCol, pse_ids::SemanticId)>())?)
        })
        .and_then(|n| n.checked_add(offsets.checked_mul(size_of::<f64>())?))
        .and_then(|n| n.checked_add(slots.checked_mul(size_of::<usize>())?))
        .ok_or(MathRuntimeError::Limit(
            "PETSc compact block binding extent",
        ))
}
/// Explicit flow or complete compiler-issued BTF composition, with its typed profile.
#[derive(Clone, Debug)]
pub struct PreparedPetsc {
    source: PreparedPetscSource,
    profile: Arc<SolverProfile>,
    settings: native_petsc::Settings,
    domain: Option<Arc<dyn PetscDomainFactory>>,
    product: Arc<Product>,
    compatibility: Compatibility,
    key: ContentHash,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PreparedPetsc {
    /// Consumed source/family/profile/supplier identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Full original source; auxiliary native status never replaces its assessment.
    pub fn source(&self) -> &PreparedPetscSource {
        &self.source
    }
    /// Typed declared profile.
    pub fn profile(&self) -> &SolverProfile {
        &self.profile
    }
    /// Original task scope shared by preparation and native execution.
    pub fn scope(&self) -> &ExecutionScope {
        self.source.scope()
    }
    /// Actual original problem identity, independent of the auxiliary profile.
    pub fn original_identity(&self) -> ContentHash {
        self.source.original_identity()
    }
    /// Frozen typed backend settings identity consumed by the strategy.
    pub fn strategy_profile(&self) -> Result<ContentHash, ProblemError> {
        Ok(profile_key(&self.profile)?.as_id())
    }
    /// Finite controls consumed by this auxiliary attempt.
    pub fn controls(&self) -> &Controls {
        &self.profile.controls
    }
    /// Exact selected backend.
    pub fn backend(&self) -> Backend {
        Backend::Petsc
    }
    /// Admitted thread extent; PETSc products currently require serial execution.
    pub fn threads(&self) -> usize {
        self.profile.controls.threads
    }
    /// Attempt duration capped again by the original absolute task deadline.
    pub fn time_limit(&self) -> std::time::Duration {
        self.profile.controls.time_limit
    }
    /// Declared finite foreign allowance, resolved by the shared deployment policy.
    pub fn declared_foreign_bytes(&self) -> usize {
        self.profile.controls.foreign_bytes.unwrap_or(0)
    }
    /// Frozen mechanism profile carried by the shared strategy.
    pub fn profile_ref(&self) -> Result<pse_model::strategy::ProfileRef, ProblemError> {
        Ok(pse_model::strategy::ProfileRef {
            backend: Backend::Petsc,
            key: self.strategy_profile()?,
        })
    }
    /// Full original specification point; it grants no permission for an auxiliary result.
    pub fn source_start(&self) -> Result<Vec<f64>, ProblemError> {
        self.source
            .source
            .prepared
            .compiled()
            .plan
            .columns()
            .iter()
            .map(|id| {
                self.source
                    .source
                    .values
                    .scalars
                    .get(id)
                    .copied()
                    .ok_or_else(|| {
                        ProblemError::Contract("PETSc original specification point missing".into())
                    })
            })
            .collect()
    }
    /// Actual selected source programs, physical interpretation, families and suppliers.
    pub fn support(&self) -> BTreeSet<ContentHash> {
        let mut result = self
            .source
            .guards
            .obligations
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        result.extend([
            self.source.original_identity(),
            self.source.preparation_identity(),
            self.source.physical.obligations().guards,
            self.source.physical.obligations().selection,
        ]);
        if let Some(domain) = &self.domain {
            result.insert(domain.source());
        }
        match self.product.as_ref() {
            Product::Flow { family, mass, .. } => {
                result.insert(family.key());
                result.insert(mass.realization());
            }
            Product::Blocks {
                blocks,
                declaration,
            } => {
                result.insert(*declaration);
                result.extend(blocks.iter().map(|b| b.family.key()));
            }
        }
        result
    }
    /// Conservative escaping native report and full original proposal allowance.
    pub(crate) fn result_bytes(&self) -> Result<usize, MathRuntimeError> {
        self.source
            .original
            .result_bytes()?
            .checked_add(self.profile.controls.report_allowance()?)
            .and_then(|n| {
                n.checked_add(
                    self.source
                        .physical
                        .coordinates()
                        .len()
                        .checked_mul(size_of::<f64>())?,
                )
            })
            .ok_or(MathRuntimeError::Limit("PETSc result allowance"))
    }
    /// Mathematical mechanism actually bound by this product.
    pub fn mechanism(&self) -> pse_model::strategy::MechanismKind {
        match self.product.as_ref() {
            Product::Flow { .. } => pse_model::strategy::MechanismKind::PseudoTransient,
            Product::Blocks { .. } => pse_model::strategy::MechanismKind::Block,
        }
    }
}

/// Exact source-declared frozen physical mass. No settings-derived mass is provided.
#[derive(Debug)]
pub struct FrozenPetscMass {
    matrix: Arc<pse_math::sparse::AssemblyMatrix>,
    structure: MassStructure,
    binding: (ContentHash, ContentHash),
    realization: ContentHash,
}

#[cfg(test)]
#[path = "petsc_tests.rs"]
mod tests;
impl FrozenPetscMass {
    /// Bind actual finite sparse coefficients and their physical producer identity.
    ///
    /// # Errors
    /// Matrix shape differs from the complete original or contains nonfinite values.
    pub fn new(
        source: &PreparedPetscSource,
        matrix: Arc<pse_math::sparse::AssemblyMatrix>,
        physical_source: ContentHash,
    ) -> Result<Self, ProblemError> {
        let n = source.physical.coordinates().len();
        let symbolic = matrix.matrix().symbolic();
        if symbolic.nrows() != n
            || symbolic.ncols() != n
            || matrix.matrix().val().iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "PETSc frozen physical mass shape or finite coefficients".into(),
            ));
        }
        let mut incidence = Vec::with_capacity(symbolic.compute_nnz());
        for col in 0..n {
            for row in symbolic.row_idx_of_col(col) {
                incidence.push(Entry::new(GlobalRow::new(row), GlobalCol::new(col)));
            }
        }
        incidence.sort_unstable();
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("source-bound-frozen-physical-mass")
            .hash(&source.original_identity())
            .hash(&source.preparation_identity())
            .hash(&physical_source);
        for edge in &incidence {
            h.u64(edge.row.get() as u64).u64(edge.col.get() as u64);
        }
        for value in matrix.matrix().val() {
            h.f64(*value);
        }
        Ok(Self {
            matrix,
            structure: MassStructure::Frozen { incidence },
            binding: (source.original_identity(), source.preparation_identity()),
            realization: h.finish_hash(),
        })
    }
}
impl PetscMassFactory for FrozenPetscMass {
    fn structure(&self) -> &MassStructure {
        &self.structure
    }
    fn realization(&self) -> ContentHash {
        self.realization
    }
    fn binding(&self) -> (ContentHash, ContentHash) {
        self.binding
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.matrix.retained_bytes()
            + match &self.structure {
                MassStructure::Frozen { incidence } => {
                    incidence.capacity() * size_of::<Entry<GlobalRow, GlobalCol>>()
                }
                _ => 0,
            }
    }
    fn worker_bytes(&self) -> usize {
        self.matrix.retained_bytes()
    }
    fn bind(&self, scope: ExecutionScope) -> Result<MassBinding<ProblemError>, ProblemError> {
        scope.check().map_err(ProblemError::from)?;
        Ok(MassBinding::Frozen(self.matrix.as_ref().clone()))
    }
}

impl PreparedSolve {
    /// Inventory of the complete selected original guard-bearing programs and actual
    /// provider configurations. It comes from source demands, never from incidence.
    ///
    /// # Errors
    /// The source is not an actual compiled algebraic preparation.
    pub fn petsc_guard_inventory(&self) -> Result<OriginalGuardInventory, ProblemError> {
        let source = match &self.representation {
            Representation::Algebraic(source) => source,
            _ => {
                return Err(ProblemError::Unsupported(
                    "PETSc compiled guard inventory requires an algebraic source".into(),
                ));
            }
        };
        let obligations = source
            .prepared
            .compiled()
            .artifacts
            .iter()
            .map(|r| r.key())
            .chain(source.providers.values().map(|p| p.configuration_key()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(OriginalGuardInventory {
            original: self.original_identity()?,
            preparation: self.preparation_identity()?,
            obligations,
        })
    }
}

struct Preparing(Option<Arc<std::sync::atomic::AtomicBool>>);
impl Drop for Preparing {
    fn drop(&mut self) {
        if let Some(flag) = &self.0 {
            flag.store(true, std::sync::atomic::Ordering::Release);
        }
    }
}
async fn scope_stopped(scope: &ExecutionScope) -> ProblemError {
    loop {
        if let Err(error) = scope.check() {
            return error.into();
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

impl MathService {
    /// Freeze the full original compiled root contract under its existing task scope.
    ///
    /// # Errors
    /// Non-root/bounded/partial sources, insufficient ordinary Jacobian support,
    /// mismatched original guard inventory, cancellation or a finite resource cap.
    pub async fn prepare_petsc_source(
        self: &Arc<Self>,
        mut original: PreparedSolve,
        guards: OriginalGuardInventory,
        scope: ExecutionScope,
        driver: &crate::CancelSource,
    ) -> Result<PreparedPetscSource, MathRuntimeError> {
        scope.check().map_err(ProblemError::from)?;
        if scope.deadline().is_none() {
            return Err(ProblemError::Contract(
                "PETSc preparation requires the original finite task deadline".into(),
            )
            .into());
        }
        if original.task_scope().as_ref().is_some_and(|declared| {
            !Arc::ptr_eq(declared.cancellation(), scope.cancellation())
                || declared.deadline() != scope.deadline()
        }) {
            return Err(ProblemError::Contract(
                "PETSc source preparation must preserve its original declared task scope".into(),
            )
            .into());
        }
        if original.composition.is_some() {
            return Err(ProblemError::Contract(
                "PETSc source must be prepared before composing its single strategy".into(),
            )
            .into());
        }
        if original.profile.intent != SolveIntent::Root
            || guards.original != original.original_identity()?
            || guards.preparation != original.preparation_identity()?
            || guards.obligations.windows(2).any(|w| w[0] >= w[1])
        {
            return Err(ProblemError::Contract(
                "PETSc original purpose or full frozen guard inventory binding".into(),
            )
            .into());
        }
        if original
            .petsc_guard_inventory()?
            .obligations
            .iter()
            .any(|key| guards.obligations.binary_search(key).is_err())
        {
            return Err(ProblemError::Contract(
                "PETSc guard inventory omits an actual selected original executable producer"
                    .into(),
            )
            .into());
        }
        let source = match &original.representation {
            Representation::Algebraic(a) => a.clone(),
            _ => {
                return Err(ProblemError::Unsupported(
                    "PETSc explicit products require a compiled original root source".into(),
                )
                .into());
            }
        };
        let plan = &source.prepared.compiled().plan;
        if plan.order() < DerivativeOrder::First
            || plan.available_order() < DerivativeOrder::First
            || !plan.structure().native().is_empty()
            || !plan.structure().requirements().is_empty()
        {
            return Err(ProblemError::Unsupported("PETSc products need actual ordinary First callbacks with no unbound native formulation requirements".into()).into());
        }
        original = original.within_task(scope.clone())?;
        let bindings = derived::binding_metadata_bytes(&source, &original.profile)?;
        let bytes = source
            .prepared
            .compiled()
            .retained_bytes()
            .checked_add(
                guards
                    .obligations
                    .len()
                    .checked_mul(size_of::<ContentHash>())
                    .ok_or(MathRuntimeError::Limit("PETSc guard inventory extent"))?,
            )
            .and_then(|n| n.checked_add(bindings))
            .ok_or(MathRuntimeError::Limit("PETSc source extent"))?;
        if bytes > self.policy.workspace_bytes {
            return Err(MathRuntimeError::Limit(
                "PETSc source preparation workspace",
            ));
        }
        let control = FlightCancellation::default();
        let task = scope.clone();
        let worker_scope = scope.clone();
        let mut preparing = Preparing(Some(scope.cancellation().clone()));
        let operation = self.job_retained_scoped(1, bytes, control.clone(), scope.deadline(), move |abort| {
            if abort.load(std::sync::atomic::Ordering::Acquire) { return Err(MathRuntimeError::Cancelled); }
            worker_scope.check().map_err(ProblemError::from)?;
            let physical = derived::physical_contract(&original, &source, worker_scope.cancellation())?;
            let zero = derived::zero_contract(&physical)?;
            if physical.coordinates().is_empty() {return Err(ProblemError::Unsupported("PETSc products require an actual nonempty original root execution; all-fixed cases retain constant assessment".into()).into());}
            if physical.coordinates().iter().any(|v| v.lower.is_finite() || v.upper.is_finite()) { return Err(ProblemError::Unsupported("PETSc explicit products do not enforce arbitrary authored bounds".into()).into()); }
            pse_structural::initialization::Plan::from_analysis(&source.prepared.compiled().structure).map_err(|e| ProblemError::Contract(e.to_string()))?;
            let bindings=derived::binding_metadata_bytes(&source,&original.profile)?;
            let owned = physical.retained_bytes().checked_add(zero.retained_bytes()).and_then(|n| n.checked_add(size_of::<PreparedPetscSource>()+size_of::<AlgebraicCase>()+size_of::<OriginalGuardInventory>()+10*size_of::<usize>())).and_then(|n| n.checked_add(guards.obligations.capacity()*size_of::<ContentHash>())).and_then(|n|n.checked_add(bindings)).ok_or(MathRuntimeError::Limit("PETSc retained source metadata"))?;
            worker_scope.check().map_err(ProblemError::from)?;
            Ok(((original, source, physical, zero, guards), owned))
        });
        tokio::pin!(operation);
        let ((original, source, physical, zero, guards), owner) = tokio::select! {
            result = &mut operation => {preparing.0=None;result?},
            () = driver.cancelled() => { task.cancellation().store(true, std::sync::atomic::Ordering::Release); control.cancel(); let _ = operation.await; return Err(MathRuntimeError::Cancelled); }
            error = scope_stopped(&task) => {preparing.0=None;control.cancel();let _=operation.await;return Err(error.into());}
        };
        task.check().map_err(ProblemError::from)?;
        Ok(PreparedPetscSource {
            original: Arc::new(original),
            source: Arc::new(source),
            physical,
            zero,
            guards: Arc::new(guards),
            scope: task,
            _owner: owner,
        })
    }
}

fn admit(
    source: &PreparedPetscSource,
    profile: &SolverProfile,
    domain: Option<&dyn PetscDomainFactory>,
    method: native_petsc::Method,
) -> Result<native_petsc::Settings, ProblemError> {
    source.scope.check().map_err(ProblemError::from)?;
    profile.controls.validate()?;
    if profile.selection != SolverSelection::Explicit(Backend::Petsc)
        || profile.intent != SolveIntent::Root
        || profile.numerics.key() != source.original.numerics.policy.key()
        || profile.controls.threads != 1
        || profile.controls.threads != source.original.threads()
        || profile.sensitivity.is_some()
        || !profile.controls.options.is_empty()
        || profile.controls.reuse == ReusePolicy::RequireReuse
        || domain.is_some_and(|d| d.inventory() != source.guards.as_ref())
    {
        return Err(ProblemError::Contract("PETSc explicit profile must retain original numerical policy, complete guards and fresh serial ownership".into()));
    }
    if domain.is_none() && source.guards.as_ref() != &source.original.petsc_guard_inventory()? {
        return Err(ProblemError::Contract(
            "PETSc supplemental original guard obligations require their actual supplier".into(),
        ));
    }
    let settings = match &profile.backend {
        BackendSettings::Petsc(s) => *s,
        _ => {
            return Err(ProblemError::Contract(
                "PETSc mathematical product requires explicit typed method settings".into(),
            ));
        }
    };
    settings.validate()?;
    if settings.method != method
        || source.original.accuracy.native_scaling
        || source.original.accuracy.acceptable.is_some()
    {
        return Err(ProblemError::Unsupported(
            "PETSc product/method requires strict original residual accuracy".into(),
        ));
    }
    if !matches!(profile.controls.foreign_bytes, Some(n) if n > 0) {
        return Err(ProblemError::Unsupported(
            "PETSc factory requires an explicit positive finite foreign allowance".into(),
        ));
    }
    Ok(settings)
}

/// Explicit numerical offset view validated by the existing original bridge. Full
/// Jacobian actions delegate to the same admitted ordinary source.
#[derive(Debug)]
struct ZeroRoot {
    actual: Box<dyn NleOracle>,
    contract: OracleContract,
}
impl ZeroRoot {
    fn new(actual: Box<dyn NleOracle>, source: &PreparedPetscSource) -> Result<Self, ProblemError> {
        actual.operations().admit(actual.contract(), false)?;
        let bridge = native::derived::RootBridge::new(
            actual,
            source.physical.clone(),
            source
                .physical
                .constraints()
                .iter()
                .map(|r| r.lower)
                .collect(),
        )?
        .zero_residuals()?;
        if bridge.numerical_contract().as_ref() != source.zero.as_ref() {
            return Err(ProblemError::Contract(
                "PETSc explicit authored equality subtraction differs from its frozen source"
                    .into(),
            ));
        }
        let (actual, _) = bridge.into_original().into_original();
        let mut contract = actual.contract().clone();
        contract.identity = source.zero.identity();
        Ok(Self { actual, contract })
    }
}
impl NleOracle for ZeroRoot {
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn operations(&self) -> RootOperations {
        self.actual.operations()
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.actual.jacobian_pattern()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.actual.residual(x, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.actual.jacobian(x, out)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        d: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.actual.jacobian_product(x, d, out)
    }
    fn observe(&self, rows: Vec<f64>) -> Result<quality::Observation, ProblemError> {
        self.actual.observe(rows)
    }
}

impl MathService {
    /// Prepare a declared artificial flow from actual physical M/DM actions.
    /// A frozen mass without an explicit refresh supplier applies to the whole trajectory.
    ///
    /// # Errors
    /// Wrong source/guard/profile binding, mass support, outer caps or finite allowance.
    pub fn prepare_petsc_flow(
        &self,
        source: PreparedPetscSource,
        profile: SolverProfile,
        definition: PetscFlowDefinition,
    ) -> Result<PreparedPetsc, MathRuntimeError> {
        let PetscFlowDefinition {
            domain,
            mass,
            pairing,
            sign,
            limits,
        } = definition;
        let settings = admit(
            &source,
            &profile,
            domain.as_deref(),
            native_petsc::Method::PseudoTransient,
        )?;
        if mass.binding() != (source.original_identity(), source.preparation_identity())
            || limits.steps == 0
            || limits.steps > i32::MAX as u32
            || !limits.artificial_time.is_finite()
            || limits.artificial_time <= 0.0
        {
            return Err(ProblemError::Contract(
                "PETSc physical mass binding or finite artificial trajectory caps".into(),
            )
            .into());
        }
        let family = Arc::new(DerivedFamily::shifted_pseudo_time(
            source.zero.clone(),
            pairing,
            sign,
            mass.structure().clone(),
        )?);
        let bytes = family
            .retained_bytes()
            .checked_add(mass.retained_bytes())
            .and_then(|n| n.checked_add(domain.as_ref().map_or(0, |d| d.retained_bytes())))
            .and_then(|n| {
                n.checked_add(
                    size_of::<PreparedPetsc>()
                        + size_of::<Product>()
                        + size_of::<SolverProfile>()
                        + 6 * size_of::<usize>(),
                )
            })
            .and_then(|n| {
                n.checked_add(derived::binding_metadata_bytes(&source.source, &profile).ok()?)
            })
            .ok_or(MathRuntimeError::Limit("PETSc retained flow extent"))?;
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("compiled-petsc-artificial-flow")
            .hash(&source.original_identity())
            .hash(&source.preparation_identity())
            .hash(&family.key())
            .hash(&mass.realization())
            .hash(&source.physical.obligations().guards)
            .hash(&profile_key(&profile)?.as_id())
            .u64(u64::from(limits.steps))
            .f64(limits.artificial_time);
        if let Some(domain) = &domain {
            h.hash(&domain.source());
        }
        let key = h.finish_hash();
        let owner = self.reserve("math:petsc-flow-product", bytes)?;
        source.scope.check().map_err(ProblemError::from)?;
        Ok(PreparedPetsc {
            source,
            profile: Arc::new(profile),
            settings,
            domain,
            product: Arc::new(Product::Flow {
                family,
                mass,
                limits,
            }),
            compatibility: Compatibility {
                layout: key,
                profile: key,
                data: key,
                backend: Backend::Petsc,
            },
            key,
            _owner: owner,
        })
    }
    /// Use the existing compiler's complete BTF producer and its selected First programs.
    /// The declaration is the source-supplied domain-safe admission witness; actual
    /// original domain checks still execute around every trial.
    ///
    /// # Errors
    /// A partial/deficient source, wrong domain/profile binding, compiler failure,
    /// inconsistent selected maps/support, cancellation or a finite resource cap.
    pub async fn prepare_petsc_blocks(
        self: &Arc<Self>,
        source: PreparedPetscSource,
        compiler: PetscCompiler,
        profile: SolverProfile,
        domain: Option<Arc<dyn PetscDomainFactory>>,
        declaration: ContentHash,
        driver: &crate::CancelSource,
    ) -> Result<PreparedPetsc, MathRuntimeError> {
        let mut preparing = Preparing(Some(source.scope.cancellation().clone()));
        let result = self
            .prepare_petsc_blocks_inner(source, compiler, profile, domain, declaration, driver)
            .await;
        preparing.0 = None;
        result
    }
    async fn prepare_petsc_blocks_inner(
        self: &Arc<Self>,
        source: PreparedPetscSource,
        compiler: PetscCompiler,
        profile: SolverProfile,
        domain: Option<Arc<dyn PetscDomainFactory>>,
        declaration: ContentHash,
        driver: &crate::CancelSource,
    ) -> Result<PreparedPetsc, MathRuntimeError> {
        let PetscCompiler {
            workspace,
            profile: compiler_profile,
        } = compiler;
        let settings = admit(
            &source,
            &profile,
            domain.as_deref(),
            native_petsc::Method::NonlinearAdditiveSchwarz,
        )?;
        let scope = source.scope.clone();
        let worker_scope = scope.clone();
        let control = FlightCancellation::default();
        let prepared = source.source.prepared.clone();
        let values = source.source.values.clone();
        let operation = self.job_retained_scoped(
            1,
            self.policy.workspace_bytes,
            control.clone(),
            scope.deadline(),
            move |abort| {
                if abort.load(std::sync::atomic::Ordering::Acquire) {
                    return Err(MathRuntimeError::Cancelled);
                }
                worker_scope.check().map_err(ProblemError::from)?;
                let _lease = workspace.lease;
                let compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                let products = compiler.prepare_bound_initialization(
                    prepared.compiled(),
                    compiler_profile,
                    worker_scope.cancellation(),
                )?;
                // Validate the actual frozen conditional views once. Callbacks below use
                // compact selected slots, not whole-parent AlgebraicOracle value validation.
                for block in products.iter() {
                    block.bind(
                        prepared.compiled().quantities.clone(),
                        &values,
                        worker_scope.cancellation(),
                    )?;
                }
                let bytes = products
                    .iter()
                    .try_fold(size_of_val(products.as_ref()), |n, p| {
                        n.checked_add(p.plan.retained_bytes())
                            .and_then(|n| n.checked_add(size_of_val(p.artifacts.as_slice())))
                    })
                    .ok_or(MathRuntimeError::Limit("PETSc compiled block products"))?;
                worker_scope.check().map_err(ProblemError::from)?;
                Ok((products, bytes))
            },
        );
        tokio::pin!(operation);
        let (products, lease) = tokio::select! {
            result = &mut operation => result?,
            () = driver.cancelled() => { scope.cancellation().store(true,std::sync::atomic::Ordering::Release); control.cancel(); let _ = operation.await; return Err(MathRuntimeError::Cancelled); }
            error = scope_stopped(&scope) => {control.cancel();let _=operation.await;return Err(error.into());}
        };
        let owner = self.shared_product(
            vec![31, Arc::as_ptr(&products) as usize],
            products.clone(),
            lease,
            vec![source.source.prepared.owner.clone()],
        )?;
        let mut blocks = Vec::with_capacity(products.len());
        let mut covered_rows = BTreeSet::new();
        let mut covered_columns = BTreeSet::new();
        for block in products.iter() {
            scope.check().map_err(ProblemError::from)?;
            let columns = block
                .plan
                .columns()
                .iter()
                .map(|id| {
                    source
                        .zero
                        .coordinates()
                        .iter()
                        .position(|c| c.id == *id)
                        .map(GlobalCol::new)
                        .ok_or_else(|| {
                            ProblemError::Contract("compiler block column outside original".into())
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let rows = block
                .plan
                .structure()
                .rows()
                .iter()
                .map(|row| {
                    source
                        .zero
                        .constraints()
                        .iter()
                        .position(|r| r.id == row.id)
                        .map(GlobalRow::new)
                        .ok_or_else(|| {
                            ProblemError::Contract("compiler block row outside original".into())
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            if columns.iter().any(|c| !covered_columns.insert(*c))
                || rows.iter().any(|r| !covered_rows.insert(*r))
            {
                return Err(ProblemError::Contract(
                    "compiler BTF products overlap their correction inventories".into(),
                )
                .into());
            }
            let family = Arc::new(DerivedFamily::block_subsystem(
                source.zero.clone(),
                columns,
                rows,
            )?);
            let mut artifacts = Vec::with_capacity(block.artifacts.len());
            for request in block.artifacts.iter() {
                scope.check().map_err(ProblemError::from)?;
                let operation = self.artifact(request.clone());
                tokio::pin!(operation);
                let artifact = tokio::select! {
                    r=&mut operation=>r?,
                    ()=driver.cancelled()=>{scope.cancellation().store(true,std::sync::atomic::Ordering::Release);return Err(MathRuntimeError::Cancelled);},
                    error=scope_stopped(&scope)=>return Err(error.into()),
                };
                scope.check().map_err(ProblemError::from)?;
                artifacts.push(artifact);
            }
            let plan = Arc::new(block.plan.as_ref().clone().with_owner(owner.clone()));
            let assembly =
                Arc::new(plan.assemble(artifacts.iter().map(|a| a.program.clone()).collect())?);
            let pattern = assembly.jacobian_pattern();
            let mut slots = Vec::with_capacity(pattern.compute_nnz());
            for col in 0..pattern.ncols() {
                for row in pattern.row_idx_of_col(col) {
                    slots.push(
                        family
                            .incidence()
                            .binary_search(&Entry::new(GlobalRow::new(row), GlobalCol::new(col)))
                            .map_err(|_| {
                                ProblemError::Contract(
                                    "compiler block derivative outside declared all-branch support"
                                        .into(),
                                )
                            })?,
                    );
                }
            }
            if slots.len() != family.incidence().len() {
                return Err(ProblemError::Contract(
                    "compiler block derivative omits original all-branch support".into(),
                )
                .into());
            }
            let local_ids = plan.columns().to_vec();
            let required = plan
                .structure()
                .instances()
                .iter()
                .flat_map(|i| i.slots.iter().map(|s| s.source()))
                .chain(local_ids.iter().copied())
                .collect::<BTreeSet<_>>();
            let compact = CaseValues {
                scalars: required
                    .iter()
                    .map(|id| {
                        source
                            .source
                            .values
                            .scalars
                            .get(id)
                            .copied()
                            .map(|v| (*id, v))
                            .ok_or_else(|| {
                                ProblemError::Contract("compiler block frozen slot missing".into())
                            })
                    })
                    .collect::<Result<_, _>>()?,
            };
            // Include actual guard-only variable inputs as well as equation incidence.
            let external = source
                .zero
                .coordinates()
                .iter()
                .enumerate()
                .filter(|(_, c)| required.contains(&c.id) && !local_ids.contains(&c.id))
                .map(|(i, c)| (GlobalCol::new(i), c.id))
                .collect::<Vec<_>>();
            let offsets = family
                .row_map()
                .ok_or_else(|| ProblemError::Contract("compiler block row map".into()))?
                .iter()
                .map(|r| source.physical.constraints()[r.get()].lower)
                .collect();
            blocks.push(CompiledBlock {
                family,
                executable: Arc::new(ExecutableCase {
                    assembly,
                    _artifacts: artifacts,
                    _owner: owner.clone(),
                }),
                values: compact,
                local_ids,
                external,
                offsets,
                jacobian_slots: slots,
            });
        }
        if covered_rows.len() != source.zero.constraints().len()
            || covered_columns.len() != source.zero.coordinates().len()
        {
            return Err(ProblemError::Contract(
                "compiler BTF products do not cover the complete original root".into(),
            )
            .into());
        }
        let mut h = FramedHasher::new(pse_ids::Frame::DerivedBindingV1);
        h.str("compiled-petsc-complete-btf")
            .hash(&source.original_identity())
            .hash(&source.preparation_identity())
            .hash(&source.physical.obligations().guards)
            .hash(&declaration)
            .hash(&profile_key(&profile)?.as_id());
        if let Some(domain) = &domain {
            h.hash(&domain.source());
        }
        for (block, product) in blocks.iter().zip(products.iter()) {
            h.hash(&block.family.key());
            for request in product.artifacts.iter() {
                h.hash(&request.key());
            }
        }
        let key = h.finish_hash();
        let base = derived::binding_metadata_bytes(&source.source, &profile)?
            .checked_add(
                size_of::<PreparedPetsc>()
                    + size_of::<Product>()
                    + size_of::<SolverProfile>()
                    + 4 * size_of::<usize>(),
            )
            .and_then(|n| n.checked_add(domain.as_ref().map_or(0, |d| d.retained_bytes())))
            .ok_or(MathRuntimeError::Limit("PETSc profile metadata"))?;
        let bytes = blocks.iter().try_fold(base, |n, b| {
            n.checked_add(b.family.retained_bytes())
                .and_then(|n| n.checked_add(block_binding_bytes(b, true).ok()?))
                .ok_or(MathRuntimeError::Limit("PETSc compact prepared blocks"))
        })?;
        let lease = self.reserve("math:petsc-compiled-blocks", bytes)?;
        scope.check().map_err(ProblemError::from)?;
        Ok(PreparedPetsc {
            source,
            profile: Arc::new(profile),
            settings,
            domain,
            product: Arc::new(Product::Blocks {
                blocks,
                declaration,
            }),
            compatibility: Compatibility {
                layout: key,
                profile: key,
                data: key,
                backend: Backend::Petsc,
            },
            key,
            _owner: lease,
        })
    }
}

#[derive(Debug)]
struct LocalBlock {
    prepared: CompiledBlock,
    worker: pse_math::assembly::CaseWorker,
    scope: ExecutionScope,
    realization: ContentHash,
    _owner: (Arc<ExecutableCase>, super::super::WorkerCharge),
}

#[derive(Debug)]
struct CompiledDomain {
    worker: pse_math::assembly::CaseWorker,
    values: CaseValues,
    columns: Vec<pse_ids::SemanticId>,
    source: ContentHash,
    scope: ExecutionScope,
    supplemental: Option<Box<dyn DomainOracle>>,
    _owner: (Arc<ExecutableCase>, super::super::WorkerCharge),
}
impl DomainOracle for CompiledDomain {
    fn source(&self) -> ContentHash {
        self.source
    }
    fn check(&mut self, point: &[f64]) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::from)?;
        if point.len() != self.columns.len() || point.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "PETSc complete original domain coordinate extent".into(),
            ));
        }
        for (id, value) in self.columns.iter().zip(point) {
            self.values.scalars.insert(*id, *value);
        }
        // These are the complete selected original demands, including body domain,
        // applicability and provider stages. A rejected claim returns its original
        // typed MathError::Applicability from the library evaluator.
        self.worker.objective(&self.values)?;
        self.worker.constraints(&self.values)?;
        self.worker.jacobian(&self.values)?;
        if self
            .worker
            .applicability_observations()
            .iter()
            .any(|o| o.required && !o.admitted)
        {
            return Err(ProblemError::Contract(
                "original compiled applicability evaluator published an unadmitted required claim"
                    .into(),
            ));
        }
        if let Some(extra) = self.supplemental.as_mut() {
            extra.check(point)?;
        }
        self.scope.check().map_err(ProblemError::from)?;
        Ok(())
    }
}
impl LocalBlock {
    fn point(
        &mut self,
        local: &[f64],
        ghosts: native_petsc::GhostCoordinates<'_>,
    ) -> Result<(), ProblemError> {
        self.scope.check().map_err(ProblemError::from)?;
        if local.len() != self.prepared.local_ids.len()
            || ghosts.columns.len() != ghosts.values.len()
            || local.iter().chain(ghosts.values).any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "PETSc compact block point extent or finite values".into(),
            ));
        }
        for (id, value) in self.prepared.local_ids.iter().zip(local) {
            self.prepared.values.scalars.insert(*id, *value);
        }
        for (column, id) in &self.prepared.external {
            let slot = ghosts.columns.binary_search(column).map_err(|_| {
                ProblemError::Contract(
                    "PETSc compact ghost lacks a compiler-selected external coordinate".into(),
                )
            })?;
            self.prepared
                .values
                .scalars
                .insert(*id, ghosts.values[slot]);
        }
        Ok(())
    }
}
impl native_petsc::BlockOracle for LocalBlock {
    fn family(&self) -> &Arc<DerivedFamily> {
        &self.prepared.family
    }
    fn realization(&self) -> ContentHash {
        self.realization
    }
    fn external_coordinates(&self) -> Vec<GlobalCol> {
        self.prepared
            .external
            .iter()
            .map(|(column, _)| *column)
            .collect()
    }
    fn residual(
        &mut self,
        local: &[f64],
        ghosts: native_petsc::GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.point(local, ghosts)?;
        let mut values = self.worker.constraints(&self.prepared.values)?;
        if values.len() != out.len() || values.len() != self.prepared.offsets.len() {
            return Err(ProblemError::Contract(
                "PETSc compiler block residual extent".into(),
            ));
        }
        for (value, offset) in values.iter_mut().zip(&self.prepared.offsets) {
            *value -= offset;
        }
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite compiler block residual"));
        }
        self.scope.check().map_err(ProblemError::from)?;
        out.copy_from_slice(&values);
        Ok(())
    }
    fn jacobian(
        &mut self,
        local: &[f64],
        ghosts: native_petsc::GhostCoordinates<'_>,
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.point(local, ghosts)?;
        if out.len() != self.prepared.family.incidence().len() {
            return Err(ProblemError::Contract(
                "PETSc compiler block Jacobian extent".into(),
            ));
        }
        let matrix = self.worker.jacobian(&self.prepared.values)?;
        if matrix.val().len() != self.prepared.jacobian_slots.len() {
            return Err(ProblemError::Contract(
                "PETSc compiler block Jacobian changed its admitted structure".into(),
            ));
        }
        let mut values = vec![0.0; out.len()];
        for (slot, value) in self.prepared.jacobian_slots.iter().zip(matrix.val()) {
            values[*slot] = *value;
        }
        if values.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite compiler block Jacobian"));
        }
        self.scope.check().map_err(ProblemError::from)?;
        out.copy_from_slice(&values);
        Ok(())
    }
}
impl MathService {
    /// Execute on the existing NativeSession worker, retaining its original admission,
    /// cancellation, progress and deadline. The return value is an auxiliary observation.
    pub(crate) fn execute_petsc(
        &self,
        prepared: &PreparedPetsc,
        mut execution: Execution,
        budget: &Arc<WorkerBudget>,
        original_start: &[f64],
    ) -> Result<DerivedAttempt, MathRuntimeError> {
        let source = &prepared.source;
        if !Arc::ptr_eq(&execution.cancel, source.scope.cancellation()) {
            return Err(ProblemError::Contract(
                "PETSc product must share original task cancellation".into(),
            )
            .into());
        }
        let outer = execution.scope()?;
        let deadline = match (outer.deadline(), source.scope.deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let scope = ExecutionScope::new(execution.cancel.clone(), deadline);
        scope.check().map_err(ProblemError::from)?;
        execution.enclosing_scope = Some(scope.clone());
        if let Some(deadline) = deadline {
            execution.time_limit = execution
                .time_limit
                .min(deadline.saturating_duration_since(execution.started));
        }
        execution.check()?;
        let n = source.physical.coordinates().len();
        if original_start.len() != n || original_start.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "PETSc original start coordinate extent or finite values".into(),
            )
            .into());
        }
        if execution
            .memory
            .is_none_or(|n| n < self.policy.foreign_allowance(&prepared.profile.controls))
        {
            return Err(ProblemError::memory(
                "PETSc attempt does not retain its resolved foreign allowance",
            )
            .into());
        }
        let product_bytes = match prepared.product.as_ref() {
            Product::Flow { mass, .. } => mass.worker_bytes(),
            Product::Blocks { blocks, .. } => blocks.iter().try_fold(0usize, |sum, b| {
                sum.checked_add(block_binding_bytes(b, false)?)
                    .ok_or(MathRuntimeError::Limit(
                        "PETSc compact block attempt extent",
                    ))
            })?,
        };
        // The original numerical supplier and the independent complete-domain supplier
        // each own one map. Compact child workers own only their admitted input slots.
        let source_values = derived::case_values_bytes(source.source.values.scalars.len())?
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(n.checked_mul(size_of::<pse_ids::SemanticId>())?))
            .ok_or(MathRuntimeError::Limit("PETSc complete domain values"))?;
        let bytes = product_bytes
            .checked_add(prepared.domain.as_ref().map_or(0, |d| d.worker_bytes()))
            .and_then(|bytes| bytes.checked_add(n.checked_mul(8 * size_of::<f64>())?))
            .and_then(|bytes| bytes.checked_add(source_values))
            .ok_or(MathRuntimeError::Limit("PETSc supplier attempt extent"))?;
        let _supplier_charge = budget.charge(bytes)?;
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self.case_worker(
            source.source.case.clone(),
            source.source.providers.clone(),
            &scope,
            budget,
        )?;
        let _original_owner = (_case, _charge);
        let _root_support =
            budget.charge(native::assembled::AlgebraicOracle::root_support_allowance(
                &source.source.prepared.compiled().plan,
                &source.source.prepared.compiled().structure,
            )?)?;
        let actual = native::assembled::AlgebraicOracle::new(worker, source.source.values.clone())?
            .with_structural_analysis(source.source.prepared.compiled().structure.clone())?
            .with_presolve_facts(source.source.prepared.compiled().presolve.clone())?
            .with_normalization(source.original.normalization.clone())?;
        actual.admit_nle()?;
        let mut original = ZeroRoot::new(Box::new(actual), source)?;
        let supplemental = match &prepared.domain {
            Some(factory) => {
                let actual = factory.bind(&source.physical, scope.clone())?;
                if factory.inventory() != source.guards.as_ref()
                    || actual.source() != source.physical.obligations().guards
                {
                    return Err(ProblemError::Contract("PETSc supplemental domain supplier changed its full frozen original obligation".into()).into());
                }
                Some(actual)
            }
            None => None,
        };
        let ExecutionWorker {
            worker,
            _case,
            _charge,
        } = self.case_worker(
            source.source.case.clone(),
            source.source.providers.clone(),
            &scope,
            budget,
        )?;
        let mut domain = CompiledDomain {
            worker,
            values: source.source.values.clone(),
            columns: source.source.prepared.compiled().plan.columns().to_vec(),
            source: source.physical.obligations().guards,
            scope: scope.clone(),
            supplemental,
            _owner: (_case, _charge),
        };
        execution.check()?;
        let mut report = match prepared.product.as_ref() {
            Product::Flow {
                family,
                mass,
                limits,
            } => {
                let actions = mass.bind(scope.clone())?;
                let mut refresh = mass.refresh(scope.clone())?;
                if mass.binding() != (source.original_identity(), source.preparation_identity()) {
                    return Err(ProblemError::Contract(
                        "PETSc physical mass source changed its frozen binding".into(),
                    )
                    .into());
                }
                if let MassBinding::StateDependent(actual) = &actions
                    && (actual.realization() != mass.realization()
                        || actual.structure() != mass.structure())
                {
                    return Err(ProblemError::Contract(
                        "PETSc actual mass/DM realization differs from its declared producer"
                            .into(),
                    )
                    .into());
                }
                let mut flow = native_petsc::ArtificialFlow::new(
                    family.clone(),
                    &mut original,
                    actions,
                    &mut domain,
                )?;
                if let Some(refresh) = refresh.as_mut() {
                    flow = flow.with_frozen_refresh(refresh.as_mut())?;
                }
                native_petsc::solve_flow(
                    &mut flow,
                    original_start,
                    &prepared.settings,
                    *limits,
                    native_petsc::SolveRequest {
                        controls: &prepared.profile.controls,
                        accuracy: &source.original.accuracy,
                        execution: execution.clone(),
                        tolerances: &source.original.tolerances,
                        warm: None,
                        compatibility: &prepared.compatibility,
                    },
                )?
            }
            Product::Blocks {
                blocks,
                declaration,
            } => {
                let mut locals = Vec::with_capacity(blocks.len());
                for block in blocks {
                    execution.check()?;
                    let ExecutionWorker {
                        worker,
                        _case,
                        _charge,
                    } = self.worker(
                        block.executable.clone(),
                        &source.source.providers,
                        scope.clone(),
                        budget,
                    )?;
                    locals.push(LocalBlock {
                        prepared: block.clone(),
                        worker,
                        scope: scope.clone(),
                        realization: prepared.key,
                        _owner: (_case, _charge),
                    });
                }
                let mut specs = Vec::with_capacity(locals.len());
                for local in &mut locals {
                    let interior = local
                        .prepared
                        .family
                        .coordinate_map()
                        .ok_or_else(|| {
                            ProblemError::Contract("PETSc compiled block has no local map".into())
                        })?
                        .to_vec();
                    specs.push(native_petsc::BlockSpec::new(
                        local,
                        interior,
                        native_petsc::DomainWitness {
                            source: source.physical.obligations().guards,
                            declaration: *declaration,
                        },
                    )?);
                }
                let mut composition =
                    native_petsc::BlockComposition::new(&mut original, &mut domain, specs)?;
                native_petsc::solve_blocks(
                    &mut composition,
                    original_start,
                    &prepared.settings,
                    native_petsc::SolveRequest {
                        controls: &prepared.profile.controls,
                        accuracy: &source.original.accuracy,
                        execution: execution.clone(),
                        tolerances: &source.original.tolerances,
                        warm: None,
                        compatibility: &prepared.compatibility,
                    },
                )?
            }
        };
        report.provenance.insert(
            "mathematical_role".into(),
            "auxiliary-original-coordinate-proposal".into(),
        );
        report.provenance.insert(
            "physical_original".into(),
            source.physical.identity().to_string(),
        );
        report.provenance.insert(
            "compiled_source".into(),
            source.preparation_identity().to_string(),
        );
        // Native callback counts remain available in Evidence. A supplied domain or mass
        // can perform additional library work whose inclusive extent is not observed here.
        report.evidence.work.evaluations = None;
        let outcome = Outcome::Native(Box::new(report));
        let observation = super::super::strategy::observe(&outcome);
        let cause = super::super::strategy::cause(&outcome);
        let permitted = derived::proposal_observation_allowed(observation)
            && cause.as_deref().is_none_or(|cause| {
                derived::proposal_observation_allowed(super::super::strategy::failure(cause))
            });
        let mut report = match outcome {
            Outcome::Native(report) => *report,
            _ => {
                return Err(ProblemError::Internal(
                    "PETSc auxiliary runner lost its native report".into(),
                )
                .into());
            }
        };
        let mut screening_failure = None;
        let mut proposal = None;
        if permitted
            && !report.evidence.callback.terminal_failure
            && let Some(candidate) = &report.candidate
        {
            let screen = (|| {
                execution.check()?;
                domain.check(&candidate.primal)?;
                let mut residual = vec![0.0; n];
                original.residual(&candidate.primal, &mut residual)?;
                if residual.iter().any(|v| !v.is_finite()) {
                    return Err(ProblemError::numerical(
                        "PETSc original proposal screen produced nonfinite source values",
                    ));
                }
                domain.check(&candidate.primal)?;
                execution.check()?;
                let coordinates = candidate.primal.clone();
                let bytes = coordinates
                    .capacity()
                    .checked_mul(size_of::<f64>())
                    .and_then(|n| n.checked_add(size_of::<OriginalProposal>()))
                    .ok_or_else(|| ProblemError::memory("PETSc original proposal extent"))?;
                let owner = self
                    .reserve("math:petsc-original-proposal", bytes)
                    .map_err(MathRuntimeError::into_problem)?;
                OriginalProposal::from_original(
                    source.original_identity(),
                    prepared.key,
                    coordinates,
                    &source.physical,
                    None,
                    owner,
                )
            })();
            match screen {
                Ok(point) => proposal = Some(point),
                Err(error) => screening_failure = Some(Arc::new(error)),
            }
        }
        // Native handles and callback suppliers end before the final clock observation.
        // A late failure preserves native code/work/candidate and withdraws proposal use.
        drop(domain);
        drop(original);
        drop(_original_owner);
        drop(_root_support);
        drop(_supplier_charge);
        if let Err(error) = execution.check() {
            proposal = None;
            report.evidence.callback.terminal_failure = true;
            report.termination.category = execution.stopped().unwrap_or(Termination::Limit);
            if report.validation_failure().is_none() {
                report.record_validation_failure(error);
            } else {
                screening_failure = Some(Arc::new(error));
            }
        }
        Ok(DerivedAttempt {
            report,
            proposal,
            screening_failure,
            refinement_product: None,
        })
    }
}

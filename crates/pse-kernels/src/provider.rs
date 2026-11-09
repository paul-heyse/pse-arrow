// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable attempt bindings and caller-owned provider completion.
use super::*;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicU64;

/// Original retirement failure attributed to its admitted provider identity.
#[derive(Debug, thiserror::Error)]
#[error("provider {} retirement: {cause}", .key.0)]
pub struct ProviderRetirementFailure {
    /// Bank membership identity, including an admitted narrowed capability.
    pub key: ProviderKey,
    /// Original provider failure; typed nested causes remain owned and inspectable.
    #[source]
    pub cause: Box<ProviderError>,
}
impl ProviderRetirementFailure {
    /// Owned failure extent, including its original source allocation.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.cause.retained_bytes())
    }
}
pse_diagnostics::impl_diagnostic! {
    ProviderRetirementFailure,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure) },
    forward(_this) { None }, help(_this) { None },
    related(this) { let cause: &dyn pse_diagnostics::TypedDiagnostic = this.cause.as_ref(); Some(Box::new(std::iter::once(cause))) },
    source(this) { Some(this.cause.as_ref()) },
    facts(this) {
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        facts.observe("provider_key", pse_diagnostics::DiagnosticObservation::Text(this.key.0.to_string()));
        facts
    }
}

use pse_contract::boundary_projection::Direction;

pse_contract::projection_enum! {
/// Exact local derivative product; numeric seeds belong to each invocation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ProviderDemand {
    projection: { name: "ProviderDemand", rust_path: crate::ProviderDemand, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Output-major complete partial arrays at the request's derivative order.
    #[default]
    CompleteJets => 1;
    /// Ordered input columns; Second returns the selected column square.
    SelectedColumns {
        /// Ordered unique local input ordinals.
        columns: Vec<usize> => { tag: 1, direction: Direction::Both, rules: [] };
    } => 2;
    /// Output-major forward actions for row-major input directions.
    JacobianAction {
        /// Number of row-major input directions.
        directions: usize => { tag: 1, direction: Direction::Both, rules: [] };
    } => 3;
    /// Cotangent-major transpose actions for row-major output cotangents.
    TransposeAction {
        /// Number of row-major selected-output cotangents.
        cotangents: usize => { tag: 1, direction: Direction::Both, rules: [] };
    } => 4;
    /// One weighted gradient and input-major weighted second actions.
    WeightedSecondAction {
        /// Number of row-major input directions.
        directions: usize => { tag: 1, direction: Direction::Both, rules: [] };
    } => 5;
}
}

/// Invocation-local numeric bindings, excluded from immutable prepared identity.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProviderRuntime<'a> {
    /// Direction-major input directions.
    pub directions: &'a [f64],
    /// Cotangent-major weights in selected output order.
    pub cotangents: &'a [f64],
    /// One weight per selected output for the weighted second product.
    pub weights: &'a [f64],
    /// Prior owner-sealed context; current effects must re-establish it.
    pub expected_linearization: Option<ProviderLinearizationToken>,
    /// Capture a passive selected-regime fact from this exact fill.
    pub observe_selection: bool,
}

/// Attempt-local proof of one provider owner's complete derivative context.
/// It never grants permission to omit a current invocation's effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderLinearizationToken {
    binding: u64,
    context: ContentHash,
    inputs: ContentHash,
}
impl ProviderLinearizationToken {
    /// Include an actually visited child premise in its enclosing owner's proof.
    pub fn frame(&self, hasher: &mut pse_ids::FramedHasher) {
        hasher
            .part(&self.binding.to_le_bytes())
            .part(self.context.as_bytes())
            .part(self.inputs.as_bytes());
    }
    fn seal(binding: u64, context: ContentHash, inputs: &[f64]) -> Self {
        let mut hash = pse_ids::FramedHasher::new(pse_ids::Frame::ProviderConfigurationV1);
        hash.part(b"attempt-linearization-inputs");
        for input in inputs {
            hash.part(&input.to_bits().to_le_bytes());
        }
        Self {
            binding,
            context,
            inputs: hash.finish_hash(),
        }
    }
}

/// The selected provider's current mathematical fact. No variant claims physical stability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderSelection {
    /// Original authored selector.
    pub selector: SemanticId,
    /// Actual winning authored regime.
    pub regime: SemanticId,
    /// Admitted selector policy and implementation configuration.
    pub policy: ContentHash,
    /// Current numerical product, including its resolved alternative configurations.
    pub product: ContentHash,
    /// Order established by the selection, independently of the filled derivative product.
    pub selection_order: DerivativeOrder,
    /// Mathematical selection qualification actually observed.
    pub regularity: ProviderSelectionRegularity,
}

/// Passive observation sealed by the actual successful provider binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderSelectionObservation {
    binding: u64,
    provider: ProviderKey,
    invocation: ContentHash,
    requested_order: DerivativeOrder,
    selection: ProviderSelection,
}
impl ProviderSelectionObservation {
    /// Actual bound provider interpretation, including its ordered ports.
    pub fn provider(&self) -> ProviderKey {
        self.provider
    }

    /// Whether two observations came from the same process-local binding.
    pub fn same_binding(&self, other: &Self) -> bool {
        self.binding == other.binding
    }

    /// Exact ordered input, request and action-seed interpretation of the fill.
    pub fn invocation(&self) -> ContentHash {
        self.invocation
    }

    /// Order of the requested and completed fill product.
    pub fn requested_order(&self) -> DerivativeOrder {
        self.requested_order
    }

    /// Supplier fact captured from this invocation.
    pub fn selection(&self) -> ProviderSelection {
        self.selection
    }

    fn seal(
        binding: &ProviderBinding,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        selection: ProviderSelection,
    ) -> Self {
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ProviderConfigurationV1);
        h.str("selected-provider-invocation-v1")
            .hash(&binding.key.0)
            .hash(&request.key.0)
            .u64(request.order as u64)
            .u64(request.outputs.len() as u64);
        for &output in &request.outputs {
            h.u64(output as u64);
        }
        match &request.demand {
            ProviderDemand::CompleteJets => {
                h.str("complete-jets");
            }
            ProviderDemand::SelectedColumns { columns } => {
                h.str("selected-columns").u64(columns.len() as u64);
                for &column in columns {
                    h.u64(column as u64);
                }
            }
            ProviderDemand::JacobianAction { directions } => {
                h.str("forward").u64(*directions as u64);
            }
            ProviderDemand::TransposeAction { cotangents } => {
                h.str("transpose").u64(*cotangents as u64);
            }
            ProviderDemand::WeightedSecondAction { directions } => {
                h.str("weighted-second").u64(*directions as u64);
            }
        }
        for values in [inputs, runtime.directions, runtime.cotangents, runtime.weights] {
            h.u64(values.len() as u64);
            for value in values {
                h.u64(value.to_bits());
            }
        }
        Self {
            binding: binding.identity,
            provider: binding.key,
            invocation: h.finish_hash(),
            requested_order: request.order,
            selection,
        }
    }
}

/// Checked result extents, in scalar cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProviderExtents {
    /// Physical values in requested output order.
    pub values: usize,
    /// First partials or first actions in the demand's declared order.
    pub jacobian: usize,
    /// Second partials or second actions in the demand's declared order.
    pub hessians: usize,
}
impl ProviderExtents {
    /// Checked total storage bytes for all demanded scalar cells.
    pub fn bytes(self) -> Result<usize, ProviderError> {
        self.values
            .checked_add(self.jacobian)
            .and_then(|n| n.checked_add(self.hessians))
            .and_then(|n| n.checked_mul(size_of::<f64>()))
            .ok_or(ProviderError::Limit("property result bytes"))
    }
}

/// Validated immutable request and descriptor, prepared before provider effects.
#[derive(Clone, Debug)]
pub struct PreparedProviderRequest {
    request: ProviderRequest,
    spec: Arc<ProviderSpec>,
    key: ProviderKey,
    extents: ProviderExtents,
    binding: Option<u64>,
}
impl PreparedProviderRequest {
    /// Validate immutable ordinals, capability and representation extents once.
    pub fn new(spec: Arc<ProviderSpec>, request: ProviderRequest) -> Result<Self, ProviderError> {
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("kernel.provider.request_preparations", "count", 1);
        }
        let n = spec.inputs.len();
        let m = request.outputs.len();
        let unique = |ordinals: &[usize], bound: usize| {
            ordinals.iter().all(|&i| i < bound)
                && ordinals
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == ordinals.len()
        };
        if m == 0
            || request.order > spec.derivatives
            || request.order > spec.smoothness
            || !unique(&request.outputs, spec.outputs.len())
        {
            return Err(ProviderError::Contract(
                "property demand or derivative profile".into(),
            ));
        }
        let product = |a: usize, b: usize| {
            a.checked_mul(b)
                .ok_or(ProviderError::Limit("property result bytes"))
        };
        let (first, second) = match &request.demand {
            ProviderDemand::CompleteJets => (n, product(n, n)?),
            ProviderDemand::SelectedColumns { columns } => {
                if request.order == DerivativeOrder::Value
                    || columns.is_empty()
                    || !unique(columns, n)
                {
                    return Err(ProviderError::Contract("selected provider columns".into()));
                }
                (columns.len(), product(columns.len(), columns.len())?)
            }
            ProviderDemand::JacobianAction { directions } => {
                if request.order != DerivativeOrder::First || *directions == 0 {
                    return Err(ProviderError::Contract(
                        "provider forward action demand".into(),
                    ));
                }
                (*directions, 0)
            }
            ProviderDemand::TransposeAction { cotangents } => {
                if request.order != DerivativeOrder::First || *cotangents == 0 {
                    return Err(ProviderError::Contract(
                        "provider transpose action demand".into(),
                    ));
                }
                let extents = ProviderExtents {
                    values: m,
                    jacobian: product(*cotangents, n)?,
                    hessians: 0,
                };
                extents.bytes()?;
                return Ok(Self {
                    key: spec.key(),
                    spec,
                    request,
                    extents,
                    binding: None,
                });
            }
            ProviderDemand::WeightedSecondAction { directions } => {
                if request.order != DerivativeOrder::Second || *directions == 0 {
                    return Err(ProviderError::Contract(
                        "provider weighted second action demand".into(),
                    ));
                }
                let extents = ProviderExtents {
                    values: m,
                    jacobian: n,
                    hessians: product(n, *directions)?,
                };
                extents.bytes()?;
                return Ok(Self {
                    key: spec.key(),
                    spec,
                    request,
                    extents,
                    binding: None,
                });
            }
        };
        let extents = ProviderExtents {
            values: m,
            jacobian: if request.order >= DerivativeOrder::First {
                product(m, first)?
            } else {
                0
            },
            hessians: if request.order >= DerivativeOrder::Second {
                product(m, second)?
            } else {
                0
            },
        };
        extents.bytes()?;
        Ok(Self {
            key: spec.key(),
            spec,
            request,
            extents,
            binding: None,
        })
    }
    /// Immutable authored demand.
    pub fn request(&self) -> &ProviderRequest {
        &self.request
    }
    /// Immutable physical descriptor.
    pub fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    /// Prepared provider identity, without descriptor hashing at invocation.
    pub fn key(&self) -> ProviderKey {
        self.key
    }
    /// Validated result populations.
    pub fn extents(&self) -> ProviderExtents {
        self.extents
    }
    /// Check a fresh worker's immutable facts and actual product support once.
    pub fn bind(&mut self, provider: &ProviderBinding) -> Result<(), ProviderError> {
        provider.check_prepared(self)?;
        self.binding = Some(provider.identity);
        Ok(())
    }
    /// Recheck current cancellation, deadline and result allowance.
    pub fn check_context(&self, context: &EvaluationContext<'_>) -> Result<(), ProviderError> {
        context.check()?;
        if self.extents.bytes()? > context.max_result_bytes {
            return Err(ProviderError::Limit("property result bytes"));
        }
        Ok(())
    }
    fn check_runtime(&self, runtime: ProviderRuntime<'_>) -> Result<(), ProviderError> {
        let n = self.spec.inputs.len();
        let m = self.request.outputs.len();
        let (directions, cotangents, weights) = match self.request.demand {
            ProviderDemand::CompleteJets | ProviderDemand::SelectedColumns { .. } => {
                (Some(0), Some(0), 0)
            }
            ProviderDemand::JacobianAction { directions } => {
                (directions.checked_mul(n), Some(0), 0)
            }
            ProviderDemand::TransposeAction { cotangents } => {
                (Some(0), cotangents.checked_mul(m), 0)
            }
            ProviderDemand::WeightedSecondAction { directions } => {
                (directions.checked_mul(n), Some(0), m)
            }
        };
        if Some(runtime.directions.len()) != directions
            || Some(runtime.cotangents.len()) != cotangents
            || runtime.weights.len() != weights
            || runtime
                .directions
                .iter()
                .chain(runtime.cotangents)
                .chain(runtime.weights)
                .any(|x| !x.is_finite())
        {
            return Err(ProviderError::Contract(
                "provider runtime product bindings".into(),
            ));
        }
        Ok(())
    }
}
impl std::ops::Deref for PreparedProviderRequest {
    type Target = ProviderRequest;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

/// Mutable invocation storage. Its contents are not a completed product.
#[derive(Debug)]
pub struct ProviderOutput<'a> {
    /// Invocation-local physical value destinations.
    pub values: &'a mut [f64],
    /// Invocation-local first derivative destinations.
    pub jacobian: &'a mut [f64],
    /// Invocation-local second derivative destinations.
    pub hessians: &'a mut [f64],
}
impl ProviderOutput<'_> {
    /// Temporarily lend destinations through another fallible wrapper.
    pub fn reborrow(&mut self) -> ProviderOutput<'_> {
        ProviderOutput {
            values: self.values,
            jacobian: self.jacobian,
            hessians: self.hessians,
        }
    }
    /// Copy an independently completed owned numerical product into exact destinations.
    pub fn copy_from(&mut self, values: &ProviderValues) -> Result<(), ProviderError> {
        if self.values.len() != values.values.len()
            || self.jacobian.len() != values.jacobian.len()
            || self.hessians.len() != values.hessians.len()
        {
            return Err(ProviderError::Contract("provider fill extent".into()));
        }
        self.values.copy_from_slice(&values.values);
        self.jacobian.copy_from_slice(&values.jacobian);
        self.hessians.copy_from_slice(&values.hessians);
        Ok(())
    }
}

/// Expiring read view issued only after successful outer invocation checks.
#[derive(Debug)]
pub struct ProviderCompletion<'a> {
    /// Successfully completed physical values.
    pub values: &'a [f64],
    /// Successfully completed first derivatives or actions.
    pub jacobian: &'a [f64],
    /// Successfully completed second derivatives or actions.
    pub hessians: &'a [f64],
    /// Complete derivative premise issued by this actual successful invocation.
    pub linearization: Option<ProviderLinearizationToken>,
    /// Optional passive fact issued by this successful invocation.
    pub selection: Option<ProviderSelectionObservation>,
}

/// Caller-owned admitted scratch; stale or partially filled cells have no read view.
#[derive(Clone, Debug)]
pub struct ProviderBuffer {
    values: Vec<f64>,
    jacobian: Vec<f64>,
    hessians: Vec<f64>,
}
impl ProviderBuffer {
    /// Allocate destinations after the caller admits their prepared extents.
    pub fn new(request: &PreparedProviderRequest) -> Self {
        let e = request.extents();
        Self {
            values: vec![f64::NAN; e.values],
            jacobian: vec![f64::NAN; e.jacobian],
            hessians: vec![f64::NAN; e.hessians],
        }
    }
    /// Fill admitted scratch and borrow only the successfully completed invocation.
    pub fn fill<'a>(
        &'a mut self,
        provider: &mut ProviderBinding,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderCompletion<'a>, ProviderError> {
        provider.fill(
            inputs,
            request,
            runtime,
            ProviderOutput {
                values: &mut self.values,
                jacobian: &mut self.jacobian,
                hessians: &mut self.hessians,
            },
            context,
        )
    }
}

/// Fresh worker paired with its immutable descriptor. The worker cannot replace it.
#[derive(Debug)]
pub struct ProviderBinding {
    identity: u64,
    spec: Arc<ProviderSpec>,
    key: ProviderKey,
    linearization_capability: bool,
    selection_observation: bool,
    worker: Box<dyn Provider>,
}
impl ProviderBinding {
    /// Consume this binding and await its actual worker's physical retirement.
    /// Returning an error still establishes completed physical retirement.
    pub fn finish(self) -> Result<(), ProviderError> {
        let key = self.key;
        self.finish_inner().map_err(|error| match error {
            error @ (ProviderError::Retirement { .. } | ProviderError::Settlement { .. }) => error,
            cause => ProviderError::Retirement {
                failures: vec![ProviderRetirementFailure {
                    key,
                    cause: Box::new(cause),
                }],
            },
        })
    }
    // The enclosing bank supplies admitted-key attribution and nonrecoverability.
    pub(crate) fn finish_inner(self) -> Result<(), ProviderError> {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.worker.finish())) {
            Ok(result) => result,
            Err(payload) => Err(ProviderError::RetirementPanic(
                pse_diagnostics::panic::settle_payload(
                    payload,
                    "non-string provider retirement panic",
                ),
            )),
        }
    }
    /// Pair fresh mutable implementation state with its immutable descriptor.
    pub fn new(spec: ProviderSpec, worker: Box<dyn Provider>) -> Self {
        let linearization_capability = worker.supports_linearization();
        let selection_observation = worker.observes_selection();
        Self {
            identity: NEXT_BANK.fetch_add(1, Ordering::Relaxed),
            key: spec.key(),
            spec: Arc::new(spec),
            linearization_capability,
            selection_observation,
            worker,
        }
    }
    /// Inspect immutable bound physical meaning.
    pub fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    /// Cached lookup identity.
    pub fn key(&self) -> ProviderKey {
        self.key
    }
    /// Validate and bind a public demand before any evaluation effects.
    pub fn prepare(
        &mut self,
        request: ProviderRequest,
    ) -> Result<PreparedProviderRequest, ProviderError> {
        let prepared = PreparedProviderRequest::new(self.spec.clone(), request)?;
        self.prepare_checked(prepared)
    }
    /// Bind an admitted demand to this worker and prepare its supported native product.
    pub fn prepare_checked(
        &mut self,
        mut prepared: PreparedProviderRequest,
    ) -> Result<PreparedProviderRequest, ProviderError> {
        prepared.bind(self)?;
        self.prepare_demand(&prepared)?;
        Ok(prepared)
    }
    /// Check prepared meaning and implemented product support at fresh binding.
    pub fn check_prepared(&self, request: &PreparedProviderRequest) -> Result<(), ProviderError> {
        self.spec
            .check_bound(request.spec(), request.request.order)?;
        if !self.worker.supports(&request.request.demand) {
            return Err(ProviderError::Contract(
                "provider product unavailable".into(),
            ));
        }
        Ok(())
    }
    /// Forward immutable capability through a wrapping binding.
    pub fn supports(&self, demand: &ProviderDemand) -> bool {
        self.worker.supports(demand)
    }
    /// Exact prepared-demand group capability, without scalar fallback.
    pub fn supports_batch(&self, request: &PreparedProviderRequest) -> bool {
        self.worker.supports_batch(request)
    }
    pub(super) fn batch_inner(
        &mut self,
        points: &[&ProviderBatchPoint],
        request: &PreparedProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderBatchResults, ProviderError> {
        self.worker.fill_batch(points, request, context)
    }
    /// Check whole-frame integrity and admitted storage without mutating native preparation.
    pub fn check_batch_frame(
        &self,
        points: &[ProviderBatchPoint],
        request: &PreparedProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        context.check()?;
        if request.binding.is_none() {
            return Err(ProviderError::Contract(
                "provider request has no executable binding".into(),
            ));
        }
        self.check_prepared(request)?;
        request.check_context(context)?;
        if !self.supports_batch(request) {
            return Err(ProviderError::Contract(
                "provider batch demand unavailable".into(),
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut bytes = 0usize;
        for point in points {
            if point.id == SemanticId::NIL || !ids.insert(point.id) {
                return Err(ProviderError::Contract(
                    "provider batch point identities".into(),
                ));
            }
            bytes = bytes
                .checked_add(point.bytes()?)
                .and_then(|n| n.checked_add(size_of::<ProviderBatchOutcome>()))
                .and_then(|n| n.checked_add(request.extents.bytes().ok()?))
                .ok_or(ProviderError::Limit("provider batch storage"))?;
        }
        if points.is_empty() || bytes > context.max_result_bytes {
            return Err(ProviderError::Limit("provider batch storage"));
        }
        Ok(())
    }
    /// Admit a whole identified frame, then gather independently admitted point inputs for one group.
    /// A malformed frame has no effects; one bad point never prevents its admitted siblings.
    pub fn evaluate_batch(
        &mut self,
        points: &[ProviderBatchPoint],
        request: &PreparedProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<Vec<ProviderBatchOutcome>, ProviderError> {
        self.check_batch_frame(points, request, context)?;
        let mut outcomes: Vec<_> = points
            .iter()
            .map(|point| ProviderBatchOutcome {
                id: point.id,
                result: Err(ProviderError::Contract(
                    "provider point not admitted".into(),
                )),
            })
            .collect();
        let mut eligible = Vec::with_capacity(points.len());
        let mut ordinals = Vec::with_capacity(points.len());
        for (ordinal, point) in points.iter().enumerate() {
            let admission = if point.inputs.len() != self.spec.inputs.len()
                || point.inputs.iter().any(|x| !x.is_finite())
            {
                Err(ProviderError::Contract(
                    "provider binding or physical inputs".into(),
                ))
            } else {
                request.check_runtime(point.runtime())
            };
            match admission {
                Ok(()) => {
                    eligible.push(point);
                    ordinals.push(ordinal);
                }
                Err(error) => outcomes[ordinal].result = Err(error),
            }
        }
        if eligible.is_empty() {
            return Ok(outcomes);
        }
        let results = self.batch_inner(&eligible, request, context)?;
        context.check()?;
        if results.len() != eligible.len() {
            return Err(ProviderError::Contract(
                "provider batch completion extent".into(),
            ));
        }
        let extent = request.extents();
        for (ordinal, result) in ordinals.into_iter().zip(results) {
            outcomes[ordinal].result = result.and_then(|value| {
                if value.values.len() != extent.values
                    || value.jacobian.len() != extent.jacobian
                    || value.hessians.len() != extent.hessians
                {
                    return Err(ProviderError::Contract(
                        "provider batch point completion extent".into(),
                    ));
                }
                if value
                    .values
                    .iter()
                    .chain(&value.jacobian)
                    .chain(&value.hessians)
                    .any(|x| !x.is_finite())
                {
                    return Err(ProviderError::Trial(
                        "nonfinite or incomplete provider batch point".into(),
                    ));
                }
                Ok(value)
            });
        }
        Ok(outcomes)
    }
    /// Whether the owner can certify its complete derivative context.
    pub fn supports_linearization(&self) -> bool {
        self.linearization_capability
    }
    pub(super) fn completed_linearization(&self) -> Option<ContentHash> {
        self.worker.completed_linearization()
    }
    pub(super) fn observes_selection(&self) -> bool {
        self.selection_observation
    }
    pub(super) fn take_completed_selection(&mut self) -> Option<ProviderSelection> {
        if self.observes_selection() {
            self.worker.take_completed_selection()
        } else {
            None
        }
    }
    pub(super) fn prepare_demand(
        &mut self,
        request: &PreparedProviderRequest,
    ) -> Result<(), ProviderError> {
        self.worker.prepare_demand(request)
    }
    pub(super) fn fill_inner(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        self.worker.fill(inputs, request, runtime, output, context)
    }
    /// Execute a fallible fill and issue completion after current-state checks.
    pub fn fill<'a>(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        mut output: ProviderOutput<'a>,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderCompletion<'a>, ProviderError> {
        if request.binding.is_none() {
            return Err(ProviderError::Contract(
                "provider request has no executable binding".into(),
            ));
        }
        // Public callers may pass another binding's prepared demand. Only the
        // exact bound worker discharges its capability premise without a check.
        if request.binding != Some(self.identity) {
            self.check_prepared(request)?;
        }
        request.check_context(context)?;
        request.check_runtime(runtime)?;
        if let Some(expected) = runtime.expected_linearization
            && (expected.binding != self.identity || !self.supports_linearization())
        {
            return Err(ProviderError::Contract(
                "foreign or unavailable provider linearization".into(),
            ));
        }
        if inputs.len() != self.spec.inputs.len() || inputs.iter().any(|x| !x.is_finite()) {
            return Err(ProviderError::Contract(
                "provider binding or physical inputs".into(),
            ));
        }
        let e = request.extents();
        if output.values.len() != e.values
            || output.jacobian.len() != e.jacobian
            || output.hessians.len() != e.hessians
        {
            return Err(ProviderError::Contract("provider fill extent".into()));
        }
        output.values.fill(f64::NAN);
        output.jacobian.fill(f64::NAN);
        output.hessians.fill(f64::NAN);
        let filled = self.fill_inner(inputs, request, runtime, output.reborrow(), context);
        let selection = if runtime.observe_selection {
            self.take_completed_selection()
        } else {
            None
        };
        filled?;
        context.check()?;
        if output
            .values
            .iter()
            .chain(output.jacobian.iter())
            .chain(output.hessians.iter())
            .any(|x| !x.is_finite())
        {
            return Err(ProviderError::Trial(
                "nonfinite or incomplete provider fill".into(),
            ));
        }
        let linearization = if self.supports_linearization() {
            self.completed_linearization()
                .map(|key| ProviderLinearizationToken::seal(self.identity, key, inputs))
        } else {
            None
        };
        if runtime.expected_linearization.is_some()
            && runtime.expected_linearization != linearization
        {
            return Err(ProviderError::LinearizationChanged);
        }
        Ok(ProviderCompletion {
            values: output.values,
            jacobian: output.jacobian,
            hessians: output.hessians,
            linearization,
            selection: selection.map(|selection| {
                ProviderSelectionObservation::seal(self, inputs, request, runtime, selection)
            }),
        })
    }
}

static NEXT_BANK: AtomicU64 = AtomicU64::new(1);
/// Attempt-local provider membership. No insert/remove API can invalidate bound ordinals.
#[derive(Debug)]
pub struct ProviderBindings {
    identity: u64,
    slots: Vec<(ProviderKey, ProviderBinding)>,
    index: BTreeMap<ProviderKey, usize>,
}
impl Default for ProviderBindings {
    fn default() -> Self {
        Self::new()
    }
}
impl ProviderBindings {
    /// Consume every owned slot, retaining every keyed original failure.
    /// Cancellation or an earlier failure never suppresses independent teardown.
    /// This returns only after every slot has physically retired, including failures.
    pub fn finish(self) -> Result<(), ProviderError> {
        let mut failures = Vec::new();
        self.finish_into(&mut failures);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(ProviderError::Retirement { failures })
        }
    }
    fn finish_into(self, failures: &mut Vec<ProviderRetirementFailure>) {
        for (key, binding) in self.slots {
            if let Err(cause) = binding.finish_inner() {
                failures.push(ProviderRetirementFailure {
                    key,
                    cause: Box::new(cause),
                });
            }
        }
    }
    /// Construct immutable membership with last-value lookup semantics.
    /// Retire each displaced duplicate before installing its replacement. If any
    /// retirement fails, consume the entire iterator and retire the final bank,
    /// returning every keyed failure instead of publishing an active bank.
    pub fn try_from_iter<T: IntoIterator<Item = (ProviderKey, ProviderBinding)>>(
        iter: T,
    ) -> Result<Self, ProviderError> {
        let mut bank = Self::new();
        let mut failures = Vec::new();
        for (key, binding) in iter {
            if let Some(&slot) = bank.index.get(&key) {
                let (_, displaced) = bank.slots.remove(slot);
                if let Err(cause) = displaced.finish_inner() {
                    failures.push(ProviderRetirementFailure {
                        key,
                        cause: Box::new(cause),
                    });
                }
                bank.slots.insert(slot, (key, binding));
            } else {
                bank.index.insert(key, bank.slots.len());
                bank.slots.push((key, binding));
            }
        }
        if failures.is_empty() {
            Ok(bank)
        } else {
            bank.finish_into(&mut failures);
            Err(ProviderError::Retirement { failures })
        }
    }
    /// Create an empty attempt binding bank with a fresh identity.
    pub fn new() -> Self {
        Self {
            identity: NEXT_BANK.fetch_add(1, Ordering::Relaxed),
            slots: Vec::new(),
            index: BTreeMap::new(),
        }
    }
    /// Stable identity for this immutable attempt membership.
    pub fn identity(&self) -> u64 {
        self.identity
    }
    /// Resolve a prepared key once when a stage first joins this attempt.
    pub fn resolve(&self, key: ProviderKey) -> Option<usize> {
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("kernel.provider.slot_resolutions", "count", 1);
        }
        self.index.get(&key).copied()
    }
    /// Inspect only a selected binding's owner capability before effects.
    pub fn supports_linearization(&self, key: ProviderKey) -> bool {
        self.index
            .get(&key)
            .is_some_and(|&slot| self.slots[slot].1.supports_linearization())
    }
    /// Whether this exact bound provider can emit a selected-regime observation.
    pub fn observes_selection(&self, key: ProviderKey) -> bool {
        self.index
            .get(&key)
            .is_some_and(|&slot| self.slots[slot].1.observes_selection())
    }
    /// Validate a prepared request against a stable slot without exposing membership mutation.
    pub fn bind_request(
        &mut self,
        slot: usize,
        request: &mut PreparedProviderRequest,
    ) -> Result<(), ProviderError> {
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("kernel.provider.request_bindings", "count", 1);
        }
        let (_, binding) = self
            .slots
            .get_mut(slot)
            .ok_or_else(|| ProviderError::Contract("missing provider slot".into()))?;
        request.bind(binding)?;
        binding.prepare_demand(request)
    }
    /// Prepare a selected immutable demand without exposing mutable membership.
    pub fn prepare_request(
        &mut self,
        request: &PreparedProviderRequest,
    ) -> Result<(), ProviderError> {
        let slot = *self
            .index
            .get(&request.key())
            .ok_or_else(|| ProviderError::Contract("missing provider worker".into()))?;
        let binding = &mut self.slots[slot].1;
        binding.check_prepared(request)?;
        binding.prepare_demand(request)
    }
    /// Bind a demand whose owner preparation was completed before numerical execution.
    pub fn bind_prepared_request(
        &self,
        slot: usize,
        request: &mut PreparedProviderRequest,
    ) -> Result<(), ProviderError> {
        let (_, binding) = self
            .slots
            .get(slot)
            .ok_or_else(|| ProviderError::Contract("missing provider slot".into()))?;
        if request.binding == Some(binding.identity) {
            return Ok(());
        }
        request.bind(binding)
    }
    /// Execute a previously resolved slot into caller-owned admitted scratch.
    pub fn fill<'a>(
        &mut self,
        slot: usize,
        buffer: &'a mut ProviderBuffer,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        runtime: ProviderRuntime<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderCompletion<'a>, ProviderError> {
        let (_, binding) = self
            .slots
            .get_mut(slot)
            .ok_or_else(|| ProviderError::Contract("missing provider slot".into()))?;
        if request.binding != Some(binding.identity) {
            return Err(ProviderError::Contract(
                "provider request belongs to a different binding".into(),
            ));
        }
        buffer.fill(binding, inputs, request, runtime, context)
    }
    /// Number of fresh bound workers.
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    /// Whether this attempt has no provider workers.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

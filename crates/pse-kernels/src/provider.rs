// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable attempt bindings and caller-owned provider completion.
use super::*;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicU64;

/// Exact local derivative product; numeric seeds belong to each invocation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ProviderDemand {
    /// Output-major complete partial arrays at the request's derivative order.
    #[default]
    CompleteJets,
    /// Ordered input columns; Second returns the selected column square.
    SelectedColumns {
        /// Ordered unique local input ordinals.
        columns: Vec<usize>,
    },
    /// Output-major forward actions for row-major input directions.
    JacobianAction {
        /// Number of row-major input directions.
        directions: usize,
    },
    /// Cotangent-major transpose actions for row-major output cotangents.
    TransposeAction {
        /// Number of row-major selected-output cotangents.
        cotangents: usize,
    },
    /// One weighted gradient and input-major weighted second actions.
    WeightedSecondAction {
        /// Number of row-major input directions.
        directions: usize,
    },
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
    worker: Box<dyn Provider>,
}
impl ProviderBinding {
    /// Pair fresh mutable implementation state with its immutable descriptor.
    pub fn new(spec: ProviderSpec, worker: Box<dyn Provider>) -> Self {
        let linearization_capability = worker.supports_linearization();
        Self {
            identity: NEXT_BANK.fetch_add(1, Ordering::Relaxed),
            key: spec.key(),
            spec: Arc::new(spec),
            linearization_capability,
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
        let mut prepared = PreparedProviderRequest::new(self.spec.clone(), request)?;
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
    /// Whether the owner can certify its complete derivative context.
    pub fn supports_linearization(&self) -> bool {
        self.linearization_capability
    }
    pub(super) fn completed_linearization(&self) -> Option<ContentHash> {
        self.worker.completed_linearization()
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
        self.fill_inner(inputs, request, runtime, output.reborrow(), context)?;
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
        })
    }
}

static NEXT_BANK: AtomicU64 = AtomicU64::new(1);
/// Attempt-local provider membership. No insert/remove API can invalidate bound ordinals.
#[derive(Debug)]
pub struct ProviderBindings {
    identity: u64,
    slots: Vec<ProviderBinding>,
    index: BTreeMap<ProviderKey, usize>,
}
impl Default for ProviderBindings {
    fn default() -> Self {
        Self::new()
    }
}
impl ProviderBindings {
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
            .is_some_and(|&slot| self.slots[slot].supports_linearization())
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
        let binding = self
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
        let binding = &mut self.slots[slot];
        binding.check_prepared(request)?;
        binding.prepare_demand(request)
    }
    /// Bind a demand whose owner preparation was completed before numerical execution.
    pub fn bind_prepared_request(
        &self,
        slot: usize,
        request: &mut PreparedProviderRequest,
    ) -> Result<(), ProviderError> {
        let binding = self
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
        let binding = self
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
impl FromIterator<(ProviderKey, ProviderBinding)> for ProviderBindings {
    fn from_iter<T: IntoIterator<Item = (ProviderKey, ProviderBinding)>>(iter: T) -> Self {
        let mut bank = Self::new();
        for (key, binding) in iter {
            // Duplicate keys have the same deterministic last-value semantics as
            // the previous BTreeMap, without mutable membership after publication.
            if let Some(&slot) = bank.index.get(&key) {
                bank.slots[slot] = binding;
            } else {
                bank.index.insert(key, bank.slots.len());
                bank.slots.push(binding);
            }
        }
        bank
    }
}
impl<const N: usize> From<[(ProviderKey, ProviderBinding); N]> for ProviderBindings {
    fn from(bindings: [(ProviderKey, ProviderBinding); N]) -> Self {
        bindings.into_iter().collect()
    }
}

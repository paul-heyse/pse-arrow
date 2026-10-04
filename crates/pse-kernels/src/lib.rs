// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical provider contracts, independent of compiler and library scalar types.
use pse_ids::{ContentHash, SemanticId};
use pse_quantity::{QuantityRegistry, QuantityTypeId, UnitId};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

mod shape;
pub use pse_model::generated::enums::ExternalDerivativeSource as DerivativeSource;
pub use shape::{ProviderShape, ProviderShapes};

/// Complete provider interpretation, including ordered ports and explicit parameter data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProviderKey(pub ContentHash);

/// A callback's exact property and derivative demand, in returned output order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderRequest {
    /// Unique output ordinals; a subset does not change provider identity.
    pub outputs: Vec<usize>,
    /// Required raw derivative order.
    pub order: DerivativeOrder,
}
impl ProviderRequest {
    /// Request all outputs in registration order.
    pub fn all(spec: &ProviderSpec, order: DerivativeOrder) -> Self {
        Self {
            outputs: (0..spec.outputs.len()).collect(),
            order,
        }
    }
    /// Reject malformed demand and bounded output allocation before a library call.
    /// # Errors
    /// Returns cancellation, malformed-demand or result-allocation errors.
    pub fn validate(
        &self,
        spec: &ProviderSpec,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        context.check()?;
        if self.outputs.is_empty()
            || self.order > spec.derivatives
            || self.order > spec.smoothness
            || self.outputs.iter().any(|&i| i >= spec.outputs.len())
            || self
                .outputs
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.outputs.len()
        {
            return Err(ProviderError::Contract(
                "property demand or derivative profile".into(),
            ));
        }
        let n = spec.inputs.len();
        let width = match self.order {
            DerivativeOrder::Value => Some(1),
            DerivativeOrder::First => n.checked_add(1),
            DerivativeOrder::Second => n
                .checked_mul(n)
                .and_then(|k| k.checked_add(n))
                .and_then(|k| k.checked_add(1)),
        };
        let bytes = width
            .and_then(|k| k.checked_mul(self.outputs.len()))
            .and_then(|k| k.checked_mul(size_of::<f64>()));
        if bytes.is_none_or(|n| n > context.max_result_bytes) {
            return Err(ProviderError::Limit("property result bytes"));
        }
        Ok(())
    }
}

/// Immutable cancellation owner and absolute deadline of one enclosing execution.
/// A direct mathematical invocation has no outer deadline; local native allowances remain finite.
#[derive(Clone, Debug)]
pub struct ExecutionScope {
    cancel: Arc<AtomicBool>,
    deadline: Option<Instant>,
}
impl ExecutionScope {
    /// Bind an execution's original deadline without starting or renewing its clock.
    pub fn new(cancel: Arc<AtomicBool>, deadline: Option<Instant>) -> Self {
        Self { cancel, deadline }
    }
    /// Same cancellation owner shared by every nested operation.
    pub fn cancellation(&self) -> &Arc<AtomicBool> {
        &self.cancel
    }
    /// Original absolute deadline; absent only for direct mathematical invocation.
    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
    /// Cancellation and time expiry remain distinct and never mutate cancellation.
    pub fn check(&self) -> Result<(), ProviderError> {
        if self.cancel.load(Ordering::Acquire) {
            Err(ProviderError::Cancelled)
        } else if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            Err(ProviderError::Deadline)
        } else {
            Ok(())
        }
    }
    /// Cap a finite local allowance by the enclosing execution's remaining time.
    pub fn remaining(&self, allowance: Duration) -> Result<Duration, ProviderError> {
        self.check()?;
        let remaining = self.deadline.map_or(allowance, |deadline| {
            allowance.min(deadline.saturating_duration_since(Instant::now()))
        });
        if remaining.is_zero() {
            self.check()?;
            Err(ProviderError::Limit("execution time allowance"))
        } else {
            Ok(remaining)
        }
    }
}

/// Worker-local controls. This allowance bounds returned buffers, not foreign allocator interception.
#[derive(Clone, Copy, Debug)]
pub struct EvaluationContext<'a> {
    /// Cooperative cancellation, checked around indivisible native calls.
    pub cancelled: &'a AtomicBool,
    /// Explicit nonzero maximum result allocation.
    pub max_result_bytes: usize,
}
impl EvaluationContext<'_> {
    /// Observe cancellation and reject a zero allocation allowance.
    /// # Errors
    /// Returns `Cancelled` or a zero-allowance limit error.
    pub fn check(&self) -> Result<(), ProviderError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(ProviderError::Cancelled);
        }
        if self.max_result_bytes == 0 {
            return Err(ProviderError::Limit("zero property allowance"));
        }
        Ok(())
    }
}

/// Required partial derivative order; distinct from physical phase and conditioning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, enum_map::Enum)]
pub enum DerivativeOrder {
    /// Values only.
    Value,
    /// Values and first partials.
    First,
    /// Values, first and second partials.
    Second,
}
#[cfg(test)]
mod requirement_tests;
/// The independent capability that refused an implicit derivative demand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerivativeCapability {
    /// Partials implemented by the residual program.
    Residual,
    /// Established regularity of the selected root's neighborhood.
    SelectorNeighborhood,
    /// Established smoothness of the implicit output.
    OutputSmoothness,
}
impl DerivativeCapability {
    /// Stable name of the capability, independent of the rendered error message.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Residual => "residual derivative",
            Self::SelectorNeighborhood => "selector neighborhood",
            Self::OutputSmoothness => "output smoothness",
        }
    }
}
/// Independent facts and demand for one selected implicit operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivativeRequirements {
    /// Available residual partials.
    pub residual_available: DerivativeOrder,
    /// Established output smoothness before selector restrictions.
    pub output_smoothness: DerivativeOrder,
    /// Established order on the selector's admitted neighborhood.
    pub selector_neighborhood: DerivativeOrder,
    /// Minimum residual order needed by the selected inner algorithm.
    pub inner_minimum: DerivativeOrder,
    /// Output partials requested by the outer consumer.
    pub requested_output: DerivativeOrder,
    /// Residual order compiled for this attempt.
    pub residual_compilation: DerivativeOrder,
}
impl DerivativeRequirements {
    /// Combine capabilities without promoting deterministic selection to smoothness.
    pub fn new(
        residual_available: DerivativeOrder,
        output_smoothness: DerivativeOrder,
        selector_neighborhood: DerivativeOrder,
        inner_minimum: DerivativeOrder,
        requested_output: DerivativeOrder,
    ) -> Result<Self, ProviderError> {
        if requested_output > residual_available {
            return Err(ProviderError::DerivativeUnavailable {
                capability: DerivativeCapability::Residual,
                requested: requested_output,
                available: residual_available,
                members: Vec::new(),
            });
        }
        if requested_output > selector_neighborhood {
            return Err(ProviderError::DerivativeUnavailable {
                capability: DerivativeCapability::SelectorNeighborhood,
                requested: requested_output,
                available: selector_neighborhood,
                members: Vec::new(),
            });
        }
        if requested_output > output_smoothness {
            return Err(ProviderError::DerivativeUnavailable {
                capability: DerivativeCapability::OutputSmoothness,
                requested: requested_output,
                available: output_smoothness,
                members: Vec::new(),
            });
        }
        let residual_compilation = inner_minimum.max(requested_output);
        if residual_compilation > residual_available {
            return Err(ProviderError::DerivativeUnavailable {
                capability: DerivativeCapability::Residual,
                requested: residual_compilation,
                available: residual_available,
                members: Vec::new(),
            });
        }
        Ok(Self {
            residual_available,
            output_smoothness,
            selector_neighborhood,
            inner_minimum,
            requested_output,
            residual_compilation,
        })
    }
    /// Available output order shared by provider descriptors and outer admission.
    pub fn output_available(&self) -> DerivativeOrder {
        self.residual_available
            .min(self.output_smoothness)
            .min(self.selector_neighborhood)
    }
}
/// Complete scalar physical port.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Port {
    /// Semantic port identity.
    pub id: SemanticId,
    /// Quantity, including basis, reference and scale.
    pub quantity: QuantityTypeId,
    /// Actual representation unit.
    pub unit: UnitId,
}
/// Immutable registration; a declaration alone is not an executable provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSpec {
    /// Explicit logical arrays over scalar ABI cells; empty for all-scalar signatures.
    pub shapes: ProviderShapes,
    /// Provenance of local partials, independent of their available order.
    pub derivative_source: DerivativeSource,
    /// Implementation identity.
    pub id: SemanticId,
    /// Algorithm and physical interpretation revision.
    pub revision: ContentHash,
    /// Exact parameter-data identity.
    pub data: ContentHash,
    /// Ordered scalar inputs.
    pub inputs: Vec<Port>,
    /// Ordered outputs from one coherent state.
    pub outputs: Vec<Port>,
    /// Implemented partial derivative order.
    pub derivatives: DerivativeOrder,
    /// Proven smooth neighborhood on the selected branch, distinct from implemented partials.
    pub smoothness: DerivativeOrder,
}
impl ProviderSpec {
    /// Check a demand-specific executable capability against its admitted mathematical meaning.
    /// Capability may narrow; all physical, data and implementation fields stay exact.
    /// # Errors
    /// Returns a contract error for changed meaning, widened capability or unavailable demand.
    pub fn check_bound(
        &self,
        admitted: &Self,
        order: DerivativeOrder,
    ) -> Result<(), ProviderError> {
        if self.shapes != admitted.shapes
            || self.derivative_source != admitted.derivative_source
            || self.id != admitted.id
            || self.revision != admitted.revision
            || self.data != admitted.data
            || self.inputs != admitted.inputs
            || self.outputs != admitted.outputs
            || self.derivatives > admitted.derivatives
            || self.smoothness > admitted.smoothness
        {
            return Err(ProviderError::Contract(
                "provider worker changed its contract".into(),
            ));
        }
        if order > self.derivatives || order > self.smoothness {
            return Err(ProviderError::Contract(
                "provider request exceeds bound derivative capability".into(),
            ));
        }
        Ok(())
    }
    /// Validate actual physical references and unique port identities.
    /// # Errors
    /// Missing references, duplicates or incompatible scalar ports.
    pub fn validate(&self, registry: &QuantityRegistry) -> Result<(), ProviderError> {
        self.shapes.validate(&self.inputs, &self.outputs)?;
        let unique = |ids: Vec<SemanticId>| {
            let count = ids.len();
            ids.into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == count
        };
        if self.inputs.len() > 4096 || self.outputs.len() > 4096 {
            return Err(ProviderError::Contract(
                "provider registration capacity".into(),
            ));
        }
        if self.outputs.is_empty()
            || !unique(self.inputs.iter().map(|p| p.id).collect())
            || !unique(self.outputs.iter().map(|p| p.id).collect())
        {
            return Err(ProviderError::Contract(
                "empty outputs or duplicate identities".into(),
            ));
        }
        for port in self.inputs.iter().chain(&self.outputs) {
            let ty = registry
                .quantity_type(port.quantity)
                .map_err(|e| ProviderError::Contract(e.to_string()))?;
            let unit = registry
                .unit(port.unit)
                .map_err(|e| ProviderError::Contract(e.to_string()))?;
            let kind = registry
                .kind(ty.key.kind)
                .map_err(|e| ProviderError::Contract(e.to_string()))?;
            if unit.dimension != kind.dimension || !ty.key.shape.is_empty() {
                return Err(ProviderError::Contract(
                    "provider port dimension or shape mismatch".into(),
                ));
            }
            let canonical = registry
                .unit(ty.canonical_unit)
                .map_err(|e| ProviderError::Contract(e.to_string()))?;
            pse_quantity::convert_spec_for_type(unit, canonical, &ty.key)
                .map_err(|e| ProviderError::Contract(e.to_string()))?;
        }
        Ok(())
    }
}
/// Output-major local derivatives, with row-major first/second partial arrays.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderValues {
    /// One scalar per output.
    pub values: Vec<f64>,
    /// Output × input entries, empty for value-only requests.
    pub jacobian: Vec<f64>,
    /// Output × input × input entries, empty below second order.
    pub hessians: Vec<f64>,
}
impl ProviderValues {
    /// Validate every requested entry before returning any output to the caller.
    /// # Errors
    /// Unsupported derivative order, wrong lengths or nonfinite values.
    /// # Errors
    /// Returns cancellation, malformed-demand or result-allocation errors.
    pub fn validate(
        &self,
        spec: &ProviderSpec,
        request: &ProviderRequest,
    ) -> Result<(), ProviderError> {
        let n = spec.inputs.len();
        let m = request.outputs.len();
        let order = request.order;
        let first = if order >= DerivativeOrder::First {
            m.checked_mul(n)
        } else {
            Some(0)
        };
        let second = if order >= DerivativeOrder::Second {
            m.checked_mul(n).and_then(|v| v.checked_mul(n))
        } else {
            Some(0)
        };
        if order > spec.derivatives
            || order > spec.smoothness
            || m == 0
            || request.outputs.iter().any(|&o| o >= spec.outputs.len())
            || request
                .outputs
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != m
            || self.values.len() != m
            || Some(self.jacobian.len()) != first
            || Some(self.hessians.len()) != second
        {
            return Err(ProviderError::Contract(
                "provider result shape or derivative order".into(),
            ));
        }
        if self
            .values
            .iter()
            .chain(&self.jacobian)
            .chain(&self.hessians)
            .any(|v| !v.is_finite())
        {
            return Err(ProviderError::Trial("nonfinite result".into()));
        }
        Ok(())
    }
}
/// Evaluation-local provider; native scratch and scalar types stay private.
pub trait Provider: std::fmt::Debug + Send {
    /// Exact executable registration.
    fn spec(&self) -> &ProviderSpec;
    /// Compute one coherent state and requested partials.
    /// # Errors
    /// Recoverable trial rejection or terminal library failure.
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError>;
}
/// Failure classes retained across the math and native boundaries.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Cooperative evaluation cancellation.
    #[error("provider evaluation cancelled")]
    Cancelled,
    /// The absolute execution deadline has expired, distinct from other resource limits.
    #[error("provider execution deadline expired")]
    Deadline,
    /// A bounded demand cannot be admitted.
    #[error("provider resource limit: {0}")]
    Limit(&'static str),
    /// Invalid registration, binding or physical contract.
    #[error("provider contract: {0}")]
    Contract(String),
    /// A valid implicit operation requests an unavailable derivative capability.
    #[error("implicit {} does not support requested derivatives: requested {requested:?}, available {available:?}", .capability.as_str())]
    DerivativeUnavailable {
        /// Capability that refuses this demand.
        capability: DerivativeCapability,
        /// Required order, including a selected inner algorithm's residual minimum.
        requested: DerivativeOrder,
        /// Established order for the refusing capability.
        available: DerivativeOrder,
        /// Actual selected unknown identities, attached by the binding owner.
        members: Vec<SemanticId>,
    },
    /// Inadmissible trial point.
    #[error("provider trial rejected: {0}")]
    Trial(String),
    /// Physical operating window rejected this trial.
    #[error("provider envelope {axis}: {value} outside [{lower}, {upper}]")]
    OutsideEnvelope {
        /// Physical coordinate.
        axis: String,
        /// Attempted canonical value.
        value: f64,
        /// Declared lower bound.
        lower: f64,
        /// Declared upper bound.
        upper: f64,
    },
    /// Nonregular implicit branch.
    #[error("provider root is singular: {0}")]
    Singular(String),
    /// A trial selects another regime than the one its derivatives are bound to: a
    /// recoverable regime crossing, refused rather than continued across the switch.
    #[error("provider regime {selector}: trial selects {selected} across bound regime {bound}")]
    RegimeCrossing {
        /// Regime selector (provider) identity.
        selector: SemanticId,
        /// Regime the first derivative request bound.
        bound: SemanticId,
        /// Regime the refused trial selected.
        selected: SemanticId,
    },
    /// Original nested mathematical/native failure and its producer-owned extent.
    #[error("{cause}")]
    Nested {
        /// Typed source owner; rendered text is never classification authority.
        #[source]
        cause: pse_model::diagnostic::DiagnosticCause,
        /// Owned source extent, excluding the shared allocation overhead.
        retained: usize,
    },
    /// Terminal failure.
    #[error("provider failed: {0}")]
    Terminal(String),
}
impl ProviderError {
    /// Owned failure extent for retaining the original typed witness.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(match self {
            Self::Contract(s) | Self::Trial(s) | Self::Singular(s) | Self::Terminal(s) => {
                s.capacity()
            }
            Self::Nested { cause, retained } => {
                retained.saturating_add(cause.allocation_overhead())
            }
            Self::OutsideEnvelope { axis, .. } => axis.capacity(),
            Self::DerivativeUnavailable { members, .. } => {
                members.capacity() * size_of::<SemanticId>()
            }
            Self::Cancelled | Self::Deadline | Self::Limit(_) | Self::RegimeCrossing { .. } => 0,
        })
    }
    /// A recoverable trial refusal: the outer method may shorten or change its step.
    pub fn recoverable(&self) -> bool {
        if let Self::Nested { cause, .. } = self {
            use pse_model::diagnostic::DiagnosticProjection;
            return cause
                .boundary_diagnostic(pse_diagnostics::DiagnosticStage::Evaluation)
                .class
                == pse_model::diagnostic::BoundaryClass::TrialRejected;
        }
        matches!(
            self,
            Self::Trial(_)
                | Self::OutsideEnvelope { .. }
                | Self::Singular(_)
                | Self::RegimeCrossing { .. }
        )
    }
}
pse_diagnostics::impl_diagnostic! {
    ProviderError,
    code(this) { match this {
        Self::Nested {..} => None,
        Self::Cancelled => Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),
        Self::Deadline => Some(pse_diagnostics::DiagnosticCode::RuntimeTimeout),
        Self::Limit(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        Self::Contract(_) => Some(pse_diagnostics::DiagnosticCode::KernelUnboundParameter),
        Self::DerivativeUnavailable { .. } => Some(pse_diagnostics::DiagnosticCode::CapabilityBackend),
        Self::Terminal(_) => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),
        Self::Trial(_) | Self::OutsideEnvelope {..} | Self::Singular(_) | Self::RegimeCrossing {..} => Some(pse_diagnostics::DiagnosticCode::MathProvider),
    } },
    forward(this) { match this { Self::Nested {cause,..} => Some(cause.as_ref()), _ => None } }, help(_this) { None }, related(_this) { None }, source(_this) { None },
    facts(this) {
        use pse_diagnostics::{DiagnosticFacts, DiagnosticObservation as O, DiagnosticRule as R};
        if matches!(this, Self::Nested {..}) {return DiagnosticFacts::default();}
        let mut facts = DiagnosticFacts { rule: Some(R::MathProvider), ..Default::default() };
        facts.observe("provider_recoverable", O::Boolean(this.recoverable()));
        match this {
            Self::OutsideEnvelope {axis,value,lower,upper} => {
                facts.observe("provider_axis",O::Text(axis.clone()));
                for (key,value) in [("provider_value",value),("provider_lower",lower),("provider_upper",upper)] {facts.observe(key,O::Number(*value));}
            }
            Self::RegimeCrossing {selector,bound,selected} => facts.sources.extend([*selector.as_bytes(),*bound.as_bytes(),*selected.as_bytes()]),
            Self::DerivativeUnavailable {capability, requested, available, members} => {
                facts.sources.extend(members.iter().map(|member| *member.as_bytes()));
                facts.observe("derivative_capability", O::Text(capability.as_str().into()));
                let order = |order| match order { DerivativeOrder::Value => 0, DerivativeOrder::First => 1, DerivativeOrder::Second => 2 };
                facts.observe("requested_derivative_order", O::Integer(order(*requested)));
                facts.observe("available_derivative_order", O::Integer(order(*available)));
            }
            Self::Nested {..} | Self::Cancelled | Self::Deadline | Self::Limit(_) | Self::Contract(_) | Self::Trial(_) | Self::Singular(_) | Self::Terminal(_) => {}
        }
        facts
    }
}

impl ProviderSpec {
    /// Key used for worker lookup; semantic implementation ID alone is insufficient.
    pub fn key(&self) -> ProviderKey {
        ProviderKey(self.identity())
    }
    /// Exact physical, algorithm and data identity; implementation-specific policy is framed by its owner.
    pub fn identity(&self) -> ContentHash {
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ProviderV4);
        self.shapes.frame(&mut h);
        h.str(self.derivative_source.as_str());
        h.id(&self.id).hash(&self.revision).hash(&self.data);
        for ports in [&self.inputs, &self.outputs] {
            h.u64(ports.len() as u64);
            for p in ports {
                h.id(&p.id).id(&p.quantity.as_id()).id(&p.unit.as_id());
            }
        }
        h.u64(self.derivatives as u64).u64(self.smoothness as u64);
        h.finish_hash()
    }
}
/// An executable provider factory, with worker-local state.
pub trait ProviderFactory: std::fmt::Debug + Send + Sync {
    /// Immutable physical and numerical interpretation.
    fn spec(&self) -> &ProviderSpec;
    /// Immutable attempt configuration beyond the compiler-owned mathematical descriptor.
    fn configuration_key(&self) -> ContentHash {
        self.spec().identity()
    }
    /// Construct fresh scratch; it must return exactly the declared provider contract.
    /// # Errors
    /// Returns the concrete provider construction failure.
    fn create(&self) -> Result<Box<dyn Provider>, ProviderError>;
    /// Construct state attached to the enclosing execution's original controls.
    /// Nested providers pass this same scope recursively; no callback renews its deadline.
    fn create_scoped(&self, scope: ExecutionScope) -> Result<Box<dyn Provider>, ProviderError> {
        scope.check()?;
        let inner = self.create()?;
        scope.check()?;
        Ok(Box::new(ScopedProvider { inner, scope }))
    }
    /// Closed output intervals every evaluation of this provider keeps, one per output
    /// in output order: a sound envelope a global export may relax the provider to
    /// (ADR-0105 §1, ADR-0120). `None` when the provider promises none. The host checks
    /// the declaration at registration, checks every evaluation against it and frames it
    /// into [`Registration::configuration_key`]; a factory does not repeat it in
    /// [`Self::configuration_key`].
    fn envelope(&self) -> Option<Vec<(f64, f64)>> {
        None
    }
}
#[derive(Debug)]
struct ScopedProvider {
    inner: Box<dyn Provider>,
    scope: ExecutionScope,
}
impl Provider for ScopedProvider {
    fn spec(&self) -> &ProviderSpec {
        self.inner.spec()
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        self.scope.check()?;
        let result = self.inner.evaluate(inputs, request, context);
        self.scope.check()?;
        result
    }
}

/// A provider's checked output envelope: one interval per output, each containing a real.
type Envelope = Arc<[(f64, f64)]>;
/// Check a declared envelope against the admitted outputs (ADR-0120 items 2 and 5).
fn checked_envelope(
    factory: &dyn ProviderFactory,
    spec: &ProviderSpec,
) -> Result<Option<Envelope>, ProviderError> {
    let Some(envelope) = factory.envelope() else {
        return Ok(None);
    };
    // Each interval is closed, ordered and meets the reals: (+inf, +inf) and (-inf, -inf)
    // are empty over the reals and would relax a provider to nothing.
    let real = |&(lower, upper): &(f64, f64)| {
        !lower.is_nan()
            && !upper.is_nan()
            && lower <= upper
            && lower < f64::INFINITY
            && upper > f64::NEG_INFINITY
    };
    if envelope.len() != spec.outputs.len() || !envelope.iter().all(real) {
        return Err(ProviderError::Contract(
            "provider envelope must hold one closed interval containing a real number per output"
                .into(),
        ));
    }
    Ok(Some(envelope.into()))
}
/// A worker whose every value is checked against its provider's declared envelope
/// (ADR-0120 item 6). A value outside it shows the declaration is false, so it is a
/// contract error, never a trial rejection. Nonfinite values stay the trial rejection
/// [`ProviderValues::validate`] makes of them.
#[derive(Debug)]
struct Enveloped {
    inner: Box<dyn Provider>,
    envelope: Envelope,
}
impl Provider for Enveloped {
    fn spec(&self) -> &ProviderSpec {
        self.inner.spec()
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        let values = self.inner.evaluate(inputs, request, context)?;
        for (&output, &value) in request.outputs.iter().zip(&values.values) {
            let Some(&(lower, upper)) = self.envelope.get(output) else {
                continue;
            };
            if value.is_finite() && !(lower <= value && value <= upper) {
                let port = self.inner.spec().outputs.get(output).map(|p| p.id);
                return Err(ProviderError::Contract(format!(
                    "provider {} output {output} ({}) = {value} lies outside its declared envelope [{lower}, {upper}]",
                    self.inner.spec().id,
                    port.map_or_else(|| "unknown port".into(), |id| id.to_string()),
                )));
            }
        }
        Ok(values)
    }
}
/// Physically admitted registration backed by an executable factory.
#[derive(Clone, Debug)]
pub struct Registration {
    factory: Arc<dyn ProviderFactory>,
    descriptor: AdmittedProvider,
    envelope: Option<Envelope>,
}
/// Immutable physically admitted provider meaning; safe for compiler inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedProvider(ProviderSpec);
impl AdmittedProvider {
    /// Admit immutable physical meaning without constructing a runtime worker.
    /// Execution still requires a matching `Registration` at the attempt boundary.
    pub fn new(spec: ProviderSpec, registry: &QuantityRegistry) -> Result<Self, ProviderError> {
        spec.validate(registry)?;
        Ok(Self(spec))
    }
    /// Complete descriptor, including implementation, phase and data identity.
    pub fn spec(&self) -> &ProviderSpec {
        &self.0
    }
    /// Derive the executable descriptor for a resolved output demand without changing meaning.
    /// # Errors
    /// Returns a contract error if the admitted capability cannot satisfy the demand.
    pub fn restrict_order(&self, order: DerivativeOrder) -> Result<Self, ProviderError> {
        if order > self.0.derivatives || order > self.0.smoothness {
            return Err(ProviderError::Contract(
                "provider request exceeds admitted derivative capability".into(),
            ));
        }
        let mut spec = self.0.clone();
        spec.derivatives = order;
        spec.smoothness = order;
        Ok(Self(spec))
    }
}
impl Registration {
    /// Bind a previously admitted descriptor without constructing mutable native state.
    /// Every subsequent worker construction checks the factory product again.
    pub fn bind(
        descriptor: AdmittedProvider,
        factory: Arc<dyn ProviderFactory>,
    ) -> Result<Self, ProviderError> {
        if factory.spec() != descriptor.spec() {
            return Err(ProviderError::Contract(
                "factory does not match the admitted descriptor".into(),
            ));
        }
        let envelope = checked_envelope(factory.as_ref(), descriptor.spec())?;
        Ok(Self {
            factory,
            descriptor,
            envelope,
        })
    }
    /// Validate physical contracts and the concrete factory product before admission.
    /// # Errors
    /// Returns invalid physical contracts, factory failures or descriptor mismatch.
    pub fn new(
        factory: Arc<dyn ProviderFactory>,
        registry: &QuantityRegistry,
    ) -> Result<Self, ProviderError> {
        let descriptor = AdmittedProvider::new(factory.spec().clone(), registry)?;
        let envelope = checked_envelope(factory.as_ref(), descriptor.spec())?;
        let value = Self {
            factory,
            descriptor,
            envelope,
        };
        value.worker()?;
        Ok(value)
    }
    /// Admitted descriptor.
    pub fn spec(&self) -> &ProviderSpec {
        self.descriptor.spec()
    }
    /// Identity of starts, bounds and controls carried by this executable capability.
    ///
    /// A declared envelope changes evaluation outcomes on every route, so the host frames
    /// the checked envelope with the factory's key (ADR-0120 item 7). A provider without
    /// one keeps its factory's key.
    pub fn configuration_key(&self) -> ContentHash {
        let factory = self.factory.configuration_key();
        let Some(envelope) = &self.envelope else {
            return factory;
        };
        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ProviderConfigurationV1);
        h.hash(&factory).u64(envelope.len() as u64);
        for &(lower, upper) in envelope.iter() {
            h.u64(lower.to_bits()).u64(upper.to_bits());
        }
        h.finish_hash()
    }
    /// The output envelope the factory declares, checked at registration: one interval
    /// per output, each closed and containing a real number.
    pub fn envelope(&self) -> Option<&[(f64, f64)]> {
        self.envelope.as_deref()
    }
    /// Factory-free immutable compiler input.
    pub fn descriptor(&self) -> AdmittedProvider {
        self.descriptor.clone()
    }
    /// Fresh evaluation-local state, checked on every construction.
    /// # Errors
    /// Returns factory failure or a contract error when its product changes meaning.
    pub fn worker(&self) -> Result<Box<dyn Provider>, ProviderError> {
        self.worker_scoped(ExecutionScope::new(Arc::new(AtomicBool::new(false)), None))
    }
    /// Construct a worker with the enclosing attempt's immutable execution scope.
    pub fn worker_scoped(&self, scope: ExecutionScope) -> Result<Box<dyn Provider>, ProviderError> {
        scope.check()?;
        let worker = self.factory.create_scoped(scope.clone())?;
        scope.check()?;
        if worker.spec() != self.spec() {
            return Err(ProviderError::Contract(
                "factory returned a different provider contract".into(),
            ));
        }
        Ok(match &self.envelope {
            Some(envelope) => Box::new(Enveloped {
                inner: worker,
                envelope: envelope.clone(),
            }),
            None => worker,
        })
    }
}

#[cfg(test)]
mod execution_scope_tests;

#[cfg(test)]
mod envelope_tests;

impl pse_model::diagnostic::DiagnosticProjection for ProviderError {
    fn boundary_diagnostic(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        match self {
            Self::Nested { cause, .. } => cause.boundary_diagnostic(stage),
            _ => pse_model::diagnostic::project_typed(self, stage),
        }
    }
    fn boundary_diagnostic_with_members(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
        bindings: &std::collections::BTreeMap<SemanticId, SemanticId>,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        match self {
            Self::Nested { cause, .. } => cause.boundary_diagnostic_with_members(stage, bindings),
            _ => self.boundary_diagnostic(stage),
        }
    }
}

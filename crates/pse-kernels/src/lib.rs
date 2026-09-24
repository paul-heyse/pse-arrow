// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical provider contracts, independent of compiler and library scalar types.
use pse_ids::{ContentHash, SemanticId};
use pse_quantity::{QuantityRegistry, QuantityTypeId, UnitId};
use std::sync::atomic::{AtomicBool, Ordering};

pub mod envelope;
pub mod feos;

/// Complete provider interpretation, including ordered ports, phase and parameter data.
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DerivativeOrder {
    /// Values only.
    Value,
    /// Values and first partials.
    First,
    /// Values, first and second partials.
    Second,
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
/// A selected physical branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase {
    /// Physical branch identity.
    pub id: SemanticId,
    /// Branch interpretation revision.
    pub revision: ContentHash,
}
/// Immutable registration; a declaration alone is not an executable provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderSpec {
    /// Declared physical operating window; absent for non-thermodynamic providers.
    pub envelope: Option<envelope::StateEnvelope>,
    /// Implementation identity.
    pub id: SemanticId,
    /// Algorithm and physical interpretation revision.
    pub revision: ContentHash,
    /// Exact parameter-data identity.
    pub data: ContentHash,
    /// Components in provider order.
    pub components: Vec<SemanticId>,
    /// Selected branch, fixed within a smooth region.
    pub phase: Phase,
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
    /// Validate actual physical references and unique component/port identities.
    /// # Errors
    /// Missing references, duplicates or incompatible scalar ports.
    pub fn validate(&self, registry: &QuantityRegistry) -> Result<(), ProviderError> {
        if let Some(envelope) = &self.envelope {
            envelope.validate()?;
        }
        let unique = |ids: Vec<SemanticId>| {
            let count = ids.len();
            ids.into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == count
        };
        if self.inputs.len() > 4096 || self.outputs.len() > 4096 || self.components.len() > 65536 {
            return Err(ProviderError::Contract(
                "provider registration capacity".into(),
            ));
        }
        if self.outputs.is_empty()
            || !unique(self.components.clone())
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
    /// A bounded demand cannot be admitted.
    #[error("provider resource limit: {0}")]
    Limit(&'static str),
    /// Invalid registration or missing derivative implementation.
    #[error("provider contract: {0}")]
    Contract(String),
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
    /// Terminal failure.
    #[error("provider failed: {0}")]
    Terminal(String),
}
pse_diagnostics::impl_diagnostic! {
    ProviderError,
    code(this) { Some(match this {
        Self::Cancelled => pse_diagnostics::DiagnosticCode::RuntimeCancelled,
        Self::Limit(_) => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
        Self::Contract(_) => pse_diagnostics::DiagnosticCode::KernelUnboundParameter,
        _ => pse_diagnostics::DiagnosticCode::SolveEvaluationError,
    }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

impl ProviderSpec {
    /// Key used for worker lookup; semantic implementation ID alone is insufficient.
    pub fn key(&self) -> ProviderKey {
        ProviderKey(self.identity())
    }
    /// Exact physical, algorithm, data and phase identity; no process handles enter this key.
    pub fn identity(&self) -> ContentHash {
        let mut h = pse_ids::FramedHasher::new("pse.provider.v2");
        h.id(&self.id)
            .hash(&self.revision)
            .hash(&self.data)
            .id(&self.phase.id)
            .hash(&self.phase.revision);
        h.u64(self.components.len() as u64);
        for id in &self.components {
            h.id(id);
        }
        for ports in [&self.inputs, &self.outputs] {
            h.u64(ports.len() as u64);
            for p in ports {
                h.id(&p.id).id(&p.quantity.as_id()).id(&p.unit.as_id());
            }
        }
        h.u64(u64::from(self.envelope.is_some()));
        if let Some(envelope) = &self.envelope {
            envelope.frame(&mut h);
        }
        h.u64(self.derivatives as u64).u64(self.smoothness as u64);
        h.finish_hash()
    }
}
/// An executable provider factory, with worker-local state.
pub trait ProviderFactory: std::fmt::Debug + Send + Sync {
    /// Immutable physical and numerical interpretation.
    fn spec(&self) -> &ProviderSpec;
    /// Construct fresh scratch; it must return exactly the declared provider contract.
    fn create(&self) -> Result<Box<dyn Provider>, ProviderError>;
}
/// Physically admitted registration backed by an executable factory.
#[derive(Clone, Debug)]
pub struct Registration {
    factory: std::sync::Arc<dyn ProviderFactory>,
    descriptor: AdmittedProvider,
}
/// Immutable physically admitted provider meaning; safe for compiler inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedProvider(ProviderSpec);
impl AdmittedProvider {
    /// Complete descriptor, including implementation, phase and data identity.
    pub fn spec(&self) -> &ProviderSpec {
        &self.0
    }
}
impl Registration {
    /// Validate physical contracts and the concrete factory product before admission.
    pub fn new(
        factory: std::sync::Arc<dyn ProviderFactory>,
        registry: &QuantityRegistry,
    ) -> Result<Self, ProviderError> {
        factory.spec().validate(registry)?;
        let descriptor = AdmittedProvider(factory.spec().clone());
        let value = Self {
            factory,
            descriptor,
        };
        value.worker()?;
        Ok(value)
    }
    /// Admitted descriptor.
    pub fn spec(&self) -> &ProviderSpec {
        self.descriptor.spec()
    }
    /// Factory-free immutable compiler input.
    pub fn descriptor(&self) -> AdmittedProvider {
        self.descriptor.clone()
    }
    /// Fresh evaluation-local state, checked on every construction.
    pub fn worker(&self) -> Result<Box<dyn Provider>, ProviderError> {
        let worker = self.factory.create()?;
        if worker.spec() != self.spec() {
            return Err(ProviderError::Contract(
                "factory returned a different provider contract".into(),
            ));
        }
        Ok(worker)
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit homogeneous-density PC-SAFT/DIPPR properties with checked chemical bindings.
use crate::{
    DerivativeOrder, EvaluationContext, Phase, Port, Provider, ProviderError, ProviderFactory,
    ProviderRequest, ProviderSpec, ProviderValues,
};
use ::feos::{ideal_gas::Dippr, pcsaft::PcSaft};
use feos_core::{Contributions, DensityInitialization, EquationOfState, SolverOptions, State};
use nalgebra::{DVector, SVector};
use num_dual::{DualNum, DualSVec64, HyperDualSVec64, gradient, partial_hessian};
use pse_ids::{ContentHash, FramedHasher, SemanticId, canonical_f64_bits, named_id};
use pse_quantity::{
    BasisKind, QuantityKindId, QuantityRegistry, ReferenceStateId, ReferenceStateKind,
};
use quantity::{Density, JOULE, KELVIN, MOL, PASCAL, Pressure, Temperature};
use std::{cell::Cell, sync::Arc};
mod data;
pub use data::{ComponentRecord, FeosData, MissingInteractionPolicy};
pub use pse_model::generated::enums::{StabilityPolicy, StabilityStatus, ThermodynamicFormulation};
type Eos = EquationOfState<Vec<Dippr>, PcSaft>;
const LANES: usize = 4;

/// Species-to-record binding; record identity is checked independently of its label.
#[derive(Clone, Debug)]
pub struct ComponentBinding {
    /// Authored species identity.
    pub species: SemanticId,
    /// PC-SAFT record CAS.
    pub pcsaft_cas: String,
    /// Ideal-gas record CAS, identifying the same substance.
    pub ideal_gas_cas: String,
}
/// Quantity-kind roles declared by the physical model, independent of port assignments.
#[derive(Clone, Debug)]
pub struct FeosKinds {
    /// Absolute temperature.
    pub temperature: QuantityKindId,
    /// Molar density.
    pub density: QuantityKindId,
    /// Mole fraction.
    pub fraction: QuantityKindId,
    /// Pressure.
    pub pressure: QuantityKindId,
    /// Molar enthalpy, distinct from internal energy.
    pub enthalpy: QuantityKindId,
    /// Molar entropy.
    pub entropy: QuantityKindId,
    /// Log fugacity coefficient.
    pub ln_fugacity: QuantityKindId,
}
/// Complete ordered physical binding of an authored property package.
#[derive(Clone, Debug)]
pub struct FeosPorts {
    /// Explicit SI operating window; empirical accuracy remains separately qualified.
    pub envelope: crate::envelope::StateEnvelope,
    /// T, molar density, then fractions in component order omitting the dependent species.
    pub inputs: Vec<Port>,
    /// P, H, S, then ln(phi) in component order.
    pub outputs: Vec<Port>,
    /// DIPPR integral from 298.15 K, without formation enthalpy.
    pub enthalpy_reference: ReferenceStateId,
    /// Ideal gas at 298.15 K and 100000 Pa, plus mixing and residual entropy.
    pub entropy_reference: ReferenceStateId,
    /// Ordered species-to-record mapping.
    pub components: Vec<ComponentBinding>,
    /// Fraction determined by one minus the independent fractions.
    pub dependent_species: SemanticId,
    /// Independent declaration of physical roles.
    pub kinds: FeosKinds,
    /// Explicit library records and provenance.
    pub data: FeosData,
    /// Selected state formulation.
    pub formulation: ThermodynamicFormulation,
    /// Selected trial admissibility policy.
    pub stability: StabilityPolicy,
}
/// Immutable library data and physically admitted factory. Workers own trial scratch.
#[derive(Clone)]
pub struct FeosPackage {
    spec: ProviderSpec,
    eos: Arc<Eos>,
    dependent: usize,
    stability: StabilityPolicy,
    records: Vec<ComponentRecord>,
}
impl std::fmt::Debug for FeosPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeosPackage")
            .field("spec", &self.spec)
            .field("records", &self.records)
            .finish_non_exhaustive()
    }
}
fn fingerprint(text: &str) -> ContentHash {
    let mut hash = FramedHasher::new("pse.feos.profile.v1");
    hash.str(text);
    hash.finish_hash()
}
fn failure(error: feos_core::FeosError) -> ProviderError {
    use feos_core::FeosError as E;
    match error {
        E::InvalidState(..)
        | E::IterationFailed(..)
        | E::NotConverged(..)
        | E::UndeterminedState(..)
        | E::SuperCritical
        | E::NoPhaseSplit
        | E::TrivialSolution => ProviderError::Trial(error.to_string()),
        _ => ProviderError::Terminal(error.to_string()),
    }
}
/// FeOS uses kB K / Å³ as the ideal-gas pressure reference. Converting to 1 bar
/// subtracts R ln(P*/1 bar). See scripts/feos_entropy_reference.py for independent evidence.
fn entropy_offset() -> f64 {
    let pressure = (quantity::KB * KELVIN / quantity::ANGSTROM.powi::<3>()).convert_into(PASCAL);
    quantity::RGAS.convert_into(JOULE / MOL / KELVIN) * (pressure / 100_000.0).ln()
}
impl FeosPackage {
    fn envelope(&self) -> Result<&crate::envelope::StateEnvelope, ProviderError> {
        self.spec
            .envelope
            .as_ref()
            .ok_or_else(|| ProviderError::Contract("FeOS operating envelope required".into()))
    }
    /// Admit chemistry, physical roles, reference states and explicit state policy.
    /// # Errors
    /// Refuses ambiguous or incompatible contracts before constructing any worker.
    pub fn new(ports: FeosPorts, registry: &QuantityRegistry) -> Result<Self, ProviderError> {
        let bad = |message: &str| ProviderError::Contract(message.into());
        let contract = |e: pse_quantity::QuantityError| ProviderError::Contract(e.to_string());
        let n = ports.components.len();
        if n == 0
            || n > 128
            || ports.inputs.len() != n + 1
            || ports.outputs.len() != n + 3
            || ports.envelope.composition.len() != n
        {
            return Err(bad("FeOS component, coordinate, output or envelope arity"));
        }
        if ports.formulation != ThermodynamicFormulation::HomogeneousDensity {
            return Err(bad(
                "phase-equilibrium formulation is not implemented by this homogeneous provider",
            ));
        }
        let dependent = ports
            .components
            .iter()
            .position(|c| c.species == ports.dependent_species)
            .ok_or_else(|| bad("dependent species is not in the ordered components"))?;
        if ports.enthalpy_reference == ports.entropy_reference {
            return Err(bad("enthalpy and entropy references must be distinct"));
        }
        for (id, pressure) in [
            (ports.enthalpy_reference, None),
            (ports.entropy_reference, Some(100_000.0)),
        ] {
            let r = registry.reference_state(id).map_err(contract)?;
            if r.kind != ReferenceStateKind::Custom
                || r.temperature != Some(298.15)
                || r.pressure != pressure
                || r.include_enthalpy_of_formation
                || r.phase.is_some()
            {
                return Err(bad(
                    "DIPPR reference requires 298.15 K, no formation enthalpy, and 1 bar for entropy only",
                ));
            }
        }
        let mut roles = vec![
            (
                ports.kinds.temperature,
                [0, 0, 0, 1, 0, 0, 0, 0],
                None,
                false,
                false,
            ),
            (
                ports.kinds.density,
                [-3, 0, 0, 0, 1, 0, 0, 0],
                None,
                false,
                false,
            ),
        ];
        roles.extend((1..n).map(|_| (ports.kinds.fraction, [0; 8], None, true, true)));
        roles.extend([
            (
                ports.kinds.pressure,
                [-1, 1, -2, 0, 0, 0, 0, 0],
                None,
                false,
                false,
            ),
            (
                ports.kinds.enthalpy,
                [2, 1, -2, 0, -1, 0, 0, 0],
                Some(ports.enthalpy_reference),
                true,
                false,
            ),
            (
                ports.kinds.entropy,
                [2, 1, -2, -1, -1, 0, 0, 0],
                Some(ports.entropy_reference),
                true,
                false,
            ),
        ]);
        roles.extend((0..n).map(|_| (ports.kinds.ln_fugacity, [0; 8], None, false, false)));
        for (port, (kind, dims, reference, molar, fraction)) in
            ports.inputs.iter().chain(&ports.outputs).zip(roles)
        {
            let unit = registry.unit(port.unit).map_err(contract)?;
            let ty = registry.quantity_type(port.quantity).map_err(contract)?;
            if ty.key.kind != kind
                || ty.key.scale_kind != pse_quantity::ScaleKind::Point
                || !ty.key.shape.is_empty()
                || ty.key.subject_kind.is_some()
                || ty.key.reference_state != reference
                || unit
                    .dimension
                    .exponents()
                    .iter()
                    .zip(dims)
                    .any(|(r, e)| r.num() != e || r.den() != 1)
                || unit.scale_to_canonical != 1.0
                || unit.offset_to_canonical != 0.0
            {
                return Err(bad(
                    "FeOS port quantity kind, dimension, shape, reference or SI representation mismatch",
                ));
            }
            if molar {
                let b = registry
                    .basis(ty.key.basis.ok_or_else(|| bad("molar basis required"))?)
                    .map_err(contract)?;
                if b.kind != BasisKind::Molar
                    || b.composition_basis
                        != fraction.then_some(pse_quantity::CompositionBasis::MoleFraction)
                    || b.rate_basis.is_some()
                    || b.reference_conditions.is_some()
                {
                    return Err(bad("FeOS port basis mismatch"));
                }
            } else if ty.key.basis.is_some() {
                return Err(bad("unexpected basis on absolute FeOS port"));
            }
        }
        let (eos, data, records) = ports.data.admit(&ports.components)?;
        let mut phase = FramedHasher::new("pse.feos.state-policy.v2");
        phase
            .str(ports.formulation.as_str())
            .str(ports.stability.as_str())
            .id(&ports.dependent_species);
        let spec = ProviderSpec {
            envelope: Some(ports.envelope),
            id: named_id(SemanticId::NIL, "pse.feos.pcsaft-dippr"),
            revision: fingerprint(
                "explicit-density:si:enthalpy29815:entropy29815-1bar:block-dual:v2",
            ),
            data,
            components: ports.components.iter().map(|c| c.species).collect(),
            phase: Phase {
                id: named_id(SemanticId::NIL, "pse.feos.homogeneous-density"),
                revision: phase.finish_hash(),
            },
            inputs: ports.inputs,
            outputs: ports.outputs,
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        };
        spec.validate(registry)?;
        Ok(Self {
            spec,
            eos: Arc::new(eos),
            dependent,
            stability: ports.stability,
            records,
        })
    }
    /// Actual bound records in the composition coordinate order.
    pub fn records(&self) -> &[ComponentRecord] {
        &self.records
    }
    /// Independent scratch and a one-trial cache.
    pub fn worker(&self) -> FeosWorker {
        FeosWorker {
            package: self.clone(),
            cache: None,
            states: Cell::new(0),
        }
    }
    fn fractions<D: DualNum<f64> + Copy>(&self, independent: &[D]) -> Vec<D> {
        let dependent = independent.iter().fold(D::from(1.0), |sum, x| sum - *x);
        let mut offset = 0;
        (0..self.spec.components.len())
            .map(|i| {
                if i == self.dependent {
                    dependent
                } else {
                    let value = independent[offset];
                    offset += 1;
                    value
                }
            })
            .collect()
    }
    fn composition(
        &self,
        temperature: f64,
        independent: &[f64],
    ) -> Result<Vec<f64>, ProviderError> {
        if independent.len() + 1 != self.spec.components.len() {
            return Err(ProviderError::Contract("FeOS composition arity".into()));
        }
        let fractions = self.fractions(independent);
        if !temperature.is_finite()
            || temperature <= 0.0
            || fractions
                .iter()
                .any(|x| !x.is_finite() || *x <= 0.0 || (fractions.len() > 1 && *x >= 1.0))
        {
            return Err(ProviderError::Trial(
                "positive temperature and interior composition required".into(),
            ));
        }
        self.envelope()?
            .thermal_composition(temperature, &fractions)?;
        Ok(fractions)
    }
    /// Library NPT initialization with separate mechanical and global diagnostics.
    /// # Errors
    /// Refuses invalid trials, envelope violations and failure of a selected stability requirement.
    pub fn initialize_npt(
        &self,
        temperature: f64,
        pressure: f64,
        composition: &[f64],
        phase: InitialPhase,
        diagnose_global: bool,
        context: &EvaluationContext<'_>,
    ) -> Result<InitialState, ProviderError> {
        context.check()?;
        let fractions = self.composition(temperature, composition)?;
        if !pressure.is_finite() || pressure <= 0.0 {
            return Err(ProviderError::Trial(
                "positive finite pressure required".into(),
            ));
        }
        let envelope = self.envelope()?;
        envelope.pressure.check("pressure", pressure)?;
        let initial = match phase {
            InitialPhase::Vapor => DensityInitialization::Vapor,
            InitialPhase::Liquid => DensityInitialization::Liquid,
        };
        let state = State::new_npt(
            &self.eos,
            Temperature::new(temperature),
            Pressure::new(pressure),
            DVector::from_vec(fractions),
            Some(initial),
        )
        .map_err(failure)?;
        let molar_density = state
            .density
            .convert_into(MOL / quantity::METER.powi::<3>());
        envelope.density.check("density", molar_density)?;
        let slope = state
            .dp_drho(Contributions::Total)
            .convert_into(PASCAL / (MOL / quantity::METER.powi::<3>()));
        let actual = state.pressure(Contributions::Total).convert_into(PASCAL);
        envelope.pressure.check("pressure", actual)?;
        if (actual - pressure).abs() > 1e-3 + 1e-8 * pressure {
            return Err(ProviderError::Trial("NPT pressure closure failed".into()));
        }
        let global_stability = if diagnose_global || self.stability == StabilityPolicy::Global {
            stability(state.is_stable(SolverOptions::default()))
        } else {
            StabilityObservation {
                status: StabilityStatus::NotRequested,
                detail: None,
            }
        };
        enforce_stability(self.stability, slope, &global_stability)?;
        context.check()?;
        Ok(InitialState {
            molar_density,
            mechanical_slope: slope,
            global_stability,
        })
    }
}
impl ProviderFactory for FeosPackage {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn create(&self) -> Result<Box<dyn Provider>, ProviderError> {
        Ok(Box::new(self.worker()))
    }
}
/// Density initialization branch, independent of phase stability.
#[derive(Clone, Copy, Debug)]
pub enum InitialPhase {
    /// Low-density guess.
    Vapor,
    /// High-density guess.
    Liquid,
}
/// Outcome of a global stability diagnostic; failures remain failures.
#[derive(Clone, Debug)]
pub struct StabilityObservation {
    /// Shared status vocabulary.
    pub status: StabilityStatus,
    /// Library failure detail, when present.
    pub detail: Option<String>,
}
fn stability(result: Result<bool, feos_core::FeosError>) -> StabilityObservation {
    match result {
        Ok(true) => StabilityObservation {
            status: StabilityStatus::Stable,
            detail: None,
        },
        Ok(false) => StabilityObservation {
            status: StabilityStatus::Unstable,
            detail: None,
        },
        Err(e) => StabilityObservation {
            status: StabilityStatus::Failed,
            detail: Some(e.to_string()),
        },
    }
}
fn enforce_stability(
    policy: StabilityPolicy,
    slope: f64,
    global: &StabilityObservation,
) -> Result<(), ProviderError> {
    if policy != StabilityPolicy::Unchecked && (!slope.is_finite() || slope <= 0.0) {
        return Err(ProviderError::Trial(
            "mechanically unstable homogeneous state".into(),
        ));
    }
    if policy == StabilityPolicy::Global && global.status != StabilityStatus::Stable {
        return Err(ProviderError::Trial(format!(
            "global stability {}: {}",
            global.status.as_str(),
            global.detail.as_deref().unwrap_or("")
        )));
    }
    Ok(())
}
/// Checked NPT candidate, separate from smooth property callbacks.
#[derive(Clone, Debug)]
pub struct InitialState {
    /// Moles per cubic metre.
    pub molar_density: f64,
    /// Actual dP/d(rho), distinct from a global stability test.
    pub mechanical_slope: f64,
    /// Requested diagnostic, including failed and not-requested outcomes.
    pub global_stability: StabilityObservation,
}
#[derive(Clone, Debug)]
struct Cached {
    inputs: Vec<u64>,
    request: ProviderRequest,
    values: ProviderValues,
}
/// Worker-local evaluator. Fixed-size dual blocks support arbitrary admitted component counts.
#[derive(Debug)]
pub struct FeosWorker {
    package: FeosPackage,
    cache: Option<Cached>,
    states: Cell<usize>,
}
impl FeosWorker {
    /// Actual state constructions, including derivative blocks.
    pub fn state_evaluations(&self) -> usize {
        self.states.get()
    }
    fn properties<D: DualNum<f64> + Copy>(
        &self,
        x: &[D],
        outputs: &[usize],
    ) -> Result<Vec<D>, ProviderError> {
        let state = State::new(
            &self.package.eos,
            Temperature::new(x[0]),
            Density::new(x[1]),
            DVector::from_vec(self.package.fractions(&x[2..])),
        )
        .map_err(failure)?;
        self.states.set(self.states.get() + 1);
        let pressure = state.pressure(Contributions::Total).convert_into(PASCAL);
        self.package
            .envelope()?
            .pressure
            .check("pressure", pressure.re())?;
        if self.package.stability != StabilityPolicy::Unchecked {
            let slope = state
                .dp_drho(Contributions::Total)
                .convert_into(PASCAL / (MOL / quantity::METER.powi::<3>()))
                .re();
            if !slope.is_finite() || slope <= 0.0 {
                return Err(ProviderError::Trial(
                    "mechanically unstable homogeneous state".into(),
                ));
            }
        }
        let phi = outputs.iter().any(|&i| i >= 3).then(|| state.ln_phi());
        outputs
            .iter()
            .map(|&i| {
                Ok(match i {
                    0 => pressure,
                    1 => state
                        .molar_enthalpy(Contributions::Total)
                        .convert_into(JOULE / MOL),
                    2 => {
                        state
                            .molar_entropy(Contributions::Total)
                            .convert_into(JOULE / MOL / KELVIN)
                            - D::from(entropy_offset())
                    }
                    _ => *phi
                        .as_ref()
                        .and_then(|p| p.get(i - 3))
                        .ok_or_else(|| ProviderError::Contract("property ordinal".into()))?,
                })
            })
            .collect()
    }
    fn evaluate_blocks(
        &self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        let n = inputs.len();
        let m = request.outputs.len();
        let mut result = ProviderValues {
            values: vec![0.0; m],
            jacobian: vec![],
            hessians: vec![],
        };
        if request.order == DerivativeOrder::Value {
            result.values = self.properties(inputs, &request.outputs)?;
            return Ok(result);
        }
        result.jacobian.resize(m * n, 0.0);
        if request.order == DerivativeOrder::First {
            for offset in (0..n).step_by(LANES) {
                context.check()?;
                let values = gradient(
                    |seed: SVector<DualSVec64<LANES>, LANES>| {
                        let mut x: Vec<_> = inputs
                            .iter()
                            .map(|&v| DualSVec64::<LANES>::from_re(v))
                            .collect();
                        for i in 0..LANES.min(n - offset) {
                            x[offset + i] += seed[i];
                        }
                        self.properties(&x, &request.outputs)
                    },
                    &SVector::<f64, LANES>::zeros(),
                )?;
                for (o, (value, derivative)) in values.into_iter().enumerate() {
                    result.values[o] = value;
                    for i in 0..LANES.min(n - offset) {
                        result.jacobian[o * n + offset + i] = derivative[i];
                    }
                }
            }
        } else {
            result.hessians.resize(m * n * n, 0.0);
            for left in (0..n).step_by(LANES) {
                for right in (left..n).step_by(LANES) {
                    context.check()?;
                    let values = partial_hessian(
                        |(a, b): (
                            SVector<HyperDualSVec64<LANES, LANES>, LANES>,
                            SVector<HyperDualSVec64<LANES, LANES>, LANES>,
                        )| {
                            let mut x: Vec<_> = inputs
                                .iter()
                                .map(|&v| HyperDualSVec64::<LANES, LANES>::from_re(v))
                                .collect();
                            for i in 0..LANES.min(n - left) {
                                x[left + i] += a[i];
                            }
                            for i in 0..LANES.min(n - right) {
                                x[right + i] += b[i];
                            }
                            self.properties(&x, &request.outputs)
                        },
                        (
                            &SVector::<f64, LANES>::zeros(),
                            &SVector::<f64, LANES>::zeros(),
                        ),
                    )?;
                    for (o, (value, dl, _, h)) in values.into_iter().enumerate() {
                        result.values[o] = value;
                        for i in 0..LANES.min(n - left) {
                            if left == right {
                                result.jacobian[o * n + left + i] = dl[i];
                            }
                            for j in 0..LANES.min(n - right) {
                                result.hessians[o * n * n + (left + i) * n + right + j] = h[(i, j)];
                                if left != right {
                                    result.hessians[o * n * n + (right + j) * n + left + i] =
                                        h[(i, j)];
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(result)
    }
}
impl Provider for FeosWorker {
    fn spec(&self) -> &ProviderSpec {
        &self.package.spec
    }
    fn evaluate(
        &mut self,
        inputs: &[f64],
        request: &ProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderValues, ProviderError> {
        let prior = self.cache.take();
        request.validate(self.spec(), context)?;
        if inputs.len() != self.spec().inputs.len() {
            return Err(ProviderError::Contract("FeOS input arity".into()));
        }
        if !inputs[1].is_finite() || inputs[1] <= 0.0 {
            return Err(ProviderError::Trial(
                "positive finite density required".into(),
            ));
        }
        let fractions = self.package.composition(inputs[0], &inputs[2..])?;
        self.package
            .envelope()?
            .density
            .check("density", inputs[1])?;
        let bits: Vec<_> = inputs.iter().copied().map(canonical_f64_bits).collect();
        let n = inputs.len();
        if let Some(c) = prior
            && c.inputs == bits
            && c.request.order >= request.order
            && request
                .outputs
                .iter()
                .all(|o| c.request.outputs.contains(o))
        {
            let mut result = ProviderValues {
                values: vec![],
                jacobian: vec![],
                hessians: vec![],
            };
            for output in &request.outputs {
                let i = c
                    .request
                    .outputs
                    .iter()
                    .position(|o| o == output)
                    .ok_or_else(|| ProviderError::Contract("cached output selection".into()))?;
                result.values.push(c.values.values[i]);
                if request.order >= DerivativeOrder::First {
                    result
                        .jacobian
                        .extend_from_slice(&c.values.jacobian[i * n..(i + 1) * n]);
                }
                if request.order >= DerivativeOrder::Second {
                    result
                        .hessians
                        .extend_from_slice(&c.values.hessians[i * n * n..(i + 1) * n * n]);
                }
            }
            self.cache = Some(c);
            return Ok(result);
        }
        if self.package.stability == StabilityPolicy::Global {
            let state = State::new(
                &self.package.eos,
                Temperature::new(inputs[0]),
                Density::new(inputs[1]),
                DVector::from_vec(fractions),
            )
            .map_err(failure)?;
            self.states.set(self.states.get() + 1);
            let slope = state
                .dp_drho(Contributions::Total)
                .convert_into(PASCAL / (MOL / quantity::METER.powi::<3>()));
            self.package.envelope()?.pressure.check(
                "pressure",
                state.pressure(Contributions::Total).convert_into(PASCAL),
            )?;
            context.check()?;
            enforce_stability(
                self.package.stability,
                slope,
                &stability(state.is_stable(SolverOptions::default())),
            )?;
        }
        let result = self.evaluate_blocks(inputs, request, context)?;
        result.validate(self.spec(), request)?;
        context.check()?;
        self.cache = Some(Cached {
            inputs: bits,
            request: request.clone(),
            values: result.clone(),
        });
        Ok(result)
    }
}
#[cfg(test)]
mod tests;

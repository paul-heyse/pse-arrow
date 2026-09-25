// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit-density PC-SAFT/DIPPR methane/ethane/propane property package.
use crate::{
    DerivativeOrder, EvaluationContext, Phase, Port, Provider, ProviderError, ProviderFactory,
    ProviderRequest, ProviderSpec, ProviderValues,
};
use ::feos::{
    ideal_gas::{Dippr, DipprParameters},
    pcsaft::{PcSaft, PcSaftParameters},
};
use feos_core::{Contributions, DensityInitialization, EquationOfState, SolverOptions, State};
use nalgebra::{Const, DVector, U1};
use num_dual::{Dual2SVec64, DualNum, DualSVec64};
use pse_ids::{ContentHash, FramedHasher, SemanticId, named_id};
use pse_quantity::{BasisKind, QuantityRegistry, ReferenceStateId, ReferenceStateKind};
use quantity::{Density, JOULE, KELVIN, MOL, PASCAL, Pressure, Temperature};
use std::sync::Arc;

const PC: &str = include_str!("../data/pcsaft-light-hydrocarbons.json");
const IG: &str = include_str!("../data/ideal-gas-light-hydrocarbons.json");
type Eos = EquationOfState<Vec<Dippr>, PcSaft>;

/// Ordered physical bindings. Quantities come from the caller's admitted model registry.
#[derive(Clone, Debug)]
pub struct FeosPorts {
    /// Explicit SI operating window; empirical accuracy remains separately qualified.
    pub envelope: crate::envelope::StateEnvelope,
    /// Temperature (K), molar density (mol/m³), methane/ethane mole fractions.
    pub inputs: [Port; 4],
    /// Pressure (Pa), enthalpy (J/mol), entropy (J/mol/K), three dimensionless ln(phi).
    pub outputs: [Port; 6],
    /// Declared custom `FeOS` caloric convention at 298.15 K, without formation enthalpy.
    pub caloric_reference: ReferenceStateId,
    /// Component identities in methane/ethane/propane order.
    pub components: [SemanticId; 3],
}

/// Immutable library data and physically admitted factory. Workers own all trial state.
#[derive(Clone)]
pub struct FeosPackage {
    spec: ProviderSpec,
    eos: Arc<Eos>,
}
impl std::fmt::Debug for FeosPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FeosPackage")
            .field("spec", &self.spec)
            .finish_non_exhaustive()
    }
}
fn fingerprint(text: &str) -> ContentHash {
    let mut hash = FramedHasher::new("pse.feos.profile.v1");
    hash.str(text);
    hash.finish_hash()
}
#[allow(
    clippy::needless_pass_by_value,
    reason = "map_err consumes the native error"
)]
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
impl FeosPackage {
    fn envelope(&self) -> Result<&crate::envelope::StateEnvelope, ProviderError> {
        self.spec
            .envelope
            .as_ref()
            .ok_or_else(|| ProviderError::Contract("FeOS requires an operating envelope".into()))
    }
    /// Admit physical roles and construct library-owned equations of state.
    /// # Errors
    /// Rejects incompatible physical ports, caloric conventions or operating envelopes.
    #[allow(
        clippy::too_many_lines,
        clippy::float_cmp,
        reason = "one physical admission table; canonical unit scale must equal one exactly"
    )]
    pub fn new(ports: FeosPorts, registry: &QuantityRegistry) -> Result<Self, ProviderError> {
        let contract = |e: pse_quantity::QuantityError| ProviderError::Contract(e.to_string());
        let reference = registry
            .reference_state(ports.caloric_reference)
            .map_err(contract)?;
        if reference.kind != ReferenceStateKind::Custom
            || reference.temperature != Some(298.15)
            || reference.pressure.is_some()
            || reference.include_enthalpy_of_formation
            || reference.phase.is_some()
        {
            return Err(ProviderError::Contract("FeOS custom caloric convention: 298.15 K integrals, no pressure datum or formation enthalpy".into()));
        }
        let dims = [
            [0, 0, 0, 1, 0, 0, 0, 0],
            [-3, 0, 0, 0, 1, 0, 0, 0],
            [0; 8],
            [0; 8],
            [-1, 1, -2, 0, 0, 0, 0, 0],
            [2, 1, -2, 0, -1, 0, 0, 0],
            [2, 1, -2, -1, -1, 0, 0, 0],
            [0; 8],
            [0; 8],
            [0; 8],
        ];
        for (i, (port, exponents)) in ports
            .inputs
            .iter()
            .chain(&ports.outputs)
            .zip(dims)
            .enumerate()
        {
            let unit = registry.unit(port.unit).map_err(contract)?;
            let ty = registry.quantity_type(port.quantity).map_err(contract)?;
            if ty.key.scale_kind != pse_quantity::ScaleKind::Point {
                return Err(ProviderError::Contract(
                    "FeOS state/property ports are point quantities".into(),
                ));
            }
            if i != 5 && i != 6 && ty.key.reference_state.is_some() {
                return Err(ProviderError::Contract(
                    "FeOS absolute state ports cannot carry a shifted datum".into(),
                ));
            }
            if unit
                .dimension
                .exponents()
                .iter()
                .zip(exponents)
                .any(|(r, e)| r.num() != e || r.den() != 1)
                || unit.scale_to_canonical != 1.0
                || unit.offset_to_canonical != 0.0
            {
                return Err(ProviderError::Contract(
                    "FeOS ports require declared SI representations".into(),
                ));
            }
            if i == 5 || i == 6 {
                let basis = ty.key.basis.ok_or_else(|| {
                    ProviderError::Contract("caloric molar basis required".into())
                })?;
                if ty.key.reference_state != Some(ports.caloric_reference)
                    || registry.basis(basis).map_err(contract)?.kind != BasisKind::Molar
                {
                    return Err(ProviderError::Contract(
                        "FeOS caloric reference or molar basis mismatch".into(),
                    ));
                }
            }
            if i == 2 || i == 3 {
                let basis = ty.key.basis.ok_or_else(|| {
                    ProviderError::Contract("mole-fraction basis required".into())
                })?;
                let basis = registry.basis(basis).map_err(contract)?;
                if basis.kind != BasisKind::Molar
                    || basis.composition_basis != Some(pse_quantity::CompositionBasis::MoleFraction)
                {
                    return Err(ProviderError::Contract(
                        "FeOS composition requires mole fractions".into(),
                    ));
                }
            }
        }
        let mut data = FramedHasher::new("pse.feos.data.v1");
        data.str(PC)
            .str(IG)
            .str("explicit-zero-binary-interactions");
        let spec = ProviderSpec {
            envelope: Some(ports.envelope),
            id: named_id(SemanticId::NIL, "pse.feos.light-hydrocarbons"),
            revision: fingerprint("pcsaft+dippr:explicit-density:si:raw-partials:v1"),
            data: data.finish_hash(),
            components: ports.components.to_vec(),
            phase: Phase {
                id: named_id(SemanticId::NIL, "pse.feos.homogeneous-density"),
                revision: fingerprint(
                    "positive-density-interior-composition:no-phase-selection:v1",
                ),
            },
            inputs: ports.inputs.to_vec(),
            outputs: ports.outputs.to_vec(),
            derivatives: DerivativeOrder::Second,
            smoothness: DerivativeOrder::Second,
        };
        spec.validate(registry)?;
        let pc = PcSaftParameters::new(
            serde_json::from_str(PC).map_err(|e| ProviderError::Terminal(e.to_string()))?,
            vec![],
        )
        .map_err(failure)?;
        let ig = DipprParameters::new(
            serde_json::from_str(IG).map_err(|e| ProviderError::Terminal(e.to_string()))?,
            vec![],
        )
        .map_err(failure)?;
        Ok(Self {
            spec,
            eos: Arc::new(EquationOfState::new(Dippr::new(ig), PcSaft::new(pc))),
        })
    }
    /// New independent scratch and one-trial cache.
    pub fn worker(&self) -> FeosWorker {
        FeosWorker {
            package: self.clone(),
            cache: None,
            states: 0,
        }
    }

    /// Library NPT initialization; the phase chooses an initial density, not a phase guarantee.
    /// # Errors
    /// Returns cancellation, invalid trials, envelope violations or native state failures.
    pub fn initialize_npt(
        &self,
        temperature: f64,
        pressure: f64,
        composition: [f64; 2],
        phase: InitialPhase,
        check_stability: bool,
        context: &EvaluationContext<'_>,
    ) -> Result<InitialState, ProviderError> {
        context.check()?;
        validate_inputs(&[temperature, 1.0, composition[0], composition[1]])?;
        if !pressure.is_finite() || pressure <= 0.0 {
            return Err(ProviderError::Trial(
                "positive finite pressure required".into(),
            ));
        }
        let envelope = self.envelope()?;
        envelope.thermal_composition(
            temperature,
            [
                composition[0],
                composition[1],
                1.0 - composition[0] - composition[1],
            ],
        )?;
        envelope.pressure.check("pressure", pressure)?;
        let n = DVector::from_vec(vec![
            composition[0],
            composition[1],
            1.0 - composition[0] - composition[1],
        ]);
        let initial = match phase {
            InitialPhase::Vapor => DensityInitialization::Vapor,
            InitialPhase::Liquid => DensityInitialization::Liquid,
        };
        let state = State::new_npt(
            &self.eos,
            Temperature::new(temperature),
            Pressure::new(pressure),
            n,
            Some(initial),
        )
        .map_err(failure)?;
        envelope.density.check(
            "density",
            state
                .density
                .convert_into(MOL / quantity::METER.powi::<3>()),
        )?;
        let slope = state
            .dp_drho(Contributions::Total)
            .convert_into(PASCAL / (MOL / quantity::METER.powi::<3>()));
        let regularity = slope / (quantity::RGAS.convert_into(JOULE / MOL / KELVIN) * temperature);
        let actual = state.pressure(Contributions::Total).convert_into(PASCAL);
        envelope.pressure.check("pressure", actual)?;
        if !regularity.is_finite() || regularity <= 1e-8 {
            return Err(ProviderError::Singular(
                "NPT density derivative is nonregular".into(),
            ));
        }
        if !actual.is_finite() || (actual - pressure).abs() > 1e-3 + 1e-8 * pressure {
            return Err(ProviderError::Trial("NPT pressure closure failed".into()));
        }
        let stable = if check_stability {
            Some(state.is_stable(SolverOptions::default()).map_err(failure)?)
        } else {
            None
        };
        context.check()?;
        Ok(InitialState {
            molar_density: state
                .density
                .convert_into(MOL / quantity::METER.powi::<3>()),
            stable,
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
/// Initialization choice, with no implication that the returned phase is globally stable.
#[derive(Clone, Copy, Debug)]
pub enum InitialPhase {
    /// Low-density initial guess.
    Vapor,
    /// High-density initial guess.
    Liquid,
}
/// A checked candidate, separate from smooth property callbacks.
#[derive(Clone, Debug)]
pub struct InitialState {
    /// Density in mol/m³.
    pub molar_density: f64,
    /// Explicit stability diagnostic, absent when not requested.
    pub stable: Option<bool>,
}

#[derive(Clone, Debug)]
struct Cached {
    inputs: [u64; 4],
    request: ProviderRequest,
    values: ProviderValues,
}
/// Worker-local evaluator and cache. It never retains a mutable `FeOS` state across trials.
#[derive(Debug)]
pub struct FeosWorker {
    package: FeosPackage,
    cache: Option<Cached>,
    states: usize,
}
impl FeosWorker {
    /// Actual state constructions, including derivative-order upgrades.
    pub fn state_evaluations(&self) -> usize {
        self.states
    }
    fn properties<D: DualNum<f64> + Copy>(
        &mut self,
        x: [D; 4],
        outputs: &[usize],
    ) -> Result<Vec<D>, ProviderError> {
        let n = DVector::from_vec(vec![x[2], x[3], D::from(1.0) - x[2] - x[3]]);
        let state = State::new(
            &self.package.eos,
            Temperature::new(x[0]),
            Density::new(x[1]),
            n,
        )
        .map_err(failure)?;
        self.states += 1;
        let pressure = state.pressure(Contributions::Total).convert_into(PASCAL);
        self.package
            .envelope()?
            .pressure
            .check("pressure", pressure.re())?;
        let phi = outputs.iter().any(|&i| i >= 3).then(|| state.ln_phi());
        outputs
            .iter()
            .map(|&i| {
                Ok(match i {
                    0 => pressure,
                    1 => state
                        .molar_enthalpy(Contributions::Total)
                        .convert_into(JOULE / MOL),
                    2 => state
                        .molar_entropy(Contributions::Total)
                        .convert_into(JOULE / MOL / KELVIN),
                    3..=5 => phi
                        .as_ref()
                        .ok_or_else(|| ProviderError::Contract("missing fugacity demand".into()))?
                        [i - 3],
                    _ => return Err(ProviderError::Contract("property ordinal".into())),
                })
            })
            .collect()
    }
}
fn validate_inputs(x: &[f64]) -> Result<[f64; 4], ProviderError> {
    let x: [f64; 4] = x
        .try_into()
        .map_err(|_| ProviderError::Contract("FeOS input arity".into()))?;
    if x.iter().any(|v| !v.is_finite())
        || x[0] <= 0.0
        || x[1] <= 0.0
        || x[2] <= 0.0
        || x[3] <= 0.0
        || x[2] + x[3] >= 1.0
    {
        return Err(ProviderError::Trial(
            "positive T/rho and interior methane/ethane/propane composition required".into(),
        ));
    }
    Ok(x)
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
        let x = validate_inputs(inputs)?;
        let envelope = self.package.envelope()?;
        envelope.thermal_composition(x[0], [x[2], x[3], 1.0 - x[2] - x[3]])?;
        envelope.density.check("density", x[1])?;
        let bits = x.map(f64::to_bits);
        if let Some(c) = prior
            && c.inputs == bits
            && c.request.order >= request.order
            && request
                .outputs
                .iter()
                .all(|o| c.request.outputs.contains(o))
        {
            let mut values = ProviderValues {
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
                values.values.push(c.values.values[i]);
                if request.order >= DerivativeOrder::First {
                    values
                        .jacobian
                        .extend_from_slice(&c.values.jacobian[i * 4..(i + 1) * 4]);
                }
                if request.order >= DerivativeOrder::Second {
                    values
                        .hessians
                        .extend_from_slice(&c.values.hessians[i * 16..(i + 1) * 16]);
                }
            }
            self.cache = Some(c);
            return Ok(values);
        }
        let mut result = ProviderValues {
            values: vec![],
            jacobian: vec![],
            hessians: vec![],
        };
        match request.order {
            DerivativeOrder::Value => result.values = self.properties(x, &request.outputs)?,
            DerivativeOrder::First => {
                let dual = std::array::from_fn(|i| DualSVec64::<4>::from_re(x[i]).derivative(i));
                for v in self.properties(dual, &request.outputs)? {
                    result.values.push(v.re);
                    result
                        .jacobian
                        .extend(v.eps.unwrap_generic(Const::<4>, U1).iter().copied());
                }
            }
            DerivativeOrder::Second => {
                let dual = std::array::from_fn(|i| Dual2SVec64::<4>::from_re(x[i]).derivative(i));
                for v in self.properties(dual, &request.outputs)? {
                    result.values.push(v.re);
                    result
                        .jacobian
                        .extend(v.v1.unwrap_generic(U1, Const::<4>).iter().copied());
                    let h = v.v2.unwrap_generic(Const::<4>, Const::<4>);
                    for i in 0..4 {
                        for j in 0..4 {
                            result.hessians.push(h[(i, j)]);
                        }
                    }
                }
            }
        }
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

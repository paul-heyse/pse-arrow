// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicitly prepared CoolProp 8 HEOS water, with source-selected phase and datum.
//!
//! ADR-0238: source declarations select an offline descriptor; a caller explicitly
//! prepares and supplies the corresponding registration. No source name loads code.
//! Release data: <https://github.com/CoolProp/CoolProp/tree/ae81610e7d23efc57f9d051c8e70a4d66e87537f>.
//! The initial Linux x86_64 image is authenticated against the library in the published
//! <https://static.crates.io/crates/coolprop-sys-linux-x86-64/coolprop-sys-linux-x86-64-8.0.0.crate>.
#![allow(
    unsafe_code,
    reason = "narrow calls through coolprop-sys 8.0.0's generated C ABI, with owned live handles and bounded error buffers"
)]

use coolprop_sys::{COOLPROP, bindings::CoolProp};
use pse_ids::{ContentHash, Frame, FramedHasher, SemanticId};
use pse_kernels::{
    AdmittedProvider, DerivativeOrder, DerivativeSource, EvaluationContext, Port,
    PreparedProviderRequest, Provider, ProviderBinding, ProviderError, ProviderFactory,
    ProviderOutput, ProviderRuntime, ProviderShapes, ProviderSpec, Registration,
};
use pse_quantity::{PhysicalName, QuantityRegistry, ReferenceStateKind};
use sha2::{Digest, Sha256};
use std::{
    ffi::{CStr, c_long},
    path::Path,
    sync::Arc,
};

/// Exact upstream implementation selected by this adapter.
pub const COOLPROP_REVISION: &str = "ae81610e7d23efc57f9d051c8e70a4d66e87537f";
/// Exact released scientific parameter source, distinct from the wrapper revision.
pub const WATER_JSON_SHA256: &str =
    "85021d3803d2b02edb1d3e90bd60f4eee4a70380e06ed6b7b44e57716b4c2266";
const LINUX_X86_64_SHA256: &str =
    "5c53697a38d09dc5afea0c4976b790a8803ac4cd51d6b10af9b8ed9c2fdc3105";
const REVISION: &str = "daab2516d788ff809465a5c8243eafc687077e938908973b87b93fb1b33b0e02";
const FITTED_GAS_CONSTANT: f64 = 8.314_371_357_587;
const MOLAR_MASS: f64 = 0.018_015_268;
const ERROR_CAPACITY: usize = 1024;

/// The admitted homogeneous branch; normal evaluations never impose a phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaterRegime {
    /// Compressed/subcritical liquid, including the high-pressure liquid region.
    Liquid,
    /// Vapor and the continuous low-pressure gas region above critical temperature.
    Vapor,
    /// Temperature and pressure both exceed the critical point; property scope only.
    Supercritical,
}
impl WaterRegime {
    /// Authored implementation name used by the existing explicit provider join.
    pub const fn implementation(self) -> &'static str {
        match self {
            Self::Liquid => "coolprop8-water-liquid",
            Self::Vapor => "coolprop8-water-vapor",
            Self::Supercritical => "coolprop8-water-supercritical",
        }
    }
    fn data(self) -> &'static str {
        match self {
            Self::Liquid => "0893df81d6f6faddbf7366978c4ada9b0b691ce06959bb2bcd95e8362a0bd879",
            Self::Vapor => "57d9eceb4f994921eedbd4f47a015df62e2cfaa967da36122a55939ba02f3ba8",
            Self::Supercritical => {
                "23a36dfd43bd0546b2e755868d91c8104c5f86c5c1f274f5ed4fb1f76aedc7eb"
            }
        }
    }
    fn accepts(self, phase: i32) -> bool {
        // CoolProp 8 DataStructures.h: liquid=0, supercritical=1,
        // supercritical_gas=2, supercritical_liquid=3, gas=5, twophase=6.
        match self {
            Self::Liquid => matches!(phase, 0 | 3),
            Self::Vapor => matches!(phase, 2 | 5),
            Self::Supercritical => phase == 1,
        }
    }
}

fn contract(message: impl Into<String>) -> ProviderError {
    ProviderError::Contract(message.into())
}
fn terminal(message: impl Into<String>) -> ProviderError {
    ProviderError::Terminal(message.into())
}

fn specification(
    registry: &QuantityRegistry,
    regime: WaterRegime,
) -> Result<ProviderSpec, ProviderError> {
    let id = pse_ids::named_id(SemanticId::from_bytes([0xc8; 16]), regime.implementation());
    let port = |name: &str, quantity: &str| -> Result<Port, ProviderError> {
        let Some(PhysicalName::QuantityType(quantity)) = registry.physical_name(quantity) else {
            return Err(contract(
                "water provider requires its named physical quantities",
            ));
        };
        let unit = registry
            .quantity_type(quantity)
            .map_err(|error| contract(error.to_string()))?
            .canonical_unit;
        Ok(Port {
            id: pse_ids::named_id(id, name),
            quantity,
            unit,
        })
    };
    Ok(ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id,
        revision: ContentHash::parse_hex(REVISION).map_err(|error| contract(error.to_string()))?,
        data: ContentHash::parse_hex(regime.data()).map_err(|error| contract(error.to_string()))?,
        inputs: vec![
            port("temperature", "Temperature")?,
            port("density", "Density")?,
        ],
        outputs: vec![
            port("pressure", "Pressure")?,
            port("enthalpy", "MolarEnthalpy")?,
            port("entropy", "MolarEntropy")?,
        ],
        derivatives: DerivativeOrder::Second,
        smoothness: DerivativeOrder::Second,
    })
}

/// Construct a physical descriptor without loading or preparing the native library.
///
/// # Errors
/// Refuses missing or inconsistent physical quantity declarations.
pub fn descriptor(
    registry: &QuantityRegistry,
    regime: WaterRegime,
) -> Result<AdmittedProvider, ProviderError> {
    AdmittedProvider::new(specification(registry, regime)?, registry)
}

#[derive(Clone, Copy, Debug)]
struct Reference {
    temperature: f64,
    pressure: f64,
    gas_constant: f64,
    molar_mass: f64,
    h_offset: f64,
    s_offset: f64,
}

/// Prepared production capability with authenticated release and immutable datum.
#[derive(Clone, Debug)]
pub struct WaterFactory {
    spec: ProviderSpec,
    regime: WaterRegime,
    keys: Keys,
    reference: Reference,
    configuration: ContentHash,
}
impl WaterFactory {
    /// Explicitly load the bundled capability, authenticate it and establish its ideal-only anchor.
    /// No process-global reference-state setter is used.
    ///
    /// # Errors
    /// Returns an attributable preparation, release, physical-contract or native failure.
    pub fn prepare(
        registry: &QuantityRegistry,
        regime: WaterRegime,
    ) -> Result<Self, ProviderError> {
        let spec = specification(registry, regime)?;
        let stock = registry
            .reference_states()
            .find(|reference| reference.name == "stock")
            .ok_or_else(|| contract("water provider requires the declared stock reference"))?;
        if stock.kind != ReferenceStateKind::IdealGasAtConditions
            || stock.include_enthalpy_of_formation
        {
            return Err(contract(
                "water provider requires a sensible ideal-gas stock reference",
            ));
        }
        for output in &spec.outputs[1..] {
            if registry
                .quantity_type(output.quantity)
                .map_err(|error| contract(error.to_string()))?
                .key
                .reference_state
                != Some(stock.id)
            {
                return Err(contract(
                    "water caloric outputs must use the selected stock reference",
                ));
            }
        }
        let condition =
            |value: Option<pse_quantity::ReferenceCondition>| -> Result<f64, ProviderError> {
                registry
                    .reference_condition(
                        &value.ok_or_else(|| contract("stock reference condition is absent"))?,
                    )
                    .map_err(|error| contract(error.to_string()))
            };
        let temperature = condition(stock.temperature)?;
        let pressure = condition(stock.pressure)?;
        if !temperature.is_finite()
            || temperature <= 0.0
            || !pressure.is_finite()
            || pressure <= 0.0
        {
            return Err(contract(
                "stock reference conditions must be finite and positive",
            ));
        }
        ensure_library()?;
        let api = COOLPROP.shared_access();
        authenticate(&api)?;
        let keys = Keys::new(&api)?;
        let mut state = NativeState::new(&api)?;
        let reference = (|| -> Result<Reference, ProviderError> {
            let gas_constant = state.output(&api, keys.gas_constant)?;
            let molar_mass = state.output(&api, keys.molar_mass)?;
            if gas_constant.to_bits() != FITTED_GAS_CONSTANT.to_bits()
                || molar_mass.to_bits() != MOLAR_MASS.to_bits()
            {
                return Err(terminal(
                    "loaded HEOS Water constants differ from the selected released data",
                ));
            }
            // This is an ideal-gas reference construction, not an equilibrium liquid
            // or vapor state. Only ideal-only h/s outputs are read from this state.
            let mut error = ErrorBuffer::default();
            // SAFETY: the handle is live; literal phase and bounded error destinations outlive the call.
            unsafe {
                api.AbstractState_specify_phase(
                    state.handle,
                    c"phase_gas".as_ptr(),
                    &raw mut error.code,
                    error.message.as_mut_ptr().cast(),
                    ERROR_CAPACITY as c_long,
                );
            }
            error.check("ideal reference phase", false)?;
            state.update(
                &api,
                keys.density_temperature,
                pressure / (gas_constant * temperature),
                temperature,
            )?;
            Ok(Reference {
                temperature,
                pressure,
                gas_constant,
                molar_mass,
                h_offset: -state.output(&api, keys.ideal_h)?,
                s_offset: -state.output(&api, keys.ideal_s)?,
            })
        })();
        // Release the read guard before destruction, including the error path.
        drop(api);
        drop(state);
        let reference = reference?;
        let mut identity = FramedHasher::new(Frame::ProviderConfigurationV1);
        identity
            .hash(&spec.identity())
            .str(COOLPROP_REVISION)
            .str(WATER_JSON_SHA256)
            .str(LINUX_X86_64_SHA256);
        for value in [
            reference.temperature,
            reference.pressure,
            reference.gas_constant,
            reference.molar_mass,
            reference.h_offset,
            reference.s_offset,
        ] {
            identity.u64(value.to_bits());
        }
        Ok(Self {
            spec,
            regime,
            keys,
            reference,
            configuration: identity.finish_hash(),
        })
    }
}
impl ProviderFactory for WaterFactory {
    fn spec(&self) -> &ProviderSpec {
        &self.spec
    }
    fn configuration_key(&self) -> ContentHash {
        self.configuration
    }
    fn create(&self) -> Result<ProviderBinding, ProviderError> {
        // Preparation authenticated and loaded this process-lifetime function
        // table; fresh attempt scratch does not re-read the immutable image.
        let api = COOLPROP.shared_access();
        let state = NativeState::new(&api)?;
        drop(api);
        Ok(ProviderBinding::new(
            self.spec.clone(),
            Box::new(WaterWorker {
                state,
                factory: self.clone(),
            }),
        ))
    }
}

/// Prepare a production registration for explicit supply to the existing runtime/package ingress.
///
/// # Errors
/// Refuses unavailable native preparation, changed release/data or descriptor mismatch.
pub fn registration(
    registry: &QuantityRegistry,
    regime: WaterRegime,
) -> Result<Registration, ProviderError> {
    let descriptor = descriptor(registry, regime)?;
    Registration::bind(
        descriptor,
        Arc::new(WaterFactory::prepare(registry, regime)?),
    )
}

fn ensure_library() -> Result<(), ProviderError> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(terminal(
            "CoolProp capability is unprepared: this platform's bundled artifact is not yet qualified",
        ));
    }
    if !Path::new(coolprop_sys::COOLPROP_PATH).is_file() {
        return Err(terminal(
            "CoolProp capability is unprepared: bundled native library is absent",
        ));
    }
    let image = std::fs::read(coolprop_sys::COOLPROP_PATH)
        .map_err(|error| terminal(format!("CoolProp prepared artifact is unreadable: {error}")))?;
    let digest = Sha256::digest(&image)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if digest != LINUX_X86_64_SHA256 {
        return Err(terminal(
            "CoolProp prepared artifact differs from the selected published platform image",
        ));
    }
    std::panic::catch_unwind(|| std::sync::LazyLock::force(&COOLPROP))
        .map(|_| ())
        .map_err(|_| {
            terminal(
                "CoolProp capability is unprepared: bundled library or required ABI failed to load",
            )
        })
}

fn global(api: &CoolProp, key: &CStr) -> Result<String, ProviderError> {
    let mut buffer = [0u8; ERROR_CAPACITY];
    // SAFETY: static key and writable buffer remain live for their declared extents.
    let result = unsafe {
        api.get_global_param_string(
            key.as_ptr(),
            buffer.as_mut_ptr().cast(),
            ERROR_CAPACITY as i32,
        )
    };
    if result != 1 {
        return Err(terminal("CoolProp release metadata is unavailable"));
    }
    Ok(buffer_string(&buffer))
}
fn authenticate(api: &CoolProp) -> Result<(), ProviderError> {
    let version = global(api, c"version")?;
    let revision = global(api, c"gitrevision")?;
    if version != "8.0.0" || revision != COOLPROP_REVISION {
        return Err(terminal(format!(
            "CoolProp capability differs from selected 8.0.0 release: version={version}, revision={revision}"
        )));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct Keys {
    temperature: c_long,
    density: c_long,
    pressure: c_long,
    h: c_long,
    s: c_long,
    u: c_long,
    gas_constant: c_long,
    molar_mass: c_long,
    ideal_h: c_long,
    ideal_s: c_long,
    density_temperature: c_long,
}
impl Keys {
    fn new(api: &CoolProp) -> Result<Self, ProviderError> {
        let parameter = |name: &CStr| -> Result<c_long, ProviderError> {
            // SAFETY: every key is an immutable nul-terminated literal.
            let key = unsafe { api.get_param_index(name.as_ptr()) };
            if key < 0 {
                Err(terminal("CoolProp selected parameter is unavailable"))
            } else {
                Ok(key)
            }
        };
        // SAFETY: immutable nul-terminated literal; no borrowed destination.
        let density_temperature = unsafe { api.get_input_pair_index(c"DmolarT_INPUTS".as_ptr()) };
        if density_temperature < 0 {
            return Err(terminal(
                "CoolProp explicit-density input pair is unavailable",
            ));
        }
        Ok(Self {
            temperature: parameter(c"T")?,
            density: parameter(c"Dmolar")?,
            pressure: parameter(c"P")?,
            h: parameter(c"Hmolar")?,
            s: parameter(c"Smolar")?,
            u: parameter(c"Umolar")?,
            gas_constant: parameter(c"gas_constant")?,
            molar_mass: parameter(c"molar_mass")?,
            ideal_h: parameter(c"Hmolar_idealgas")?,
            ideal_s: parameter(c"Smolar_idealgas")?,
            density_temperature,
        })
    }
}

#[derive(Debug)]
struct ErrorBuffer {
    code: c_long,
    message: [u8; ERROR_CAPACITY],
}
impl Default for ErrorBuffer {
    fn default() -> Self {
        Self {
            code: 0,
            message: [0; ERROR_CAPACITY],
        }
    }
}
impl ErrorBuffer {
    fn check(self, operation: &str, trial: bool) -> Result<(), ProviderError> {
        if self.code == 0 {
            return Ok(());
        }
        let detail = format!(
            "CoolProp {operation}: code {}: {}",
            self.code,
            buffer_string(&self.message)
        );
        if trial {
            Err(ProviderError::Trial(detail))
        } else {
            Err(terminal(detail))
        }
    }
}
fn buffer_string(buffer: &[u8]) -> String {
    let end = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());
    String::from_utf8_lossy(&buffer[..end]).into_owned()
}
fn finite(value: f64) -> Result<f64, ProviderError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ProviderError::Trial("nonfinite CoolProp result".into()))
    }
}

#[derive(Debug)]
struct NativeState {
    handle: c_long,
}
impl NativeState {
    fn new(api: &CoolProp) -> Result<Self, ProviderError> {
        let mut error = ErrorBuffer::default();
        // SAFETY: literals and bounded error destinations are live; returned ownership is unique.
        let handle = unsafe {
            api.AbstractState_factory(
                c"HEOS".as_ptr(),
                c"Water".as_ptr(),
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            )
        };
        error.check("state construction", false)?;
        if handle < 0 {
            return Err(terminal("CoolProp did not return a live state"));
        }
        Ok(Self { handle })
    }
    fn update(
        &mut self,
        api: &CoolProp,
        pair: c_long,
        first: f64,
        second: f64,
    ) -> Result<(), ProviderError> {
        let mut error = ErrorBuffer::default();
        // SAFETY: this owner alone mutates its live state; error destinations are bounded and live.
        unsafe {
            api.AbstractState_update(
                self.handle,
                pair,
                first,
                second,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            );
        }
        error.check("state update", true)
    }
    fn output(&self, api: &CoolProp, key: c_long) -> Result<f64, ProviderError> {
        let mut error = ErrorBuffer::default();
        // SAFETY: live exclusively owned state, selected key and bounded live error destinations.
        let value = unsafe {
            api.AbstractState_keyed_output(
                self.handle,
                key,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            )
        };
        error.check("state output", true)?;
        finite(value)
    }
    fn phase(&self, api: &CoolProp) -> Result<i32, ProviderError> {
        let mut error = ErrorBuffer::default();
        // SAFETY: live state and bounded writable error destinations.
        let value = unsafe {
            api.AbstractState_phase(
                self.handle,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            )
        };
        error.check("phase selection", true)?;
        Ok(value)
    }
    fn first(
        &self,
        api: &CoolProp,
        output: c_long,
        wrt: c_long,
        held: c_long,
    ) -> Result<f64, ProviderError> {
        let mut error = ErrorBuffer::default();
        // SAFETY: live homogeneous state, selected parameter keys and bounded error destinations.
        let value = unsafe {
            api.AbstractState_first_partial_deriv(
                self.handle,
                output,
                wrt,
                held,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            )
        };
        error.check("first homogeneous partial", true)?;
        finite(value)
    }
    fn second(
        &self,
        api: &CoolProp,
        output: c_long,
        wrt: c_long,
        held: c_long,
        wrt2: c_long,
        held2: c_long,
    ) -> Result<f64, ProviderError> {
        let mut error = ErrorBuffer::default();
        // SAFETY: live homogeneous state, selected parameter keys and bounded error destinations.
        let value = unsafe {
            api.AbstractState_second_partial_deriv(
                self.handle,
                output,
                wrt,
                held,
                wrt2,
                held2,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            )
        };
        error.check("second homogeneous partial", true)?;
        finite(value)
    }
}
impl Drop for NativeState {
    fn drop(&mut self) {
        let api = COOLPROP.shared_access();
        let mut error = ErrorBuffer::default();
        // SAFETY: the unique owner frees its still-live handle once, after all calls have returned.
        unsafe {
            api.AbstractState_free(
                self.handle,
                &raw mut error.code,
                error.message.as_mut_ptr().cast(),
                ERROR_CAPACITY as c_long,
            );
        }
    }
}

#[derive(Debug)]
struct WaterWorker {
    state: NativeState,
    factory: WaterFactory,
}
impl Provider for WaterWorker {
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        _runtime: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        context.check()?;
        let [temperature, density] = inputs else {
            return Err(contract("water provider requires T and molar density"));
        };
        for (axis, value, lower, upper) in [
            ("temperature", *temperature, 273.16, 1200.0),
            ("molar density", *density, 1e-8, 70000.0),
        ] {
            if !value.is_finite() || value < lower || value > upper {
                return Err(ProviderError::OutsideEnvelope {
                    axis: axis.into(),
                    value,
                    lower,
                    upper,
                });
            }
        }
        // One shared access guard spans the coherent update, values and jets.
        // It permits independent HEOS states, while excluding global mutation.
        let api = COOLPROP.shared_access();
        let keys = self.factory.keys;
        self.state
            .update(&api, keys.density_temperature, *density, *temperature)?;
        let phase = self.state.phase(&api)?;
        if !self.factory.regime.accepts(phase) {
            return Err(ProviderError::Trial(format!(
                "water {:?} provider refuses actual CoolProp phase {phase}",
                self.factory.regime
            )));
        }
        let pressure = self.state.output(&api, keys.pressure)?;
        if pressure <= 0.0 || pressure > 100_000_000.0 {
            return Err(ProviderError::OutsideEnvelope {
                axis: "pressure".into(),
                value: pressure,
                lower: 0.0,
                upper: 100_000_000.0,
            });
        }
        if self
            .state
            .first(&api, keys.pressure, keys.density, keys.temperature)?
            <= 0.0
        {
            return Err(ProviderError::Trial(
                "water state lacks positive isothermal mechanical response".into(),
            ));
        }
        let h = self.state.output(&api, keys.h)? + self.factory.reference.h_offset;
        let s = self.state.output(&api, keys.s)? + self.factory.reference.s_offset;
        // Private native consistency observation; no public enthalpy-typed u alias.
        let u = self.state.output(&api, keys.u)? + self.factory.reference.h_offset;
        let mismatch = h - u - pressure / density;
        if mismatch.abs() > 1e-9 * (1.0 + h.abs() + u.abs() + (pressure / density).abs()) {
            return Err(terminal("water h/u datum translation violates h=u+p/rho"));
        }
        let values = [pressure, h, s];
        let native_outputs = [keys.pressure, keys.h, keys.s];
        let wrt = [keys.temperature, keys.density];
        for (position, ordinal) in request.outputs.iter().copied().enumerate() {
            output.values[position] = values[ordinal];
            if request.order >= DerivativeOrder::First {
                for (first, first_wrt) in wrt.iter().copied().enumerate() {
                    output.jacobian[position * 2 + first] = self.state.first(
                        &api,
                        native_outputs[ordinal],
                        first_wrt,
                        wrt[1 - first],
                    )?;
                }
            }
            if request.order == DerivativeOrder::Second {
                for (first, first_wrt) in wrt.iter().copied().enumerate() {
                    for (second, second_wrt) in wrt.iter().copied().enumerate() {
                        output.hessians[position * 4 + first * 2 + second] = self.state.second(
                            &api,
                            native_outputs[ordinal],
                            first_wrt,
                            wrt[1 - first],
                            second_wrt,
                            wrt[1 - second],
                        )?;
                    }
                }
            }
            context.check()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

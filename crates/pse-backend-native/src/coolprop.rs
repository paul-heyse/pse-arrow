// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicitly prepared CoolProp 8 HEOS water, with source-selected phase and datum.
//!
//! ADR-0238: source declarations select an offline descriptor; a caller explicitly
//! prepares and supplies the corresponding registration. No source name loads code.
//! Release data: <https://github.com/CoolProp/CoolProp/tree/ae81610e7d23efc57f9d051c8e70a4d66e87537f>.
//! One explicitly prepared source/patch/dependency/toolchain image serves homogeneous and
//! saturated calls; build admission and runtime bytes retain the same original receipt.
//! The admitted process profile has no global CoolProp reference/configuration mutation. A newly
//! constructed scalar or grouped state relies on that lifetime premise; a common native h/u shift
//! would preserve energy consistency while changing the factory's original datum. An embedding that
//! admits global setters must requalify fresh-state reference meaning before using this profile.
#![allow(
    unsafe_code,
    reason = "narrow calls through coolprop-sys 8.0.0's generated C ABI, with owned live handles and bounded error buffers"
)]

use coolprop_sys::{COOLPROP, bindings::CoolProp};
use pse_ids::{ContentHash, Frame, FramedHasher, SemanticId};
use pse_kernels::{
    AdmittedProvider, DerivativeOrder, DerivativeSource, EvaluationContext, Port,
    PreparedProviderRequest, Provider, ProviderBatchPoint, ProviderBatchResults, ProviderBinding,
    ProviderDemand, ProviderError, ProviderFactory, ProviderOutput, ProviderRuntime,
    ProviderShapes, ProviderSpec, ProviderValues, Registration,
};
use pse_quantity::{QuantityRegistry, ReferenceStateKind};
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

// Fixed consumed roles declared in packages/reference/physical/materials/physical.yaml.
// Other nominal mappings need an explicitly declared native binding; neither names nor
// equal dimensions grant this ABI's scientific interpretation. Stock conditions remain data.
const WATER_MOLAR_BASIS: pse_quantity::BasisId =
    pse_quantity::BasisId::from_bytes(0x04d4600b17cbfb717ec549e688ea3503_u128.to_be_bytes());
const WATER_STOCK_DATUM: pse_quantity::ReferenceStateId =
    pse_quantity::ReferenceStateId::from_bytes(
        0xebfd9ee6148d4c6497d47099a39e0bfa_u128.to_be_bytes(),
    );

fn specification(
    registry: &QuantityRegistry,
    regime: WaterRegime,
) -> Result<ProviderSpec, ProviderError> {
    let id = pse_ids::named_id(SemanticId::from_bytes([0xc8; 16]), regime.implementation());
    // The C ABI consumes SI Kelvin, mol/m³, Pa, J/mol and J/(mol K).
    // A named canonical unit need not have unit SI scale in an admitted registry.
    let port = |name: &str,
                quantity: u128,
                nominal_kind: u128,
                dimensions: [i32; 8],
                molar: bool,
                caloric: bool|
     -> Result<Port, ProviderError> {
        let quantity = pse_quantity::QuantityTypeId::from_bytes(quantity.to_be_bytes());
        let nominal_kind = pse_quantity::QuantityKindId::from_bytes(nominal_kind.to_be_bytes());
        let ty = registry
            .quantity_type(quantity)
            .map_err(|error| contract(error.to_string()))?;
        let unit = registry
            .unit(ty.canonical_unit)
            .map_err(|error| contract(error.to_string()))?;
        let kind = registry
            .kind(ty.key.kind)
            .map_err(|error| contract(error.to_string()))?;
        let expected = dimensions
            .into_iter()
            .map(|exponent| pse_quantity::Ratio::new(exponent, 1))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| contract(error.to_string()))?;
        let basis = ty
            .key
            .basis
            .map(|basis| {
                registry
                    .basis(basis)
                    .map_err(|error| contract(error.to_string()))
            })
            .transpose()?;
        if ty.key.kind != nominal_kind
            || ty.key.basis != molar.then_some(WATER_MOLAR_BASIS)
            || ty.key.reference_state != caloric.then_some(WATER_STOCK_DATUM)
            || kind.dimension.exponents().as_slice() != expected.as_slice()
            || kind.extensive
            || kind.category.is_some()
            || kind.addition_kind
                != if molar && !caloric {
                    pse_quantity::QuantityAdditionKind::Additive
                } else {
                    pse_quantity::QuantityAdditionKind::OriginSensitive
                }
            || unit.scale_to_canonical != 1.0
            || unit.offset_to_canonical != 0.0
            || unit.is_affine
            || ty.key.scale_kind != pse_quantity::ScaleKind::Point
            || !ty.key.shape.is_empty()
            || ty.key.subject_kind.is_some()
            || match basis {
                Some(basis) => {
                    !molar
                        || basis.kind != pse_quantity::BasisKind::Molar
                        || basis.composition_basis.is_some()
                        || basis.rate_basis.is_some()
                        || basis.reference_conditions.is_some()
                }
                None => molar,
            }
        {
            return Err(contract(format!(
                "water provider requires its scalar SI {quantity} physical contract"
            )));
        }
        Ok(Port {
            id: pse_ids::named_id(id, name),
            quantity,
            unit: unit.id,
        })
    };
    Ok(ProviderSpec {
        shapes: ProviderShapes::default(),
        derivative_source: DerivativeSource::Analytic,
        id,
        revision: ContentHash::parse_hex(REVISION).map_err(|error| contract(error.to_string()))?,
        data: ContentHash::parse_hex(regime.data()).map_err(|error| contract(error.to_string()))?,
        inputs: vec![
            port(
                "temperature",
                0xc64b96975a4a59755f8711d3bf628bc9,
                0x57a7ab92753b012231a543d3850b926c,
                [0, 0, 0, 1, 0, 0, 0, 0],
                false,
                false,
            )?,
            port(
                "density",
                0xb6edcb08fc072d81a9f99b45f7f9c90a,
                0x89a563673835109d4c3a68d4603bab31,
                [-3, 0, 0, 0, 1, 0, 0, 0],
                true,
                false,
            )?,
        ],
        outputs: vec![
            port(
                "pressure",
                0xa0fa145fd8b2ecec448c39666960debc,
                0xf789180fbded8b11fd73e2436d33fc9e,
                [-1, 1, -2, 0, 0, 0, 0, 0],
                false,
                false,
            )?,
            port(
                "enthalpy",
                0x1831d0d72dc74b299ba8ecb6d4da6f53,
                0x99263c30741ca81f1c4c3d4b987d4085,
                [2, 1, -2, 0, -1, 0, 0, 0],
                true,
                true,
            )?,
            port(
                "entropy",
                0x571c90b8909e43c0ae859554cc2e3479,
                0xfaa6f95459144650ac414fd389e16e27,
                [2, 1, -2, -1, -1, 0, 0, 0],
                true,
                true,
            )?,
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
    /// Products supplied directly by this implementation's homogeneous partial APIs.
    pub fn supports(&self, demand: &ProviderDemand) -> bool {
        matches!(
            demand,
            ProviderDemand::CompleteJets | ProviderDemand::SelectedColumns { .. }
        )
    }
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
            .reference_state(WATER_STOCK_DATUM)
            .map_err(|error| contract(error.to_string()))?;
        if stock.kind != ReferenceStateKind::IdealGasAtConditions
            || stock.include_enthalpy_of_formation
            || stock.subject.is_some()
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
        let condition = |value: Option<pse_quantity::ReferenceCondition>,
                         expected: pse_quantity::QuantityTypeId|
         -> Result<f64, ProviderError> {
            let value = value.ok_or_else(|| contract("stock reference condition is absent"))?;
            if value.quantity_type != expected {
                return Err(contract(
                    "stock condition differs from the selected nominal water role",
                ));
            }
            registry
                .reference_condition(&value)
                .map_err(|error| contract(error.to_string()))
        };
        let temperature = condition(stock.temperature, spec.inputs[0].quantity)?;
        let pressure = condition(stock.pressure, spec.outputs[0].quantity)?;
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
            .str(coolprop_sys::COOLPROP_IMAGE_SHA256)
            .str(include_str!(concat!(
                env!("OUT_DIR"),
                "/coolprop-profile.json"
            )));
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
            "CoolProp capability is unprepared: this source-prepared target is not yet qualified",
        ));
    }
    if !Path::new(coolprop_sys::COOLPROP_PATH).is_file() {
        return Err(terminal(
            "CoolProp capability is unprepared: selected native library is absent",
        ));
    }
    let image = std::fs::read(coolprop_sys::COOLPROP_PATH)
        .map_err(|error| terminal(format!("CoolProp prepared artifact is unreadable: {error}")))?;
    let digest = Sha256::digest(&image)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if digest != coolprop_sys::COOLPROP_IMAGE_SHA256 {
        return Err(terminal(
            "CoolProp prepared artifact differs from the authenticated compiled image",
        ));
    }
    std::panic::catch_unwind(|| std::sync::LazyLock::force(&COOLPROP))
        .map(|_| ())
        .map_err(|_| {
            terminal(
                "CoolProp capability is unprepared: selected library or required ABI failed to load",
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
fn water_inputs(inputs: &[f64]) -> Result<[f64; 2], ProviderError> {
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
    Ok([*temperature, *density])
}
#[derive(Clone, Copy)]
enum WaterField {
    Pressure = 0,
    Enthalpy = 1,
    Entropy = 2,
    InternalEnergy = 3,
    Mechanical = 5,
}
/// One native scientific owner for both scalar and group phase, stability, pressure and datum policy.
fn qualified_water_values(
    factory: &WaterFactory,
    density: f64,
    phase: i32,
    mut observe: impl FnMut(WaterField) -> Result<f64, ProviderError>,
) -> Result<[f64; 3], ProviderError> {
    if !factory.regime.accepts(phase) {
        return Err(ProviderError::Trial(format!(
            "water {:?} provider refuses actual CoolProp phase {phase}",
            factory.regime
        )));
    }
    let pressure = finite(observe(WaterField::Pressure)?)?;
    if pressure <= 0.0 || pressure > 100_000_000.0 {
        return Err(ProviderError::OutsideEnvelope {
            axis: "pressure".into(),
            value: pressure,
            lower: 0.0,
            upper: 100_000_000.0,
        });
    }
    if finite(observe(WaterField::Mechanical)?)? <= 0.0 {
        return Err(ProviderError::Trial(
            "water state lacks positive isothermal mechanical response".into(),
        ));
    }
    let h = finite(observe(WaterField::Enthalpy)?)? + factory.reference.h_offset;
    let s = finite(observe(WaterField::Entropy)?)? + factory.reference.s_offset;
    let u = finite(observe(WaterField::InternalEnergy)?)? + factory.reference.h_offset;
    let mismatch = h - u - pressure / density;
    if mismatch.abs() > 1e-9 * (1.0 + h.abs() + u.abs() + (pressure / density).abs()) {
        return Err(terminal("water h/u datum translation violates h=u+p/rho"));
    }
    Ok([pressure, h, s])
}
/// Reject every missing slot, including private physical guards, before admitting any row.
fn qualified_water_row(
    factory: &WaterFactory,
    density: f64,
    row: &[f64],
) -> Result<[f64; 3], ProviderError> {
    if row.len() != 6 || row.iter().any(|value| !value.is_finite()) {
        return Err(ProviderError::Trial(
            "incomplete CoolProp batch point".into(),
        ));
    }
    let phase = row[4];
    if phase.fract() != 0.0 || !(0.0..=8.0).contains(&phase) {
        return Err(ProviderError::Trial("invalid CoolProp batch phase".into()));
    }
    qualified_water_values(factory, density, phase as i32, |field| {
        Ok(row[field as usize])
    })
}
#[cfg(test)]
thread_local! { static BATCH_NATIVE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
impl Provider for WaterWorker {
    fn supports(&self, demand: &ProviderDemand) -> bool {
        self.factory.supports(demand)
    }
    fn supports_batch(&self, request: &PreparedProviderRequest) -> bool {
        request.order == DerivativeOrder::Value
            && matches!(request.demand, ProviderDemand::CompleteJets)
    }
    fn fill_batch(
        &mut self,
        points: &[&ProviderBatchPoint],
        request: &PreparedProviderRequest,
        context: &EvaluationContext<'_>,
    ) -> Result<ProviderBatchResults, ProviderError> {
        context.check()?;
        if !self.supports_batch(request) {
            return Err(contract("water batch demand unavailable"));
        }
        // Account for Rust input/mapping/output storage and the library's copied input and result matrix.
        let bytes = points
            .len()
            .checked_mul(20 * size_of::<f64>() + size_of::<usize>())
            .ok_or(ProviderError::Limit("water native batch storage"))?;
        if bytes > context.max_result_bytes {
            return Err(ProviderError::Limit("water native batch storage"));
        }
        let mut results: ProviderBatchResults = points
            .iter()
            .map(|_| Err(contract("water point not admitted")))
            .collect();
        let mut temperatures = Vec::with_capacity(points.len());
        let mut densities = Vec::with_capacity(points.len());
        let mut ordinals = Vec::with_capacity(points.len());
        for (ordinal, point) in points.iter().enumerate() {
            match water_inputs(&point.inputs) {
                Ok([temperature, density]) => {
                    temperatures.push(temperature);
                    densities.push(density);
                    ordinals.push(ordinal);
                }
                Err(error) => results[ordinal] = Err(error),
            }
        }
        if ordinals.is_empty() {
            return Ok(results);
        }
        let count = c_long::try_from(ordinals.len())
            .map_err(|_| ProviderError::Limit("water batch point count"))?;
        let mut rows = count;
        let mut columns = 6;
        let cells = ordinals
            .len()
            .checked_mul(6)
            .ok_or(ProviderError::Limit("water batch matrix"))?;
        let mut matrix = vec![f64::NAN; cells];
        let mut backend = *b"HEOS\0";
        let fractions = [1.0];
        // PropsSImulti uses the process-global error slot. Keep exclusive access through retrieval,
        // never retry a failed group, and never mutate global delimiter/reference configuration.
        let api = COOLPROP.exclusive_access();
        let _previous_error = global(&api, c"errstring")?;
        context.check()?;
        #[cfg(test)]
        BATCH_NATIVE_CALLS.with(|calls| calls.set(calls.get() + 1));
        // SAFETY: checked equal input lengths, checked owned point-major output capacity, mutable
        // backend bytes and all literals/fractions remain live. The group owns all buffers until return.
        unsafe {
            api.PropsSImulti(
                c"P,Hmolar,Smolar,Umolar,Phase,d(P)/d(Dmolar)|T".as_ptr(),
                c"T".as_ptr(),
                temperatures.as_mut_ptr(),
                count,
                c"Dmolar".as_ptr(),
                densities.as_mut_ptr(),
                count,
                backend.as_mut_ptr().cast(),
                c"Water".as_ptr(),
                fractions.as_ptr(),
                1,
                matrix.as_mut_ptr(),
                &raw mut rows,
                &raw mut columns,
            );
        }
        let detail = global(&api, c"errstring")?;
        if rows != count || columns != 6 {
            return Err(terminal(format!(
                "CoolProp batch matrix {rows}x{columns}, expected {count}x6: {detail}"
            )));
        }
        context.check()?;
        for (native, ordinal) in ordinals.into_iter().enumerate() {
            results[ordinal] = qualified_water_row(
                &self.factory,
                densities[native],
                &matrix[native * 6..native * 6 + 6],
            )
            .map(|values| ProviderValues {
                values: request
                    .outputs
                    .iter()
                    .map(|&ordinal| values[ordinal])
                    .collect(),
                jacobian: vec![],
                hessians: vec![],
            });
        }
        context.check()?;
        Ok(results)
    }
    fn fill(
        &mut self,
        inputs: &[f64],
        request: &PreparedProviderRequest,
        _runtime: ProviderRuntime<'_>,
        output: ProviderOutput<'_>,
        context: &EvaluationContext<'_>,
    ) -> Result<(), ProviderError> {
        context.check()?;
        let columns: &[usize] = match &request.demand {
            ProviderDemand::CompleteJets => &[0, 1],
            ProviderDemand::SelectedColumns { columns } => columns,
            _ => return Err(contract("water scalar product unavailable")),
        };
        let width = columns.len();
        let [temperature, density] = water_inputs(inputs)?;
        // One shared access guard spans the coherent update, values and jets.
        // It permits independent HEOS states, while excluding global mutation.
        let api = COOLPROP.shared_access();
        let keys = self.factory.keys;
        self.state
            .update(&api, keys.density_temperature, density, temperature)?;
        let phase = self.state.phase(&api)?;
        let values = qualified_water_values(&self.factory, density, phase, |field| match field {
            WaterField::Pressure => self.state.output(&api, keys.pressure),
            WaterField::Mechanical => {
                self.state
                    .first(&api, keys.pressure, keys.density, keys.temperature)
            }
            WaterField::Enthalpy => self.state.output(&api, keys.h),
            WaterField::Entropy => self.state.output(&api, keys.s),
            WaterField::InternalEnergy => self.state.output(&api, keys.u),
        })?;
        let native_outputs = [keys.pressure, keys.h, keys.s];
        let wrt = [keys.temperature, keys.density];
        for (position, ordinal) in request.outputs.iter().copied().enumerate() {
            output.values[position] = values[ordinal];
            if request.order >= DerivativeOrder::First {
                for (first, &column) in columns.iter().enumerate() {
                    output.jacobian[position * width + first] = self.state.first(
                        &api,
                        native_outputs[ordinal],
                        wrt[column],
                        wrt[1 - column],
                    )?;
                }
            }
            if request.order == DerivativeOrder::Second {
                for (first, &column) in columns.iter().enumerate() {
                    for (second, &second_column) in columns.iter().enumerate() {
                        output.hessians[position * width * width + first * width + second] =
                            self.state.second(
                                &api,
                                native_outputs[ordinal],
                                wrt[column],
                                wrt[1 - column],
                                wrt[second_column],
                                wrt[1 - second_column],
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

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Library-owned arithmetic with explicit physical and guarded evaluation boundaries.
pub mod assembly;
pub mod binding;
pub mod coefficients;
pub mod convexity;
mod error;
mod execution;
pub mod facts;
mod functions;
pub mod guarded;
pub mod jets;
pub mod library;
pub mod normalization;
pub mod numerics;
pub mod presolve;
pub mod sparse;
pub mod typed;
pub use error::MathError;
pub use functions::{Function, Implementation, UnaryFunction};
#[cfg(test)]
mod assembly_tests;
#[cfg(test)]
mod derivative_tests;
#[cfg(test)]
mod guarded_tests;
#[cfg(test)]
mod typed_tests;

/// Initialized process-global symbol vocabulary. Construction is an explicit effect;
/// every mathematical helper only reads this immutable context.
#[derive(Debug)]
pub struct SymbolicContext {
    /// Actual linked native math environment captured at initialization.
    pub environment: MathEnvironment,
    pub(crate) formals: Vec<symbolica::atom::Atom>,
    pub(crate) functions: Vec<symbolica::atom::Symbol>,
}
static CONTEXT: std::sync::OnceLock<Result<SymbolicContext, String>> = std::sync::OnceLock::new();

/// Initialize Symbolica and register the bounded vocabulary in stable order before
/// admitting any model. The existing optional license configuration is unchanged.
/// Secrets never enter artifacts, identities or diagnostics.
/// # Errors
/// Invalid existing license configuration or incompatible symbol registration.
pub fn initialize() -> Result<&'static SymbolicContext, MathError> {
    CONTEXT
        .get_or_init(|| {
            match std::env::var("SYMBOLICA_LICENSE") {
                Ok(key) => symbolica::license::LicenseManager::set_license_key(&key)
                    .map_err(|_| "Symbolica license activation failed".to_owned())?,
                Err(std::env::VarError::NotPresent) => {}
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err("Symbolica license environment is not Unicode".to_owned());
                }
            }
            symbolica::GLOBAL_SETTINGS
                .initialize_tracing
                .store(false, std::sync::atomic::Ordering::Relaxed);
            library::register_symbols()
        })
        .as_ref()
        .map_err(|message| MathError::Library(message.clone()))
}

/// Read the initialized context without effects. Explicit initialization is required
/// at a compiler/runtime entry boundary, never performed from a tracked query.
/// # Errors
/// The application has not initialized its mathematical runtime.
pub fn context() -> Result<&'static SymbolicContext, MathError> {
    CONTEXT
        .get()
        .ok_or_else(|| {
            MathError::Contract(
                "initialize the symbolic runtime before mathematical construction".into(),
            )
        })?
        .as_ref()
        .map_err(|message| MathError::Library(message.clone()))
}

/// Opaque lifetime anchor; allocation policy belongs to the runtime, never semantic equality.
pub trait AllocationOwner: std::fmt::Debug + Send + Sync + 'static {}
impl<T: std::fmt::Debug + Send + Sync + 'static> AllocationOwner for T {}

/// Runtime facts, distinct from declared dependency pins and case policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MathEnvironment {
    /// Linked GMP runtime version.
    pub gmp: String,
    /// Linked MPFR runtime version.
    pub mpfr: String,
}
impl MathEnvironment {
    /// Environment projection used alongside build and optimization profile identities.
    pub fn identity(&self) -> pse_ids::ContentHash {
        let mut h = pse_ids::FramedHasher::new("pse.math.environment.v1");
        h.str(&self.gmp)
            .str(&self.mpfr)
            .str("stable-formals-and-functions-v1;semantic-and-numerical-agreement;binary64");
        h.finish_hash()
    }
}
#[allow(
    unsafe_code,
    reason = "ADR-0088: read immutable process-lifetime GMP/MPFR version strings"
)]
fn linked_environment() -> Result<MathEnvironment, String> {
    // SAFETY: the pinned GMP library exports an immutable process-lifetime pointer.
    let gmp_ptr = unsafe { gmp_mpfr_sys::gmp::version };
    // SAFETY: GMP's version pointer is nonnull and NUL-terminated.
    let gmp = unsafe { std::ffi::CStr::from_ptr(gmp_ptr) };
    // SAFETY: the pinned MPFR function reads no mutable state and takes no arguments.
    let mpfr_ptr = unsafe { gmp_mpfr_sys::mpfr::get_version() };
    // SAFETY: MPFR returns a nonnull, NUL-terminated process-lifetime string.
    let mpfr = unsafe { std::ffi::CStr::from_ptr(mpfr_ptr) };
    Ok(MathEnvironment {
        gmp: gmp.to_str().map_err(|_| "invalid GMP version")?.to_owned(),
        mpfr: mpfr
            .to_str()
            .map_err(|_| "invalid MPFR version")?
            .to_owned(),
    })
}

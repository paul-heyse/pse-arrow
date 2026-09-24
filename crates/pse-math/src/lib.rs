// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Library-owned arithmetic with explicit physical and guarded evaluation boundaries.
pub mod assembly;
pub mod binding;
pub mod coefficients;
mod error;
mod execution;
pub mod facts;
mod functions;
pub mod guarded;
pub mod jets;
pub mod library;
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

/// Initialize Symbolica once, before constructing any library values. An optional
/// `SYMBOLICA_LICENSE` is supplied to its public license API; secrets never enter
/// artifacts, identities or diagnostics. No subscriber is installed in the host.
/// # Errors
/// A supplied license cannot be activated, or its environment value is not Unicode.
pub fn initialize() -> Result<(), MathError> {
    static INITIALIZED: std::sync::OnceLock<Result<(), &'static str>> = std::sync::OnceLock::new();
    INITIALIZED
        .get_or_init(|| {
            match std::env::var("SYMBOLICA_LICENSE") {
                Ok(key) => symbolica::license::LicenseManager::set_license_key(&key)
                    .map_err(|_| "Symbolica license activation failed")?,
                Err(std::env::VarError::NotPresent) => {}
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err("Symbolica license environment is not Unicode");
                }
            }
            symbolica::GLOBAL_SETTINGS
                .initialize_tracing
                .store(false, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        })
        .map_err(|message| MathError::Library(message.to_string()))
}

/// Opaque lifetime anchor; allocation policy belongs to the runtime, never semantic equality.
pub trait AllocationOwner: std::fmt::Debug + Send + Sync + 'static {}
impl<T: std::fmt::Debug + Send + Sync + 'static> AllocationOwner for T {}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit registrations of form-recognizing inner-solver capabilities.
use super::{CubicRoots, InnerSolver};
use crate::{MathError, guarded::PreparedBody, jets::EvaluationLimits};
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Recognize an admitted residual form and build a bounded solver for that form.
/// Iteration/root isolation belongs to the registered numerical library. Original
/// residual validation and implicit derivatives remain the common owner's work.
pub trait Accelerator: std::fmt::Debug + Send + Sync {
    /// Refuse an unsupported form, cancellation or a resource excess before returning.
    fn admit(
        &self,
        body: &PreparedBody,
        unknowns: usize,
        limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<dyn InnerSolver>, MathError>;
}
/// An immutable-at-use inventory, supplied by the composition root, never populated
/// from source strings. A source reference can select only an admitted registration.
#[derive(Clone, Debug, Default)]
pub struct Accelerators {
    entries: BTreeMap<String, Arc<dyn Accelerator>>,
}
impl Accelerators {
    /// The initial supported library bindings. Callers can instead supply an empty
    /// inventory or register independent capabilities without compiler changes.
    pub fn standard() -> Self {
        let cubic: Arc<dyn Accelerator> = Arc::new(Cubic);
        Self {
            entries: BTreeMap::from([("cubic_roots".into(), cubic)]),
        }
    }
    /// Add a unique, bounded reference; duplicates never replace existing authority.
    pub fn register(
        &mut self,
        reference: String,
        capability: Arc<dyn Accelerator>,
    ) -> Result<(), MathError> {
        if reference.is_empty() || reference.len() > 256 || self.entries.len() >= 64 {
            return Err(MathError::Limit("accelerator registration extent"));
        }
        if self.entries.contains_key(&reference) {
            return Err(MathError::Contract(
                "duplicate accelerator reference".into(),
            ));
        }
        self.entries.insert(reference, capability);
        Ok(())
    }
    /// Resolve the explicit reference and let its owner recognize the original body.
    pub fn admit(
        &self,
        reference: &str,
        body: &PreparedBody,
        unknowns: usize,
        limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<dyn InnerSolver>, MathError> {
        if cancel.load(Ordering::Acquire) {
            return Err(MathError::Cancelled);
        }
        limits.check()?;
        self.entries
            .get(reference)
            .ok_or_else(|| {
                MathError::Contract(format!("unregistered implicit accelerator: {reference}"))
            })?
            .admit(body, unknowns, limits, cancel)
    }
}
/// Symbolica-certified polynomial roots, with the existing cubic form admission.
#[derive(Debug)]
pub struct Cubic;
impl Accelerator for Cubic {
    fn admit(
        &self,
        body: &PreparedBody,
        unknowns: usize,
        limits: EvaluationLimits,
        cancel: &Arc<AtomicBool>,
    ) -> Result<Arc<dyn InnerSolver>, MathError> {
        if unknowns != 1 {
            return Err(MathError::Contract(
                "cubic accelerator requires one unknown".into(),
            ));
        }
        Ok(Arc::new(CubicRoots::new(body, limits, cancel)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{guarded::Stage, library};
    use symbolica::atom::{Atom, AtomCore};
    #[test]
    fn accelerators_use_only_registered_capabilities_and_recognized_forms() {
        crate::initialize().unwrap();
        let body = |degree| {
            PreparedBody::new(
                2,
                3,
                vec![2],
                vec![Stage::Block {
                    expressions: vec![
                        library::formal(0).unwrap().pow(Atom::num(degree))
                            - library::formal(1).unwrap(),
                    ],
                    outputs: vec![2],
                    source: pse_ids::SemanticId::NIL,
                }],
                pse_kernels::DerivativeOrder::Second,
            )
            .unwrap()
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let limits = EvaluationLimits::default();
        let mut registry = Accelerators::default();
        assert!(
            registry
                .admit("cubic_roots", &body(2), 1, limits, &cancel)
                .is_err()
        );
        registry
            .register("custom_reference".into(), Arc::new(Cubic))
            .unwrap();
        assert!(
            registry
                .register("custom_reference".into(), Arc::new(Cubic))
                .is_err()
        );
        assert!(
            registry
                .admit("custom_reference", &body(2), 1, limits, &cancel)
                .is_ok()
        );
        assert!(
            registry
                .admit("custom_reference", &body(4), 1, limits, &cancel)
                .is_err()
        );
        assert!(
            registry
                .admit("custom_reference", &body(2), 2, limits, &cancel)
                .is_err()
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            registry.admit("custom_reference", &body(2), 1, limits, &cancel),
            Err(MathError::Cancelled)
        ));
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fixed operation-local slots for neutral CaseScratch; no providers or numerical products survive.
use super::*;
use pse_math::assembly::{CaseAssembly, CaseScratch};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Exactly one passive slot for one actual case worker in a finite operation/chain.
pub(crate) struct NeutralCase {
    assembly: Arc<CaseAssembly>,
    pending: Mutex<Option<CaseScratch>>,
    owner_thread: Mutex<Option<std::thread::ThreadId>>,
    valid: Arc<AtomicBool>,
    _owner: Arc<dyn pse_math::AllocationOwner>,
    #[cfg(all(test, feature = "solver-ipopt"))]
    reuses: std::sync::atomic::AtomicUsize,
}
impl std::fmt::Debug for NeutralCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NeutralCase")
            .field("valid", &self.valid.load(Ordering::Acquire))
            .finish()
    }
}
impl NeutralCase {
    pub(crate) fn allowance(assembly: &CaseAssembly) -> Option<usize> {
        assembly
            .numeric_worker_bytes()
            .checked_mul(2)?
            .checked_add(2048)
    }
    pub(crate) fn new(
        assembly: Arc<CaseAssembly>,
        owner: Arc<dyn pse_math::AllocationOwner>,
        valid: Arc<AtomicBool>,
    ) -> Arc<Self> {
        Arc::new(Self {
            assembly,
            pending: Mutex::new(None),
            owner_thread: Mutex::new(None),
            valid,
            _owner: owner,
            #[cfg(all(test, feature = "solver-ipopt"))]
            reuses: std::sync::atomic::AtomicUsize::new(0),
        })
    }
    pub(crate) fn compatible(&self, assembly: &Arc<CaseAssembly>) -> bool {
        Arc::ptr_eq(assembly, &self.assembly)
    }
    pub(crate) fn discard(&self) {
        self.valid.store(false, Ordering::Release);
        if let Ok(mut pending) = self.pending.try_lock() {
            let old = pending.take();
            drop(pending);
            drop(old);
        }
    }
    #[cfg(all(test, feature = "solver-ipopt"))]
    pub(crate) fn reuses(&self) -> usize {
        self.reuses.load(Ordering::Acquire)
    }
    pub(crate) fn worker(
        self: &Arc<Self>,
        assembly: &Arc<CaseAssembly>,
        registrations: &BTreeMap<pse_kernels::ProviderKey, pse_kernels::Registration>,
        scope: pse_kernels::ExecutionScope,
    ) -> Result<CaseWorker, ProblemError> {
        scope.check().map_err(ProblemError::Provider)?;
        let compatible = Arc::ptr_eq(assembly, &self.assembly)
            && self.owner_thread.try_lock().is_ok_and(|mut owner| {
                let current = std::thread::current().id();
                if let Some(owner) = *owner {
                    owner == current
                } else {
                    *owner = Some(current);
                    true
                }
            });
        if !compatible {
            self.discard();
        }
        // Reset happens before fresh provider factories. A refused reset owns no source effects.
        let scratch = if self.valid.load(Ordering::Acquire) {
            self.pending
                .try_lock()
                .ok()
                .and_then(|mut pending| pending.take())
                .and_then(|scratch| scratch.compatible(assembly).then_some(scratch))
                .and_then(|scratch| scratch.reset().ok())
        } else {
            None
        };
        let reused = scratch.is_some();
        let providers = crate::math::attempt_providers(registrations, &scope)
            .map_err(ProblemError::Provider)?;
        let mut worker = match scratch {
            Some(scratch) => assembly.worker_from_scratch(scratch, providers, scope)?,
            None => assembly.worker_scoped(providers, scope),
        };
        worker.prepare_providers()?;
        if self.valid.load(Ordering::Acquire) {
            let slot = self.clone();
            worker.recycle_on_drop(move |scratch| {
                if slot.valid.load(Ordering::Acquire)
                    && let Ok(mut pending) = slot.pending.try_lock()
                    && pending.is_none()
                {
                    *pending = Some(scratch);
                }
            });
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count(
                if reused {
                    "fit.neutral_case_reuses"
                } else {
                    "fit.neutral_case_constructions"
                },
                "count",
                1,
            );
        }
        #[cfg(all(test, feature = "solver-ipopt"))]
        if reused {
            self.reuses.fetch_add(1, Ordering::Relaxed);
        }
        Ok(worker)
    }
}
/// The fixed function/guard population of one actual DynamicProgram. Frames, role
/// caches, snapshots, provider banks and native IVP workspaces are deliberately absent.
#[derive(Debug)]
pub(crate) struct DynamicScratch {
    functions: BTreeMap<(usize, Function), Arc<NeutralCase>>,
    guards: Vec<Option<Arc<NeutralCase>>>,
}
impl DynamicScratch {
    pub(crate) fn allowance(program: &DynamicProgram) -> Option<usize> {
        program
            .programs
            .iter()
            .try_fold(2048usize, |bytes, p| {
                bytes.checked_add(NeutralCase::allowance(&p.case.assembly)?)
            })?
            .checked_add(program.modes.iter().try_fold(0usize, |bytes, m| {
                bytes.checked_add(match &m.guard {
                    Some(guard) => NeutralCase::allowance(&guard.case.assembly)?,
                    None => 0,
                })
            })?)
    }
    pub(crate) fn new(
        program: &DynamicProgram,
        owner: Arc<dyn pse_math::AllocationOwner>,
        valid: Arc<AtomicBool>,
    ) -> Self {
        Self {
            functions: program
                .programs
                .iter()
                .map(|p| {
                    (
                        (p.mode, p.function),
                        NeutralCase::new(p.case.assembly.clone(), owner.clone(), valid.clone()),
                    )
                })
                .collect(),
            guards: program
                .modes
                .iter()
                .map(|mode| {
                    mode.guard.as_ref().map(|g| {
                        NeutralCase::new(g.case.assembly.clone(), owner.clone(), valid.clone())
                    })
                })
                .collect(),
        }
    }
    pub(crate) fn discard(&self) {
        for slot in self.functions.values().chain(self.guards.iter().flatten()) {
            slot.discard();
        }
    }
    pub(super) fn function(&self, mode: usize, function: Function) -> Option<&Arc<NeutralCase>> {
        self.functions.get(&(mode, function))
    }
    pub(super) fn guard(&self, mode: usize) -> Option<&Arc<NeutralCase>> {
        self.guards.get(mode).and_then(Option::as_ref)
    }
}

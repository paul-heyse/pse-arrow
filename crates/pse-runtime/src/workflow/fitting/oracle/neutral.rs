// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finite actual-role slots for one fit operation or one joined profile chain.
use super::*;
use crate::workflow::dynamics::{DynamicScratch, NeutralCase};
use std::sync::atomic::{AtomicBool, Ordering};

enum Slots {
    Steady(Arc<NeutralCase>),
    Transient(DynamicScratch),
}
/// No point, outer free/rank map, provider bank, execution or native state is stored.
pub(in crate::workflow::fitting) struct FitScratch {
    slots: BTreeMap<InstanceId, Slots>,
    valid: Arc<AtomicBool>,
}
impl std::fmt::Debug for FitScratch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FitScratch")
            .field("roles", &self.slots.len())
            .finish()
    }
}
impl FitScratch {
    /// Optional extra resident/reset overlap is reserved before any current provider is made.
    pub(in crate::workflow::fitting) fn new(
        p: &FitProblem,
        math: &Arc<MathService>,
    ) -> Option<Arc<Self>> {
        let bytes = p
            .experiments
            .iter()
            .try_fold(4096usize, |bytes, experiment| {
                bytes.checked_add(match experiment {
                    Experiment::Steady(s) => NeutralCase::allowance(&s.case.assembly)?,
                    Experiment::Transient(s) => DynamicScratch::allowance(&s.program)?,
                })
            })?;
        let owner = math.reserve("fit:neutral-case-slots", bytes).ok()?;
        let owner: Arc<dyn pse_math::AllocationOwner> = owner;
        let valid = Arc::new(AtomicBool::new(true));
        let slots = p
            .declaration
            .experiments
            .iter()
            .zip(&p.experiments)
            .map(|(role, experiment)| {
                (
                    role.experiment_id,
                    match experiment {
                        Experiment::Steady(s) => Slots::Steady(NeutralCase::new(
                            s.case.assembly.clone(),
                            owner.clone(),
                            valid.clone(),
                        )),
                        Experiment::Transient(s) => Slots::Transient(DynamicScratch::new(
                            &s.program,
                            owner.clone(),
                            valid.clone(),
                        )),
                    },
                )
            })
            .collect();
        Some(Arc::new(Self { slots, valid }))
    }
    pub(in crate::workflow::fitting) fn discard(&self) {
        self.valid.store(false, Ordering::Release);
        for slot in self.slots.values() {
            match slot {
                Slots::Steady(slot) => slot.discard(),
                Slots::Transient(slots) => slots.discard(),
            }
        }
    }
    pub(in crate::workflow::fitting) fn steady(
        &self,
        role: InstanceId,
        assembly: &Arc<pse_math::assembly::CaseAssembly>,
    ) -> Option<&Arc<NeutralCase>> {
        if !self.valid.load(Ordering::Acquire) {
            return None;
        }
        match self.slots.get(&role) {
            Some(Slots::Steady(slot)) if slot.compatible(assembly) => Some(slot),
            _ => None,
        }
    }
    #[cfg(feature = "solver-diffsol")]
    pub(super) fn transient(&self, role: InstanceId) -> Option<&DynamicScratch> {
        if !self.valid.load(Ordering::Acquire) {
            return None;
        }
        match self.slots.get(&role) {
            Some(Slots::Transient(slots)) => Some(slots),
            _ => None,
        }
    }
    #[cfg(all(test, feature = "solver-ipopt"))]
    pub(super) fn steady_reuses(&self) -> usize {
        self.slots
            .values()
            .map(|slot| match slot {
                Slots::Steady(slot) => slot.reuses(),
                _ => 0,
            })
            .sum()
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One selected PETSc handler, independent from all pre-existing handlers.
use super::*;
use pse_diagnostics::observation::{Producer, ProducerRole};
const FAMILY: &str = "petsc_event.v1";
const EVENTS: [&CStr; 13] = [
    c"SNESSolve", c"SNESFunctionEval", c"SNESJacobianEval", c"KSPSolve",
    c"KSPSetUp", c"PCSetUp", c"PCApply", c"SNESNPCSolve", c"SNESLineSearch",
    c"TSStep", c"TSFunctionEval", c"TSJacobianEval", c"TSPseudoCmptTStp",
];
/// Must be constructed and dropped inside the actual serialized PETSc admission.
/// No native reference escapes; checkpoints run only on that same owning thread.
pub(super) struct Log {
    raw: ffi::Handle,
    producer: Producer,
    execution: Execution,
}
impl Log {
    pub(super) fn selected(execution: &Execution) -> Option<Self> {
        let producer = pse_diagnostics::observation::current()?.producer(ProducerRole::NativeFactor, "petsc.events");
        if !producer.detail_selected(FAMILY) { return None; }
        let mut raw = ptr::null_mut();
        // SAFETY: caller retains process serialization and initialized PETSc.
        let code = unsafe { ffi::pse_petsc_log_create(&raw) };
        if code != 0 || raw.is_null() {
            producer.context().unavailable("petsc.logging", "selected handler unavailable or registration refused");
            return None;
        }
        Some(Self { raw, producer, execution: execution.clone() })
    }
    pub(super) fn checkpoint(&self, phase: &str, owner_returned: bool) {
        if !self.producer.detail_selected(FAMILY) { return; }
        if catch_unwind(AssertUnwindSafe(|| self.sample(phase, owner_returned))).is_err() {
            let _ = catch_unwind(AssertUnwindSafe(|| self.producer.context().unavailable(
                "petsc.logging", "selected event projection failed")));
        }
    }
    fn sample(&self, phase: &str, owner_returned: bool) {
        for event in EVENTS {
            let mut info = ffi::EventPerf::default();
            // SAFETY: independent live handler on its serialized native owner;
            // getter copies scalars and never invokes numerical work.
            let code = unsafe { ffi::pse_petsc_log_event(self.raw, event.as_ptr(), &mut info) };
            let measured = code == 0 && info.available != 0 && info.entered_calls >= 0
                && info.active_depth >= 0 && info.completed_seconds.is_finite()
                && info.completed_seconds >= 0. && info.completed_flops.is_finite()
                && info.completed_flops >= 0.;
            let unavailable = if code == 0 && info.registered != 0 && info.activated == 0 {
                UnavailableReason::NotRequested
            } else { UnavailableReason::NotComputed };
            let value = |value| if measured { Metric::Integer(value) } else { Metric::Unavailable(unavailable) };
            let real = |value| if measured { Metric::Real(value) } else { Metric::Unavailable(unavailable) };
            self.execution.progress.push_from(Event {
                source: Some(EventSource::new("petsc_event", None, None).with_role(ProducerRole::NativeFactor)),
                phase: phase.into(), elapsed: self.execution.started.elapsed(), incumbent: None,
                values: std::collections::BTreeMap::from([
                    ("backend".into(), Metric::Text("petsc".into())),
                    ("native_event".into(), Metric::Text(event.to_string_lossy().into_owned())),
                    ("scope".into(), Metric::Text("selected_handler_current_stage".into())),
                    ("count_semantics".into(), Metric::Text("outermost_native_event_entries".into())),
                    ("time_semantics".into(), Metric::Text("inclusive_completed_event_intervals".into())),
                    ("source_available".into(), Metric::Bool(measured)),
                    ("event_registered".into(), Metric::Bool(code == 0 && info.registered != 0)),
                    ("event_activated".into(), Metric::Bool(code == 0 && info.activated != 0)),
                    ("stage".into(), value(i64::from(info.stage))),
                    ("entered_calls".into(), value(i64::from(info.entered_calls))),
                    ("native_event_depth".into(), value(i64::from(info.active_depth))),
                    ("owner_returned".into(), Metric::Bool(owner_returned)),
                    ("completed_seconds".into(), real(info.completed_seconds)),
                    ("completed_flops".into(), real(info.completed_flops)),
                ]),
            }, &self.producer);
        }
    }
}
impl Drop for Log {
    fn drop(&mut self) {
        // SAFETY: lexical caller ownership drops this handler before releasing
        // PETSc admission. Stop/destroy touches only this handler, never a global toggle.
        let code = unsafe { ffi::pse_petsc_log_destroy(&mut self.raw) };
        if code != 0 {
            let _ = catch_unwind(AssertUnwindSafe(|| self.producer.context().unavailable(
                "petsc.logging.cleanup", "selected handler stop/destroy failed")));
        }
    }
}

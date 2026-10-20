// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One selected PETSc handler, independent from all pre-existing handlers.
use super::*;
use pse_diagnostics::observation::{ObservationContext, Producer, ProducerRole};
const FAMILY: &str = "petsc_event.v1";
const EVENTS: [&CStr; 13] = [
    c"SNESSolve",
    c"SNESFunctionEval",
    c"SNESJacobianEval",
    c"KSPSolve",
    c"KSPSetUp",
    c"PCSetUp",
    c"PCApply",
    c"SNESNPCSolve",
    c"SNESLineSearch",
    c"TSStep",
    c"TSFunctionEval",
    c"TSJacobianEval",
    c"TSPseudoCmptTStp",
];
/// Must be constructed and dropped inside the actual serialized PETSc admission.
/// No native reference escapes; checkpoints run only on that same owning thread.
pub(super) struct Log {
    raw: ffi::Handle,
    producers: [Producer; 13],
    context: ObservationContext,
    ordinal: Cell<Option<u64>>,
    execution: Execution,
}
impl Log {
    pub(super) fn selected(execution: &Execution) -> Option<Self> {
        let context = pse_diagnostics::observation::current()?;
        // Check the explicit selection before any producer/channel allocation.
        let selected = context.record_tap().is_some_and(|tap| {
            catch_unwind(AssertUnwindSafe(|| tap.selected(FAMILY))).unwrap_or(false)
        });
        if !selected {
            return None;
        }
        let mut raw = ptr::null_mut();
        // SAFETY: caller retains process serialization and initialized PETSc.
        let code = unsafe { ffi::pse_petsc_log_create(&mut raw) };
        if code != 0 || raw.is_null() {
            context.record_source_unavailable(FAMILY);
            context.unavailable(
                "petsc.logging",
                "selected handler unavailable or registration refused",
            );
            return None;
        }
        let producers = std::array::from_fn(|index| {
            let producer = context.producer(
                ProducerRole::NativeFactor,
                &format!("petsc.events.{}", EVENTS[index].to_string_lossy()),
            );
            producer.activate_record_family(FAMILY);
            producer
        });
        Some(Self {
            raw,
            producers,
            context,
            ordinal: Cell::new(Some(1)),
            execution: execution.clone(),
        })
    }
    pub(super) fn checkpoint(&self, phase: &str, owner_returned: bool) {
        if !self.producers[0].detail_selected(FAMILY) {
            return;
        }
        let ordinal = self.ordinal.get();
        self.ordinal
            .set(ordinal.and_then(|value| value.checked_add(1)));
        if catch_unwind(AssertUnwindSafe(|| {
            self.sample(phase, owner_returned, ordinal)
        }))
        .is_err()
        {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.context.record_source_unavailable(FAMILY);
                self.context
                    .unavailable("petsc.logging", "selected event projection failed");
            }));
        }
    }
    fn sample(&self, phase: &str, owner_returned: bool, ordinal: Option<u64>) {
        for (event, producer) in EVENTS.into_iter().zip(&self.producers) {
            if !producer.enabled() {
                self.context.record_source_unavailable(FAMILY);
                continue;
            }
            let mut info = ffi::EventPerf::default();
            // SAFETY: independent live handler on its serialized native owner;
            // getter copies scalars and never invokes numerical work.
            let code = unsafe { ffi::pse_petsc_log_event(self.raw, event.as_ptr(), &mut info) };
            let measured = code == 0
                && info.available != 0
                && info.entered_calls >= 0
                && info.active_depth >= 0
                && info.completed_seconds.is_finite()
                && info.completed_seconds >= 0.
                && info.completed_flops.is_finite()
                && info.completed_flops >= 0.;
            let unavailable = if code == 0 && info.registered != 0 && info.activated == 0 {
                UnavailableReason::NotRequested
            } else {
                UnavailableReason::NotComputed
            };
            let flag = |value| {
                if code == 0 {
                    Metric::Bool(value)
                } else {
                    Metric::Unavailable(UnavailableReason::Unknown)
                }
            };
            let value = |value| {
                if measured {
                    Metric::Integer(value)
                } else {
                    Metric::Unavailable(unavailable)
                }
            };
            let real = |value| {
                if measured {
                    Metric::Real(value)
                } else {
                    Metric::Unavailable(unavailable)
                }
            };
            self.execution.progress.push_from(
                Event {
                    source: Some(
                        EventSource::new("petsc_event", None, None)
                            .with_role(ProducerRole::NativeFactor),
                    ),
                    phase: phase.into(),
                    elapsed: self.execution.started.elapsed(),
                    incumbent: None,
                    values: std::collections::BTreeMap::from([
                        ("backend".into(), Metric::Text("petsc".into())),
                        (
                            "checkpoint_ordinal".into(),
                            ordinal.and_then(|value| i64::try_from(value).ok()).map_or(
                                Metric::Unavailable(UnavailableReason::Unknown),
                                Metric::Integer,
                            ),
                        ),
                        (
                            "native_event".into(),
                            Metric::Text(event.to_string_lossy().into_owned()),
                        ),
                        (
                            "scope".into(),
                            Metric::Text("selected_handler_current_stage".into()),
                        ),
                        (
                            "count_semantics".into(),
                            Metric::Text("outermost_native_event_entries".into()),
                        ),
                        (
                            "time_semantics".into(),
                            Metric::Text("inclusive_completed_event_intervals".into()),
                        ),
                        ("source_available".into(), Metric::Bool(measured)),
                        ("event_registered".into(), flag(info.registered != 0)),
                        ("event_activated".into(), flag(info.activated != 0)),
                        ("stage".into(), value(i64::from(info.stage))),
                        ("entered_calls".into(), value(i64::from(info.entered_calls))),
                        (
                            "native_event_depth".into(),
                            value(i64::from(info.active_depth)),
                        ),
                        ("owner_returned".into(), Metric::Bool(owner_returned)),
                        ("completed_seconds".into(), real(info.completed_seconds)),
                        ("completed_flops".into(), real(info.completed_flops)),
                    ]),
                },
                producer,
            );
        }
    }
}
impl Drop for Log {
    fn drop(&mut self) {
        // SAFETY: lexical caller ownership drops this handler before releasing
        // PETSc admission. Stop/destroy touches only this handler, never a global toggle.
        let code = unsafe { ffi::pse_petsc_log_destroy(&mut self.raw) };
        if code != 0 {
            let _ = catch_unwind(AssertUnwindSafe(|| {
                self.context.unavailable(
                    "petsc.logging.cleanup",
                    "selected handler stop/destroy failed",
                )
            }));
        }
    }
}

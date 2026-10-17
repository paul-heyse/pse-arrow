// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Copies only work already requested by the implicit scientific owner.

pub(super) fn observe<T, E: miette::Diagnostic>(
    phase: &'static str,
    calls: &'static str,
    elapsed: &'static str,
    failures: &'static str,
    completed: &'static str,
    operation: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    let context = pse_diagnostics::observation::current().filter(|context| context.is_active());
    let _phase = context.as_ref().map(|context| context.phase(phase));
    let started = context.as_ref().map(|_| std::time::Instant::now());
    if let Some(context) = &context {
        context.count(calls, "calls", 1);
    }
    let result = operation();
    if let Some(context) = &context {
        context.text("implicit.last_stage", phase);
        context.boolean("implicit.stage.failed", result.is_err());
        if let Err(error) = &result {
            context.count(failures, "calls", 1);
            if let Some(code) = miette::Diagnostic::code(error) {
                context.text("implicit.last_failure_code", &code.to_string());
            }
        } else {
            context.count(completed, "calls", 1);
        }
        if let Some(started) = started {
            context.duration(elapsed, started.elapsed());
        }
    }
    result
}

/// Coarse IFT stages share the already-selected observation owner and callback flag.
/// Cancellation is checked around opaque library work; no unperformed stage completes.
pub(super) fn stage<T>(
    phase: &'static str,
    calls: &'static str,
    elapsed: &'static str,
    failures: &'static str,
    completed: &'static str,
    cancel: &std::sync::atomic::AtomicBool,
    operation: impl FnOnce() -> Result<T, crate::MathError>,
) -> Result<T, crate::MathError> {
    observe(phase, calls, elapsed, failures, completed, || {
        let check = || {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) {
                Err(crate::MathError::Cancelled)
            } else {
                Ok(())
            }
        };
        check()?;
        let value = operation()?;
        check()?;
        Ok(value)
    })
}

pub(super) trait ProofStatus {
    fn status(&self) -> &'static str;
}
fn refusal(reason: super::SelectionProofRefusal) -> &'static str {
    match reason {
        super::SelectionProofRefusal::Unsupported => "unsupported",
        super::SelectionProofRefusal::Chart => "chart-unestablished",
        super::SelectionProofRefusal::Coverage => "coverage-unestablished",
        super::SelectionProofRefusal::Resource => "resource-refused",
        super::SelectionProofRefusal::Boundary => "boundary-unestablished",
    }
}
impl ProofStatus for super::SelectionEvidence {
    fn status(&self) -> &'static str {
        match self {
            Self::Unique(_) => "unique",
            Self::Multiple => "multiple",
            Self::Incomplete(reason) => refusal(*reason),
        }
    }
}
impl ProofStatus for super::ChartChainEvidence {
    fn status(&self) -> &'static str {
        match self {
            Self::Connected(_) => "connected",
            Self::Incomplete(reason) | Self::Refused { reason, .. } => refusal(*reason),
            Self::Interrupted(_) => "interrupted",
        }
    }
}
macro_rules! enclosure_status {
    ($type:ty) => {
        impl ProofStatus for $type {
            fn status(&self) -> &'static str {
                match self {
                    Self::Enclosed { .. } => "enclosed",
                    Self::Incomplete { reason, .. } => refusal(*reason),
                    Self::Interrupted { .. } => "interrupted",
                }
            }
        }
    };
}
enclosure_status!(super::RootPointEvidence);
enclosure_status!(super::RootActionEvidence);
enclosure_status!(super::RootNeighborhoodEvidence);
impl ProofStatus for bool {
    fn status(&self) -> &'static str {
        if *self {
            "connected"
        } else {
            "connection-unestablished"
        }
    }
}
pub(super) fn observe_proof<T: ProofStatus>(
    phase: &'static str,
    calls: &'static str,
    elapsed: &'static str,
    failures: &'static str,
    completed: &'static str,
    operation: impl FnOnce() -> Result<T, crate::MathError>,
) -> Result<T, crate::MathError> {
    let result = observe(phase, calls, elapsed, failures, completed, operation);
    if let Some(context) = pse_diagnostics::observation::current() {
        context.text("implicit.proof.last_kind", phase);
        context.text(
            "implicit.proof.last_status",
            result.as_ref().map_or("failed", ProofStatus::status),
        );
    }
    result
}

pub(super) fn request(request: &super::SelectionProofRequest<'_>) {
    if let Some(context) = pse_diagnostics::observation::current() {
        context.source_ref("implicit.proof.selection", request.selection.as_bytes());
        context.integer(
            "implicit.proof.order",
            "derivative-order",
            request.order as i64,
        );
        context.integer(
            "implicit.proof.alternatives",
            "alternatives",
            request.alternatives.len() as i64,
        );
        context.integer(
            "implicit.proof.winner",
            "alternative-index",
            request.winner as i64,
        );
        context.integer(
            "implicit.proof.unknowns",
            "physical-coordinates",
            request.candidate.len() as i64,
        );
        context.gauge(
            "implicit.proof.remaining_allowance",
            "seconds",
            request.time_limit.as_secs_f64(),
        );
        if let Some(alternative) = request.alternatives.get(request.winner) {
            context.source_ref(
                "implicit.proof.original_residual",
                alternative.residual_identity.as_bytes(),
            );
        }
    }
}

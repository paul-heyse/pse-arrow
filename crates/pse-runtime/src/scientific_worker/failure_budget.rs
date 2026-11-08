// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounds on restoring a scientific worker failure after structural decoding.
use pse_kernels::worker_protocol::Failure;
use pse_model::HeapUsage as _;

pub(super) const FAILURE_DEPTH: usize = 32;
pub(super) const FAILURE_NODES: usize = 1024;
pub(super) const FAILURE_RETAINED_TEXT: usize = 64 << 10;

/// Semantic restoration has its own finite tree/retention allowance, after
/// the generated codec's independent frame, nesting and allocation limits.
pub(super) fn failure_within_limits(failure: &Failure) -> bool {
    let mut pending = vec![(failure, 0usize)];
    let mut nodes = FAILURE_NODES;
    let mut text = FAILURE_RETAINED_TEXT;
    while let Some((failure, depth)) = pending.pop() {
        if depth > FAILURE_DEPTH || nodes == 0 {
            return false;
        }
        nodes -= 1;
        let bytes = match failure {
            Failure::ProviderRetirement { failures } => {
                if failures.is_empty() || failures.len() > 64 {
                    return false;
                }
                pending.extend(
                    failures
                        .iter()
                        .map(|failure| (failure.cause.as_ref(), depth + 1)),
                );
                0
            }
            Failure::ProviderSettlement { operation, cleanup } => {
                pending.push((operation, depth + 1));
                pending.push((cleanup, depth + 1));
                0
            }
            Failure::Retirement { causes } => {
                if causes.is_empty() || causes.len() > 128 {
                    return false;
                }
                pending.extend(causes.iter().map(|cause| (cause, depth + 1)));
                0
            }
            Failure::Diagnostic(diagnostic) => {
                let mut observed = vec![(diagnostic.as_ref(), depth + 1)];
                while let Some((diagnostic, depth)) = observed.pop() {
                    if depth > FAILURE_DEPTH || nodes == 0 {
                        return false;
                    }
                    nodes -= 1;
                    if diagnostic.causes.len() > 128 {
                        return false;
                    }
                    observed.extend(diagnostic.causes.iter().map(|cause| (cause, depth + 1)));
                }
                diagnostic.heap_bytes()
            }
            Failure::Resource(value)
            | Failure::Contract(value)
            | Failure::Trial(value)
            | Failure::Terminal(value)
            | Failure::UnknownCompletion(value)
            | Failure::ProviderRetirementPanic(value)
            | Failure::NativePanic(value)
            | Failure::Singular(value) => value.len(),
            Failure::OutsideEnvelope { axis, .. } => axis.len(),
            Failure::Cancelled
            | Failure::Deadline
            | Failure::LinearizationChanged
            | Failure::RegimeCrossing { .. } => 0,
        };
        let Some(remaining) = text.checked_sub(bytes) else {
            return false;
        };
        text = remaining;
    }
    true
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrow coordinate leaves from generated complete values for numerical algorithms.
//! No rows, alternate subject objects or detached inventories are constructed.
macro_rules! member {
    ($subject:expr, $selection:ident, $value:ident) => {{
        let subject = $subject;
        subject
            .species
            .as_ref()
            .and_then(|arm| arm.member.$selection.as_ref())
            .map(|value| value.$value)
            .or_else(|| {
                subject
                    .element
                    .as_ref()
                    .and_then(|arm| arm.member.$selection.as_ref())
                    .map(|value| value.$value)
            })
            .or_else(|| {
                subject
                    .phase_species
                    .as_ref()
                    .and_then(|arm| arm.member.$selection.as_ref())
                    .map(|value| value.$value)
            })
    }};
}
macro_rules! phase {
    ($subject:expr, $selection:ident, $value:ident) => {{
        let subject = $subject;
        subject
            .total
            .as_ref()
            .and_then(|arm| arm.phase.as_ref())
            .and_then(|coordinate| coordinate.$selection.as_ref())
            .map(|value| value.$value)
            .or_else(|| {
                subject
                    .energy
                    .as_ref()
                    .and_then(|arm| arm.phase.as_ref())
                    .and_then(|coordinate| coordinate.$selection.as_ref())
                    .map(|value| value.$value)
            })
            .or_else(|| {
                subject
                    .momentum
                    .as_ref()
                    .and_then(|arm| arm.phase.as_ref())
                    .and_then(|coordinate| coordinate.$selection.as_ref())
                    .map(|value| value.$value)
            })
            .or_else(|| {
                subject
                    .phase_species
                    .as_ref()
                    .and_then(|arm| arm.phase.$selection.as_ref())
                    .map(|value| value.$value)
            })
    }};
}
macro_rules! law_phase {
    ($subject:expr, $selection:ident, $value:ident) => {{
        let subject = $subject;
        $crate::passes::physical_subject::phase!(subject, $selection, $value)
            .or_else(|| {
                subject
                    .species
                    .as_ref()
                    .and_then(|arm| arm.phase.as_ref())
                    .and_then(|coordinate| coordinate.$selection.as_ref())
                    .map(|value| value.$value)
            })
            .or_else(|| {
                subject
                    .element
                    .as_ref()
                    .and_then(|arm| arm.phase.as_ref())
                    .and_then(|coordinate| coordinate.$selection.as_ref())
                    .map(|value| value.$value)
            })
    }};
}
pub(super) use {law_phase, member, phase};

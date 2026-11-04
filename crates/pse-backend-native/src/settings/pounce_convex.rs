// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The POUNCE-convex settings document (Plan 22 N5): the interior-point method's typed
//! choices and the complete native FERAL configuration. Its stopping tolerance comes from
//! the resolved accuracy and its iteration budget from the shared controls; nothing here
//! restates them.

include!("pounce_convex/declarations.rs");
impl Default for Settings {
    /// The library's own defaults: the self-dual embedding, equilibration, no crossover and
    /// FERAL's defaults.
    fn default() -> Self {
        Self {
            self_dual: true,
            equilibrate: true,
            crossover: false,
            linear: super::pounce::LinearSettings::default(),
        }
    }
}

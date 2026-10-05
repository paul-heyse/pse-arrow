// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library composition controls and separately selected M22 native acceptance.
// The acceptance journeys instantiate the catalog publication futures, which prove `Send`
// through deep async nesting: the crate needs pse-catalog's own recursion limit.
#![recursion_limit = "256"]
#[cfg(all(test, feature = "native-profiles"))]
mod native_profiles;

#[cfg(all(test, feature = "native-acceptance"))]
mod acceptance;

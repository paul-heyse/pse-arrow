// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library composition controls and separately selected M22 native acceptance.
// Native acceptance journeys prove `Send` through deeply nested workflow futures.
#![recursion_limit = "256"]
#[cfg(all(test, feature = "native-profiles"))]
mod native_profiles;

#[cfg(all(test, feature = "native-acceptance"))]
mod acceptance;

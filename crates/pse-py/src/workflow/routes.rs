// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Mechanical conversion of retained native decisions into generated documents.
use crate::documents::DocumentValue;
use pse_backend_native::routing::{Eligibility, Route};
pub(super) type NativeRoute = DocumentValue<pse_runtime::workflow::RouteDocument>;
pub(super) type NativeEligibility = DocumentValue<pse_runtime::workflow::EligibilityDocument>;
impl From<Route> for NativeRoute {
    fn from(route: Route) -> Self {
        Self(route.into())
    }
}
impl From<&Eligibility> for NativeEligibility {
    fn from(eligibility: &Eligibility) -> Self {
        Self(eligibility.into())
    }
}

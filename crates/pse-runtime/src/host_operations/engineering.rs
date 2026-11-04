// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit local engineering service; scientific policy stays with native workflow owners.

mod session;
pub use session::{
    EngineeringPhysicalOutput, EngineeringResultOutput, EngineeringService, EngineeringServiceError,
};

/// Source-generated wire messages, constraints and independent client/service bindings.
#[cfg(feature = "engineering-service")]
pub mod generated;

#[cfg(feature = "engineering-service")]
mod rpc;
#[cfg(feature = "engineering-service")]
pub use rpc::{EngineeringClient, EngineeringEndpoint, LocalAccessPolicy, serve};

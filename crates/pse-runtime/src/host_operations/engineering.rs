// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure engineering contracts and the explicitly activated local service.
//! Scientific policy stays with native workflow owners.

#[cfg(feature = "engineering-service")]
mod readiness;
#[cfg(feature = "engineering-service")]
mod session;
#[cfg(feature = "engineering-service")]
pub use readiness::{EngineeringFixtureReady, EngineeringFrontendReady};
#[cfg(feature = "engineering-service")]
pub use session::{
    EngineeringModelingOutput, EngineeringPhysicalOutput, EngineeringResultOutput,
    EngineeringService, EngineeringServiceError, EngineeringStudyOutput,
};

/// Complete encoded ceiling for fixed contract, workbench and prepared-study documents.
#[cfg(feature = "engineering-service")]
const DOCUMENT_RESPONSE_BYTES: usize = 16 << 20;

/// Source-generated wire messages, constraints and independent client/service bindings.
pub mod generated;

#[cfg(feature = "engineering-service")]
mod rpc;
#[cfg(feature = "engineering-service")]
pub use rpc::{EngineeringClient, EngineeringEndpoint, LocalAccessPolicy, serve};

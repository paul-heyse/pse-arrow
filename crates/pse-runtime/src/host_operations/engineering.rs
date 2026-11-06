// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit local engineering service; scientific policy stays with native workflow owners.

mod readiness;
mod session;
pub use readiness::{EngineeringFixtureReady, EngineeringFrontendReady};
pub use session::{
    EngineeringModelingOutput, EngineeringPhysicalOutput, EngineeringResultOutput,
    EngineeringService, EngineeringServiceError, EngineeringStudyOutput,
};

/// Complete encoded ceiling for fixed contract, workbench and prepared-study documents.
const DOCUMENT_RESPONSE_BYTES: usize = 16 << 20;

/// Source-generated wire messages, constraints and independent client/service bindings.
#[cfg(feature = "engineering-service")]
pub mod generated;

#[cfg(feature = "engineering-service")]
mod rpc;
#[cfg(feature = "engineering-service")]
pub use rpc::{EngineeringClient, EngineeringEndpoint, LocalAccessPolicy, serve};

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
    EngineeringModelingOutput, EngineeringPhysicalOutput, EngineeringProviderPreparation,
    EngineeringResultOutput, EngineeringService, EngineeringServiceError, EngineeringStudyOutput,
};

/// Complete encoded ceiling for fixed contract, workbench and prepared-study documents.
#[cfg(feature = "engineering-service")]
use pse_host::engineering_client::DOCUMENT_RESPONSE_BYTES;

/// Source-generated wire messages, constraints and independent client/service bindings.
#[cfg(feature = "engineering-service")]
pub mod generated;

/// Finite model-inspection messages and reflection, available without the service.
#[cfg(feature = "engineering-inspection")]
pub mod inspection;

// Own conversions between registry source types and host-owned wire types.
#[cfg(feature = "engineering-inspection")]
pub(crate) struct NativeAdapterScope;

#[cfg(feature = "engineering-service")]
mod rpc;
#[cfg(feature = "engineering-service")]
pub use rpc::{EngineeringClient, EngineeringEndpoint, LocalAccessPolicy, serve};

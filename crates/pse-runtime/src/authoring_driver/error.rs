// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native ingestion failures preserve their original classified cause.
/// Classified failures at the effectful document and relation boundary.
#[derive(Debug, thiserror::Error)]
pub enum AuthoringDriverError {
    /// Pure authoring syntax, identity, budget or reference failure.
    #[error(transparent)]
    Authoring(#[from] pse_authoring::AuthoringError),
    /// Native planning or execution failure.
    #[error(transparent)]
    Catalog(#[from] pse_engine::EngineError),
    /// Shared native allocation admission failure.
    #[error(transparent)]
    Resource(#[from] pse_columnar::ReserveError),
    /// Cancellation or checked allocation extent.
    #[error(transparent)]
    Allocation(#[from] pse_columnar::CanonError),
    /// Generated Arrow contract admission.
    #[error(transparent)]
    Relation(#[from] pse_relations::RelationError),
}
pse_diagnostics::impl_diagnostic! {
    AuthoringDriverError,
    code(_this) { None },
    forward(this) { match this {
        Self::Authoring(value)=>Some(value), Self::Catalog(value)=>Some(value),
        Self::Resource(value)=>Some(value), Self::Allocation(value)=>Some(value),
        Self::Relation(value)=>Some(value),
    } },
    help(_this) { None }, related(_this) { None }, source(_this) { None }
}
impl From<datafusion::common::DataFusionError> for AuthoringDriverError {
    fn from(error: datafusion::common::DataFusionError) -> Self {
        Self::Catalog(error.into())
    }
}
pse_columnar::impl_native_error!(AuthoringDriverError);

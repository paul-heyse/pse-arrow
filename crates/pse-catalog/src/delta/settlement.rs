// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared effect versus observation failures; domain receipts remain exact request proofs.
use datafusion::common::{DataFusionError, Result};
/// A native mutation could have committed even when outcome delivery failed.
#[derive(Debug, thiserror::Error)]
pub enum SettlementError {
    /// Inspect the native Delta log before retrying an uncertain command.
    #[error("Delta mutation outcome is unresolved: {source}")]
    Unresolved {
        /// Original operation or outcome observation error.
        source: deltalake::DeltaTableError,
    },
    /// A maintenance fence committed; destructive work may be partially complete.
    #[error("Delta maintenance after fence {fence_version} is incomplete: {source}")]
    MaintenanceInterrupted {
        /// Known durable fence, not a claim that maintenance completed.
        fence_version: u64,
        /// Original operation or result observation error.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// The native commit completed; only result observation failed.
    #[error("Delta mutation committed version {version}; subsequent work failed: {source}")]
    Committed {
        /// Actual committed Delta version.
        version: u64,
        /// Result observation error.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

pub(crate) fn unresolved(source: deltalake::DeltaTableError) -> DataFusionError {
    uncertain(DataFusionError::External(Box::new(
        SettlementError::Unresolved { source },
    )))
}
pub(super) fn committed(
    version: u64,
    error: impl std::error::Error + Send + Sync + 'static,
) -> DataFusionError {
    uncertain(DataFusionError::External(Box::new(
        SettlementError::Committed {
            version,
            source: Box::new(error),
        },
    )))
}

pub(super) fn maintenance_interrupted(
    fence_version: u64,
    source: DataFusionError,
) -> DataFusionError {
    uncertain(DataFusionError::External(Box::new(
        SettlementError::MaintenanceInterrupted {
            fence_version,
            source: Box::new(source),
        },
    )))
}

/// Keep both the operation failure and a failure to observe its effects.
pub(super) fn recovery_failed(
    primary: std::sync::Arc<deltalake::DeltaTableError>,
    observation: DataFusionError,
) -> DataFusionError {
    uncertain(DataFusionError::Collection(vec![
        DataFusionError::External(Box::new(primary)),
        observation,
    ]))
}

fn uncertain(source: DataFusionError) -> DataFusionError {
    pse_engine::operation::FailureKind::UncertainEffects.error(source)
}

/// Preserve a known native version when receipt or metrics observation fails.
pub(super) fn observed<T>(version: Option<u64>, result: Result<T>) -> Result<T> {
    result.map_err(|error| match version {
        Some(version) => committed(version, error),
        None => error,
    })
}
pse_diagnostics::impl_diagnostic! {
    SettlementError,
    code(this) { match this {
            Self::Unresolved { .. } | Self::Committed { .. } | Self::MaintenanceInterrupted { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),

            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

/// A successful native builder must report the exact admitted transition.
pub(super) fn expected_version(actual: Option<u64>, expected: u64) -> Result<()> {
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(DataFusionError::Execution(
            "native commit returned an unexpected version".into(),
        ))
    }
}

#[cfg(test)]
mod delta_boundary_unit {
    use super::*;
    use std::error::Error;
    #[test]
    fn observation_failure_preserves_native_source_and_the_known_commit() {
        let error = observed::<()>(
            Some(9),
            Err(DataFusionError::Execution("lost metrics".into())),
        )
        .unwrap_err();
        let observations = pse_columnar::observe(&error, pse_columnar::PlanOrigin::Analytics);
        assert_eq!(observations.len(), 1);
        assert_eq!(
            observations[0].code,
            pse_diagnostics::DiagnosticCode::RuntimeInfrastructure
        );
        let source: &dyn Error = observations[0].domain_cause.unwrap();
        let completion = source
            .downcast_ref::<pse_engine::operation::CompletionFailure>()
            .unwrap();
        assert_eq!(
            completion.kind,
            pse_engine::operation::FailureKind::UncertainEffects
        );
        let DataFusionError::External(source) = &completion.source else {
            panic!("settlement cause must survive")
        };
        let settlement = source.downcast_ref::<SettlementError>().unwrap();
        assert!(matches!(
            settlement,
            SettlementError::Committed { version: 9, .. }
        ));
        assert!(
            settlement
                .source()
                .unwrap()
                .downcast_ref::<DataFusionError>()
                .is_some()
        );
        assert_eq!(observed(None, Ok(0)).unwrap(), 0);
    }
}

pse_columnar::impl_native_error!(SettlementError);

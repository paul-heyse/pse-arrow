// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact receipt facts shared only by owners with an immutable request witness.
//! This operation supplies no clock, retry, fence or scientific permission.

use super::CanonicalError;

/// Preserve both failed observations and the original immutable recovery intent.
pub(crate) fn unresolved(
    owner: &'static str,
    operation: &str,
    request: &[u8],
    source: CanonicalError,
    readback: Option<CanonicalError>,
) -> CanonicalError {
    CanonicalError::SettlementUnresolved {
        owner,
        operation: operation.to_owned(),
        request: request.to_vec(),
        source: Box::new(source),
        readback: readback.map(Box::new),
    }
}

/// Read once in the caller's existing task and clock after a failed write/ack.
/// Only the owner may supply a refusal/absence proof for its actual transaction.
pub(crate) async fn settle_exact<T>(
    owner: &'static str,
    operation: &str,
    request: &[u8],
    write: Result<T, CanonicalError>,
    readback: impl Future<Output = Result<Option<T>, CanonicalError>>,
    matches_request: impl Fn(&T) -> bool,
    absence_proven: impl FnOnce(&CanonicalError) -> bool,
) -> Result<T, CanonicalError> {
    let source = match write {
        Ok(receipt) => {
            return if matches_request(&receipt) {
                Ok(receipt)
            } else {
                Err(CanonicalError::OperationReused)
            };
        }
        Err(source) => source,
    };
    match readback.await {
        Ok(Some(receipt)) if matches_request(&receipt) => Ok(receipt),
        Ok(Some(_)) => Err(CanonicalError::OperationReused),
        Ok(None) if absence_proven(&source) => Err(source),
        Ok(None) => Err(unresolved(owner, operation, request, source, None)),
        // An owner can authenticate reuse while decoding its immutable
        // operation receipt. That is a settled identity fact, not failed IO.
        Err(CanonicalError::OperationReused) => Err(CanonicalError::OperationReused),
        Err(readback) => Err(unresolved(
            owner,
            operation,
            request,
            source,
            Some(readback),
        )),
    }
}

#[cfg(test)]
mod canonical_settlement_unit {
    #![allow(
        clippy::panic,
        reason = "a polled readback after exact acknowledgement is a test failure"
    )]
    use super::*;

    async fn settle(
        write: Result<Vec<u8>, CanonicalError>,
        readback: Result<Option<Vec<u8>>, CanonicalError>,
    ) -> Result<Vec<u8>, CanonicalError> {
        settle_exact(
            "research-dataset",
            "original-key",
            b"original-request",
            write,
            async { readback },
            |receipt| receipt == b"original-request",
            CanonicalError::source_transaction_refused,
        )
        .await
    }

    #[tokio::test]
    async fn exact_acknowledgement_does_not_poll_readback_or_refusal_policy() {
        let result = settle_exact(
            "project",
            "original-key",
            b"request",
            Ok(b"request".to_vec()),
            async { panic!("acknowledged write must not dispatch readback") },
            |receipt| receipt == b"request",
            |_| panic!("no absence policy needed"),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn exact_readback_settles_original_lost_acknowledgement() {
        assert!(
            settle(
                Err(CanonicalError::Timeout),
                Ok(Some(b"original-request".to_vec()))
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn differing_immutable_request_is_reuse_for_ack_and_readback() {
        assert!(matches!(
            settle(Ok(b"different".to_vec()), Ok(None)).await,
            Err(CanonicalError::OperationReused)
        ));
        assert!(matches!(
            settle(
                Err(CanonicalError::Timeout),
                Ok(Some(b"different".to_vec()))
            )
            .await,
            Err(CanonicalError::OperationReused)
        ));
        assert!(matches!(
            settle(
                Err(CanonicalError::Timeout),
                Err(CanonicalError::OperationReused)
            )
            .await,
            Err(CanonicalError::OperationReused)
        ));
    }

    #[tokio::test]
    async fn missing_receipt_never_proves_timeout_rollback() {
        let error = settle(Err(CanonicalError::Timeout), Ok(None)).await.err();
        assert!(
            matches!(error, Some(CanonicalError::SettlementUnresolved { owner: "research-dataset", operation, request, source, readback: None })
            if operation == "original-key" && request == b"original-request" && matches!(*source, CanonicalError::Timeout))
        );
    }

    #[tokio::test]
    async fn owner_refusal_and_absent_receipt_retain_original_refusal() {
        let error = CanonicalError::ConflictExhausted {
            operation: "original-transaction",
            attempts: 32,
        };
        assert!(matches!(
            settle(Err(error), Ok(None)).await,
            Err(CanonicalError::ConflictExhausted {
                operation: "original-transaction",
                attempts: 32
            })
        ));
    }

    #[tokio::test]
    async fn shared_operation_cannot_promote_statement_refusal_without_owner_proof() {
        let error = settle_exact::<Vec<u8>>(
            "staged-owner",
            "original-key",
            b"request",
            Err(CanonicalError::ConflictExhausted {
                operation: "a-statement",
                attempts: 32,
            }),
            async { Ok(None) },
            |_| true,
            |_| false,
        )
        .await
        .err();
        assert!(matches!(
            error,
            Some(CanonicalError::SettlementUnresolved { readback: None, .. })
        ));
    }

    #[tokio::test]
    async fn failed_readback_retains_both_causes_and_intent_even_after_refusal() {
        for source in [
            CanonicalError::Timeout,
            CanonicalError::ConflictExhausted {
                operation: "original-transaction",
                attempts: 32,
            },
        ] {
            let error = settle(Err(source), Err(CanonicalError::ConnectionUnavailable))
                .await
                .err();
            assert!(
                matches!(error, Some(CanonicalError::SettlementUnresolved { owner: "research-dataset", operation, request, source, readback: Some(readback) })
                if operation == "original-key" && request == b"original-request"
                    && matches!(*source, CanonicalError::Timeout | CanonicalError::ConflictExhausted { .. })
                    && matches!(*readback, CanonicalError::ConnectionUnavailable))
            );
        }
    }

    #[tokio::test]
    async fn settlement_keeps_existing_task_local_deadline() {
        use super::super::transport::{current_deadline, within_clock};
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1);
        let result = within_clock(
            deadline,
            settle_exact(
                "project",
                "original-key",
                b"request",
                Err(CanonicalError::Timeout),
                async {
                    assert_eq!(current_deadline(), Some(deadline));
                    Ok(Some(b"request".to_vec()))
                },
                |receipt| receipt == b"request",
                |_| false,
            ),
        )
        .await;
        assert!(result.is_ok());
    }

    #[test]
    fn nested_causes_keep_infrastructure_and_uncertainty_classification() {
        let error = unresolved(
            "project",
            "original-key",
            b"request",
            CanonicalError::OperationReused,
            Some(CanonicalError::Timeout),
        );
        assert!(error.is_infrastructure());
        assert!(error.effect_is_unknown());
        // A nested fresh-premise or retryable cause cannot authorize replay of
        // an unresolved enclosing operation.
        assert!(!error.needs_fresh_premise());
        assert!(!error.retryable());
        let wrapped = CanonicalError::SourceEditUnsettled {
            problem: "problem".into(),
            operation: "original-key".into(),
            source: Box::new(error),
        };
        assert!(wrapped.is_infrastructure());
        assert!(wrapped.effect_is_unknown());
    }
}

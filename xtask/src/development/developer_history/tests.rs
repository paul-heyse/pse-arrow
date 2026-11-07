// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_operations::{
    canonical::CanonicalOptions,
    native_fixture::FixtureSelection,
    transport::client::{IndexedResults, Root},
};
use std::{
    sync::atomic::Ordering,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn fact(created_unix_ms: i64) -> HistoryFact {
    HistoryFact {
        invocation: uuid::Uuid::new_v4().to_string(),
        repository_id: "1".repeat(64),
        checkout_id: "2".repeat(64),
        control_manifest_sha256: "3".repeat(64),
        revision: None,
        created_unix_ms,
        interpretation: 3,
        outcome: "failed".into(),
        report_sha256: "4".repeat(64),
        inventory_sha256: "5".repeat(64),
        settlement_sha256: "6".repeat(64),
        selected_count: 2,
        omitted_evidence: Some(1),
        failure_codes: vec!["runtime.infrastructure".into()],
        failed: vec![FailedCase {
            package: "pse-host".into(),
            target_kind: "lib".into(),
            target_name: "pse_host".into(),
            enabled_features: vec!["feature_a".into()],
            absent_features: vec!["feature_b".into()],
            binary_id: "pse-host".into(),
            test_name: "workflow::fails".into(),
        }],
        profile: "default".into(),
        platform: None,
        filters: HistoryFilters {
            ignore_default_filter: false,
            words: vec![],
            expressions: vec!["test(workflow::fails)".into()],
        },
        ignored_policy: "default".into(),
    }
}

#[test]
fn history_publication_bounds_and_original_commitment() {
    let fact = fact(1);
    let first = HistoryPublication::new(fact.clone(), 100).unwrap();
    assert_eq!(first, HistoryPublication::new(fact.clone(), 100).unwrap());
    assert_ne!(
        first.content_sha256,
        HistoryPublication::new(fact.clone(), 101)
            .unwrap()
            .content_sha256
    );
    assert!(HistoryPublication::new(fact.clone(), 1).is_err());
    let mut changed = first.clone();
    changed.fact.failed[0].test_name = "workflow::other".into();
    assert!(changed.verify(&fact.invocation).is_err());
    let mut duplicate = fact.clone();
    duplicate.failed.push(duplicate.failed[0].clone());
    assert!(HistoryPublication::new(duplicate, 100).is_err());
    let mut oversized = fact;
    oversized.filters.words = vec!["a".repeat(MAX_STRING_BYTES); MAX_FILTERS];
    assert!(matches!(
        HistoryPublication::new(oversized, 100),
        Err(HistoryError::Invalid("encoded projection exceeds the pilot bound"))
    ));
}

impl HistoryStore {
    async fn fault_ack(
        &self,
        reader: &Self,
        acknowledgement: Result<IndexedResults, CanonicalError>,
    ) -> Result<IndexedResults, CanonicalError> {
        let fault = self.ack_fault.swap(0, Ordering::SeqCst);
        if fault != 0 && acknowledgement.is_ok() {
            if fault == 2 {
                reader.disconnect().await.unwrap();
            }
            return Err(CanonicalError::IncompleteResponse);
        }
        acknowledgement
    }
}

async fn install(
    options: &HistoryOptions,
    service: &CanonicalOptions,
    deadline: Instant,
) -> Result<(), HistoryError> {
    let scope = Scope::from_options(options)?;
    let client = Client::connect(scope.endpoint, RESPONSE_LIMITS.wire_bytes).await?;
    let context = || RequestContext::control(deadline);
    let result = async {
        transport::request(
            client
                .signin(Root {
                    username: service.username.clone(),
                    password: service.password.clone(),
                })
                .request_context(context()),
        )
        .await?;
        transport::bounded_query(
            client
                .query(format!("DEFINE NAMESPACE {};", scope.namespace))
                .request_context(context()),
        )
        .await?;
        transport::request(client.use_ns(&scope.namespace).request_context(context())).await?;
        transport::bounded_query(
            client
                .query(format!("DEFINE DATABASE {};", scope.database))
                .request_context(context()),
        )
        .await?;
        transport::request(
            client
                .use_ns(&scope.namespace)
                .use_db(&scope.database)
                .request_context(context()),
        )
        .await?;
        // Both credentials are generated hex values for this original disposable fixture.
        let ddl = format!(
            "BEGIN; {} CREATE developer_history_deployment:current CONTENT $marker; \
             DEFINE USER {} ON DATABASE PASSWORD '{}' ROLES VIEWER; \
             DEFINE USER {} ON DATABASE PASSWORD '{}' ROLES EDITOR; COMMIT;",
            schema(),
            options.reader.username,
            options.reader.password,
            options.writer.username,
            options.writer.password,
        );
        transport::bounded_query(
            client
                .query(ddl)
                .bind(("marker", Deployment {
                    issuer: scope.issuer,
                    schema_sha256: scope.schema_sha256,
                }))
                .request_context(context()),
        )
        .await?;
        Ok::<_, HistoryError>(())
    }
    .await;
    let drain = client.disconnect().await;
    result?;
    drain?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires the explicitly selected disposable native evidence fixture"]
async fn history_native_roles_replay_conflict_and_unknown_ack() {
    let fixture = FixtureSelection::selected().unwrap();
    let service = CanonicalOptions::from_state(fixture.state()).unwrap();
    let credential = |name: &str| HistoryCredential {
        username: name.into(),
        password: uuid::Uuid::new_v4().simple().to_string(),
    };
    let mut options = HistoryOptions {
        endpoint: service.endpoint.strip_prefix("grpc://").unwrap().parse().unwrap(),
        namespace: format!("pse_developer_history_{}", uuid::Uuid::new_v4().simple()),
        database: "pilot".into(),
        issuer: uuid::Uuid::new_v4().to_string(),
        reader: credential("history_reader"),
        writer: credential("history_writer"),
    };
    let deadline = Instant::now() + Duration::from_secs(60);
    install(&options, &service, deadline).await.unwrap();
    let writer = HistoryStore::open(&options, HistoryRole::Writer, deadline).await.unwrap();
    let reader = HistoryStore::open(&options, HistoryRole::Reader, deadline).await.unwrap();
    let now = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap();
    let expiry = now + 60_000;
    let first = HistoryPublication::new(fact(now), expiry).unwrap();
    let id = &first.fact.invocation;

    assert_eq!(reader.read(id, now, deadline).await.unwrap(), HistoryRead::Missing);
    let _ = transport::bounded_query(
        reader.query("CREATE ONLY type::record('developer_test_history', $invocation) CONTENT $row;")
            .unwrap()
            .bind(("invocation", id.as_str()))
            .bind(("row", first.clone())),
    )
    .await;
    assert_eq!(writer.read(id, now, deadline).await.unwrap(), HistoryRead::Missing);
    assert!(matches!(
        reader.publish(&reader, &first, deadline).await,
        Err(HistoryError::WrongDeployment)
    ));
    assert_eq!(writer.publish(&reader, &first, deadline).await.unwrap(), Published::Created);
    assert_eq!(reader.read(id, now, deadline).await.unwrap(), HistoryRead::Present(Box::new(first.clone())));
    assert_eq!(writer.publish(&reader, &first, deadline).await.unwrap(), Published::Existing);

    let mut changed = first.fact.clone();
    changed.report_sha256 = "a".repeat(64);
    let conflict = HistoryPublication::new(changed, expiry).unwrap();
    assert!(matches!(writer.publish(&reader, &conflict, deadline).await, Err(HistoryError::ReusedIdentity)));
    assert_eq!(reader.read(id, now, deadline).await.unwrap(), HistoryRead::Present(Box::new(first.clone())));
    let candidates = reader.preview_expired(expiry, 1, deadline).await.unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(&candidates[0].invocation, id);
    assert_eq!(candidates[0].content_sha256, first.content_sha256);
    assert_eq!(reader.read(id, expiry, deadline).await.unwrap(), HistoryRead::Expired(candidates.into_iter().next().unwrap()));

    let second = HistoryPublication::new(fact(now), expiry).unwrap();
    writer.ack_fault.store(1, Ordering::SeqCst);
    assert_eq!(writer.publish(&reader, &second, deadline).await.unwrap(), Published::Reconciled);
    let third = HistoryPublication::new(fact(now), expiry).unwrap();
    writer.ack_fault.store(2, Ordering::SeqCst);
    assert!(matches!(
        writer.publish(&reader, &third, deadline).await,
        Err(HistoryError::Unsettled { invocation, readback: Some(_), .. }) if invocation == third.fact.invocation
    ));
    let replacement = HistoryStore::open(&options, HistoryRole::Reader, deadline).await.unwrap();
    assert_eq!(writer.publish(&replacement, &third, deadline).await.unwrap(), Published::Existing);
    let competitor = HistoryStore::open(&options, HistoryRole::Writer, deadline).await.unwrap();
    let mut left = fact(now);
    let mut right = left.clone();
    left.report_sha256 = "c".repeat(64);
    right.report_sha256 = "d".repeat(64);
    let left = HistoryPublication::new(left, expiry).unwrap();
    let right = HistoryPublication::new(right, expiry).unwrap();
    let (left_result, right_result) = tokio::join!(
        writer.publish(&replacement, &left, deadline),
        competitor.publish(&replacement, &right, deadline),
    );
    let winner = match (left_result, right_result) {
        (Ok(Published::Created), Err(HistoryError::ReusedIdentity)) => left,
        (Err(HistoryError::ReusedIdentity), Ok(Published::Created)) => right,
        other => panic!("expected one created and one exact-identity refusal: {other:?}"),
    };
    assert_eq!(
        replacement.read(&winner.fact.invocation, now, deadline).await.unwrap(),
        HistoryRead::Present(Box::new(winner)),
    );
    competitor.disconnect().await.unwrap();
    assert!(transport::bounded_query(
        writer.query(&format!(
            "USE NS `{}` DB `{}`; CREATE cross_scope_probe:forbidden;",
            service.namespace, service.database
        ))
        .unwrap()
    ).await.is_err());
    options.issuer = uuid::Uuid::new_v4().to_string();
    assert!(matches!(HistoryStore::open(&options, HistoryRole::Reader, deadline).await, Err(HistoryError::WrongDeployment)));
    writer.disconnect().await.unwrap();
    replacement.disconnect().await.unwrap();
}

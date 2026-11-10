// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fresh design-phase reconstruction retains both physical owners and resets authority.
use crate::{canonical::study_module::StudyModuleReceipt, native_service::NativeCanonicalService};
use std::{
    fs,
    time::{Duration, Instant},
};

#[tokio::test]
async fn fresh_reconstruction_rejects_nested_target_and_keeps_original_until_explicit_disposal() {
    let spec = super::owned_test_service_spec();
    let mut source = NativeCanonicalService::new(spec.clone()).unwrap();
    let source_path = source.state().to_path_buf();
    let work_end = Instant::now() + Duration::from_secs(240);
    let settlement_end = work_end + Duration::from_secs(60);
    let (issuer,module)=source.run_until(work_end,settlement_end,|source|Box::pin(async move {
        source.launch(work_end).await?;
        source.fixture_query("CREATE canonical_guards:discard_fixture SET key='discard_fixture',generation=17u64;",surrealdb_types::Object::new()).await?;
        Ok((source.ready()?.fence().issuer().to_owned(),StudyModuleReceipt::read(source.state())?.sha256.clone()))
    })).await.unwrap();
    let stopped = source.try_into_stopped().unwrap();
    let intent = stopped
        .empty_reconstruction_intent(uuid::Uuid::new_v4(), module)
        .unwrap();
    let mut nested = spec.clone();
    nested.state_parent = source_path.clone();
    let failure = stopped
        .prepare_empty_reconstruction(intent.clone(), nested, work_end)
        .unwrap_err();
    let (stopped, target, returned, _) = failure.into_parts();
    assert!(
        target.is_none(),
        "nested placement must refuse before constructing a replacement"
    );
    assert_eq!(returned, intent);
    assert!(source_path.is_dir());

    let wrong = stopped
        .empty_reconstruction_intent(uuid::Uuid::new_v4(), "0".repeat(64))
        .unwrap();
    let failure = stopped
        .prepare_empty_reconstruction(wrong, spec.clone(), work_end)
        .unwrap_err();
    let (stopped, target, _, _) = failure.into_parts();
    assert!(
        target.is_none(),
        "wrong module must refuse before constructing a replacement"
    );
    let mut controller = stopped
        .prepare_empty_reconstruction(intent.clone(), spec.clone(), work_end)
        .unwrap();
    let (entered, _resume) = controller.pause_after_launch_for_test();
    {
        let launch = controller.launch(work_end);
        tokio::pin!(launch);
        tokio::select! {
            result=&mut launch=>panic!("controlled launch completed before cancellation: {result:?}"),
            ()=entered.notified()=>{},
        }
        // Dropping the actual launch future after dispatch cannot drop the controller.
    }
    assert_eq!(controller.original(), source_path);
    let failure = controller.into_ready().unwrap_err();
    let mut controller = failure.into_controller().unwrap();
    controller.reconcile(work_end).await.unwrap();
    let mut replacement = controller.into_ready().unwrap();
    let original_target_path = replacement.target().state().to_path_buf();
    let original_target_invocation = replacement.target().intent().unit().to_owned();
    let failure = replacement.suppress_ack_for_test();
    let mut controller = failure.into_controller().unwrap();
    controller.reconcile(work_end).await.unwrap();
    let mut replacement = controller.into_ready().unwrap();
    assert_eq!(replacement.target().state(), original_target_path);
    assert_eq!(
        replacement.target().intent().unit(),
        original_target_invocation
    );
    assert_eq!(replacement.intent(), &intent);
    assert_eq!(replacement.original(), source_path);
    let target = replacement.target();
    assert_ne!(target.state(), source_path);
    assert_ne!(target.ready().unwrap().fence().issuer(), issuer);
    assert!(
        fs::read(source_path.join("credentials.json")).unwrap()
            != fs::read(target.state().join("credentials.json")).unwrap(),
        "fresh target must mint its own credentials"
    );
    let mut observed = target
        .fixture_query(
            "SELECT * FROM ONLY canonical_guards:discard_fixture;",
            surrealdb_types::Object::new(),
        )
        .await
        .unwrap();
    assert!(
        observed
            .take::<Option<surrealdb_types::Object>>(0)
            .unwrap()
            .is_none(),
        "fresh target must not copy disposable old rows"
    );
    assert!(
        source_path.is_dir(),
        "validation does not erase the stopped original"
    );
    let (original, mut target) = replacement.into_parts();
    let replay = original
        .prepare_empty_reconstruction(intent.clone(), spec, work_end)
        .unwrap_err();
    let (mut original, duplicate, _, _) = replay.into_parts();
    assert!(
        duplicate.is_none(),
        "replaying the recorded operation must not create another target"
    );
    let operation = uuid::Uuid::parse_str(intent.operation()).unwrap();
    let origin = target.reconstruction_origin().unwrap().unwrap();
    assert_eq!(origin.operation, intent.operation());
    assert_eq!(
        original.reconstruction_status(operation).unwrap(),
        Some(origin.clone())
    );
    original.dispose().unwrap();
    assert!(!source_path.exists());
    assert_eq!(
        target.reconstruction_origin().unwrap(),
        Some(origin.clone())
    );
    target.shutdown(settlement_end).await.unwrap();
    let mut target = target.try_into_stopped().unwrap().reopen().unwrap();
    target
        .run_until(work_end, settlement_end, |target| {
            Box::pin(async move {
                target.launch(work_end).await?;
                assert_eq!(target.reconstruction_origin()?, Some(origin));
                Ok(())
            })
        })
        .await
        .unwrap();
    target.dispose().unwrap();
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use crate::observation::{Diagnostics, read_checkpoint};
use pse_diagnostics::observation::{Outcome, Policy};
mod canonical;
mod follow;
mod investigation;
mod publication;

fn request(correlation: &str, request: Request) -> Vec<u8> {
    serde_json::to_vec(&ReaderRequest {
        protocol: Protocol::ProductReader,
        version: Version,
        correlation: correlation.into(),
        access: Access::Passive,
        request,
    })
    .unwrap()
}

#[test]
fn canonical_refusal_redacts_the_native_message_and_ordinary_causes() {
    let secret = "original-private-canonical-password";
    let policy =
        pse_host::envelope::RedactionPolicy::new([("CANONICAL_PASSWORD".into(), secret.into())]);
    let error = ReaderError::Canonical(Box::new(
        pse_operations::canonical::CanonicalError::SettlementUnresolved {
            owner: "project",
            operation: "original-operation".into(),
            request: Vec::new(),
            source: Box::new(pse_operations::canonical::CanonicalError::Configuration(
                secret.into(),
            )),
            readback: None,
        },
    ));
    let encoded = encode_response(response_with_policy(
        "canonical-refusal".into(),
        Err(error),
        &policy,
    ))
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let failure = &value["payload"]["failure"];
    assert_eq!(failure["code"], "request.refused");
    assert_eq!(failure["message"], failure["native"]["message"]);
    assert_eq!(failure["causes"], failure["native"]["causes"]);
    assert!(
        !std::str::from_utf8(encoded.bytes())
            .unwrap()
            .contains(secret)
    );
    assert!(
        failure
            .to_string()
            .contains("<redacted:CANONICAL_PASSWORD>")
    );
}

#[test]
fn managed_launch_candidate_is_transport_only_and_mismatch_never_relabels_selected_process() {
    use pse_diagnostics::process_facts::{ManagedInvocationCandidate, ProcessRead, ProcessReader};
    let directory = tempfile::tempdir().unwrap();
    let owner = Diagnostics::new(Policy::default(), directory.path().to_owned());
    let operation = owner.hub().start("original selected process");
    owner.publish().unwrap();
    let mut image = read_checkpoint(&owner.checkpoint_path()).unwrap();
    let ProcessRead::Present(facts) = ProcessReader::default().process(std::process::id()) else {
        panic!("fixture publisher has actual kernel identity");
    };
    let candidate = ManagedInvocationCandidate {
        invocation: "original-host-routing-candidate".into(),
        process: facts.identity,
    };
    image.managed_invocation = Some(candidate.clone());
    let selected = Request::Inspect {
        source: CheckpointSelection::Retained {
            checkpoint: owner.checkpoint_path(),
            expected: None,
        },
        request: InspectionRequest {
            version: Version,
            operation: operation.context().id(),
            window: std::time::Duration::from_secs(30),
            include_related: false,
            related_limit: 1,
            numerical_history_policy: Default::default(),
        },
        invocation: None,
    };
    for mismatched in [false, true] {
        if mismatched {
            image
                .managed_invocation
                .as_mut()
                .unwrap()
                .process
                .start_ticks += 1;
        }
        std::fs::write(owner.checkpoint_path(), serde_json::to_vec(&image).unwrap()).unwrap();
        let encoded = encode(
            &request("observer", selected.clone()),
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap();
        assert_eq!(encoded.exit_code(), 0);
        let value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
        assert_eq!(value["payload"]["kind"], "inspection");
        assert!(
            value["payload"]["process"]
                .get("managed_invocation")
                .is_none()
        );
        if mismatched {
            assert!(value["managed_invocation"].is_null());
        } else {
            assert_eq!(
                value["managed_invocation"],
                serde_json::to_value(&candidate).unwrap()
            );
        }
    }
}

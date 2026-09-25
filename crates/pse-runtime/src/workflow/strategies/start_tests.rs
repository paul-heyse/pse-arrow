// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::tests::profile;
use super::*;
use crate::math::solves::Outcome;
use crate::workflow::{
    ModelBuilder, RunReport,
    tests::{compiler_profile, declaration, id, physical, runtime},
};
use std::collections::BTreeMap;

#[tokio::test]
async fn multi_root_choice_is_explained_by_seed_policy_independently_of_allocation() {
    let mut d = declaration();
    d.cases[0].variables[0].fixed = false;
    d.cases[0].values[0].value = 1.0;
    let mut second = d.cases[0].clone();
    second.case_id = id(6);
    second.values[0].value = -1.0;
    d.cases.push(second);
    let runtime = runtime();
    let revision = ModelBuilder::from_declaration(runtime.clone(), d, physical())
        .freeze()
        .unwrap();
    let cancel = crate::CancelSource::new();
    let positive = revision
        .prepare(
            id(5),
            profile(SolveIntent::Root),
            compiler_profile(),
            &cancel,
        )
        .await
        .unwrap();
    let negative = revision
        .prepare(
            id(6),
            profile(SolveIntent::Root),
            compiler_profile(),
            &cancel,
        )
        .await
        .unwrap();
    let cold = negative.start().unwrap().wait().await.unwrap();
    let RunReport::Solves(cold) = cold.report().unwrap() else {
        panic!()
    };
    let Outcome::Native(cold) = &cold.outcomes[0] else {
        panic!()
    };
    assert!((cold.candidate.as_ref().unwrap().primal[0] + 2.0).abs() < 1e-6);
    assert!(!cold.start_receipt.as_ref().unwrap().submitted);
    for reuse in [ReusePolicy::Fresh, ReusePolicy::AllowRebuild] {
        for policy in [StartPolicy::NoPriorStart, StartPolicy::PreviousAccepted] {
            let mut profile = profile(SolveIntent::Root);
            profile.controls.start = policy;
            profile.controls.reuse = reuse;
            let next = revision
                .prepare(id(6), profile, compiler_profile(), &cancel)
                .await
                .unwrap();
            let result = runtime
                .start(vec![positive.clone(), next], false)
                .unwrap()
                .wait()
                .await
                .unwrap();
            let RunReport::Solves(report) = result.report().unwrap() else {
                panic!()
            };
            let Outcome::Native(second) = &report.outcomes[1] else {
                panic!("{:?}", report.outcomes)
            };
            let receipt = second.start_receipt.as_ref().unwrap();
            let expected = if policy == StartPolicy::PreviousAccepted {
                2.0
            } else {
                -2.0
            };
            assert!((second.candidate.as_ref().unwrap().primal[0] - expected).abs() < 1e-6);
            assert_eq!(receipt.submitted, policy == StartPolicy::PreviousAccepted);
            assert_eq!(
                receipt.previous_attempt,
                (policy == StartPolicy::PreviousAccepted).then_some(0)
            );
            if let Some(seed) = &receipt.seed {
                assert_eq!(seed.origin.as_ref().unwrap().attempt, 0);
            }
            let output = second.warm_start.as_ref().unwrap().origin.as_ref().unwrap();
            assert_eq!(output.run, Some(result.run_id));
            assert_eq!(output.attempt, 1);
        }
    }
    let explicit = negative
        .with_primal_start(BTreeMap::from([(id(1), 1.0)]))
        .unwrap();
    let result = explicit.start().unwrap().wait().await.unwrap();
    let RunReport::Solves(report) = result.report().unwrap() else {
        panic!()
    };
    let Outcome::Native(report) = &report.outcomes[0] else {
        panic!()
    };
    assert!((report.candidate.as_ref().unwrap().primal[0] - 2.0).abs() < 1e-6);
    assert!(report.start_receipt.as_ref().unwrap().submitted);
}

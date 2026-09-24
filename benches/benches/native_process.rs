// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete cold preparation and warm source edit through joined solve, result access and teardown.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "qualification workloads fail on incomplete or invalid execution"
)]
#[path = "../../tests/support/plan14.rs"]
mod fixture;
use criterion::{Criterion, criterion_group, criterion_main};
use fixture::*;
use pse_backend_native::solve::Backend;
use std::{num::NonZeroUsize, path::PathBuf};
fn process(c: &mut Criterion) {
    let id = std::env::var("PSE_PLAN14_COST_CASE")
        .expect("guarded measurement recipe supplies exact case");
    let parts: Vec<_> = id.split('-').collect();
    assert_eq!(parts.len(), 3);
    let blocks = match parts[1] {
        "small" => 1,
        "medium" => 8,
        "large" => 32,
        _ => panic!("unknown shape"),
    };
    let threads = match parts[2] {
        "1" => 1,
        "4" => 4,
        _ => panic!("unknown thread profile"),
    };
    let warm = match parts[0] {
        "cold" => false,
        "warm" => true,
        _ => panic!("unknown reuse profile"),
    };
    let output = PathBuf::from(std::env::var("PSE_PLAN14_COST_OUTPUT").unwrap());
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let case = sid(&json("bindings.json")["root_case"]);
    let mut retained = if warm {
        let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
        let mut b = executor.block_on(builder(&owner));
        resize(&mut b, blocks);
        let revision = b.freeze().unwrap();
        success(
            executor
                .block_on(solve(&revision, case, Backend::Ipopt, false))
                .as_ref(),
        );
        Some((owner, revision))
    } else {
        None
    };
    let mut peak = 0;
    let mut rss = 0;
    let mut iterations = 0;
    let mut group = c.benchmark_group("process");
    group.sample_size(10);
    group.bench_function(&id, |b| {
        b.iter(|| {
            if let Some((owner, revision)) = &mut retained {
                owner.runtime.reset_observation_peak();
                let mut edit = revision.edit();
                let source = edit
                    .declaration_mut()
                    .cases
                    .iter_mut()
                    .find(|c| c.case_id == case)
                    .unwrap();
                // Value-only source change exercises incremental preparation, including physical data.
                source.values[0].value += 1e-9;
                *revision = edit.freeze().unwrap();
                let result = executor.block_on(solve(revision, case, Backend::Ipopt, false));
                success(&result);
                std::hint::black_box(result.table("runtime.solve_variables").unwrap());
                drop(result);
                peak = peak.max(owner.runtime.observation_peak_bytes());
                rss = rss.max(
                    owner
                        .runtime
                        .report()
                        .unwrap()
                        .process_peak_rss_bytes
                        .unwrap(),
                );
            } else {
                let owner =
                    WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
                let mut draft = executor.block_on(builder(&owner));
                resize(&mut draft, blocks);
                let revision = draft.freeze().unwrap();
                let result = executor.block_on(solve(&revision, case, Backend::Ipopt, false));
                success(&result);
                std::hint::black_box(result.table("runtime.solve_variables").unwrap());
                drop(result);
                drop(revision);
                peak = peak.max(owner.runtime.observation_peak_bytes());
                rss = rss.max(
                    owner
                        .runtime
                        .report()
                        .unwrap()
                        .process_peak_rss_bytes
                        .unwrap(),
                );
                drop(owner);
            }
            iterations += 1;
        })
    });
    group.finish();
    drop(retained);
    drop(executor);
    std::fs::write(output.join(format!("{id}-memory.json")),serde_json::to_vec_pretty(&serde_json::json!({
        "id":id,"iterations":iterations,"pool_peak_bytes":peak,"process_peak_rss_bytes":rss,
        "blocks":blocks,"variables":3*blocks,"threads":threads,"native_threads":1,
        "scope":"source preparation/edit, compile, joined native solve, source-space quality, Arrow result access and attempt teardown",
        "memory_scope":"pool observation per operation; process lifetime VmHWM from the dedicated benchmark process"
    })).unwrap()).unwrap();
}
fn configuration() -> Criterion {
    Criterion::default().output_directory(
        &PathBuf::from(std::env::var("PSE_PLAN14_COST_OUTPUT").unwrap()).join("criterion"),
    )
}
criterion_group! {name=benches;config=configuration();targets=process}
criterion_main!(benches);

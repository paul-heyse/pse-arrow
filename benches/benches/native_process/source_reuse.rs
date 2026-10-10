// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete unchanged authored-byte copy, parse and canonical declaration edit.
use super::{k4_support as support, *};
use serde_json::{Value, json};

pub(super) fn measure(
    c: &mut Criterion,
    spec: &Value,
    output: &std::path::Path,
    phases: &phases::Phases,
) {
    let name = spec["id"].as_str().unwrap();
    let blocks = usize::try_from(spec["blocks"].as_u64().unwrap()).unwrap();
    let threads = usize::try_from(spec["threads"].as_u64().unwrap()).unwrap();
    assert_eq!(spec["reuse"], "unchanged-bytes");
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .build()
        .unwrap();
    let owner = WorkflowRuntime::with_threads(NonZeroUsize::new(threads).unwrap()).unwrap();
    let runtime = runtime(&owner);
    let mut source = String::from("package unchanged {");
    for block in 0..blocks {
        source.push_str(&format!(
            "def Unit{block} {{ var x:Scalar; eq balance:x==2; }}"
        ));
    }
    source.push('}');
    let authored = source.into_bytes();
    let document = pse_ids::named_id(pse_ids::SemanticId::NIL, "unchanged-authored-benchmark");
    let parse = |bytes: &[u8]| {
        pse_authoring::language::parse(
            std::str::from_utf8(bytes).unwrap(),
            document,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    };
    let original = parse(&authored);
    // Canonical declaration reads use identity order; authored sibling order
    // remains explicit in each row's parent/ordinal fields.
    let mut expected = original.clone();
    expected.sort_by_key(|row| row.declaration_id);
    let physical = executor.block_on(support::physical(
        &runtime,
        &owner,
        &support::sources(support::SOURCE),
    ));
    // This ingress deliberately owns typed declarations, so an unchanged edit
    // has no document-text removal or first-edit normalization hidden in it.
    let package = executor
        .block_on(runtime.modeling_package(original.clone(), physical))
        .unwrap();
    let revision = package.canonical_revision().clone();
    owner.runtime.reset_observation_peak();
    phases.reset();
    let mut iterations = 0_u64;
    let mut group = c.benchmark_group("process");
    group
        .sample_size(10)
        .sampling_mode(criterion::SamplingMode::Flat)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_secs(1));
    group.bench_function(name, |b| {
        b.iter_custom(|count| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..count {
                let started = Instant::now();
                let copied = authored.clone();
                let rows = parse(&copied);
                let edited = executor.block_on(package.with_declarations(rows)).unwrap();
                elapsed += started.elapsed();
                assert_eq!(copied, authored);
                assert_eq!(edited.canonical_revision(), &revision);
                let actual = executor.block_on(edited.declarations()).unwrap();
                assert_eq!(actual.len(), expected.len());
                assert!(
                    actual
                        .iter()
                        .zip(&expected)
                        .all(|(actual, expected)| actual == expected),
                    "complete canonical declarations changed after unchanged authored edit"
                );
                iterations += 1;
            }
            elapsed
        })
    });
    group.finish();
    let pool = owner.runtime.pool();
    let peak = owner.runtime.observation_peak_bytes();
    let rss = owner.runtime.report().unwrap().process_peak_rss_bytes;
    let retained = pool.reserved();
    drop(package);
    drop(runtime);
    executor.block_on(owner.cleanup_fixtures()).unwrap();
    drop(owner);
    assert_eq!(pool.reserved(), 0);
    let summary = json!({"id":name,"workload":spec,"iterations":iterations,
        "threads":threads,"native_threads":1,"variables_observed":[blocks],
        "authored_bytes":authored.len(),"declarations":original.len(),
        "pool_peak_bytes":peak,"process_peak_rss_bytes":rss,"retained_runtime_bytes":retained,
        "after_retained_runtime_teardown_bytes":0,"compiler_phases":phases.report(iterations),
        "timed_scope":"copy exact authored UTF-8 bytes, parse with stable named document identity, complete canonical declaration edit",
        "untimed_scope":"physical and initial declaration admission, exact byte/declaration/revision checks, fixture teardown",
        "scope":"unchanged declaration ingress; not document reimport, scientific compilation or a solver speedup",
        "oracle":"exact authored bytes and complete declarations in canonical identity order, including authored parent/ordinal fields; unchanged canonical revision and sequence"});
    std::fs::write(
        output.join(format!("{name}-memory.json")),
        serde_json::to_vec_pretty(&summary).unwrap(),
    )
    .unwrap();
}

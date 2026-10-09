// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

// Disposable test module, included inside typed_math_portable::tests with the
// temporary compiler persistence-probe feature. No production persistence path.
use super::*;
use salsa::{Database, Setter};
use std::sync::atomic::AtomicUsize;
use std::time::Instant;

static PERSISTED_EXECUTIONS: AtomicUsize = AtomicUsize::new(0);
static CONTROL_EXECUTIONS: AtomicUsize = AtomicUsize::new(0);

#[salsa::db]
#[derive(Clone, Default)]
struct RecipeProbeDb {
    storage: salsa::Storage<Self>,
}
#[salsa::db]
impl Database for RecipeProbeDb {}

#[salsa::input(persist, singleton)]
struct RecipeProbeInput {
    #[returns(ref)]
    payload: Vec<u8>,
    #[returns(ref)]
    interpretation: String,
}

fn checked_recipe(db: &dyn Database, input: RecipeProbeInput) -> Recipe {
    let recipe: Recipe = serde_json::from_slice(input.payload(db)).unwrap();
    // An actual tracked interpretation leaf participates even when the changed
    // interpretation does not change the decoded record in this feasibility test.
    assert!(!input.interpretation(db).is_empty());
    assert_eq!(recipe.interpretation, INTERPRETATION);
    assert!(recipe.semantic_identity.is_some());
    assert!(recipe.checked_members.is_some());
    recipe
}

#[salsa::tracked(returns(clone), persist)]
fn persisted_recipe(db: &dyn Database, input: RecipeProbeInput) -> Recipe {
    PERSISTED_EXECUTIONS.fetch_add(1, Ordering::SeqCst);
    checked_recipe(db, input)
}

#[salsa::tracked(returns(clone))]
fn control_recipe(db: &dyn Database, input: RecipeProbeInput) -> Recipe {
    CONTROL_EXECUTIONS.fetch_add(1, Ordering::SeqCst);
    checked_recipe(db, input)
}

#[salsa::input]
struct EvictionInput {
    #[returns(ref)]
    payload: Vec<u8>,
}

#[salsa::tracked(returns(clone), lru = 2)]
fn evicted_recipe(db: &dyn Database, input: EvictionInput) -> Recipe {
    CONTROL_EXECUTIONS.fetch_add(1, Ordering::SeqCst);
    serde_json::from_slice(input.payload(db)).unwrap()
}

fn snapshot(db: &mut RecipeProbeDb) -> Vec<u8> {
    serde_json::to_vec(&<dyn Database>::as_serialize(db)).unwrap()
}

fn restore_disposable(bytes: &[u8]) -> Result<RecipeProbeDb, serde_json::Error> {
    // Do not mutate or return a live compiler workspace on failed restore.
    let mut db = RecipeProbeDb::default();
    <dyn Database>::deserialize(&mut db, &mut serde_json::Deserializer::from_slice(bytes))?;
    Ok(db)
}

#[test]
fn real_recipe_persistence_feasibility() {
    let source = r#"package p {
        entity kind convention {} entity convention ideal {}
        coordinate map coords(x:Length) {slot x=x/1{m};}
        reconstruction family for coords(x:Length)->Length reference ideal=1{m};
        fn law(x:Coordinate<coords.x>)->Reduced<family>=0*x;
        fn potential(x:Length)->Length=reconstruct(family,law,x);
        response response_value from potential(x:Length,potential:Fn(x:Length)->Length)->Length=potential(x);
        def Root {var x:Length;eq residual:response_value(x,potential)==0{m};}
    }"#;
    let (registry, body) = authored(source);
    let payload = body.portable_payload().unwrap();
    let expected: Recipe = serde_json::from_slice(&payload).unwrap();
    let wire: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    assert!(!wire["proofs"]["entries"].as_array().unwrap().is_empty());
    let changed_source = source.replace("=0*x", "=1*x");
    let (_, changed_body) = authored(&changed_source);
    let changed_payload = changed_body.portable_payload().unwrap();
    let changed_expected: Recipe = serde_json::from_slice(&changed_payload).unwrap();
    assert_ne!(expected, changed_expected);

    PERSISTED_EXECUTIONS.store(0, Ordering::SeqCst);
    CONTROL_EXECUTIONS.store(0, Ordering::SeqCst);
    let mut db = RecipeProbeDb::default();
    let input = RecipeProbeInput::new(&db, payload.clone(), INTERPRETATION.to_string());
    assert_eq!(persisted_recipe(&db, input), expected);
    assert_eq!(control_recipe(&db, input), expected);
    assert_eq!(PERSISTED_EXECUTIONS.load(Ordering::SeqCst), 1);
    assert_eq!(CONTROL_EXECUTIONS.load(Ordering::SeqCst), 1);
    let bytes = snapshot(&mut db);
    drop(db);

    let mut restored = restore_disposable(&bytes).unwrap();
    let restored_input = RecipeProbeInput::get(&restored);
    assert_eq!(persisted_recipe(&restored, restored_input), expected);
    assert_eq!(PERSISTED_EXECUTIONS.load(Ordering::SeqCst), 1);
    assert_eq!(control_recipe(&restored, restored_input), expected);
    assert_eq!(CONTROL_EXECUTIONS.load(Ordering::SeqCst), 2);

    restored_input.set_payload(&mut restored).to(changed_payload);
    assert_eq!(persisted_recipe(&restored, restored_input), changed_expected);
    assert_eq!(PERSISTED_EXECUTIONS.load(Ordering::SeqCst), 2);
    restored_input
        .set_interpretation(&mut restored)
        .to(format!("{INTERPRETATION}/changed-probe-context"));
    assert_eq!(persisted_recipe(&restored, restored_input), changed_expected);
    assert_eq!(PERSISTED_EXECUTIONS.load(Ordering::SeqCst), 3);

    assert!(restore_disposable(&bytes[..bytes.len() / 2]).is_err());
    let clean = restore_disposable(&bytes).unwrap();
    assert_eq!(persisted_recipe(&clean, RecipeProbeInput::get(&clean)), expected);

    // LRU controls use this same real payload, not integers as query results.
    let mut eviction_db = RecipeProbeDb::default();
    let inputs: Vec<_> = (0..8)
        .map(|_| EvictionInput::new(&eviction_db, payload.clone()))
        .collect();
    for input in &inputs {
        assert_eq!(evicted_recipe(&eviction_db, *input), expected);
    }
    eviction_db.synthetic_write(salsa::Durability::HIGH);
    let before = CONTROL_EXECUTIONS.load(Ordering::SeqCst);
    assert_eq!(evicted_recipe(&eviction_db, inputs[0]), expected);
    assert_eq!(CONTROL_EXECUTIONS.load(Ordering::SeqCst), before + 1);

    // Whole restore is compared to whole ordinary Recipe decode, not a warm
    // tracked-query fetch against a cold compiler or mathematical reconstruction.
    let decode_iterations = 100;
    let restore_iterations = 10;
    let mut decode_ns = Vec::new();
    let mut restore_ns = Vec::new();
    let mut reconstruction_ns = Vec::new();
    let mut reconstructed_compile_ns = Vec::new();
    let mut original_compile_ns = Vec::new();
    let mut evaluation_ns = Vec::new();
    let cancelled = Arc::new(AtomicBool::new(false));
    for _ in 0..7 {
        let start = Instant::now();
        for _ in 0..decode_iterations {
            let recipe: Recipe = serde_json::from_slice(std::hint::black_box(&payload)).unwrap();
            std::hint::black_box(recipe);
        }
        decode_ns.push(start.elapsed().as_nanos() / decode_iterations);
        let start = Instant::now();
        for _ in 0..restore_iterations {
            let restored = restore_disposable(std::hint::black_box(&bytes)).unwrap();
            std::hint::black_box(persisted_recipe(&restored, RecipeProbeInput::get(&restored)));
        }
        restore_ns.push(start.elapsed().as_nanos() / restore_iterations);
        let start = Instant::now();
        let rebuilt = reconstruct_fixture(
            &payload,
            portable_payload_hash(&payload),
            &body.spec,
            &registry,
            &cancelled,
        ).unwrap();
        reconstruction_ns.push(start.elapsed().as_nanos());
        let start = Instant::now();
        let compiled = rebuilt.math.compile(
            &[0], &[0], DerivativeOrder::First, Optimization::default(),
            pse_math::jets::EvaluationLimits::default(), &cancelled,
        ).unwrap();
        reconstructed_compile_ns.push(start.elapsed().as_nanos());
        let start = Instant::now();
        let original_compiled = body.math.compile(
            &[0], &[0], DerivativeOrder::First, Optimization::default(),
            pse_math::jets::EvaluationLimits::default(), &cancelled,
        ).unwrap();
        original_compile_ns.push(start.elapsed().as_nanos());
        let start = Instant::now();
        let jet = compiled.worker().evaluate(
            &[2.0], DerivativeOrder::First, &mut BTreeMap::new(), &cancelled,
        ).unwrap();
        evaluation_ns.push(start.elapsed().as_nanos());
        let original_jet = original_compiled.worker().evaluate(
            &[2.0], DerivativeOrder::First, &mut BTreeMap::new(), &cancelled,
        ).unwrap();
        assert_eq!(jet.values, original_jet.values);
        assert_eq!(jet.jacobian, original_jet.jacobian);
        assert_eq!(jet.values, vec![0.0]);
    }
    println!("RECIPE_PERSISTENCE_PROBE {}", serde_json::json!({
        "payload_bytes": payload.len(),
        "snapshot_bytes": bytes.len(),
        "recipe_retained_bytes": expected.bytes(),
        "original_body_descriptor_bytes": body.descriptor_bytes(),
        "original_body_math_retained_bytes": body.math().retained_bytes(),
        "fresh_restore_persisted_query_executions": 0,
        "fresh_restore_unpersisted_query_executions": 1,
        "payload_change_reexecutions": 1,
        "context_change_reexecutions": 1,
        "corrupt_truncated_restore_refused": true,
        "lru_recomputation": true,
        "decode_iterations_per_sample": decode_iterations,
        "restore_iterations_per_sample": restore_iterations,
        "decode_ns_per_operation": decode_ns,
        "restore_and_fetch_ns_per_operation": restore_ns,
        "qualified_fixture_reconstruction_ns_per_operation": reconstruction_ns,
        "reconstructed_body_evaluator_compile_ns_per_operation": reconstructed_compile_ns,
        "original_body_evaluator_compile_ns_per_operation": original_compile_ns,
        "reconstructed_body_worker_and_evaluation_ns_per_operation": evaluation_ns,
        "reconstruction_and_evaluator_iterations_per_sample": 1,
        "protected_read_and_runtime_qualification_measured": false,
        "mathematical_reconstruction_avoided": false,
    }));
}

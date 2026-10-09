// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Disposable inquiry module, included inside workflow::modeling::reuse_tests for execution.
use super::*;
use std::{hash::{Hash, Hasher}, hint::black_box, time::Instant};
#[derive(Default)]
struct Count { bytes: usize, writes: usize }
impl Hasher for Count {
    fn finish(&self) -> u64 { self.bytes as u64 }
    fn write(&mut self, bytes: &[u8]) { self.bytes += bytes.len(); self.writes += 1; }
}
#[tokio::test]
#[ignore = "bounded preparation-key inquiry; prints operation timings, no performance assertion"]
async fn graph_hash_basis_key_cost_inquiry() {
    let runtime = super::super::super::tests::runtime();
    let source = "package p {def Root {param a:Scalar=2;var x:Scalar;eq e:x*x==a;}def Other {var z:Scalar;eq e:z==3;}}";
    let rows = declarations(source);
    let root = rows.iter().find(|r| r.name == "Root").unwrap().declaration_id;
    let instance = pse_modeling::specialize::root_instance(root);
    let package = runtime.modeling_package(rows, super::super::super::tests::physical()).await.unwrap();
    let initial = prepared(&package).await;
    let selection = package.checked_selection(&[root], &crate::CancelSource::new()).await.unwrap();
    let service = runtime.shared.math();
    let key = service.basis_key(&selection.admission,root,instance,Bindings::default(),Limits::default()).unwrap();
    let mut count = Count::default();
    key.hash(&mut count);
    let loops = 20_000;
    let mut records = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        for _ in 0..loops {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            black_box(&key).hash(&mut hasher);
            black_box(hasher.finish());
        }
        let hash = start.elapsed().as_secs_f64()/f64::from(loops);
        let start = Instant::now();
        for _ in 0..loops {
            black_box(service.basis_key(&selection.admission,root,instance,Bindings::default(),Limits::default()).unwrap());
        }
        let construction = start.elapsed().as_secs_f64()/f64::from(loops);
        let start = Instant::now();
        for _ in 0..20 { black_box(prepared(&package).await); }
        let warm = start.elapsed().as_secs_f64()/20.0;
        records.push(serde_json::json!({"hash_seconds":hash,"key_construction_seconds":construction,"warm_preparation_seconds":warm}));
    }
    println!("BASIS_KEY_INQUIRY {}",serde_json::json!({"workload":"scalar-authored-exact-preparation","hash_bytes":count.bytes,"hash_writes":count.writes,"retained_bytes":key.retained_bytes(),"samples":records,"limits":"same-process operation timings; no candidate hash or end-to-end speedup claim"}));
    runtime.canonical.store().release(selection.read.selection()).await.unwrap();
    drop(initial);
    service.clear_program_cache();
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A06 bounded read-only feasibility probes, with no production scheduler/configuration.
use super::*;
use std::sync::{Barrier, atomic::AtomicUsize};

fn source_rows(text: &str) -> Vec<Declaration> {
    pse_authoring::language::parse(
        text,
        SemanticId::from_bytes([191; 16]),
        pse_authoring::language::IdentityPolicy::Named,
        pse_authoring::ParseBudget::default(),
    )
    .unwrap()
}
fn stage(db: &CompilerDatabase, row: Declaration) -> Source {
    let row = Arc::new(row);
    Source::new(db, row.clone(), row, None, None)
}

#[test]
fn cloned_readers_drain_success_error_and_local_cancellation_before_repair() {
    let mut db = CompilerDatabase::default();
    let row = source_rows("package p {fn law(x:Scalar)->Scalar=x;}")
        .into_iter()
        .find(|row| row.name == "law")
        .unwrap();
    let valid = stage(&db, row.clone());
    let mut malformed = row.clone();
    malformed.value.function.as_mut().unwrap().body = Some("x +".into());
    let invalid = stage(&db, malformed);
    let cancelled = stage(&db, row.clone());
    let barrier = Arc::new(Barrier::new(3));
    let active = Arc::new(AtomicUsize::new(0));
    let output = std::thread::scope(|threads| {
        let mut readers = Vec::new();
        for (position, input) in [valid, invalid, cancelled].into_iter().enumerate() {
            let handle = db.clone();
            let barrier = barrier.clone();
            let active = active.clone();
            readers.push(threads.spawn(move || {
                active.fetch_add(1, Ordering::AcqRel);
                barrier.wait();
                if position == 2 {
                    handle.cancellation_token().cancel();
                }
                let answer = salsa::Cancelled::catch(std::panic::AssertUnwindSafe(|| {
                    parsed(&handle, input)
                }));
                active.fetch_sub(1, Ordering::AcqRel);
                answer
            }));
        }
        readers
            .into_iter()
            .map(|reader| reader.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(active.load(Ordering::Acquire), 0);
    assert!(output[0].as_ref().unwrap().is_ok());
    assert!(output[1].as_ref().unwrap().is_err());
    assert!(output[2].is_err());
    assert!(!db.cancel.load(Ordering::Acquire));
    // All cloned handles were joined/dropped before the sole owner mutates input.
    invalid.set_row(&mut db).to(Arc::new(row));
    assert!(parsed(&db, invalid).is_ok());
    assert!(parsed(&db, cancelled).is_ok());
    assert!(Arc::ptr_eq(
        output[0].as_ref().unwrap().as_ref().unwrap(),
        &parsed(&db, valid).unwrap()
    ));
}

#[test]
fn one_worker_parse_work_is_compared_with_complete_candidate_staging() {
    let mut text = String::from("package p {");
    for index in 0..64 {
        text.push_str(&format!("fn law_{index}(x:Scalar)->Scalar=x+x+x+x;"));
    }
    text.push_str("def Root {var x:Scalar;eq e:x==law_0(x);}}");
    let rows = source_rows(&text);
    let db = CompilerDatabase::default();
    let began = std::time::Instant::now();
    let sources = rows
        .iter()
        .cloned()
        .map(|row| stage(&db, row))
        .collect::<Vec<_>>();
    for source in sources {
        parsed(&db, source).unwrap();
    }
    let parse_and_stage = began.elapsed();
    let mut workspace = CompilerWorkspace::new(
        crate::authored_transfer_tests::context(),
        WorkspaceLimits::default(),
    )
    .unwrap();
    let began = std::time::Instant::now();
    let revision = workspace
        .publish_modeling(rows.clone(), PhysicalScope::default())
        .unwrap();
    let complete = began.elapsed();
    assert_eq!(revision.declarations().len(), rows.len());
    eprintln!(
        "A06 synthetic cold serial probe: rows={}, worker_count=1, parse_plus_source_staging_us={}, complete_candidate_us={}",
        rows.len(),
        parse_and_stage.as_micros(),
        complete.as_micros()
    );
}

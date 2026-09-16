// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Finished physical encodings preserve logical contents under direct decoded admission.

#[path = "../src/canon_fixtures.rs"]
mod canon_fixtures;

use canon_fixtures::{canonical, fixture};
use pse_catalog::store::{encode, verify};
use pse_ids::{CancellationToken, Envelope, FixedBudget};

#[test]
fn finished_ipc_and_parquet_have_distinct_transport_bytes_and_equal_canonical_contents() {
    let (reg, contract, batch) = fixture(false);
    let budget = FixedBudget::new(64 << 20);
    let cancel = CancellationToken::default();
    let spec = reg.relation("authored.values").expect("relation");
    let expected = canonical(&contract, std::slice::from_ref(&batch));
    let ipc = encode::ipc_file(&batch, budget.as_ref(), &cancel).expect("finished IPC");
    let parquet = encode::parquet_file(&batch, budget.as_ref(), &cancel).expect("finished Parquet");
    assert_ne!(ipc.bytes, parquet.bytes);
    assert_ne!(
        pse_ids::encoding_checksum(&ipc.bytes),
        pse_ids::encoding_checksum(&parquet.bytes)
    );
    let ipc_rows = verify::ipc_file(
        &ipc.bytes,
        &reg,
        spec,
        budget.as_ref(),
        &cancel,
        Envelope::DEFAULT,
    )
    .expect("admitted IPC");
    let parquet_rows = verify::parquet_file(
        &parquet.bytes,
        &reg,
        spec,
        budget.as_ref(),
        &cancel,
        Envelope::DEFAULT,
    )
    .expect("admitted Parquet");
    for actual in [ipc_rows, parquet_rows] {
        pse_relations::validate::validate_batch(&reg, spec, &actual)
            .expect("registry/value admission");
        let actual = canonical(&contract, &[actual]);
        assert_eq!(actual.preimage, expected.preimage);
        assert_eq!(actual.logical_hash, expected.logical_hash);
    }
    drop(ipc);
    drop(parquet);
    assert_eq!(budget.reserved(), 0);
}

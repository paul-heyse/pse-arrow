// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native Arrow IPC and Parquet preserve declared logical content.
#[path = "../src/canon_fixtures.rs"]
mod canon_fixtures;
use canon_fixtures::{canonical, fixture};
use datafusion::arrow::{
    compute::concat_batches,
    ipc::{reader::FileReader, writer::FileWriter},
};
use parquet::arrow::{ArrowWriter, arrow_reader::ParquetRecordBatchReaderBuilder};
use std::io::Cursor;
#[test]
fn native_ipc_and_parquet_have_distinct_bytes_and_equal_canonical_content() {
    let (registry, contract, batch) = fixture(false);
    let spec = registry.relation("authored.values").expect("relation");
    let expected = canonical(&contract, std::slice::from_ref(&batch));
    let mut ipc = FileWriter::try_new(Vec::new(), &batch.schema()).expect("IPC writer");
    ipc.write(&batch).expect("IPC batch");
    let ipc = ipc.into_inner().expect("finished IPC");
    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("value.parquet");
    let mut parquet = ArrowWriter::try_new(
        std::fs::File::create(&path).expect("file"),
        batch.schema(),
        None,
    )
    .expect("Parquet writer");
    parquet.write(&batch).expect("Parquet batch");
    parquet.close().expect("finished Parquet");
    let parquet_bytes = std::fs::read(&path).expect("bytes");
    assert_ne!(
        pse_ids::encoding_checksum(&ipc),
        pse_ids::encoding_checksum(&parquet_bytes)
    );
    let ipc = FileReader::try_new(Cursor::new(ipc), None)
        .expect("IPC reader")
        .collect::<Result<Vec<_>, _>>()
        .expect("IPC batches");
    let parquet =
        ParquetRecordBatchReaderBuilder::try_new(std::fs::File::open(&path).expect("file"))
            .expect("Parquet reader")
            .build()
            .expect("reader")
            .collect::<Result<Vec<_>, _>>()
            .expect("Parquet batches");
    for batches in [ipc, parquet] {
        let actual = concat_batches(&batch.schema(), &batches).expect("decoded relation");
        pse_relations::validate::validate_schema(&registry, spec, actual.schema().as_ref())
            .expect("exact declared admission");
        let actual = canonical(&contract, &[actual]);
        assert_eq!(actual.preimage, expected.preimage);
        assert_eq!(actual.logical_hash, expected.logical_hash);
    }
}

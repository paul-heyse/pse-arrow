// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Package data documents (ADR-0125, Plan 23 KR9): Parquet bytes loaded with a package,
//! decoded by Arrow type and admitted through the relation a dataset declaration names.
use super::*;
use crate::authoring_driver::document::{
    OwnedDocumentSet, UNIT_METADATA, load_package_documents_owned, package_checksum,
};
use datafusion::arrow::array::{
    ArrayRef, BooleanArray, DictionaryArray, FixedSizeBinaryArray, Float64Array, Int64Array,
    RecordBatch, StringArray,
};
use datafusion::arrow::datatypes::{Field, Int32Type, Schema};
use pse_modeling::specialize::Value;
use std::collections::BTreeMap;
use std::sync::Arc;
use tests::{physical, runtime, runtime_with};

const MANIFEST: &[u8] =
    include_bytes!("../../../../tests/fixtures/packages/minimal_named/package.toml");
const DATA: &str = "data/bank.parquet";

const BANK: &str = r#"package minimal_named {
 entity kind source provenance { attribute title: Text; }
 enum role { given }
 entity source s { title = "KR9 fixture" }
 identifier scheme cas;
 entity kind item { attribute cas: Id<cas>?; }
 entity item a { cas = Id<cas>("71-43-2") }
 entity item b { cas = Id<cas>("108-88-3") }
 enum phase { liquid, vapor }
 table bank[j: item by cas, ph: phase]: {h: Energy storage {kJ}, T: Temperature? storage {degC}, n: Integer, note: Text, ok: Boolean} missing optional;
 dataset banked: bank provenance(s, role.given) from "data/bank.parquet";
}"#;

/// One named column of a document, with the unit its metadata states, if any.
type Column = (&'static str, ArrayRef, Option<&'static str>);

/// Parquet bytes of `columns`, each field nullable, written by the Rust `parquet` crate.
fn parquet(columns: Vec<Column>) -> Vec<u8> {
    let schema = Arc::new(Schema::new(
        columns
            .iter()
            .map(|(name, array, unit)| {
                let field = Field::new(*name, array.data_type().clone(), true);
                match unit {
                    Some(unit) => field.with_metadata(
                        [(UNIT_METADATA.to_owned(), (*unit).to_owned())].into(),
                    ),
                    None => field,
                }
            })
            .collect::<Vec<_>>(),
    ));
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        columns.into_iter().map(|(_, array, _)| array).collect(),
    )
    .unwrap();
    let mut bytes = Vec::new();
    let mut writer = parquet::arrow::ArrowWriter::try_new(&mut bytes, schema, None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    bytes
}
fn bank_columns() -> Vec<Column> {
    vec![
        (
            "j",
            Arc::new(StringArray::from(vec!["71-43-2", "108-88-3"])),
            None,
        ),
        (
            "ph",
            Arc::new(DictionaryArray::<Int32Type>::from_iter(["liquid", "vapor"])),
            None,
        ),
        ("h", Arc::new(Float64Array::from(vec![1.5, -2.0])), Some("kJ")),
        ("T", Arc::new(Float64Array::from(vec![Some(25.0), None])), None),
        ("n", Arc::new(Int64Array::from(vec![1, 2])), None),
        (
            "note",
            Arc::new(StringArray::from(vec!["first", "second"])),
            None,
        ),
        ("ok", Arc::new(BooleanArray::from(vec![true, false])), None),
    ]
}
fn sources(text: &str, document: Vec<u8>) -> BTreeMap<String, Vec<u8>> {
    BTreeMap::from([
        ("package.toml".to_owned(), MANIFEST.to_vec()),
        ("models/bank.pse".to_owned(), text.as_bytes().to_vec()),
        (DATA.to_owned(), document),
    ])
}
fn load(rt: &Runtime, sources: &BTreeMap<String, Vec<u8>>) -> Result<ModelingPackage, WorkflowError> {
    let token = pse_columnar::CancellationToken::new();
    let pool = rt.shared.pool();
    let bundle = load_package_documents_owned(
        sources,
        &rt.registry,
        pse_authoring::ParseBudget::default(),
        &pool,
        &token,
    )?;
    let documents = OwnedDocumentSet::try_from_bundles(vec![bundle], &pool, &token)?;
    rt.modeling_from_documents(&documents, physical())
}
fn refusal(rt: &Runtime, sources: &BTreeMap<String, Vec<u8>>) -> String {
    match load(rt, sources) {
        Ok(_) => panic!("admitted"),
        Err(error) => error.to_string(),
    }
}
fn entity(package: &ModelingPackage, name: &str) -> Value {
    let p = package.revision.checked();
    Value::Entity {
        id: p.entry(&format!("minimal_named.{name}")).unwrap(),
        kind: p.entry("minimal_named.item").unwrap(),
    }
}
fn member(package: &ModelingPackage, name: &str) -> Value {
    let p = package.revision.checked();
    let enumeration = p.entry("minimal_named.phase").unwrap();
    Value::Enum {
        enumeration,
        member: p.declaration(enumeration).unwrap().value.enumeration.as_ref().unwrap()
            .members
            .iter()
            .find(|m| m.name == name)
            .unwrap()
            .member_id,
    }
}
fn magnitude(value: &Value) -> f64 {
    match value {
        Value::Number { bits, .. } => f64::from_bits(*bits),
        other => panic!("a quantity: {other:?}"),
    }
}
fn table<'p>(package: &'p ModelingPackage, name: &str) -> &'p pse_modeling::data::Table {
    let p = package.revision.checked();
    p.table(p.entry(&format!("minimal_named.{name}")).unwrap()).unwrap()
}

/// ADR-0125: a Parquet document loaded with its package is decoded by Arrow type and its
/// rows admitted through the declared relation: magnitudes converted from the declared
/// storage unit (a matching stated unit accepted), a dictionary naming members, a null of
/// an optional column absent, and integers, text and Booleans as themselves.
#[test]
fn parquet_dataset_admits_through_declared_relation() {
    let rt = runtime();
    let package = load(&rt, &sources(BANK, parquet(bank_columns()))).unwrap();
    let bank = table(&package, "bank");
    assert_eq!(bank.rows.len(), 2);
    let row = &bank.rows[&vec![entity(&package, "a"), member(&package, "liquid")]];
    assert_eq!(magnitude(&row.cells[0]), 1500.0);
    assert!((magnitude(&row.cells[1]) - 298.15).abs() < 1e-9);
    assert_eq!(row.cells[2], Value::Integer(1));
    assert_eq!(row.cells[3], Value::Text("first".into()));
    assert_eq!(row.cells[4], Value::Boolean(true));
    let row = &bank.rows[&vec![entity(&package, "b"), member(&package, "vapor")]];
    assert_eq!(magnitude(&row.cells[0]), -2000.0);
    assert_eq!(row.cells[1], Value::Missing);
    // The data document is a package document: its identity is `named_id(package, path)`.
    let documents = package.revision.documents();
    let (id, document) = documents.documents.iter().next().unwrap();
    assert_eq!(document.path, DATA);
    assert_eq!(
        *id,
        pse_ids::named_id(
            *documents.packages.values().next().unwrap(),
            DATA
        )
    );
}

/// A column whose Arrow type does not store its declared type is refused, naming the
/// column and its declared type.
#[test]
fn parquet_column_type_mismatch_refused() {
    let rt = runtime();
    let mut columns = bank_columns();
    columns[2] = ("h", Arc::new(StringArray::from(vec!["1.5", "-2.0"])), None);
    let error = refusal(&rt, &sources(BANK, parquet(columns)));
    assert!(
        error.contains("column h") && error.contains("declared") && error.contains("Utf8"),
        "{error}"
    );
    // A column the declaration does not name, and a declared column missing.
    let mut columns = bank_columns();
    columns.push(("extra", Arc::new(Int64Array::from(vec![1, 2])), None));
    assert!(refusal(&rt, &sources(BANK, parquet(columns))).contains("has column extra"));
    let mut columns = bank_columns();
    columns.remove(4);
    assert!(refusal(&rt, &sources(BANK, parquet(columns))).contains("has no column n"));
    // An Arrow type no data document stores is refused where the document is decoded.
    let mut columns = bank_columns();
    columns[4] = ("n", Arc::new(datafusion::arrow::array::Int32Array::from(vec![1, 2])), None);
    let error = refusal(&rt, &sources(BANK, parquet(columns)));
    assert!(error.contains("column n has Arrow type Int32"), "{error}");
}

/// Units come only from the declaration: a stated unit other than the declared storage
/// unit is refused, never used.
#[test]
fn parquet_metadata_unit_disagreement_is_refused() {
    let rt = runtime();
    let mut columns = bank_columns();
    columns[2].2 = Some("J");
    let error = refusal(&rt, &sources(BANK, parquet(columns)));
    assert!(
        error.contains("column h") && error.contains("states unit J") && error.contains("disagrees"),
        "{error}"
    );
    // A unit stated on a column that is no quantity is refused too.
    let mut columns = bank_columns();
    columns[4].2 = Some("K");
    assert!(refusal(&rt, &sources(BANK, parquet(columns))).contains("only a quantity column"));
}

/// Utf8 values of a reference column resolve through the identifier index of the declared
/// scheme; identities resolve directly; an unknown value is refused.
#[test]
fn parquet_reference_by_identifier_scheme_resolves() {
    let rt = runtime();
    let package = load(&rt, &sources(BANK, parquet(bank_columns()))).unwrap();
    let a = entity(&package, "a");
    assert!(
        table(&package, "bank")
            .rows
            .contains_key(&vec![a.clone(), member(&package, "liquid")])
    );
    let mut columns = bank_columns();
    columns[0] = (
        "j",
        Arc::new(StringArray::from(vec!["71-43-2", "0-00-0"])),
        None,
    );
    let error = refusal(&rt, &sources(BANK, parquet(columns)));
    assert!(error.contains("\"0-00-0\"") && error.contains("column j"), "{error}");
    // By identity, without a scheme: FixedSizeBinary(16) declaration identities.
    let identity = |value: Value| match value {
        Value::Entity { id, .. } => *id.as_id().as_bytes(),
        other => panic!("an entity: {other:?}"),
    };
    let by_identity = BANK.replace("item by cas", "item");
    let mut columns = bank_columns();
    columns[0] = (
        "j",
        Arc::new(
            FixedSizeBinaryArray::try_from_iter(
                [identity(a), identity(entity(&package, "b"))].into_iter(),
            )
            .unwrap(),
        ),
        None,
    );
    let package = load(&rt, &sources(&by_identity, parquet(columns))).unwrap();
    assert_eq!(table(&package, "bank").rows.len(), 2);
}

/// ADR-0123 Outcome 8, ADR-0125: the bytes of a data document enter the package checksum
/// and the source revision; unchanged bytes reproduce both.
#[test]
fn data_bytes_enter_package_checksum_and_source_revision() {
    let rt = runtime();
    let original = sources(BANK, parquet(bank_columns()));
    // One changed byte changes the package checksum, which frames exact bytes.
    let mut flipped = original.clone();
    let bytes = flipped.get_mut(DATA).unwrap();
    let last = bytes.len() / 2;
    bytes[last] ^= 1;
    assert_ne!(package_checksum(&original), package_checksum(&flipped));
    assert_eq!(package_checksum(&original), package_checksum(&original.clone()));
    // A document whose bytes differ in one value is another package and another source
    // revision; the same bytes reproduce both.
    let mut columns = bank_columns();
    columns[4] = ("n", Arc::new(Int64Array::from(vec![1, 3])), None);
    let changed = sources(BANK, parquet(columns));
    let first = load(&rt, &original).unwrap();
    let again = load(&rt, &original).unwrap();
    let other = load(&rt, &changed).unwrap();
    assert_eq!(first.revision.identity(), again.revision.identity());
    assert_ne!(first.revision.identity(), other.revision.identity());
    // The loaded package header carries the checksum over its exact documents.
    let header = |sources: &BTreeMap<String, Vec<u8>>| {
        load_package_documents_owned(
            sources,
            &rt.registry,
            pse_authoring::ParseBudget::default(),
            &rt.shared.pool(),
            &pse_columnar::CancellationToken::new(),
        )
        .unwrap()
        .bundle()
        .package
        .content_hash
    };
    assert_eq!(header(&original), package_checksum(&original));
    assert_eq!(header(&changed), package_checksum(&changed));
    assert_ne!(header(&original), header(&changed));
}

/// A table of 10⁵ document rows admits through the relation without a single cell: the
/// declaration carries no rows, its IR does not grow with the document, and every row is
/// admitted typed column by column.
#[test]
fn large_table_admits_without_cell_evaluation() {
    const ROWS: i64 = 100_000;
    let rt = runtime_with(256 << 20, 1 << 20, 1 << 30);
    let text = r#"package minimal_named {
 entity kind source provenance { attribute title: Text; }
 enum role { given }
 entity source s { title = "KR9 fixture" }
 table level[n: Integer]: Energy storage {J} missing optional;
 dataset levels: level provenance(s, role.given) from "data/bank.parquet";
}"#;
    let document = |rows: i64| {
        parquet(vec![
            ("n", Arc::new(Int64Array::from_iter_values(0..rows)), None),
            (
                "value",
                Arc::new(Float64Array::from_iter_values((0..rows).map(|n| n as f64 * 0.5))),
                None,
            ),
        ])
    };
    let large = load(&rt, &sources(text, document(ROWS))).unwrap();
    let small = load(&rt, &sources(text, document(10))).unwrap();
    let level = table(&large, "level");
    assert_eq!(level.rows.len(), ROWS as usize);
    assert_eq!(magnitude(&level.rows[&vec![Value::Integer(ROWS - 1)]].cells[0]), (ROWS - 1) as f64 * 0.5);
    assert_eq!(table(&small, "level").rows.len(), 10);
    // The declarations are the same rows, cell-free, whatever the document holds.
    assert_eq!(large.declarations(), small.declarations());
    assert!(
        large
            .declarations()
            .iter()
            .filter_map(|d| d.value.dataset.as_ref())
            .all(|d| d.rows.is_empty() && d.document.as_deref() == Some(DATA))
    );
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Standalone library characterization; this does not test a pse implementation.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use arrow::array::{
    Array, ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray, UInt32Array,
};
use arrow::buffer::{NullBuffer, ScalarBuffer};
use arrow::compute::{
    CastOptions, can_cast_types, cast_with_options, concat_batches, take_record_batch,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::ipc::MetadataVersion;
use arrow::ipc::writer::{FileWriter, IpcWriteOptions, StreamWriter};
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use datafusion::execution::session_state::SessionStateBuilder;
use datafusion_catalog::TableProvider;
use datafusion_common::types::DFExtensionType;
use datafusion_common::{Result, TableReference, exec_err, not_impl_err, plan_err};
use datafusion_expr::registry::{
    ExtensionTypeRegistration, ExtensionTypeRegistry, MemoryExtensionTypeRegistry,
};
use datafusion_expr::{
    ColumnarValue, Expr, LogicalPlan, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
    Volatility,
};
use datafusion_proto::bytes::logical_plan_to_bytes_with_extension_codec;
use datafusion_proto::logical_plan::LogicalExtensionCodec;
use parquet::arrow::ArrowWriter;

static FACTORY_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct SemanticId;

impl DFExtensionType for SemanticId {
    fn storage_type(&self) -> DataType {
        DataType::FixedSizeBinary(16)
    }

    fn serialize_metadata(&self) -> Option<String> {
        None
    }
}

fn semantic_field(name: &str, extension: &str) -> Field {
    Field::new(name, DataType::Utf8, false).with_metadata(HashMap::from([
        ("ARROW:extension:name".to_owned(), extension.to_owned()),
        (
            "ARROW:extension:metadata".to_owned(),
            "{\"v\":1}".to_owned(),
        ),
    ]))
}

async fn extension_validation() -> Result<()> {
    let registry = Arc::new(MemoryExtensionTypeRegistry::new_with_canonical_extension_types());
    registry.add_extension_type_registration(ExtensionTypeRegistration::new_arc(
        "pse.semantic_id",
        |storage, _metadata| {
            FACTORY_CALLS.fetch_add(1, Ordering::SeqCst);
            if storage != &DataType::FixedSizeBinary(16) {
                return plan_err!("semantic ID requires FixedSizeBinary(16), got {storage}");
            }
            Ok(Arc::new(SemanticId) as Arc<dyn DFExtensionType>)
        },
    ))?;
    let wrong = semantic_field("id", "pse.semantic_id");
    let unknown = semantic_field("unknown", "pse.not_registered");
    println!(
        "E1 direct validation rejects wrong storage: {}",
        registry.create_extension_type_for_field(&wrong).is_err()
    );
    println!(
        "E1 direct validation rejects unknown name: {}",
        registry.create_extension_type_for_field(&unknown).is_err()
    );
    FACTORY_CALLS.store(0, Ordering::SeqCst);
    let ctx = SessionContext::new_with_state(
        SessionStateBuilder::new()
            .with_default_features()
            .with_extension_type_registry(registry)
            .build(),
    );
    let batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![wrong, unknown])),
        vec![
            Arc::new(StringArray::from(vec!["not-an-id"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["unregistered"])) as ArrayRef,
        ],
    )?;
    ctx.register_batch("malformed_extensions", batch)?;
    let result = ctx
        .sql("SELECT id, unknown FROM malformed_extensions")
        .await?
        .collect()
        .await;
    println!("E1 ordinary SELECT accepts both fields: {}", result.is_ok());
    println!(
        "E1 registry factory calls during SELECT: {}",
        FACTORY_CALLS.load(Ordering::SeqCst)
    );
    assert!(result.is_ok());
    assert_eq!(FACTORY_CALLS.load(Ordering::SeqCst), 0);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct FailingArgument {
    signature: Signature,
}

impl ScalarUDFImpl for FailingArgument {
    fn name(&self) -> &str {
        "failing_argument"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Int64)
    }
    fn invoke_with_args(&self, _: ScalarFunctionArgs) -> Result<ColumnarValue> {
        exec_err!("unselected argument evaluated")
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct DeclaredConditional {
    signature: Signature,
}

impl ScalarUDFImpl for DeclaredConditional {
    fn name(&self) -> &str {
        "declared_conditional"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Int64)
    }
    fn short_circuits(&self) -> bool {
        true
    }
    fn conditional_arguments<'a>(
        &self,
        args: &'a [Expr],
    ) -> Option<(Vec<&'a Expr>, Vec<&'a Expr>)> {
        Some((vec![&args[0]], vec![&args[1], &args[2]]))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        // All probe rows select the else branch. Reaching this body is enough.
        Ok(args.args[2].clone())
    }
}

async fn conditional_evaluation() -> Result<()> {
    let ctx = SessionContext::new();
    ctx.register_udf(ScalarUDF::from(FailingArgument {
        signature: Signature::exact(vec![DataType::Int64], Volatility::Immutable),
    }));
    ctx.register_udf(ScalarUDF::from(DeclaredConditional {
        signature: Signature::exact(
            vec![DataType::Boolean, DataType::Int64, DataType::Int64],
            Volatility::Immutable,
        ),
    }));
    ctx.register_batch(
        "inputs",
        RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)])),
            vec![Arc::new(Int64Array::from(vec![1, 2]))],
        )?,
    )?;
    let custom = ctx
        .sql("SELECT declared_conditional(x < 0, failing_argument(x), x) FROM inputs")
        .await?
        .collect()
        .await;
    println!(
        "E2 conditional UDF with both flags errors: {}",
        custom.is_err()
    );
    if let Err(ref error) = custom {
        println!("E2 error: {error}");
    }
    assert!(custom.is_err());
    let builtin = ctx
        .sql("SELECT CASE WHEN x < 0 THEN failing_argument(x) ELSE x END FROM inputs")
        .await?
        .collect()
        .await;
    println!("E2 builtin CASE masks unselected rows: {}", builtin.is_ok());
    assert!(builtin.is_ok());
    Ok(())
}

fn cast_fidelity() -> Result<()> {
    let options = CastOptions {
        safe: false,
        ..Default::default()
    };
    let input = Float64Array::from(vec![1.9]);
    let converted = cast_with_options(&input, &DataType::Int64, &options)?;
    let value = converted
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    println!(
        "E3 can_cast Float64 to Int64: {}",
        can_cast_types(&DataType::Float64, &DataType::Int64)
    );
    println!("E3 safe=false casts 1.9 to: {value}");
    assert_eq!(value, 1);
    let large = Int64Array::from(vec![9_007_199_254_740_993]);
    let float = cast_with_options(&large, &DataType::Float64, &options)?;
    let roundtrip = cast_with_options(float.as_ref(), &DataType::Int64, &options)?;
    let rounded = roundtrip
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    println!("E3 safe=false Int64 -> Float64 -> Int64: 9007199254740993 -> {rounded}");
    assert_eq!(rounded, 9_007_199_254_740_992);
    Ok(())
}

fn stream(batch: &RecordBatch) -> Result<Vec<u8>> {
    let options = IpcWriteOptions::try_new(64, false, MetadataVersion::V5)?;
    let mut bytes = Vec::new();
    {
        let mut writer = StreamWriter::try_new_with_options(&mut bytes, &batch.schema(), options)?;
        writer.write(batch)?;
        writer.finish()?;
    }
    Ok(bytes)
}

fn representation_identity() -> Result<()> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("x", DataType::Int64, true),
    ]));
    let make = |hidden: i64| -> Result<RecordBatch> {
        let a = Int64Array::new(
            ScalarBuffer::from(vec![7, hidden]),
            Some(NullBuffer::from(vec![true, false])),
        );
        let b = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![0, 1])), Arc::new(a)],
        )?;
        // PKs already ascend. Exercise take as well as concat, so a sorting
        // implementation that always materializes rows is covered.
        let sorted = take_record_batch(&b, &UInt32Array::from(vec![0, 1]))?;
        Ok(concat_batches(&schema, [&sorted])?)
    };
    let left = make(0)?;
    let right = make(42)?;
    println!(
        "E4 equal visible arrays with different null payload: {}",
        left.column(1) == right.column(1)
    );
    let left_bytes = stream(&left)?;
    let right_bytes = stream(&right)?;
    println!(
        "E4 canonical-option IPC stream bytes equal: {}",
        left_bytes == right_bytes
    );
    assert_eq!(left.column(1), right.column(1));
    assert_ne!(left_bytes, right_bytes);
    println!(
        "E4 null-payload stream hashes: {} / {}",
        blake3::hash(&left_bytes),
        blake3::hash(&right_bytes)
    );

    let mut file = Vec::new();
    {
        let mut writer = FileWriter::try_new_with_options(
            &mut file,
            &schema,
            IpcWriteOptions::try_new(64, false, MetadataVersion::V5)?,
        )?;
        writer.write(&left)?;
        writer.finish()?;
    }
    let mut parquet = Vec::new();
    {
        let mut writer = ArrowWriter::try_new(&mut parquet, schema, None)?;
        writer.write(&left)?;
        writer.close()?;
    }
    println!(
        "E4 IPC stream / IPC file / Parquet lengths: {} / {} / {}",
        left_bytes.len(),
        file.len(),
        parquet.len()
    );
    println!(
        "E4 physical encodings match canonical stream digest: file={} parquet={}",
        blake3::hash(&file) == blake3::hash(&left_bytes),
        blake3::hash(&parquet) == blake3::hash(&left_bytes)
    );
    assert_ne!(blake3::hash(&file), blake3::hash(&left_bytes));
    assert_ne!(blake3::hash(&parquet), blake3::hash(&left_bytes));
    Ok(())
}

#[derive(Debug)]
struct FixedProviderCodec;

impl LogicalExtensionCodec for FixedProviderCodec {
    fn try_encode(
        &self,
        _: &datafusion_expr::logical_plan::Extension,
        _: &mut Vec<u8>,
    ) -> Result<()> {
        not_impl_err!("probe has no extension nodes")
    }
    fn try_decode(
        &self,
        _: &[u8],
        _: &[LogicalPlan],
        _: &TaskContext,
    ) -> Result<datafusion_expr::logical_plan::Extension> {
        not_impl_err!("probe is encode-only")
    }
    fn try_encode_table_provider(
        &self,
        _: &TableReference,
        _: Arc<dyn TableProvider>,
        bytes: &mut Vec<u8>,
    ) -> Result<()> {
        bytes.extend_from_slice(b"fixed-snapshot-relation");
        Ok(())
    }
    fn try_decode_table_provider(
        &self,
        _: &[u8],
        _: &TableReference,
        _: SchemaRef,
        _: &TaskContext,
    ) -> Result<Arc<dyn TableProvider>> {
        not_impl_err!("probe is encode-only")
    }
}

async fn sorted_metadata_proto() -> Result<()> {
    let ctx = SessionContext::new();
    let mut encodings = BTreeSet::new();
    let mut ipc_encodings = BTreeSet::new();
    for _ in 0..32 {
        // Lexicographically sorted insertion exactly as blueprint section 4.3 prescribes.
        let mut metadata = HashMap::new();
        for key in ["a", "b", "c", "d", "e", "f", "g", "h"] {
            metadata.insert(key.to_owned(), "same-value".to_owned());
        }
        let schema: SchemaRef = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Int64, false).with_metadata(metadata),
        ]));
        let batch = RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1]))])?;
        let dataframe = ctx.read_batch(batch.clone())?.select_columns(&["x"])?;
        encodings.insert(
            logical_plan_to_bytes_with_extension_codec(
                dataframe.logical_plan(),
                &FixedProviderCodec,
            )?
            .to_vec(),
        );
        ipc_encodings.insert(stream(&batch)?);
    }
    println!(
        "E5 sorted metadata insertion, distinct proto encodings / 32: {}",
        encodings.len()
    );
    println!(
        "E5 sorted metadata insertion, distinct IPC encodings / 32: {}",
        ipc_encodings.len()
    );
    assert!(encodings.len() > 1);
    assert_eq!(ipc_encodings.len(), 1);
    Ok(())
}

fn floating_point_rewrites() {
    let x = std::hint::black_box(1000.0_f64);
    println!("E6 log(exp(1000))={} versus rewritten x={x}", x.exp().ln());
    let a = std::hint::black_box(1.0e16_f64);
    let b = std::hint::black_box(-1.0e16_f64);
    let c = std::hint::black_box(1.0_f64);
    println!(
        "E6 reassociation: (a+b)+c={} a+(b+c)={}",
        (a + b) + c,
        a + (b + c)
    );
    assert!(x.exp().ln().is_infinite());
    assert_ne!((a + b) + c, a + (b + c));
}

#[tokio::main]
async fn main() -> Result<()> {
    extension_validation().await?;
    conditional_evaluation().await?;
    cast_fidelity()?;
    representation_identity()?;
    sorted_metadata_proto().await?;
    floating_point_rewrites();
    println!("Characterization groups completed: 6; assertion failures: 0.");
    Ok(())
}

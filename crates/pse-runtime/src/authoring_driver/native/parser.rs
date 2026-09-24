// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::authoring_driver::{DriverError, ParseBudget, document::load_package_sources_owned};
use datafusion::{
    arrow::{
        array::{Array, FixedSizeBinaryArray, ListArray, RecordBatch, StringArray, StructArray},
        buffer::OffsetBuffer,
        compute::concat_batches,
        datatypes::{DataType, Field, FieldRef, Schema, SchemaRef},
    },
    common::{DataFusionError, Result},
    logical_expr::{
        ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
    },
};
use pse_columnar::{CancellationToken, MemoryPool};
use pse_ids::SemanticId;
use pse_schema::Registry;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug)]
struct Binding {
    registry: Arc<Registry>,
    relation: SemanticId,
    budget: ParseBudget,
    pool: Arc<dyn MemoryPool>,
    cancel: CancellationToken,
    schema: SchemaRef,
    item: FieldRef,
    output: FieldRef,
}
#[derive(Debug, Clone)]
pub(super) struct Parser {
    binding: Arc<Binding>,
    name: String,
    signature: Signature,
}
impl PartialEq for Parser {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.binding, &other.binding)
    }
}
impl Eq for Parser {}
impl Hash for Parser {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.binding).hash(state);
    }
}
impl Parser {
    pub(super) fn new(
        registry: Arc<Registry>,
        relation: SemanticId,
        budget: ParseBudget,
        pool: Arc<dyn MemoryPool>,
        cancel: CancellationToken,
    ) -> Result<Self> {
        let spec = registry
            .relation_by_id(relation)
            .ok_or_else(|| super::invalid("source output declaration absent"))?;
        let schema =
            Arc::new(pse_schema::arrow::relation_schema(&registry, spec).map_err(external)?);
        let item = Arc::new(Field::new(
            "item",
            DataType::Struct(schema.fields().clone()),
            false,
        ));
        let output = Arc::new(Field::new(
            "parsed",
            DataType::List(Arc::clone(&item)),
            false,
        ));
        Ok(Self {
            name: format!("pse_parse_{}", spec.fingerprint.to_hex()),
            binding: Arc::new(Binding {
                registry,
                relation,
                budget,
                pool,
                cancel,
                schema,
                item,
                output,
            }),
            signature: Signature::user_defined(Volatility::Immutable),
        })
    }
    fn parse(&self, package: &[u8], sources: &dyn Array) -> Result<RecordBatch> {
        let sources = sources
            .as_any()
            .downcast_ref::<StructArray>()
            .ok_or_else(|| super::invalid("source inventory is not a struct"))?;
        let ids = sources
            .column_by_name("document_id")
            .and_then(|column| column.as_any().downcast_ref::<FixedSizeBinaryArray>())
            .ok_or_else(|| super::invalid("document identity representation changed"))?;
        if ids.null_count() != 0 || ids.value_length() != 16 {
            return Err(super::invalid(
                "source inventory contains invalid document identities",
            ));
        }
        let paths = strings(sources, "path")?;
        let texts = strings(sources, "source_text")?;
        if sources.null_count() != 0 || paths.null_count() != 0 || texts.null_count() != 0 {
            return Err(super::invalid("source inventory contains null text/path"));
        }
        let bundle = load_package_sources_owned(
            (0..sources.len()).map(|index| (paths.value(index), texts.value(index).as_bytes())),
            &self.binding.registry,
            self.binding.budget,
            &self.binding.pool,
            &self.binding.cancel,
        )
        .map_err(external)?;
        if bundle.bundle().package.package_id.as_bytes().as_slice() != package {
            return Err(external(DriverError::Authoring(
                pse_authoring::AuthoringError::Contract {
                    at: None,
                    reason: "source package grouping disagrees with its actual header identity"
                        .into(),
                },
            )));
        }
        for index in 0..sources.len() {
            let expected =
                pse_ids::named_id(bundle.bundle().package.package_id, paths.value(index));
            if expected.as_bytes().as_slice() != ids.value(index) {
                return Err(external(DriverError::Authoring(
                    pse_authoring::AuthoringError::Contract {
                        at: None,
                        reason: "document identity disagrees with its package and path".into(),
                    },
                )));
            }
        }
        Ok(bundle
            .bundle()
            .batches
            .get(&self.binding.relation)
            .map_or_else(
                || RecordBatch::new_empty(Arc::clone(&self.binding.schema)),
                |batch| batch.batch().clone(),
            ))
    }
}
impl ScalarUDFImpl for Parser {
    fn name(&self) -> &str {
        &self.name
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(self.binding.output.data_type().clone())
    }
    fn return_field_from_args(&self, _: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        Ok(Arc::clone(&self.binding.output))
    }
    fn coerce_types(&self, arguments: &[DataType]) -> Result<Vec<DataType>> {
        let [DataType::FixedSizeBinary(16), DataType::List(child)] = arguments else {
            return Err(super::invalid(
                "parser requires a package identity and document list",
            ));
        };
        let DataType::Struct(fields) = child.data_type() else {
            return Err(super::invalid("document list requires path/text structs"));
        };
        if fields.len() != 3
            || fields[0].name() != "document_id"
            || fields[0].data_type() != &DataType::FixedSizeBinary(16)
            || fields[1].name() != "path"
            || fields[2].name() != "source_text"
            || fields[1..]
                .iter()
                .any(|field| field.data_type() != &DataType::Utf8)
        {
            return Err(super::invalid(
                "document input must contain an identity and exact UTF-8 path and source_text",
            ));
        }
        Ok(arguments.to_vec())
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let binding = &self.binding;
        binding.cancel.checkpoint().map_err(external)?;
        let [packages, sources] = args.args.as_slice() else {
            return Err(super::invalid("parser requires two arguments"));
        };
        let packages = packages.to_array(args.number_rows)?;
        let packages = packages
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .ok_or_else(|| super::invalid("package identity representation changed"))?;
        let sources = sources.to_array(args.number_rows)?;
        let sources = sources
            .as_any()
            .downcast_ref::<ListArray>()
            .ok_or_else(|| super::invalid("source list representation changed"))?;
        let reservation = pse_columnar::MemoryConsumer::new("authoring:native-parser-output")
            .register(&binding.pool);
        reservation.try_grow(
            args.number_rows
                .checked_add(1)
                .and_then(|rows| rows.checked_mul(size_of::<RecordBatch>() + size_of::<i32>()))
                .ok_or_else(exhausted)?,
        )?;
        let mut batches = Vec::with_capacity(args.number_rows);
        let mut offsets = Vec::with_capacity(args.number_rows + 1);
        offsets.push(0i32);
        let mut length = 0i32;
        for index in 0..args.number_rows {
            binding.cancel.checkpoint().map_err(external)?;
            if packages.is_null(index) || sources.is_null(index) {
                return Err(super::invalid(
                    "parser received a null package or document list",
                ));
            }
            let batch = self.parse(packages.value(index), sources.value(index).as_ref())?;
            length = length
                .checked_add(i32::try_from(batch.num_rows()).map_err(|_| exhausted())?)
                .ok_or_else(exhausted)?;
            reservation.try_grow(batch.get_array_memory_size())?;
            batches.push(batch);
            offsets.push(length);
        }
        let values = StructArray::from(concat_batches(&binding.schema, &batches)?);
        let values = ListArray::try_new(
            Arc::clone(&binding.item),
            OffsetBuffer::new(offsets.into()),
            Arc::new(values),
            None,
        )?;
        let batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Arc::clone(&binding.output)])),
            vec![Arc::new(values)],
        )?;
        let retained =
            pse_columnar::owned_buffer::attach_reservation(batch, reservation).map_err(external)?;
        Ok(ColumnarValue::Array(Arc::clone(retained.column(0))))
    }
}
fn strings<'a>(values: &'a StructArray, name: &str) -> Result<&'a StringArray> {
    values
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| super::invalid("source path/text representation changed"))
}
fn exhausted() -> DataFusionError {
    DataFusionError::ResourcesExhausted("native parser output exceeds its supported extent".into())
}
fn external(
    error: impl pse_diagnostics::TypedDiagnostic + Send + Sync + 'static,
) -> DataFusionError {
    pse_columnar::external(error)
}

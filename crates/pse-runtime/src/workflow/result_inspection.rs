// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded readable observations over the original protected scientific result owner.

use super::{PackResultSelection, PackScientificState, Runtime, WorkflowError, contract};
use datafusion::arrow::{
    array::{
        Array, ArrayRef, BinaryArray, BooleanArray, FixedSizeBinaryArray, Float32Array,
        Float64Array, Int8Array, Int16Array, Int32Array, Int64Array, LargeBinaryArray,
        LargeStringArray, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    },
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    sync::Arc,
};

const DOCUMENT_BYTES: usize = 64 << 10;
const TEXT_BYTES: usize = 1024;
const CONTEXT_FIELDS: usize = 32;

/// Select one exact immutable result; schema discovery does not read its Arrow rows.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultInspectionRequest {
    /// Original canonical run, never a latest-run alias.
    pub run: String,
    /// Exact original terminal attempt.
    pub attempt: String,
    /// Qualified original scientific relation.
    pub relation: String,
    /// Inclusive original row coordinate.
    pub start: u64,
    /// Exclusive original row coordinate.
    pub end: u64,
    /// Optional closure digest returned by an earlier discovery; mismatch refuses.
    #[serde(default)]
    pub expected_manifest: Option<String>,
    /// Bounded schema page or readable scalar selection.
    pub view: ResultInspectionView,
}

/// Physical column selection; this does not reinterpret a scientific output predicate.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultInspectionView {
    /// Page the original field inventory without decoding any result batch.
    Schema {
        /// Zero-based original field position.
        offset: usize,
        /// At most sixteen original fields.
        limit: usize,
    },
    /// Read selected original scalar fields together with all declared non-value context.
    Values {
        /// One to eight unique exact original field names.
        fields: Vec<String>,
        /// At most sixteen original rows; continuation retains source coordinates.
        maximum_rows: usize,
    },
}

/// Admitted registry Arrow field, including nested datatype/field metadata.
/// Schema-only discovery does not independently validate the stored file schema.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultInspectionField {
    /// Position in the original complete relation schema.
    pub index: usize,
    /// Exact original field name.
    pub name: String,
    /// Original Arrow datatype using arrow_schema's own JSON encoding, including nested fields.
    pub datatype_json: String,
    /// Original outer field nullability.
    pub nullable: bool,
    /// Every original outer field metadata entry, including quantity and extension semantics.
    pub metadata: BTreeMap<String, String>,
}

/// IEEE classification independent of exact bit payload and scientific usability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultFloatClass {
    /// Includes positive and negative zero, whose exact bits remain distinct.
    Finite,
    /// A NaN; payload/sign bits remain available.
    Nan,
    /// Positive IEEE infinity.
    PositiveInfinity,
    /// Negative IEEE infinity.
    NegativeInfinity,
}
impl ResultFloatClass {
    fn of(value: f64) -> Self {
        if value.is_nan() {
            Self::Nan
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                Self::PositiveInfinity
            } else {
                Self::NegativeInfinity
            }
        } else {
            Self::Finite
        }
    }
}

/// Exact scalar storage observation. Field metadata and row context retain its interpretation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultInspectionCell {
    /// Original validity bit is false; distinct from nonfinite and unsupported storage.
    Null,
    /// Exact Boolean storage.
    Boolean {
        /// Original value.
        value: bool,
    },
    /// Signed integer; original width and temporal/quantity semantics remain in the field.
    Integer {
        /// Exact original integer.
        value: i64,
    },
    /// Unsigned integer with no lossy floating-point conversion.
    Unsigned {
        /// Exact original integer.
        value: u64,
    },
    /// IEEE binary32, retaining NaN payloads and signed zero.
    Float32 {
        /// Exact original IEEE bits.
        bits: u32,
        /// Convenient finite value; absent exactly for a nonfinite observation.
        finite_value: Option<f32>,
        /// Explicit original IEEE classification.
        class: ResultFloatClass,
    },
    /// IEEE binary64, retaining NaN payloads and signed zero.
    Float64 {
        /// Exact original IEEE bits.
        bits: u64,
        /// Convenient finite value; absent exactly for a nonfinite observation.
        finite_value: Option<f64>,
        /// Explicit original IEEE classification.
        class: ResultFloatClass,
    },
    /// Complete bounded UTF-8 text; never a truncated prefix.
    Text {
        /// Original complete text.
        value: String,
    },
    /// Complete bounded bytes; fixed-width semantic identities retain their original field metadata.
    Binary {
        /// Lowercase exact hexadecimal bytes.
        hex: String,
    },
    /// The exact value remains available through the protected Arrow IPC reader.
    Unavailable {
        /// Why the readable view cannot present the original cell.
        reason: ResultCellUnavailable,
    },
}

/// Unavailable readable detail never becomes a null or a normalized scientific value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultCellUnavailable {
    /// This view deliberately does not reinterpret nested, dictionary or specialized storage.
    UnsupportedStorage,
    /// Complete text/bytes exceed the readable cell allowance; no prefix is returned.
    ValueExtent,
}

/// One original row; selected cells and independently retained context use descriptor order.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultInspectionRow {
    /// Exact original coordinate, including across missing recorded ranges.
    pub source_row: u64,
    /// Cells in the selected_fields descriptor order.
    pub values: Vec<ResultInspectionCell>,
    /// Cells in context_fields order; all original non-value-role columns are retained.
    pub context: Vec<ResultInspectionCell>,
    /// False when context storage is unavailable: do not infer complete physical interpretation.
    pub context_complete: bool,
}

/// Explicit finite-view refusal, independent of native source errors or scientific outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResultInspectionRefusal {
    /// Request bounds, duplicate fields or empty selections cannot describe this finite view.
    Selection,
    /// A requested physical field is not declared by the original relation.
    UnknownField,
    /// The caller's discovery digest names another immutable closure.
    ManifestChanged,
    /// Original context cannot fit the finite readable view.
    ContextExtent,
    /// Complete required descriptors/values exceed the encoded answer allowance.
    ResponseExtent,
}

/// Original protected inspection outcome. Copying does not upgrade scientific usability.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultInspectionDocument {
    /// A page of the admitted registry field inventory, without validating stored Arrow rows.
    Schema {
        /// Exact original source/terminal/range; usability remains Unknown.
        selection: PackResultSelection,
        /// Actual immutable closure digest, suitable for expected_manifest.
        manifest: String,
        /// Original schema metadata including contract version/fingerprint.
        schema_metadata: BTreeMap<String, String>,
        /// Complete original field population, independent of this page.
        field_count: usize,
        /// Zero-based original field offset.
        offset: usize,
        /// Selected original descriptors.
        fields: Vec<ResultInspectionField>,
        /// Next original field offset; absent only on the final page.
        next: Option<usize>,
        /// Recorded rows intersecting the selected range; holes are not invented.
        recorded_rows: u64,
        /// Native allocation ownership follows the response until FFI serialization completes.
        #[serde(skip)]
        #[schemars(skip)]
        owner: Option<Arc<MemoryReservation>>,
    },
    /// Readable bounded original scalar observations; exact IPC remains the bulk interface.
    Values {
        /// Exact original source/terminal/range; usability remains Unknown.
        selection: PackResultSelection,
        /// Actual immutable closure digest.
        manifest: String,
        /// Original schema metadata including quantity/context checks and fingerprint.
        schema_metadata: BTreeMap<String, String>,
        /// Exact original descriptors, in requested order.
        selected_fields: Vec<ResultInspectionField>,
        /// Every non-value-role original field not already selected; no name heuristics.
        context_fields: Vec<ResultInspectionField>,
        /// Recorded rows intersecting the complete selected range, not just the returned prefix.
        recorded_rows: u64,
        /// Original row observations in source order.
        rows: Vec<ResultInspectionRow>,
        /// Exact next original coordinate, preserving holes; repeat with start=next_row.
        next_row: Option<u64>,
        /// Every returned selected and context cell was representable, independent of population/scientific usability.
        readable_complete: bool,
        /// Native allocation ownership follows the response until FFI serialization completes.
        #[serde(skip)]
        #[schemars(skip)]
        owner: Option<Arc<MemoryReservation>>,
    },
    /// The finite readable request cannot produce a truthful complete answer of this shape.
    Refused {
        /// Typed refusal; no required fields were silently discarded.
        reason: ResultInspectionRefusal,
        /// Existing exact IPC route, not a newly issued read handle.
        guidance: String,
    },
}

fn refused(reason: ResultInspectionRefusal) -> ResultInspectionDocument {
    ResultInspectionDocument::Refused {
        reason,
        guidance: "Narrow the field/page/row selection or use the existing exact protected Arrow IPC result reader; no values were truncated.".into(),
    }
}

struct Budget {
    remaining: usize,
}
impl Budget {
    fn charge(&mut self, bytes: usize) -> Result<(), ResultInspectionRefusal> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or(ResultInspectionRefusal::ResponseExtent)?;
        Ok(())
    }
    fn metadata(
        &mut self,
        metadata: &std::collections::HashMap<String, String>,
    ) -> Result<BTreeMap<String, String>, ResultInspectionRefusal> {
        let extent = metadata
            .iter()
            .try_fold(0usize, |sum, (key, value)| {
                sum.checked_add(key.len())
                    .and_then(|sum| sum.checked_add(value.len()))
                    .and_then(|sum| sum.checked_add(128))
            })
            .ok_or(ResultInspectionRefusal::ResponseExtent)?;
        self.charge(extent)?;
        Ok(metadata
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect())
    }
    fn field(
        &mut self,
        index: usize,
        field: &Field,
    ) -> Result<ResultInspectionField, ResultInspectionRefusal> {
        // Bound retained native structure before formatting a nested datatype. The
        // independent final counting serializer checks actual escaped wire extent.
        self.charge(field.size().saturating_add(256))?;
        let datatype_json = serde_json::to_string(field.data_type())
            .map_err(|_| ResultInspectionRefusal::ResponseExtent)?;
        if datatype_json.len() > DOCUMENT_BYTES {
            return Err(ResultInspectionRefusal::ResponseExtent);
        }
        Ok(ResultInspectionField {
            index,
            name: field.name().clone(),
            datatype_json,
            nullable: field.is_nullable(),
            metadata: field
                .metadata()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        })
    }
}

struct EncodedExtent(usize);
impl Write for EncodedExtent {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|size| *size <= DOCUMENT_BYTES)
            .ok_or_else(|| std::io::Error::other("readable result extent"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn bound_document(document: ResultInspectionDocument) -> ResultInspectionDocument {
    // Include the standalone envelope overhead in the original encoding's final safeguard.
    let mut encoded = EncodedExtent(64);
    if serde_json::to_writer(&mut encoded, &document).is_err() {
        refused(ResultInspectionRefusal::ResponseExtent)
    } else {
        document
    }
}

fn cell(array: &ArrayRef, row: usize) -> Result<ResultInspectionCell, WorkflowError> {
    if row >= array.len() {
        return Err(contract("result inspection row exceeds admitted batch"));
    }
    if array.is_null(row) {
        return Ok(ResultInspectionCell::Null);
    }
    macro_rules! number {
        ($ty:ty, $variant:ident, $convert:expr) => {{
            let values = array.as_any().downcast_ref::<$ty>().ok_or_else(|| {
                contract("result inspection storage differs from admitted datatype")
            })?;
            ResultInspectionCell::$variant {
                value: $convert(values.value(row)),
            }
        }};
    }
    let value = match array.data_type() {
        DataType::Null => ResultInspectionCell::Null,
        DataType::Boolean => number!(BooleanArray, Boolean, std::convert::identity),
        DataType::Int8 => number!(Int8Array, Integer, i64::from),
        DataType::Int16 => number!(Int16Array, Integer, i64::from),
        DataType::Int32 => number!(Int32Array, Integer, i64::from),
        DataType::Int64 => number!(Int64Array, Integer, std::convert::identity),
        DataType::UInt8 => number!(UInt8Array, Unsigned, u64::from),
        DataType::UInt16 => number!(UInt16Array, Unsigned, u64::from),
        DataType::UInt32 => number!(UInt32Array, Unsigned, u64::from),
        DataType::UInt64 => number!(UInt64Array, Unsigned, std::convert::identity),
        DataType::Float32 => {
            let values = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| contract("result float32 storage differs"))?;
            let value = values.value(row);
            ResultInspectionCell::Float32 {
                bits: value.to_bits(),
                finite_value: value.is_finite().then_some(value),
                class: ResultFloatClass::of(f64::from(value)),
            }
        }
        DataType::Float64 => {
            let values = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| contract("result float64 storage differs"))?;
            let value = values.value(row);
            ResultInspectionCell::Float64 {
                bits: value.to_bits(),
                finite_value: value.is_finite().then_some(value),
                class: ResultFloatClass::of(value),
            }
        }
        DataType::Utf8 | DataType::LargeUtf8 => {
            let text = if let Some(values) = array.as_any().downcast_ref::<StringArray>() {
                values.value(row)
            } else if let Some(values) = array.as_any().downcast_ref::<LargeStringArray>() {
                values.value(row)
            } else {
                return Err(contract("result UTF-8 storage differs"));
            };
            if text.len() > TEXT_BYTES {
                ResultInspectionCell::Unavailable {
                    reason: ResultCellUnavailable::ValueExtent,
                }
            } else {
                ResultInspectionCell::Text { value: text.into() }
            }
        }
        DataType::Binary | DataType::LargeBinary | DataType::FixedSizeBinary(_) => {
            let bytes = if let Some(values) = array.as_any().downcast_ref::<BinaryArray>() {
                values.value(row)
            } else if let Some(values) = array.as_any().downcast_ref::<LargeBinaryArray>() {
                values.value(row)
            } else if let Some(values) = array.as_any().downcast_ref::<FixedSizeBinaryArray>() {
                values.value(row)
            } else {
                return Err(contract("result binary storage differs"));
            };
            if bytes.len() > TEXT_BYTES / 2 {
                ResultInspectionCell::Unavailable {
                    reason: ResultCellUnavailable::ValueExtent,
                }
            } else {
                let mut hex = String::with_capacity(bytes.len() * 2);
                use std::fmt::Write as _;
                for byte in bytes {
                    write!(&mut hex, "{byte:02x}")
                        .map_err(|_| contract("result byte formatting refused"))?;
                }
                ResultInspectionCell::Binary { hex }
            }
        }
        _ => ResultInspectionCell::Unavailable {
            reason: ResultCellUnavailable::UnsupportedStorage,
        },
    };
    // Strings were bounded before copying. The fixed native owner prepays the
    // maximum 16 * (8 selected + 32 context) complete scalar observations.
    Ok(value)
}

fn row_values(
    batch: &RecordBatch,
    row: usize,
    indices: &[usize],
) -> Result<Vec<ResultInspectionCell>, WorkflowError> {
    indices
        .iter()
        .map(|index| cell(batch.column(*index), row))
        .collect()
}

fn preflight(
    request: &ResultInspectionRequest,
    schema: &Schema,
) -> Option<ResultInspectionRefusal> {
    if request.start > request.end
        || [&request.run, &request.attempt, &request.relation]
            .iter()
            .any(|value| {
                value.is_empty() || value.len() > pse_schema::catalog::substrate::METADATA_ID_BYTES
            })
        || request
            .expected_manifest
            .as_ref()
            .is_some_and(|value| value.len() > 128)
    {
        return Some(ResultInspectionRefusal::Selection);
    }
    match &request.view {
        ResultInspectionView::Schema { offset, limit }
            if *limit == 0 || *limit > 16 || *offset > schema.fields().len() =>
        {
            Some(ResultInspectionRefusal::Selection)
        }
        ResultInspectionView::Values {
            fields,
            maximum_rows,
        } => {
            if fields.is_empty()
                || fields.len() > 8
                || *maximum_rows == 0
                || *maximum_rows > 16
                || fields.iter().collect::<BTreeSet<_>>().len() != fields.len()
            {
                return Some(ResultInspectionRefusal::Selection);
            }
            fields
                .iter()
                .any(|name| schema.index_of(name).is_err())
                .then_some(ResultInspectionRefusal::UnknownField)
        }
        _ => None,
    }
}

impl Runtime {
    /// Observe one protected exact result without exporting its full Arrow data.
    ///
    /// # Errors
    /// Original protected acquisition, validation, cancellation or allocation fails.
    /// Finite-view limits and stale expected manifests produce a typed Refused document.
    pub async fn inspect_result(
        &self,
        request: ResultInspectionRequest,
        cancel: CancellationToken,
    ) -> Result<ResultInspectionDocument, WorkflowError> {
        self.scope_activity(async {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let spec = self
                .registry()
                .relation(&request.relation)
                .ok_or_else(|| contract("unknown result inspection relation"))?;
            let schema = pse_schema::arrow::relation_schema_ref(self.registry(), spec)
                .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))?;
            if let Some(reason) = preflight(&request, schema.as_ref()) {
                return Ok(refused(reason));
            }
            let owner = MemoryConsumer::new("canonical:readable-result-observation")
                .register(&self.shared.pool());
            // Covers retained descriptors/rows and escaped serialization scratch independently
            // from the protected reader's existing original-batch/metadata owners.
            owner
                .try_grow(16 * DOCUMENT_BYTES)
                .map_err(pse_engine::EngineError::from)?;
            let owner = Arc::new(owner);
            let mut budget = Budget {
                remaining: DOCUMENT_BYTES,
            };
            let schema_metadata = match budget.metadata(schema.metadata()) {
                Ok(metadata) => metadata,
                Err(reason) => return Ok(refused(reason)),
            };
            let (selected, context) = match &request.view {
                ResultInspectionView::Schema { .. } => (Vec::new(), Vec::new()),
                ResultInspectionView::Values { fields, .. } => {
                    let selected = fields
                        .iter()
                        .map(|field| {
                            schema
                                .index_of(field)
                                .map_err(|_| contract("selected result field disappeared"))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let context = spec
                        .columns
                        .iter()
                        .enumerate()
                        .filter(|(index, field)| {
                            !selected.contains(index)
                                && field.result_selection_role()
                                    != Some(pse_schema::model::ResultSelectionRole::Value)
                        })
                        .map(|(index, _)| index)
                        .collect::<Vec<_>>();
                    if context.len() > CONTEXT_FIELDS {
                        return Ok(refused(ResultInspectionRefusal::ContextExtent));
                    }
                    (selected, context)
                }
            };
            let descriptors = |indices: &[usize], budget: &mut Budget| {
                indices
                    .iter()
                    .map(|index| budget.field(*index, schema.field(*index)))
                    .collect::<Result<Vec<_>, _>>()
            };
            let selected_fields = match descriptors(&selected, &mut budget) {
                Ok(fields) => fields,
                Err(reason) => return Ok(refused(reason)),
            };
            let context_fields = match descriptors(&context, &mut budget) {
                Ok(fields) => fields,
                Err(_) => return Ok(refused(ResultInspectionRefusal::ContextExtent)),
            };
            let mut reader = self
                .results(
                    &request.run,
                    &request.attempt,
                    &request.relation,
                    request.start,
                    request.end,
                    cancel.clone(),
                )
                .await?;
            let outcome = async {
                let manifest = reader.selection().manifest().digest.clone();
                if request
                    .expected_manifest
                    .as_ref()
                    .is_some_and(|expected| expected != &manifest)
                {
                    return Ok(refused(ResultInspectionRefusal::ManifestChanged));
                }
                let selection = reader.pack_selection(PackScientificState::Unknown)?;
                let snapshot = reader.snapshot();
                let recorded_rows = snapshot.blocks().iter().try_fold(0u64, |count, block| {
                    count
                        .checked_add(
                            request
                                .end
                                .min(block.end)
                                .saturating_sub(request.start.max(block.start)),
                        )
                        .ok_or_else(|| contract("result inspection population overflow"))
                })?;
                match request.view {
                    ResultInspectionView::Schema { offset, limit } => {
                        let end = offset.saturating_add(limit).min(schema.fields().len());
                        let fields =
                            match descriptors(&(offset..end).collect::<Vec<_>>(), &mut budget) {
                                Ok(fields) => fields,
                                Err(reason) => return Ok(refused(reason)),
                            };
                        Ok(bound_document(ResultInspectionDocument::Schema {
                            selection,
                            manifest,
                            schema_metadata,
                            field_count: schema.fields().len(),
                            offset,
                            fields,
                            next: (end < schema.fields().len()).then_some(end),
                            recorded_rows,
                            owner: Some(owner),
                        }))
                    }
                    ResultInspectionView::Values { maximum_rows, .. } => {
                        let mut rows = Vec::new();
                        let mut next_row = None;
                        for block in snapshot.blocks() {
                            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                            let start = request.start.max(block.start);
                            let end = request.end.min(block.end);
                            if start >= end {
                                continue;
                            }
                            if rows.len() == maximum_rows {
                                next_row = Some(start);
                                break;
                            }
                            let batch = reader.next_batch().await?.ok_or_else(|| {
                                contract("result inspection frozen block is absent")
                            })?;
                            if batch.num_rows() as u64 != end - start {
                                return Err(contract(
                                    "result inspection source coordinates differ",
                                ));
                            }
                            for row in 0..batch.num_rows() {
                                if rows.len() == maximum_rows {
                                    next_row = Some(start + row as u64);
                                    break;
                                }
                                let values = row_values(&batch, row, &selected)?;
                                let context = row_values(&batch, row, &context)?;
                                let context_complete = !context.iter().any(|value| {
                                    matches!(value, ResultInspectionCell::Unavailable { .. })
                                });
                                rows.push(ResultInspectionRow {
                                    source_row: start + row as u64,
                                    values,
                                    context,
                                    context_complete,
                                });
                            }
                            if next_row.is_some() {
                                break;
                            }
                        }
                        let readable_complete = rows.iter().all(|row| {
                            row.context_complete
                                && !row.values.iter().any(|value| {
                                    matches!(value, ResultInspectionCell::Unavailable { .. })
                                })
                        });
                        Ok(bound_document(ResultInspectionDocument::Values {
                            selection,
                            manifest,
                            schema_metadata,
                            selected_fields,
                            context_fields,
                            recorded_rows,
                            rows,
                            next_row,
                            readable_complete,
                            owner: Some(owner),
                        }))
                    }
                }
            }
            .await;
            reader.close();
            outcome
        })
        .await
    }
}

#[cfg(test)]
mod result_inspection_unit {
    use super::*;

    #[test]
    fn readable_cells_keep_original_ieee_bits_null_and_complete_text() {
        let nan = f64::from_bits(0x7ff8_0000_0000_0042);
        let floats: ArrayRef = Arc::new(Float64Array::from(vec![
            Some(-0.0),
            Some(nan),
            Some(f64::INFINITY),
            None,
        ]));
        assert_eq!(
            cell(&floats, 0).unwrap(),
            ResultInspectionCell::Float64 {
                bits: (-0.0f64).to_bits(),
                finite_value: Some(-0.0),
                class: ResultFloatClass::Finite
            }
        );
        assert_eq!(
            cell(&floats, 1).unwrap(),
            ResultInspectionCell::Float64 {
                bits: nan.to_bits(),
                finite_value: None,
                class: ResultFloatClass::Nan
            }
        );
        assert_eq!(
            cell(&floats, 2).unwrap(),
            ResultInspectionCell::Float64 {
                bits: f64::INFINITY.to_bits(),
                finite_value: None,
                class: ResultFloatClass::PositiveInfinity
            }
        );
        assert_eq!(cell(&floats, 3).unwrap(), ResultInspectionCell::Null);
        let integers: ArrayRef = Arc::new(UInt64Array::from(vec![u64::MAX]));
        assert_eq!(
            cell(&integers, 0).unwrap(),
            ResultInspectionCell::Unsigned { value: u64::MAX }
        );
        let texts: ArrayRef = Arc::new(StringArray::from(vec![
            "complete",
            &"x".repeat(TEXT_BYTES + 1),
        ]));
        assert_eq!(
            cell(&texts, 0).unwrap(),
            ResultInspectionCell::Text {
                value: "complete".into()
            }
        );
        assert_eq!(
            cell(&texts, 1).unwrap(),
            ResultInspectionCell::Unavailable {
                reason: ResultCellUnavailable::ValueExtent
            }
        );
        let nested: ArrayRef =
            Arc::new(datafusion::arrow::array::ListArray::from_iter_primitive::<
                datafusion::arrow::datatypes::Int32Type,
                _,
                _,
            >(vec![Some(vec![Some(1), Some(2)])]));
        assert_eq!(
            cell(&nested, 0).unwrap(),
            ResultInspectionCell::Unavailable {
                reason: ResultCellUnavailable::UnsupportedStorage
            }
        );
    }

    #[test]
    fn field_descriptor_is_the_original_arrow_type_and_metadata() {
        let nested = Field::new("child", DataType::Float64, true).with_metadata(
            std::collections::HashMap::from([(
                "original.child".into(),
                "kelvin/basis/context".into(),
            )]),
        );
        let datatype = DataType::Struct(vec![Arc::new(nested)].into());
        let field = Field::new("physical", datatype.clone(), false).with_metadata(
            std::collections::HashMap::from([("original.unit".into(), "K".into())]),
        );
        let projected = Budget {
            remaining: DOCUMENT_BYTES,
        }
        .field(5, &field)
        .unwrap();
        assert_eq!(projected.index, 5);
        assert_eq!(projected.name, "physical");
        assert_eq!(
            serde_json::from_str::<DataType>(&projected.datatype_json).unwrap(),
            datatype
        );
        assert_eq!(projected.metadata["original.unit"], "K");
        assert!(!projected.nullable);
    }

    #[test]
    fn encoded_extent_is_a_typed_refusal_not_truncated_required_content() {
        let original = ResultInspectionDocument::Refused {
            reason: ResultInspectionRefusal::ContextExtent,
            guidance: "x".repeat(DOCUMENT_BYTES + 1),
        };
        assert!(matches!(
            bound_document(original),
            ResultInspectionDocument::Refused {
                reason: ResultInspectionRefusal::ResponseExtent,
                ..
            }
        ));
        let mut budget = Budget {
            remaining: DOCUMENT_BYTES,
        };
        let field = Field::new("huge", DataType::Utf8, true).with_metadata(
            std::collections::HashMap::from([("original".into(), "x".repeat(DOCUMENT_BYTES))]),
        );
        assert!(matches!(
            budget.field(0, &field),
            Err(ResultInspectionRefusal::ResponseExtent)
        ));
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_result_inspection_server_unit;

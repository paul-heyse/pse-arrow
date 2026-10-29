// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded readable observations over the original protected scientific result owner.

use super::{PackResultSelection, PackScientificState, RetainedDataServices, Runtime, WorkflowError, contract};
use datafusion::arrow::{
    array::{
        Array, ArrayRef, BinaryArray, BooleanArray, FixedSizeBinaryArray, Float32Array,
        Float64Array, Int8Array, Int16Array, Int32Array, Int64Array, LargeBinaryArray,
        LargeStringArray, ListArray, StringArray, StructArray, UInt8Array, UInt16Array,
        UInt32Array, UInt64Array,
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
/// Schema example identities are illustrative, not live eligible result references.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = result_request_example())]
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
    /// Original-window/path-bound continuation; never a payload permission.
    #[serde(default)]
    pub cursor: Option<ResultInspectionCursor>,
}
fn result_request_example() -> ResultInspectionRequest {
    ResultInspectionRequest {
        run: "illustrative-original-run".into(),
        attempt: "illustrative-terminal-attempt".into(),
        relation: "runtime.study_outcomes".into(),
        start: 0,
        end: 1,
        expected_manifest: None,
        cursor: None,
        view: ResultInspectionView::Values {
            fields: vec![ArrowFieldPath {
                field: "start".into(),
                steps: vec![ArrowFieldStep::StructField {
                    name: "kind".into(),
                }],
            }],
            maximum_rows: 1,
        },
    }
}

/// Exact typed Arrow navigation; an empty step list selects the original root field.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct ArrowFieldPath {
    /// Exact original root column name.
    pub field: String,
    /// At most sixteen native struct/list operations.
    pub steps: Vec<ArrowFieldStep>,
}
impl From<&str> for ArrowFieldPath {
    fn from(field: &str) -> Self {
        Self {
            field: field.into(),
            steps: Vec::new(),
        }
    }
}
impl From<String> for ArrowFieldPath {
    fn from(field: String) -> Self {
        Self {
            field,
            steps: Vec::new(),
        }
    }
}
/// A native array operation, not a JSON pointer or a string expression.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ArrowFieldStep {
    /// Exact declared struct child.
    StructField {
        /// Original exact child name.
        name: String,
    },
    /// Selected element relative to each original row's list offsets.
    ListElement {
        /// Zero-based position within each original valid list.
        index: u64,
    },
}
/// Immutable request coordinates retained across pages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultInspectionBasis {
    /// Original run.
    pub run: String,
    /// Original terminal attempt.
    pub attempt: String,
    /// Original relation.
    pub relation: String,
    /// Complete requested row window.
    pub start: u64,
    /// Exclusive original window end.
    pub end: u64,
    /// Original view/path/page allowance, unchanged by continuation.
    pub view: ResultInspectionView,
}
/// Original governing permission and retained physical interpretation, independent of selected-product usability.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultInspectionOriginal {
    /// Completion-owned original scientific permission. Missing original fact is unknown.
    pub governing_permission: Option<bool>,
    /// Original physical source closure availability, never a current-context substitution.
    pub physical: ResultPhysicalContext,
    /// Exact read and source protection custody retained through FFI serialization.
    #[serde(skip)]
    #[schemars(skip)]
    _custody: Option<Arc<ResultInspectionCustody>>,
}
#[derive(Debug)]
struct ResultInspectionCustody {
    _read: pse_operations::canonical_results::ResultRead,
    _physical: Option<super::PhysicalContext>,
}
/// Original-context locator state, also bound into every immutable continuation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultPhysicalContext {
    /// Original rows were protected and admitted through the physical owner.
    Available {
        /// Original full physical context identity.
        identity: pse_ids::ContentHash,
        /// Exact retained revision closure digest.
        closure: String,
    },
    /// Original closure cannot supply readable physical meaning; raw result values remain readable.
    Unavailable {
        /// Explicit original-closure failure; no alternate context was consulted.
        reason: String,
    },
}

/// Store/selection-bound continuation; each call reacquires original protection.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultInspectionCursor {
    /// Original canonical store authority; coordinates alone never cross stores.
    pub issuing_store: String,
    /// Exact original protected selection, including issuing authority.
    pub selection: PackResultSelection,
    /// Exact closure digest.
    pub manifest: String,
    /// Exact admitted registry relation contract, independent of its numeric version.
    pub schema_fingerprint: String,
    /// Original request identity.
    pub basis: ResultInspectionBasis,
    /// Original physical context and exact closure availability.
    pub physical: ResultPhysicalContext,
    /// Next original row or descriptor/metadata position.
    pub position: u64,
}
impl ResultInspectionRequest {
    fn basis(&self) -> ResultInspectionBasis {
        ResultInspectionBasis {
            run: self.run.clone(),
            attempt: self.attempt.clone(),
            relation: self.relation.clone(),
            start: self.start,
            end: self.end,
            view: self.view.clone(),
        }
    }
}

/// Physical column selection; this does not reinterpret a scientific output predicate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultInspectionView {
    /// Page the original field inventory without decoding any result batch.
    Schema {
        /// Zero-based original field position.
        offset: usize,
        /// At most sixteen original fields.
        limit: usize,
    },
    /// Immediate native children of one exact field, without result data batches.
    Children {
        /// Original parent field.
        path: ArrowFieldPath,
        /// Original immediate-child offset.
        offset: usize,
        /// One to sixteen children.
        limit: usize,
    },
    /// Exact metadata entries of one original field, without result data batches.
    Metadata {
        /// Original field owning the metadata.
        path: ArrowFieldPath,
        /// Original sorted metadata-entry offset.
        offset: usize,
        /// One to sixteen entries.
        limit: usize,
    },
    /// Read a finite physical declaration page from the original retained context.
    Physical {
        /// Same physical-owner query used by direct Python and MCP inspection.
        request: super::PhysicalInspectionRequest,
    },
    /// Read selected original scalar fields together with all declared non-value context.
    Values {
        /// One to eight unique exact original root/child paths.
        fields: Vec<ArrowFieldPath>,
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
    /// Exact original field location.
    pub path: ArrowFieldPath,
    /// Shallow storage descriptor; exact nested detail uses the same native reader.
    pub storage: ResultInspectionStorage,
    /// Original outer field nullability.
    pub nullable: bool,
    /// Inline original metadata, including quantity and extension semantics.
    /// Explicit omissions are available through the Metadata view at this path.
    pub metadata: BTreeMap<String, String>,
    /// Original entries deferred without truncation to exact detail reads.
    pub metadata_omitted: Vec<ResultMetadataOmission>,
    /// Inline original ancestor metadata in path order, with explicit omissions below.
    pub ancestor_metadata: Vec<BTreeMap<String, String>>,
    /// Deferred ancestor entries in the same path order as ancestor_metadata.
    pub ancestor_metadata_omitted: Vec<Vec<ResultMetadataOmission>>,
}
/// One original metadata entry deliberately absent from a shallow descriptor.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResultMetadataOmission {
    /// Original key; the Metadata view reads this field's original entries.
    pub key: String,
    /// Complete original UTF-8 value extent, before JSON escaping.
    pub value_bytes: usize,
}
/// Compact native storage node; recursive field trees are not repeated in ordinary answers.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultInspectionStorage {
    /// Exact original scalar datatype using Arrow's JSON encoding.
    Scalar {
        /// Arrow's exact original datatype encoding.
        datatype_json: String,
    },
    /// Immediate children can be selected through the Children view.
    Struct {
        /// Complete immediate child population.
        child_count: usize,
    },
    /// Ordinary Arrow List; child detail uses ListElement.
    List {
        /// Original list element field name.
        element_name: String,
    },
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
    /// An ancestor container is null; its underlying child buffer was not exposed.
    ParentNull {
        /// Zero-based path step whose parent is null.
        depth: usize,
    },
    /// A valid list lacks the selected element; empty and nonempty lists retain their lengths.
    MissingElement {
        /// Zero-based list path step.
        depth: usize,
        /// Original valid list length, including zero.
        length: usize,
    },
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
    /// Continuation belongs to another issuer/selection/window/path or original view.
    CursorChanged,
}

/// Original protected inspection outcome. Copying does not upgrade scientific usability.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResultInspectionDocument {
    /// A page of the admitted registry field inventory, without validating stored Arrow rows.
    Schema {
        /// Exact original source/terminal/range and available completion-owned product permission.
        selection: PackResultSelection,
        /// Original governing permission and physical closure; selected-product usability is independent.
        original: ResultInspectionOriginal,
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
        /// Exact next page while retaining the complete original request.
        next_cursor: Option<ResultInspectionCursor>,
        /// Recorded rows intersecting the selected range; holes are not invented.
        recorded_rows: u64,
        /// Native allocation ownership follows the response until FFI serialization completes.
        #[serde(skip)]
        #[schemars(skip)]
        owner: Option<Arc<MemoryReservation>>,
    },
    /// Readable bounded original scalar observations; exact IPC remains the bulk interface.
    Values {
        /// Exact original source/terminal/range and available completion-owned product permission.
        selection: PackResultSelection,
        /// Original governing permission and physical closure; selected-product usability is independent.
        original: ResultInspectionOriginal,
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
        /// Informational next original coordinate; use next_cursor with the unchanged original window.
        next_row: Option<u64>,
        /// Exact row continuation; retain the original request window and view.
        next_cursor: Option<ResultInspectionCursor>,
        /// Every returned selected and context cell was representable, independent of population/scientific usability.
        readable_complete: bool,
        /// Native allocation ownership follows the response until FFI serialization completes.
        #[serde(skip)]
        #[schemars(skip)]
        owner: Option<Arc<MemoryReservation>>,
    },
    /// Exact metadata page from one admitted original field.
    Metadata {
        /// Original protected result selection.
        selection: PackResultSelection,
        /// Original governing permission and physical closure; selected-product usability is independent.
        original: ResultInspectionOriginal,
        /// Original closure digest.
        manifest: String,
        /// Original selected field.
        path: ArrowFieldPath,
        /// Complete entry population.
        entry_count: usize,
        /// Original entry offset.
        offset: usize,
        /// Complete selected metadata entries, never truncated strings.
        entries: BTreeMap<String, String>,
        /// Exact next page, if any.
        next_cursor: Option<ResultInspectionCursor>,
        /// Native retained custody through serialization.
        #[serde(skip)]
        #[schemars(skip)]
        owner: Option<Arc<MemoryReservation>>,
    },
    /// A bounded physical-owner page tied to this exact protected result.
    Physical {
        /// Exact original selected result; selected product usability remains unknown.
        selection: PackResultSelection,
        /// Original aggregate permission and physical closure locator.
        original: ResultInspectionOriginal,
        /// Exact result closure digest.
        manifest: String,
        /// Complete selected physical page; absent only when the original closure is unavailable.
        page: Option<super::PhysicalInspectionDocument>,
        /// Original-query-bound next physical declaration page.
        next_cursor: Option<ResultInspectionCursor>,
        /// Allocation custody through serialization.
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
        self.field_at(
            index,
            field,
            ArrowFieldPath {
                field: field.name().clone(),
                steps: Vec::new(),
            },
            &[],
        )
    }
    fn field_at(
        &mut self,
        index: usize,
        field: &Field,
        path: ArrowFieldPath,
        ancestors: &[&Field],
    ) -> Result<ResultInspectionField, ResultInspectionRefusal> {
        self.charge(field.name().len().saturating_add(256))?;
        let storage = match field.data_type() {
            DataType::Struct(children) => ResultInspectionStorage::Struct {
                child_count: children.len(),
            },
            DataType::List(element) => ResultInspectionStorage::List {
                element_name: element.name().clone(),
            },
            datatype => {
                let datatype_json = serde_json::to_string(datatype)
                    .map_err(|_| ResultInspectionRefusal::ResponseExtent)?;
                self.charge(datatype_json.len())?;
                ResultInspectionStorage::Scalar { datatype_json }
            }
        };
        let (metadata, metadata_omitted) = self.field_metadata(field)?;
        let (ancestor_metadata, ancestor_metadata_omitted) = ancestors
            .iter()
            .map(|field| self.field_metadata(field))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        Ok(ResultInspectionField {
            index,
            name: field.name().clone(),
            path,
            storage,
            nullable: field.is_nullable(),
            metadata,
            metadata_omitted,
            ancestor_metadata,
            ancestor_metadata_omitted,
        })
    }
    fn field_metadata(
        &mut self,
        field: &Field,
    ) -> Result<(BTreeMap<String, String>, Vec<ResultMetadataOmission>), ResultInspectionRefusal>
    {
        let composite = matches!(field.data_type(), DataType::Struct(_) | DataType::List(_));
        let mut metadata = BTreeMap::new();
        let mut omitted = Vec::new();
        for (key, value) in field.metadata() {
            if composite && key == pse_schema::arrow::KEY_LOGICAL_TYPE {
                // Composite logical identities encode the complete recursive
                // Arrow tree. Child/path pages retain its exact native structure.
                self.charge(key.len().saturating_add(128))?;
                omitted.push(ResultMetadataOmission {
                    key: key.clone(),
                    value_bytes: value.len(),
                });
            } else {
                self.charge(key.len().saturating_add(value.len()).saturating_add(128))?;
                metadata.insert(key.clone(), value.clone());
            }
        }
        Ok((metadata, omitted))
    }
}

fn path_fields<'a>(schema: &'a Schema, path: &ArrowFieldPath) -> Option<(usize, Vec<&'a Field>)> {
    let root = schema.index_of(&path.field).ok()?;
    let mut fields = vec![schema.field(root)];
    for step in &path.steps {
        let field = *fields.last()?;
        let child = match (field.data_type(), step) {
            (DataType::Struct(children), ArrowFieldStep::StructField { name }) => {
                children.iter().find(|field| field.name() == name)?.as_ref()
            }
            (DataType::List(element), ArrowFieldStep::ListElement { .. }) => element.as_ref(),
            _ => return None,
        };
        fields.push(child);
    }
    Some((root, fields))
}

fn selected_cell(
    array: &ArrayRef,
    row: usize,
    steps: &[ArrowFieldStep],
    depth: usize,
) -> Result<ResultInspectionCell, WorkflowError> {
    if steps.is_empty() {
        return cell(array, row);
    }
    if row >= array.len() {
        return Err(contract("nested result coordinate exceeds admitted array"));
    }
    if array.is_null(row) {
        return Ok(ResultInspectionCell::ParentNull { depth });
    }
    match (&steps[0], array.data_type()) {
        (ArrowFieldStep::StructField { name }, DataType::Struct(_)) => {
            let values = array
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or_else(|| contract("admitted struct storage differs"))?;
            let child = values
                .column_by_name(name)
                .ok_or_else(|| contract("selected struct child disappeared"))?;
            selected_cell(child, row, &steps[1..], depth + 1)
        }
        (ArrowFieldStep::ListElement { index }, DataType::List(_)) => {
            let values = array
                .as_any()
                .downcast_ref::<ListArray>()
                .ok_or_else(|| contract("admitted list storage differs"))?;
            let offsets = values.value_offsets();
            let start =
                usize::try_from(offsets[row]).map_err(|_| contract("negative list offset"))?;
            let end =
                usize::try_from(offsets[row + 1]).map_err(|_| contract("negative list offset"))?;
            let length = end.saturating_sub(start);
            let Ok(index) = usize::try_from(*index) else {
                return Ok(ResultInspectionCell::MissingElement { depth, length });
            };
            if index >= length {
                return Ok(ResultInspectionCell::MissingElement { depth, length });
            }
            selected_cell(values.values(), start + index, &steps[1..], depth + 1)
        }
        _ => Ok(ResultInspectionCell::Unavailable {
            reason: ResultCellUnavailable::UnsupportedStorage,
        }),
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
    let valid_path = |path: &ArrowFieldPath| {
        path.steps.len() <= 16
            && path.field.len() <= 1024
            && path.steps.iter().all(|step| match step {
                ArrowFieldStep::StructField { name } => name.len() <= 1024,
                _ => true,
            })
    };
    match &request.view {
        ResultInspectionView::Physical { request } => {
            (!request.valid_bounds()).then_some(ResultInspectionRefusal::Selection)
        }
        ResultInspectionView::Schema { offset, limit } => {
            (*limit == 0 || *limit > 16 || *offset > schema.fields().len())
                .then_some(ResultInspectionRefusal::Selection)
        }
        ResultInspectionView::Children { path, limit, .. }
        | ResultInspectionView::Metadata { path, limit, .. } => {
            if *limit == 0 || *limit > 16 || !valid_path(path) {
                Some(ResultInspectionRefusal::Selection)
            } else {
                path_fields(schema, path)
                    .is_none()
                    .then_some(ResultInspectionRefusal::UnknownField)
            }
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
                || fields.iter().any(|path| !valid_path(path))
            {
                Some(ResultInspectionRefusal::Selection)
            } else {
                fields
                    .iter()
                    .any(|path| path_fields(schema, path).is_none())
                    .then_some(ResultInspectionRefusal::UnknownField)
            }
        }
    }
}

impl Runtime {
    /// Inspect the exact original result through retained-data capabilities.
    pub async fn inspect_result(
        &self,
        request: ResultInspectionRequest,
        cancel: CancellationToken,
    ) -> Result<ResultInspectionDocument, WorkflowError> {
        self.retained_data_services().inspect_result(request, cancel).await
    }
}
impl RetainedDataServices {
    /// Observe one protected exact result through native shallow/path pages.
    /// # Errors
    /// Original protection, full-batch validation, cancellation and allocation failures propagate.
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
            let basis = request.basis();
            let issuer = self.canonical_store()?.issuing_store()?.to_owned();
            let schema_fingerprint = schema
                .metadata()
                .get(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT)
                .cloned()
                .ok_or_else(|| contract("registered result schema fingerprint absent"))?;
            if request.cursor.as_ref().is_some_and(|cursor| {
                cursor.basis != basis
                    || cursor.issuing_store != issuer
                    || cursor.schema_fingerprint != schema_fingerprint
            }) {
                return Ok(refused(ResultInspectionRefusal::CursorChanged));
            }
            let owner = MemoryConsumer::new("canonical:readable-result-observation")
                .register(&self.pool());
            owner
                .try_grow(16 * DOCUMENT_BYTES)
                .map_err(pse_engine::EngineError::from)?;
            let owner = Arc::new(owner);
            let mut budget = Budget {
                remaining: DOCUMENT_BYTES,
            };
            let schema_metadata = match budget.metadata(schema.metadata()) {
                Ok(value) => value,
                Err(reason) => return Ok(refused(reason)),
            };
            let descriptor = |path: &ArrowFieldPath, budget: &mut Budget| {
                let (index, chain) =
                    path_fields(&schema, path).ok_or(ResultInspectionRefusal::UnknownField)?;
                let (field, ancestors) = chain
                    .split_last()
                    .ok_or(ResultInspectionRefusal::UnknownField)?;
                budget.field_at(index, field, path.clone(), ancestors)
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
                    .is_some_and(|value| value != &manifest)
                {
                    return Ok(refused(ResultInspectionRefusal::ManifestChanged));
                }
                use pse_model::generated::enums::DerivedQuantity;
                use pse_relations::generated::runtime::{parameter_covariances, parameter_intervals, profile_points};
                let quantities: &[DerivedQuantity] = if spec.id == parameter_covariances::RELATION_ID { &[DerivedQuantity::ParameterCovariance] }
                    else if spec.id == profile_points::RELATION_ID { &[DerivedQuantity::ProfileInterval] }
                    else if spec.id == parameter_intervals::RELATION_ID { &[DerivedQuantity::WaldInterval, DerivedQuantity::ProfileInterval] }
                    else { &[] };
                let mut usability = PackScientificState::Unknown;
                for quantity in quantities {
                    if let Some(permitted) = super::durable::result_product_permission(self.canonical_store()?, &self.pool(), reader.selection(), *quantity).await? {
                        if !permitted { usability = PackScientificState::Unusable; break; }
                        usability = PackScientificState::Usable;
                    }
                }
                let selection = reader.pack_selection(usability)?;
                if let Some(cursor) = &request.cursor
                    && (cursor.manifest != manifest
                        || cursor.selection.usability != selection.usability
                        || reader.validate_pack_selection(&cursor.selection).is_err())
                {
                    return Ok(refused(ResultInspectionRefusal::CursorChanged));
                }
                let (physical, physical_info) = match self.original_result_physical(reader.selection(), &cancel).await {
                    Ok((context, closure)) => {
                        let info = ResultPhysicalContext::Available { identity: context.identity(), closure };
                        (Some(context), info)
                    }
                    Err(error) => {
                        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                        tracing::debug!(error = %error, "original result physical closure unavailable");
                        let reason = match &error {
                            WorkflowError::Input(reason) | WorkflowError::Integrity(reason) if reason.len() <= TEXT_BYTES => reason.clone(),
                            _ => "original retained physical closure failed native admission".into(),
                        };
                        (None, ResultPhysicalContext::Unavailable { reason })
                    }
                };
                if request.cursor.as_ref().is_some_and(|cursor| cursor.physical != physical_info) {
                    return Ok(refused(ResultInspectionRefusal::CursorChanged));
                }
                let governing_permission = super::durable::result_governing_permission(self.canonical_store()?, &self.pool(), reader.selection()).await?;
                let original = ResultInspectionOriginal { governing_permission, physical: physical_info.clone(), _custody: Some(Arc::new(ResultInspectionCustody { _read: reader.selection().clone(), _physical: physical.clone() })) };
                let next_cursor = |position: u64| ResultInspectionCursor {
                    issuing_store: issuer.clone(),
                    selection: selection.clone(),
                    manifest: manifest.clone(),
                    schema_fingerprint: schema_fingerprint.clone(),
                    basis: basis.clone(),
                    physical: physical_info.clone(),
                    position,
                };
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
                match &request.view {
                    ResultInspectionView::Physical { request: physical_request } => {
                        let mut query = physical_request.clone();
                        if let Some(cursor) = &request.cursor { query.offset = usize::try_from(cursor.position).map_err(|_| contract("physical cursor exceeds platform"))?; }
                        let page = physical.as_ref().map(|context| context.describe(&query)).transpose()?;
                        let next = match &page { Some(super::PhysicalInspectionDocument::Page { next, .. }) => *next, _ => None };
                        Ok(bound_document(ResultInspectionDocument::Physical { selection: selection.clone(), original: original.clone(), manifest: manifest.clone(), page, next_cursor: next.map(|position| next_cursor(position as u64)), owner: Some(owner) }))
                    }
                    ResultInspectionView::Schema { offset, limit }
                    | ResultInspectionView::Children { offset, limit, .. } => {
                        let offset = match request.cursor.as_ref() {
                            Some(cursor) => usize::try_from(cursor.position)
                                .map_err(|_| contract("descriptor cursor exceeds platform"))?,
                            None => *offset,
                        };
                        let paths =
                            if let ResultInspectionView::Children { path, .. } = &request.view {
                                let (_, chain) = path_fields(&schema, path)
                                    .ok_or_else(|| contract("selected child disappeared"))?;
                                match chain.last().map(|field| field.data_type()) {
                                    Some(DataType::Struct(children)) => children
                                        .iter()
                                        .map(|field| {
                                            let mut child = path.clone();
                                            child.steps.push(ArrowFieldStep::StructField {
                                                name: field.name().clone(),
                                            });
                                            child
                                        })
                                        .collect::<Vec<_>>(),
                                    Some(DataType::List(_)) => {
                                        let mut child = path.clone();
                                        child.steps.push(ArrowFieldStep::ListElement { index: 0 });
                                        vec![child]
                                    }
                                    _ => Vec::new(),
                                }
                            } else {
                                schema
                                    .fields()
                                    .iter()
                                    .map(|field| ArrowFieldPath {
                                        field: field.name().clone(),
                                        steps: Vec::new(),
                                    })
                                    .collect()
                            };
                        if offset > paths.len() {
                            return Ok(refused(ResultInspectionRefusal::Selection));
                        }
                        let end = offset.saturating_add(*limit).min(paths.len());
                        let fields = match paths[offset..end]
                            .iter()
                            .map(|path| descriptor(path, &mut budget))
                            .collect::<Result<Vec<_>, _>>()
                        {
                            Ok(value) => value,
                            Err(reason) => return Ok(refused(reason)),
                        };
                        let next = (end < paths.len()).then_some(end);
                        Ok(bound_document(ResultInspectionDocument::Schema {
                            selection: selection.clone(),
                            original: original.clone(),
                            manifest: manifest.clone(),
                            schema_metadata,
                            field_count: paths.len(),
                            offset,
                            fields,
                            next,
                            next_cursor: next.map(|position| next_cursor(position as u64)),
                            recorded_rows,
                            owner: Some(owner),
                        }))
                    }
                    ResultInspectionView::Metadata {
                        path,
                        offset,
                        limit,
                    } => {
                        let offset = match request.cursor.as_ref() {
                            Some(cursor) => usize::try_from(cursor.position)
                                .map_err(|_| contract("metadata cursor exceeds platform"))?,
                            None => *offset,
                        };
                        let (_, chain) = path_fields(&schema, path)
                            .ok_or_else(|| contract("selected metadata disappeared"))?;
                        let field = chain
                            .last()
                            .ok_or_else(|| contract("selected metadata field absent"))?;
                        let mut entries = field.metadata().iter().collect::<Vec<_>>();
                        entries.sort_by(|a, b| a.0.cmp(b.0));
                        if offset > entries.len() {
                            return Ok(refused(ResultInspectionRefusal::Selection));
                        }
                        let end = offset.saturating_add(*limit).min(entries.len());
                        let selected = entries[offset..end]
                            .iter()
                            .map(|(key, value)| ((*key).clone(), (*value).clone()))
                            .collect::<std::collections::HashMap<_, _>>();
                        let selected = match budget.metadata(&selected) {
                            Ok(value) => value,
                            Err(reason) => return Ok(refused(reason)),
                        };
                        Ok(bound_document(ResultInspectionDocument::Metadata {
                            selection: selection.clone(),
                            original: original.clone(),
                            manifest: manifest.clone(),
                            path: path.clone(),
                            entry_count: entries.len(),
                            offset,
                            entries: selected,
                            next_cursor: (end < entries.len()).then(|| next_cursor(end as u64)),
                            owner: Some(owner),
                        }))
                    }
                    ResultInspectionView::Values {
                        fields,
                        maximum_rows,
                    } => {
                        let resume = request
                            .cursor
                            .as_ref()
                            .map_or(request.start, |cursor| cursor.position);
                        if resume < request.start || resume > request.end {
                            return Ok(refused(ResultInspectionRefusal::CursorChanged));
                        }
                        let selected_fields = match fields
                            .iter()
                            .map(|path| descriptor(path, &mut budget))
                            .collect::<Result<Vec<_>, _>>()
                        {
                            Ok(value) => value,
                            Err(reason) => return Ok(refused(reason)),
                        };
                        let roots = fields
                            .iter()
                            .filter(|path| path.steps.is_empty())
                            .filter_map(|path| schema.index_of(&path.field).ok())
                            .collect::<BTreeSet<_>>();
                        let context = spec
                            .columns
                            .iter()
                            .enumerate()
                            .filter(|(index, field)| {
                                !roots.contains(index)
                                    && field.result_selection_role()
                                        != Some(pse_schema::model::ResultSelectionRole::Value)
                            })
                            .map(|(index, _)| index)
                            .collect::<Vec<_>>();
                        if context.len() > CONTEXT_FIELDS {
                            return Ok(refused(ResultInspectionRefusal::ContextExtent));
                        }
                        let context_fields = match context
                            .iter()
                            .map(|index| budget.field(*index, schema.field(*index)))
                            .collect::<Result<Vec<_>, _>>()
                        {
                            Ok(value) => value,
                            Err(_) => return Ok(refused(ResultInspectionRefusal::ContextExtent)),
                        };
                        let descriptor_context_complete =
                            selected_fields.iter().chain(&context_fields).all(|field| {
                                field.metadata_omitted.is_empty()
                                    && field.ancestor_metadata_omitted.iter().all(Vec::is_empty)
                            });
                        reader.skip_recorded_prefix(resume)?;
                        let mut rows = Vec::new();
                        let mut next_row = None;
                        for block in snapshot.blocks() {
                            let start = request.start.max(block.start);
                            let end = request.end.min(block.end);
                            if start >= end || end <= resume {
                                continue;
                            }
                            if rows.len() == *maximum_rows {
                                next_row = Some(start.max(resume));
                                break;
                            }
                            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                            let batch = reader.next_batch().await?.ok_or_else(|| {
                                contract("result inspection frozen block is absent")
                            })?;
                            if batch.num_rows() as u64 != end - start {
                                return Err(contract(
                                    "result inspection source coordinates differ",
                                ));
                            }
                            let first = usize::try_from(resume.saturating_sub(start))
                                .map_err(|_| contract("result cursor exceeds platform"))?;
                            for row in first..batch.num_rows() {
                                if rows.len() == *maximum_rows {
                                    next_row = Some(start + row as u64);
                                    break;
                                }
                                let values = fields
                                    .iter()
                                    .map(|path| {
                                        let index = schema
                                            .index_of(&path.field)
                                            .map_err(|_| contract("selected root disappeared"))?;
                                        selected_cell(batch.column(index), row, &path.steps, 0)
                                    })
                                    .collect::<Result<Vec<_>, WorkflowError>>()?;
                                let context = row_values(&batch, row, &context)?;
                                let context_complete = descriptor_context_complete
                                    && !context.iter().any(|value| {
                                        matches!(value, ResultInspectionCell::Unavailable { .. })
                                    });
                                for value in values.iter().chain(context.iter()) {
                                    if let Err(reason) = budget.charge(
                                        serde_json::to_vec(value)
                                            .map_err(|error| contract(error.to_string()))?
                                            .len(),
                                    ) {
                                        return Ok(refused(reason));
                                    }
                                }
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
                            selection: selection.clone(),
                            original: original.clone(),
                            manifest: manifest.clone(),
                            schema_metadata,
                            selected_fields,
                            context_fields,
                            recorded_rows,
                            rows,
                            next_row,
                            next_cursor: next_row.map(next_cursor),
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
    fn physical_query_names_are_bounded_before_result_basis_or_protection() {
        let mut request = result_request_example();
        request.view = ResultInspectionView::Physical {
            request: super::super::PhysicalInspectionRequest {
                query: super::super::PhysicalInspectionQuery::QuantityNamed {
                    name: "x".repeat(1025),
                },
                offset: 0,
                limit: 1,
            },
        };
        assert_eq!(
            preflight(&request, &Schema::empty()),
            Some(ResultInspectionRefusal::Selection)
        );
    }

    #[test]
    fn registered_nested_descriptors_fit_the_finite_readable_context() {
        let registry = pse_schema::shared_registry().unwrap();
        let spec = registry.relation("runtime.study_outcomes").unwrap();
        let schema = pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap();
        let raw_metadata_bytes: usize = schema
            .fields()
            .iter()
            .map(|field| serde_json::to_vec(field.metadata()).unwrap().len())
            .sum();
        assert!(
            raw_metadata_bytes > DOCUMENT_BYTES,
            "the registered recursive identities must exercise the original extent fault"
        );
        let mut budget = Budget {
            remaining: DOCUMENT_BYTES,
        };
        budget.metadata(schema.metadata()).unwrap();
        let mut descriptors = Vec::new();
        for (index, field) in schema.fields().iter().enumerate() {
            let remaining = budget.remaining;
            let descriptor = budget.field(index, field).unwrap_or_else(|error| {
                panic!(
                    "original context {} refused {error:?}, remaining {remaining}",
                    field.name()
                )
            });
            for omitted in &descriptor.metadata_omitted {
                assert_eq!(omitted.key, pse_schema::arrow::KEY_LOGICAL_TYPE);
                assert_eq!(omitted.value_bytes, field.metadata()[&omitted.key].len());
                assert!(!descriptor.metadata.contains_key(&omitted.key));
            }
            descriptors.push(descriptor);
        }
        assert_eq!(
            descriptors
                .iter()
                .map(|field| field.metadata_omitted.len())
                .sum::<usize>(),
            3
        );
        assert!(
            serde_json::to_vec(&descriptors).unwrap().len() < 24 << 10,
            "shallow descriptors must not copy recursive container identities"
        );
    }

    #[test]
    fn nested_cells_distinguish_parent_validity_empty_missing_and_null_element() {
        let lists: ArrayRef = Arc::new(ListArray::from_iter_primitive::<
            datafusion::arrow::datatypes::Int32Type,
            _,
            _,
        >(vec![
            Some(vec![Some(7), None]),
            Some(vec![]),
            None,
        ]));
        let step = [ArrowFieldStep::ListElement { index: 1 }];
        assert_eq!(
            selected_cell(&lists, 0, &step, 0).unwrap(),
            ResultInspectionCell::Null
        );
        assert_eq!(
            selected_cell(&lists, 1, &step, 0).unwrap(),
            ResultInspectionCell::MissingElement {
                depth: 0,
                length: 0
            }
        );
        assert_eq!(
            selected_cell(&lists, 2, &step, 0).unwrap(),
            ResultInspectionCell::ParentNull { depth: 0 }
        );
        assert_eq!(
            selected_cell(&lists, 0, &[ArrowFieldStep::ListElement { index: 0 }], 0).unwrap(),
            ResultInspectionCell::Integer { value: 7 }
        );
        let values: ArrayRef = Arc::new(Int64Array::from(vec![999, 11]));
        let structures: ArrayRef = Arc::new(StructArray::new(
            vec![Arc::new(Field::new("child", DataType::Int64, false))].into(),
            vec![values],
            Some(datafusion::arrow::buffer::NullBuffer::from(vec![
                false, true,
            ])),
        ));
        let step = [ArrowFieldStep::StructField {
            name: "child".into(),
        }];
        assert_eq!(
            selected_cell(&structures, 0, &step, 0).unwrap(),
            ResultInspectionCell::ParentNull { depth: 0 }
        );
        assert_eq!(
            selected_cell(&structures, 1, &step, 0).unwrap(),
            ResultInspectionCell::Integer { value: 11 }
        );
    }

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
        let nested: ArrayRef = Arc::new(ListArray::from_iter_primitive::<
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
    fn field_descriptor_retains_shallow_storage_and_original_metadata() {
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
        assert!(matches!(
            projected.storage,
            ResultInspectionStorage::Struct { child_count: 1 }
        ));
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
        assert!(
            matches!(
                Budget {
                    remaining: DOCUMENT_BYTES
                }
                .metadata(field.metadata()),
                Err(ResultInspectionRefusal::ResponseExtent)
            ),
            "exact metadata detail refuses a genuinely oversized original entry"
        );
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_result_inspection_server_unit;

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! What the `pse.canon.v2` canonicalizer is allowed to consume (blueprint §5.3).
//!
//! A [`CanonicalContract`] is plain values: a relation ID, a schema version, the registry
//! fingerprint, the declared Arrow schema with its metadata, the primary-key column
//! ordinals and one [`Layout`] per column. No registry type appears here, because
//! `pse-ids` sits below `pse-schema` in the graph and identity must not need the registry
//! to exist.
//!
//! The point of [`Layout`] is the rule in §5.3 step 2: *only registered logical types and
//! their declared canonical storage are supported; an unknown Arrow layout is rejected,
//! never hashed opportunistically.* An unmodelled layout that hashed "somehow" would give
//! two readers two different opinions about what a relation's identity covers, and only
//! one of them would ever find out.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use arrow_schema::{DataType, Field, Schema, SchemaRef, TimeUnit};

use crate::error::CanonError;
use crate::id::{ContentHash, SchemaVersion, SemanticId};

/// Metadata keys a contract may carry: the platform's own namespace, …
const METADATA_PREFIX_PSE: &str = "pse.";

/// … and Arrow's canonical extension namespace (blueprint §4.4).
const METADATA_PREFIX_EXTENSION: &str = "ARROW:extension:";

/// Metadata keys that identify the artifact rather than the contract, and are therefore
/// excluded from the hash frame and from contract comparison (blueprint §5.3 step 3).
const VOLATILE_METADATA_KEYS: [&str; 2] = ["pse.snapshot_id", "pse.producer_pass_id"];

/// The canonical UTC timezone spelling; the registry writes exactly one.
const UTC: &str = "UTC";

/// A position in a schema: zero-based registry column and child ordinals in ASCII decimal
/// without leading zeros, joined by `/` (blueprint §5.3 step 3).
///
/// The empty path denotes the schema itself. `0/2` is the third child of the first column.
/// Ordinals rather than names, because a column rename must not change a field path, and
/// `/` rather than `.` because a struct field name may contain a dot.
///
/// ```
/// use pse_ids::FieldPath;
///
/// assert_eq!(FieldPath::root().as_str(), "");
/// assert_eq!(FieldPath::root().child(0).child(2).as_str(), "0/2");
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldPath(String);

impl FieldPath {
    /// The schema itself.
    pub fn root() -> Self {
        Self(String::new())
    }

    /// The `ordinal`-th child of this path.
    #[must_use]
    pub fn child(&self, ordinal: usize) -> Self {
        if self.0.is_empty() {
            Self(ordinal.to_string())
        } else {
            Self(format!("{}/{ordinal}", self.0))
        }
    }

    /// The rendered path, which is what the canonical metadata relation stores.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The key width of a dictionary-encoded enum column (blueprint §4.4 `pse.enum`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DictKey {
    /// `Int8` keys: up to 128 members.
    Int8,
    /// `Int32` keys: the wide form.
    Int32,
}

impl DictKey {
    /// The Arrow key type this stands for.
    pub const fn data_type(self) -> DataType {
        match self {
            Self::Int8 => DataType::Int8,
            Self::Int32 => DataType::Int32,
        }
    }

    /// The Arrow key type as a [`DictKey`], or `None` for a width the contract refuses.
    const fn from_data_type(data_type: &DataType) -> Option<Self> {
        match data_type {
            DataType::Int8 => Some(Self::Int8),
            DataType::Int32 => Some(Self::Int32),
            _ => None,
        }
    }
}

impl fmt::Display for DictKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int8 => f.write_str("Int8"),
            Self::Int32 => f.write_str("Int32"),
        }
    }
}

/// The supported canonical storage matrix (blueprint §5.3 step 2).
///
/// Every variant is a layout `pse.canon.v2` knows how to normalize and frame. Anything
/// else — `Decimal128`, `LargeUtf8`, `Map`, `Union`, `RunEndEncoded`, a non-UTC or
/// non-nanosecond timestamp, a dictionary without a declared enum domain — is rejected at
/// [`CanonicalContract::try_new`] rather than normalized by guesswork.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    /// `Boolean`.
    Boolean,
    /// `Int8`.
    Int8,
    /// `Int16`.
    Int16,
    /// `Int32`.
    Int32,
    /// `Int64`.
    Int64,
    /// `UInt8`.
    UInt8,
    /// `UInt16`.
    UInt16,
    /// `UInt32`.
    UInt32,
    /// `UInt64`.
    UInt64,
    /// `Float32`; hashed through [`crate::canonical_f32_bits`].
    Float32,
    /// `Float64`; hashed through [`crate::canonical_f64_bits`].
    Float64,
    /// `Timestamp(Nanosecond, Some("UTC"))`: the one temporal storage the registry writes.
    TimestampNsUtc,
    /// `Utf8` with `i32` offsets.
    Utf8,
    /// `Binary` with `i32` offsets.
    Binary,
    /// `FixedSizeBinary(n)`: `pse.semantic_id` at 16, `pse.content_hash` at 32.
    FixedSizeBinary(i32),
    /// A string enumeration with a declared member domain. External dictionary inputs
    /// decode to values after checking bounds and membership; codes carry no identity.
    Enum {
        /// The optional dictionary key width; `None` is canonical `Utf8` storage.
        key: Option<DictKey>,
        /// The declared members, in declared order.
        members: Arc<[String]>,
    },
    /// `List` with `i32` offsets.
    List(Box<Layout>),
    /// `FixedSizeList` of a fixed width.
    FixedSizeList(Box<Layout>, i32),
    /// `Struct`, in declared field order.
    Struct(Vec<(String, Layout)>),
}

impl Layout {
    /// Whether this layout is floating point, which disqualifies it as a key (ADR-0030).
    pub const fn is_floating(&self) -> bool {
        matches!(self, Self::Float32 | Self::Float64)
    }
}

/// A terse, deterministic description of an Arrow type for an error message.
///
/// Arrow's own `Display` formats nested field metadata out of a `HashMap`, so its order is
/// not stable; a message that changes between runs is a message nobody can diff.
fn describe(data_type: &DataType) -> String {
    match data_type {
        DataType::List(field) => format!("List<{}>", describe(field.data_type())),
        DataType::LargeList(field) => format!("LargeList<{}>", describe(field.data_type())),
        DataType::ListView(field) => format!("ListView<{}>", describe(field.data_type())),
        DataType::LargeListView(field) => format!("LargeListView<{}>", describe(field.data_type())),
        DataType::FixedSizeList(field, width) => {
            format!("FixedSizeList<{}, {width}>", describe(field.data_type()))
        }
        DataType::Struct(fields) => {
            let rendered: Vec<String> = fields
                .iter()
                .map(|field| format!("{}: {}", field.name(), describe(field.data_type())))
                .collect();
            format!("Struct<{}>", rendered.join(", "))
        }
        DataType::Dictionary(key, value) => {
            format!("Dictionary<{}, {}>", describe(key), describe(value))
        }
        DataType::Map(field, _) => format!("Map<{}>", describe(field.data_type())),
        DataType::RunEndEncoded(run_ends, values) => format!(
            "RunEndEncoded<{}, {}>",
            describe(run_ends.data_type()),
            describe(values.data_type())
        ),
        DataType::Union(fields, _) => format!("Union<{} variants>", fields.len()),
        // Leaf types have no fields, so Arrow's own rendering is deterministic.
        leaf => leaf.to_string(),
    }
}

/// Renders a metadata map deterministically for an error message.
fn describe_metadata(metadata: &BTreeMap<&str, &str>) -> String {
    let rendered: Vec<String> = metadata
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    format!("{{{}}}", rendered.join(", "))
}

/// The contract metadata of a map, with the volatile artifact keys removed.
fn contract_metadata(metadata: &std::collections::HashMap<String, String>) -> BTreeMap<&str, &str> {
    metadata
        .iter()
        .filter(|(key, _)| !VOLATILE_METADATA_KEYS.contains(&key.as_str()))
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect()
}

/// Rejects a metadata key outside the registered namespaces (blueprint §5.3 step 3).
fn check_metadata_keys(
    path: &FieldPath,
    metadata: &std::collections::HashMap<String, String>,
) -> Result<(), CanonError> {
    // Sorted, so the first rejected key does not depend on hash iteration order.
    let mut keys: Vec<&str> = metadata.keys().map(String::as_str).collect();
    keys.sort_unstable();
    for key in keys {
        if !key.starts_with(METADATA_PREFIX_PSE) && !key.starts_with(METADATA_PREFIX_EXTENSION) {
            return Err(CanonError::UnregisteredMetadata {
                path: path.clone(),
                key: key.to_owned(),
            });
        }
    }
    Ok(())
}

/// Derives the [`Layout`] of a field, rejecting anything outside the matrix.
fn layout_of(
    path: &FieldPath,
    field: &Field,
    enum_domains: &BTreeMap<FieldPath, Arc<[String]>>,
) -> Result<Layout, CanonError> {
    check_metadata_keys(path, field.metadata())?;

    let unsupported = || CanonError::UnsupportedLayout {
        path: path.clone(),
        what: describe(field.data_type()),
    };

    let layout = match field.data_type() {
        DataType::Boolean => Layout::Boolean,
        DataType::Int8 => Layout::Int8,
        DataType::Int16 => Layout::Int16,
        DataType::Int32 => Layout::Int32,
        DataType::Int64 => Layout::Int64,
        DataType::UInt8 => Layout::UInt8,
        DataType::UInt16 => Layout::UInt16,
        DataType::UInt32 => Layout::UInt32,
        DataType::UInt64 => Layout::UInt64,
        DataType::Float32 => Layout::Float32,
        DataType::Float64 => Layout::Float64,
        DataType::Utf8 => enum_domains
            .get(path)
            .map_or(Layout::Utf8, |members| Layout::Enum {
                key: None,
                members: Arc::clone(members),
            }),
        DataType::Binary => Layout::Binary,
        DataType::FixedSizeBinary(width) => Layout::FixedSizeBinary(*width),
        DataType::Timestamp(TimeUnit::Nanosecond, Some(zone)) if zone.as_ref() == UTC => {
            Layout::TimestampNsUtc
        }
        DataType::Dictionary(key, value) => {
            let Some(key) = DictKey::from_data_type(key) else {
                return Err(unsupported());
            };
            if value.as_ref() != &DataType::Utf8 {
                return Err(unsupported());
            }
            let Some(members) = enum_domains.get(path) else {
                return Err(CanonError::UnsupportedLayout {
                    path: path.clone(),
                    what: format!(
                        "{} without a declared enum domain",
                        describe(field.data_type())
                    ),
                });
            };
            Layout::Enum {
                key: Some(key),
                members: Arc::clone(members),
            }
        }
        DataType::List(child) => Layout::List(Box::new(layout_of(
            &path.child(0),
            child.as_ref(),
            enum_domains,
        )?)),
        DataType::FixedSizeList(child, width) => Layout::FixedSizeList(
            Box::new(layout_of(&path.child(0), child.as_ref(), enum_domains)?),
            *width,
        ),
        DataType::Struct(children) => {
            let mut members = Vec::with_capacity(children.len());
            for (ordinal, child) in children.iter().enumerate() {
                members.push((
                    child.name().to_owned(),
                    layout_of(&path.child(ordinal), child.as_ref(), enum_domains)?,
                ));
            }
            Layout::Struct(members)
        }
        _ => return Err(unsupported()),
    };
    Ok(layout)
}

/// The selected resource envelope of a canonicalization (blueprint §5.3 step 8).
///
/// A stated bound rather than an implicit one: preflight uses checked arithmetic against
/// these numbers and reserves before allocating, so exceeding one is
/// `runtime.resource_limit` and never a partial publication.
///
/// ```
/// use pse_ids::Envelope;
///
/// assert_eq!(Envelope::DEFAULT.max_rows, 100_000_000);
/// assert!(Envelope::new(1_000, 1 << 20).is_ok());
/// // Deployment settings do not change canonical framing or identity.
/// assert!(Envelope::new(200_000_000, 1 << 30).is_ok());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// Rows in the complete relation.
    pub max_rows: u64,
    /// Bytes of normalized value, offset and validity buffers.
    pub max_normalized_bytes: u64,
}

impl Envelope {
    /// Selected default: 100 million rows and 32 GiB of normalized buffers on 64-bit
    /// hosts. This is a configurable resource policy, not a measured workload promise.
    pub const DEFAULT: Self = Self {
        max_rows: 100_000_000,
        max_normalized_bytes: if isize::BITS >= 64 {
            32 << 30
        } else {
            isize::MAX as u64
        },
    };

    /// Select a deployment envelope within actual row-index and addressability limits.
    /// Zero admits only empty data. Variable-width offsets retain their own checked
    /// representation limits; selecting a larger policy never widens an Arrow offset.
    ///
    /// # Errors
    ///
    /// [`CanonError::Envelope`] when a policy exceeds canonical `u32` row indices or
    /// addressable buffer sizes. No allocation is performed by this constructor.
    pub fn new(max_rows: u64, max_normalized_bytes: u64) -> Result<Self, CanonError> {
        let row_limit = u64::from(u32::MAX).min(isize::MAX as u64);
        if max_rows > row_limit {
            return Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Rows,
                limit: row_limit,
                actual: max_rows,
            });
        }
        if max_normalized_bytes > isize::MAX as u64 {
            return Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Bytes,
                limit: isize::MAX as u64,
                actual: max_normalized_bytes,
            });
        }
        Ok(Self {
            max_rows,
            max_normalized_bytes,
        })
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Everything `pse.canon.v2` needs about a relation, and nothing about the registry.
#[derive(Clone, Debug)]
pub struct CanonicalContract {
    /// The relation's registry identity; enters the hash frame (§5.3 step 4).
    pub relation_id: SemanticId,
    /// The relation's declared schema version; enters the hash frame as `u32`.
    pub schema_version: SchemaVersion,
    /// The registry fingerprint under which this contract was read; enters the frame.
    pub registry_fingerprint: ContentHash,
    /// The declared schema, with its contract, semantic and extension metadata.
    pub schema: SchemaRef,
    /// Primary-key column ordinals, in declared key order.
    pub primary_key: Vec<usize>,
    /// One layout per top-level column, parallel to `schema.fields()`.
    pub layouts: Vec<Layout>,
}

impl CanonicalContract {
    /// Admits a declared schema, or says exactly why it cannot be canonicalized.
    ///
    /// `primary_key` names columns in key order; `enum_domains` declares the member domain
    /// of every dictionary-encoded column, keyed by [`FieldPath`].
    ///
    /// An **empty** `primary_key` is admitted here and is a caller error everywhere else:
    /// §5.3 step 1 orders a relation by its unique primary key, and without one "equal
    /// content hashes equally across row order" has nothing to stand on. The registry
    /// declares a key for every relation, so this is not rejected at this layer only
    /// because this layer cannot tell an intentional keyless contract from a registry bug.
    ///
    /// # Errors
    ///
    /// - [`CanonError::UnsupportedLayout`] for an Arrow layout outside the matrix,
    ///   including a dictionary with no declared enum domain.
    /// - [`CanonError::UnregisteredMetadata`] for a metadata key outside `pse.` and
    ///   `ARROW:extension:` — `SERDE_ARROW:*` is the one that has actually happened.
    /// - [`CanonError::InvalidKey`] for an unknown, nullable or floating key column. A
    ///   nullable key cannot order (§5.3 step 1) and a float key merges values the engine
    ///   considers equal and the reader does not (ADR-0030).
    pub fn try_new(
        relation_id: SemanticId,
        schema_version: SchemaVersion,
        registry_fingerprint: ContentHash,
        schema: SchemaRef,
        primary_key: &[&str],
        enum_domains: &BTreeMap<FieldPath, Arc<[String]>>,
    ) -> Result<Self, CanonError> {
        check_metadata_keys(&FieldPath::root(), schema.metadata())?;

        let mut layouts = Vec::with_capacity(schema.fields().len());
        for (ordinal, field) in schema.fields().iter().enumerate() {
            layouts.push(layout_of(
                &FieldPath::root().child(ordinal),
                field.as_ref(),
                enum_domains,
            )?);
        }

        let mut key_ordinals = Vec::with_capacity(primary_key.len());
        for name in primary_key {
            let Some((ordinal, field)) = schema
                .fields()
                .iter()
                .enumerate()
                .find(|(_, field)| field.name() == name)
            else {
                return Err(CanonError::InvalidKey {
                    column: (*name).to_owned(),
                    reason: "no such column in the declared schema".to_owned(),
                });
            };
            if key_ordinals.contains(&ordinal) {
                return Err(CanonError::InvalidKey {
                    column: (*name).to_owned(),
                    reason: "named twice in the primary key".to_owned(),
                });
            }
            if field.is_nullable() {
                return Err(CanonError::InvalidKey {
                    column: (*name).to_owned(),
                    reason: "a nullable column cannot order a relation (blueprint §5.3 step 1)"
                        .to_owned(),
                });
            }
            if layouts.get(ordinal).is_some_and(Layout::is_floating) {
                return Err(CanonError::InvalidKey {
                    column: (*name).to_owned(),
                    reason: "a floating-point column is never a key (ADR-0030)".to_owned(),
                });
            }
            key_ordinals.push(ordinal);
        }

        Ok(Self {
            relation_id,
            schema_version,
            registry_fingerprint,
            schema,
            primary_key: key_ordinals,
            layouts,
        })
    }

    /// Checks a batch's schema against the contract before anything is hashed.
    ///
    /// Names, types and nullability must match exactly. Metadata must match except for
    /// `pse.snapshot_id` and `pse.producer_pass_id`, which name the artifact rather than
    /// the contract and are excluded from the hash frame for the same reason (§5.3 step 3).
    ///
    /// # Errors
    ///
    /// [`CanonError::ContractMismatch`] naming the field path that disagrees.
    pub fn check_batch_schema(&self, actual: &Schema) -> Result<(), CanonError> {
        let root = FieldPath::root();
        let expected_meta = contract_metadata(self.schema.metadata());
        let actual_meta = contract_metadata(actual.metadata());
        if expected_meta != actual_meta {
            return Err(CanonError::ContractMismatch {
                path: root.clone(),
                expected: describe_metadata(&expected_meta),
                actual: describe_metadata(&actual_meta),
            });
        }
        if self.schema.fields().len() != actual.fields().len() {
            return Err(CanonError::ContractMismatch {
                path: root,
                expected: format!("{} columns", self.schema.fields().len()),
                actual: format!("{} columns", actual.fields().len()),
            });
        }
        for (ordinal, (expected, offered)) in self
            .schema
            .fields()
            .iter()
            .zip(actual.fields().iter())
            .enumerate()
        {
            compare_field(&root.child(ordinal), expected, offered)?;
        }
        Ok(())
    }

    /// The key columns as declared names, in key order.
    pub fn key_column_names(&self) -> Vec<&str> {
        self.primary_key
            .iter()
            .filter_map(|ordinal| self.schema.fields().get(*ordinal))
            .map(|field| field.name().as_str())
            .collect()
    }
}

/// Compares one declared field with one offered field.
fn compare_field(path: &FieldPath, expected: &Field, actual: &Field) -> Result<(), CanonError> {
    if expected.name() != actual.name() {
        return Err(CanonError::ContractMismatch {
            path: path.clone(),
            expected: format!("column `{}`", expected.name()),
            actual: format!("column `{}`", actual.name()),
        });
    }
    if expected.is_nullable() != actual.is_nullable() {
        return Err(CanonError::ContractMismatch {
            path: path.clone(),
            expected: nullability(expected.is_nullable()).to_owned(),
            actual: nullability(actual.is_nullable()).to_owned(),
        });
    }
    let expected_meta = contract_metadata(expected.metadata());
    let actual_meta = contract_metadata(actual.metadata());
    if expected_meta != actual_meta {
        return Err(CanonError::ContractMismatch {
            path: path.clone(),
            expected: describe_metadata(&expected_meta),
            actual: describe_metadata(&actual_meta),
        });
    }
    compare_data_type(path, expected.data_type(), actual.data_type())
}

/// `nullable` / `non-null`, for a message.
const fn nullability(nullable: bool) -> &'static str {
    if nullable { "nullable" } else { "non-null" }
}

/// Compares two data types, recursing so that child metadata is compared modulo the
/// volatile keys rather than by raw `PartialEq`.
fn compare_data_type(
    path: &FieldPath,
    expected: &DataType,
    actual: &DataType,
) -> Result<(), CanonError> {
    let mismatch = || CanonError::ContractMismatch {
        path: path.clone(),
        expected: describe(expected),
        actual: describe(actual),
    };
    match (expected, actual) {
        (DataType::List(left), DataType::List(right)) => compare_field(&path.child(0), left, right),
        (
            DataType::FixedSizeList(left, left_width),
            DataType::FixedSizeList(right, right_width),
        ) => {
            if left_width != right_width {
                return Err(mismatch());
            }
            compare_field(&path.child(0), left, right)
        }
        (DataType::Struct(left), DataType::Struct(right)) => {
            if left.len() != right.len() {
                return Err(mismatch());
            }
            for (ordinal, (left_field, right_field)) in left.iter().zip(right.iter()).enumerate() {
                compare_field(&path.child(ordinal), left_field, right_field)?;
            }
            Ok(())
        }
        (left, right) if left == right => Ok(()),
        _ => Err(mismatch()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const RELATION: SemanticId = SemanticId::from_bytes([0x11; 16]);
    const VERSION: SchemaVersion = SchemaVersion(1);
    const FINGERPRINT: ContentHash = ContentHash::from_bytes([0x22; 32]);

    fn no_domains() -> BTreeMap<FieldPath, Arc<[String]>> {
        BTreeMap::new()
    }

    fn contract_of(
        fields: Vec<Field>,
        primary_key: &[&str],
        enum_domains: &BTreeMap<FieldPath, Arc<[String]>>,
    ) -> Result<CanonicalContract, CanonError> {
        CanonicalContract::try_new(
            RELATION,
            VERSION,
            FINGERPRINT,
            Arc::new(Schema::new(fields)),
            primary_key,
            enum_domains,
        )
    }

    fn key_field() -> Field {
        Field::new("id", DataType::FixedSizeBinary(16), false)
    }

    #[test]
    fn a_field_path_is_ordinals_not_names() {
        assert_eq!(FieldPath::root().as_str(), "");
        assert_eq!(FieldPath::root().to_string(), "");
        assert_eq!(FieldPath::root().child(0).as_str(), "0");
        assert_eq!(FieldPath::root().child(0).child(2).as_str(), "0/2");
        assert_eq!(FieldPath::root().child(10).child(0).as_str(), "10/0");

        // Byte order is what the canonical metadata relation sorts by.
        let mut paths = [
            FieldPath::root().child(1),
            FieldPath::root(),
            FieldPath::root().child(0).child(2),
        ];
        paths.sort();
        assert_eq!(
            paths.iter().map(FieldPath::as_str).collect::<Vec<_>>(),
            vec!["", "0/2", "1"]
        );
    }

    #[test]
    fn every_supported_scalar_layout_is_admitted() {
        let cases: Vec<(DataType, Layout)> = vec![
            (DataType::Boolean, Layout::Boolean),
            (DataType::Int8, Layout::Int8),
            (DataType::Int16, Layout::Int16),
            (DataType::Int32, Layout::Int32),
            (DataType::Int64, Layout::Int64),
            (DataType::UInt8, Layout::UInt8),
            (DataType::UInt16, Layout::UInt16),
            (DataType::UInt32, Layout::UInt32),
            (DataType::UInt64, Layout::UInt64),
            (DataType::Float32, Layout::Float32),
            (DataType::Float64, Layout::Float64),
            (DataType::Utf8, Layout::Utf8),
            (DataType::Binary, Layout::Binary),
            (DataType::FixedSizeBinary(16), Layout::FixedSizeBinary(16)),
            (
                DataType::Timestamp(TimeUnit::Nanosecond, Some(UTC.into())),
                Layout::TimestampNsUtc,
            ),
        ];
        for (data_type, expected) in cases {
            let contract = contract_of(
                vec![key_field(), Field::new("value", data_type.clone(), true)],
                &["id"],
                &no_domains(),
            );
            match contract {
                Ok(contract) => assert_eq!(contract.layouts.get(1), Some(&expected)),
                Err(err) => panic!("{data_type} was refused: {err}"),
            }
        }
    }

    #[test]
    fn nested_struct_of_list_of_utf8_is_admitted() {
        let inner = Field::new("item", DataType::Utf8, true);
        let list = Field::new("names", DataType::List(Arc::new(inner)), false);
        let nested = Field::new("payload", DataType::Struct(vec![list].into()), true);

        let contract = contract_of(vec![key_field(), nested], &["id"], &no_domains());
        let layouts = contract.ok().map(|contract| contract.layouts);
        assert_eq!(
            layouts.as_deref().and_then(|layouts| layouts.get(1)),
            Some(&Layout::Struct(vec![(
                "names".to_owned(),
                Layout::List(Box::new(Layout::Utf8))
            )]))
        );
    }

    #[test]
    fn a_fixed_size_list_keeps_its_width() {
        let item = Field::new("item", DataType::Float64, false);
        let field = Field::new("vector", DataType::FixedSizeList(Arc::new(item), 8), false);
        let contract = contract_of(vec![key_field(), field], &["id"], &no_domains());
        assert_eq!(
            contract
                .map(|contract| contract.layouts)
                .ok()
                .as_deref()
                .and_then(|l| l.get(1)),
            Some(&Layout::FixedSizeList(Box::new(Layout::Float64), 8))
        );
    }

    #[test]
    fn a_dictionary_with_a_declared_domain_becomes_an_enum() {
        let members: Arc<[String]> = Arc::from(vec!["liquid".to_owned(), "vapor".to_owned()]);
        let mut domains = no_domains();
        domains.insert(FieldPath::root().child(1), Arc::clone(&members));

        let field = Field::new(
            "phase",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            false,
        );
        let contract = contract_of(vec![key_field(), field], &["id"], &domains);
        assert_eq!(
            contract
                .map(|contract| contract.layouts)
                .ok()
                .as_deref()
                .and_then(|l| l.get(1)),
            Some(&Layout::Enum {
                key: Some(DictKey::Int8),
                members
            })
        );
    }

    #[test]
    fn a_dictionary_without_a_declared_domain_is_refused() {
        let field = Field::new(
            "phase",
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
            false,
        );
        let refused = contract_of(vec![key_field(), field], &["id"], &no_domains());
        assert!(matches!(
            refused,
            Err(CanonError::UnsupportedLayout { ref path, ref what })
                if path.as_str() == "1" && what.contains("without a declared enum domain")
        ));
    }

    #[test]
    fn a_dictionary_with_an_unsupported_key_or_value_is_refused() {
        let members: Arc<[String]> = Arc::from(vec!["a".to_owned()]);
        let mut domains = no_domains();
        domains.insert(FieldPath::root().child(1), Arc::clone(&members));

        let wide_key = Field::new(
            "phase",
            DataType::Dictionary(Box::new(DataType::Int64), Box::new(DataType::Utf8)),
            false,
        );
        assert!(matches!(
            contract_of(vec![key_field(), wide_key], &["id"], &domains),
            Err(CanonError::UnsupportedLayout { .. })
        ));

        let wrong_value = Field::new(
            "phase",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Int32)),
            false,
        );
        assert!(matches!(
            contract_of(vec![key_field(), wrong_value], &["id"], &domains),
            Err(CanonError::UnsupportedLayout { .. })
        ));
    }

    #[test]
    fn layouts_outside_the_matrix_are_refused_rather_than_guessed() {
        let refusals: Vec<Field> = vec![
            Field::new("amount", DataType::Decimal128(38, 10), true),
            Field::new("text", DataType::LargeUtf8, true),
            Field::new("blob", DataType::LargeBinary, true),
            Field::new("half", DataType::Float16, true),
            Field::new("day", DataType::Date32, true),
            Field::new(
                "local",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(
                "millis",
                DataType::Timestamp(TimeUnit::Millisecond, Some(UTC.into())),
                true,
            ),
            Field::new(
                "offset",
                DataType::Timestamp(TimeUnit::Nanosecond, Some("+00:00".into())),
                true,
            ),
            Field::new(
                "wide_list",
                DataType::LargeList(Arc::new(Field::new("item", DataType::Utf8, true))),
                true,
            ),
        ];
        for field in refusals {
            let name = field.name().to_owned();
            let refused = contract_of(vec![key_field(), field], &["id"], &no_domains());
            assert!(
                matches!(refused, Err(CanonError::UnsupportedLayout { .. })),
                "`{name}` was admitted"
            );
        }
    }

    #[test]
    fn an_unsupported_layout_nested_inside_a_struct_is_refused_with_its_path() {
        let inner = Field::new("amount", DataType::Decimal128(10, 2), true);
        let nested = Field::new("payload", DataType::Struct(vec![inner].into()), true);
        let refused = contract_of(vec![key_field(), nested], &["id"], &no_domains());
        assert!(matches!(
            refused,
            Err(CanonError::UnsupportedLayout { ref path, .. }) if path.as_str() == "1/0"
        ));
    }

    #[test]
    fn a_float_key_is_refused() {
        let refused = contract_of(
            vec![Field::new("weight", DataType::Float64, false)],
            &["weight"],
            &no_domains(),
        );
        assert!(matches!(
            refused,
            Err(CanonError::InvalidKey { ref column, ref reason })
                if column == "weight" && reason.contains("ADR-0030")
        ));
    }

    #[test]
    fn a_nullable_key_is_refused() {
        let refused = contract_of(
            vec![Field::new("id", DataType::FixedSizeBinary(16), true)],
            &["id"],
            &no_domains(),
        );
        assert!(matches!(
            refused,
            Err(CanonError::InvalidKey { ref reason, .. }) if reason.contains("nullable")
        ));
    }

    #[test]
    fn an_unknown_or_repeated_key_column_is_refused() {
        assert!(matches!(
            contract_of(vec![key_field()], &["missing"], &no_domains()),
            Err(CanonError::InvalidKey { .. })
        ));
        assert!(matches!(
            contract_of(vec![key_field()], &["id", "id"], &no_domains()),
            Err(CanonError::InvalidKey { ref reason, .. }) if reason.contains("twice")
        ));
    }

    #[test]
    fn a_composite_key_keeps_its_declared_order() {
        let contract = contract_of(
            vec![
                key_field(),
                Field::new("ordinal", DataType::UInt32, false),
                Field::new("value", DataType::Float64, true),
            ],
            &["ordinal", "id"],
            &no_domains(),
        );
        match contract {
            Ok(contract) => {
                assert_eq!(contract.primary_key, vec![1, 0]);
                assert_eq!(contract.key_column_names(), vec!["ordinal", "id"]);
            }
            Err(err) => panic!("a composite key was refused: {err}"),
        }
    }

    #[test]
    fn an_unregistered_metadata_key_is_an_error_not_a_strip() {
        let mut schema = Schema::new(vec![key_field()]);
        schema = schema.with_metadata(HashMap::from([(
            "SERDE_ARROW:strategy".to_owned(),
            "x".to_owned(),
        )]));
        let refused = CanonicalContract::try_new(
            RELATION,
            VERSION,
            FINGERPRINT,
            Arc::new(schema),
            &["id"],
            &no_domains(),
        );
        assert!(matches!(
            refused,
            Err(CanonError::UnregisteredMetadata { ref path, ref key })
                if path.as_str().is_empty() && key == "SERDE_ARROW:strategy"
        ));

        let tagged =
            key_field().with_metadata(HashMap::from([("whatever".to_owned(), "x".to_owned())]));
        assert!(matches!(
            contract_of(vec![tagged], &["id"], &no_domains()),
            Err(CanonError::UnregisteredMetadata { ref path, .. }) if path.as_str() == "0"
        ));
    }

    #[test]
    fn registered_metadata_namespaces_are_kept() {
        let field = key_field().with_metadata(HashMap::from([
            ("pse.quantity".to_owned(), "none".to_owned()),
            (
                "ARROW:extension:name".to_owned(),
                "pse.semantic_id".to_owned(),
            ),
        ]));
        assert!(contract_of(vec![field], &["id"], &no_domains()).is_ok());
    }

    #[test]
    fn a_matching_batch_schema_is_accepted_and_the_volatile_keys_are_ignored() {
        let declared = Schema::new(vec![key_field()]).with_metadata(HashMap::from([(
            "pse.relation".to_owned(),
            "authored.entities".to_owned(),
        )]));
        let contract = CanonicalContract::try_new(
            RELATION,
            VERSION,
            FINGERPRINT,
            Arc::new(declared.clone()),
            &["id"],
            &no_domains(),
        );
        let Ok(contract) = contract else {
            panic!("the declared schema was refused");
        };

        assert!(contract.check_batch_schema(&declared).is_ok());

        let stamped = declared.clone().with_metadata(HashMap::from([
            ("pse.relation".to_owned(), "authored.entities".to_owned()),
            ("pse.snapshot_id".to_owned(), "blake3:00".to_owned()),
            ("pse.producer_pass_id".to_owned(), "P1".to_owned()),
        ]));
        assert!(contract.check_batch_schema(&stamped).is_ok());

        let altered = declared.with_metadata(HashMap::from([(
            "pse.relation".to_owned(),
            "authored.packages".to_owned(),
        )]));
        assert!(matches!(
            contract.check_batch_schema(&altered),
            Err(CanonError::ContractMismatch { .. })
        ));
    }

    #[test]
    fn a_batch_schema_that_differs_in_name_type_or_nullability_is_refused() {
        let declared = Schema::new(vec![
            key_field(),
            Field::new("value", DataType::Float64, true),
        ]);
        let Ok(contract) = CanonicalContract::try_new(
            RELATION,
            VERSION,
            FINGERPRINT,
            Arc::new(declared),
            &["id"],
            &no_domains(),
        ) else {
            panic!("the declared schema was refused");
        };

        let renamed = Schema::new(vec![
            key_field(),
            Field::new("val", DataType::Float64, true),
        ]);
        assert!(matches!(
            contract.check_batch_schema(&renamed),
            Err(CanonError::ContractMismatch { ref path, .. }) if path.as_str() == "1"
        ));

        let retyped = Schema::new(vec![
            key_field(),
            Field::new("value", DataType::Float32, true),
        ]);
        assert!(matches!(
            contract.check_batch_schema(&retyped),
            Err(CanonError::ContractMismatch { .. })
        ));

        let required = Schema::new(vec![
            key_field(),
            Field::new("value", DataType::Float64, false),
        ]);
        assert!(matches!(
            contract.check_batch_schema(&required),
            Err(CanonError::ContractMismatch { .. })
        ));

        let short = Schema::new(vec![key_field()]);
        assert!(matches!(
            contract.check_batch_schema(&short),
            Err(CanonError::ContractMismatch { ref path, .. }) if path.as_str().is_empty()
        ));
    }

    #[test]
    fn a_nested_child_mismatch_names_the_child_path() {
        let declared = Schema::new(vec![
            key_field(),
            Field::new(
                "payload",
                DataType::Struct(vec![Field::new("name", DataType::Utf8, true)].into()),
                true,
            ),
        ]);
        let Ok(contract) = CanonicalContract::try_new(
            RELATION,
            VERSION,
            FINGERPRINT,
            Arc::new(declared),
            &["id"],
            &no_domains(),
        ) else {
            panic!("the declared schema was refused");
        };

        let offered = Schema::new(vec![
            key_field(),
            Field::new(
                "payload",
                DataType::Struct(vec![Field::new("name", DataType::Utf8, false)].into()),
                true,
            ),
        ]);
        assert!(matches!(
            contract.check_batch_schema(&offered),
            Err(CanonError::ContractMismatch { ref path, .. }) if path.as_str() == "1/0"
        ));
    }

    #[test]
    fn envelope_policy_is_configurable_with_checked_representation_limits() {
        assert_eq!(Envelope::default(), Envelope::DEFAULT);
        assert_eq!(
            Envelope::new(1_000, 1 << 20).ok(),
            Some(Envelope {
                max_rows: 1_000,
                max_normalized_bytes: 1 << 20
            })
        );
        assert!(matches!(
            Envelope::new(u64::from(u32::MAX) + 1, 0),
            Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Rows,
                ..
            })
        ));
        assert!(matches!(
            Envelope::new(0, isize::MAX as u64 + 1),
            Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Bytes,
                ..
            })
        ));
        assert!(Envelope::new(200_000_000, 1 << 30).is_ok());
        assert!(Envelope::new(0, 0).is_ok());
    }
}

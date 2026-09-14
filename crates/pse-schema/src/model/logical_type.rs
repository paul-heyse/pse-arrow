// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The logical type of a column: the §4.5 physical scalars, the composites built from
//! them, and the §4.4 extension uses.
//!
//! Two renderings matter and they are deliberately separate. [`LogicalType::name`] is the
//! *registry* name — `f64`, `enum:MaterialBalanceType`, `list<semantic_id>` — which is
//! what `reference.schema_logical_types.name` holds and what the field metadata key
//! `pse.semantic.logical_type` carries. [`render_data_type`] is the *storage* rendering
//! that `arrow_storage` holds.
//!
//! Neither is a `Debug` rendering. Both feed the registry fingerprint, and §5.3 admits no
//! `Debug`/display serialization on a hashing path — a `{:?}` of an Arrow type is a
//! library's formatting choice, reproducible only until the library changes it.

use core::fmt;

use arrow_schema::DataType;
use pse_ids::SemanticId;

use crate::error::SchemaError;
use crate::model::extension::{EXTENSION_TYPES, ExtensionTypeSpec, timestamp_storage};

/// A column's declared type (blueprint §4.4, §4.5).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogicalType {
    /// `Float64`: every numerical value in `compiled` and `runtime`.
    F64,
    /// `Int64`: counts and signed ordinals.
    I64,
    /// `Int32`.
    I32,
    /// `UInt8`.
    U8,
    /// `UInt16`: opcodes and small ordinals.
    U16,
    /// `UInt32`: ordinals, positions, versions.
    U32,
    /// `UInt64`: large ordinals and row counts.
    U64,
    /// `Boolean`.
    Bool,
    /// `Utf8`: names, docs, paths.
    Text,
    /// `Timestamp(Nanosecond, "UTC")`: run and revision timestamps.
    Timestamp,
    /// A variable-length list whose elements are uniform.
    List(Box<LogicalType>),
    /// A fixed-length list of the declared width.
    FixedList(Box<LogicalType>, i32),
    /// A struct of `(name, type, nullable)` children.
    Struct(Vec<(&'static str, LogicalType, bool)>),
    /// One of the eleven `pse.*` extension types (blueprint §4.4).
    Ext(ExtensionUse),
}

/// A use of one of the eleven extension types, with the parameters the use needs
/// (blueprint §4.4).
///
/// The parameter is on the *use*, not on the type: `pse.enum` is one registered extension
/// whose metadata names which enumeration this column holds, which is why an unaware
/// reader still sees a dictionary rather than eleven near-identical types.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtensionUse {
    /// `pse.semantic_id`.
    SemanticId,
    /// `pse.content_hash`.
    ContentHash,
    /// `pse.dimension_vector`.
    DimensionVector,
    /// `pse.quantity_value`.
    QuantityValue,
    /// `pse.bound`.
    Bound,
    /// `pse.index_tuple`.
    IndexTuple,
    /// `pse.ordinal_ref`, naming the relation whose ordinals it references.
    OrdinalRef {
        /// The qualified relation name, for example `compiled.math_expr_nodes`.
        target: &'static str,
    },
    /// `pse.source_span`.
    SourceSpan,
    /// `pse.enum`, naming the declared enumeration.
    Enum(&'static str),
    /// `pse.expr_dsl`.
    ExprDsl,
    /// `pse.target_path`.
    TargetPath,
}

impl ExtensionUse {
    /// The index of this use's type in [`EXTENSION_TYPES`].
    const fn index(&self) -> usize {
        match self {
            Self::SemanticId => 0,
            Self::ContentHash => 1,
            Self::DimensionVector => 2,
            Self::QuantityValue => 3,
            Self::Bound => 4,
            Self::IndexTuple => 5,
            Self::OrdinalRef { .. } => 6,
            Self::SourceSpan => 7,
            Self::Enum(_) => 8,
            Self::ExprDsl => 9,
            Self::TargetPath => 10,
        }
    }

    /// The extension type this use refers to.
    pub const fn spec(&self) -> &'static ExtensionTypeSpec {
        &EXTENSION_TYPES[self.index()]
    }

    /// The `ARROW:extension:name` value, for example `pse.semantic_id`.
    pub const fn extension_name(&self) -> &'static str {
        self.spec().name
    }

    /// The registry logical-type name of this use.
    ///
    /// Parameterised uses carry their parameter, because `enum:PhaseType` and
    /// `enum:CubicType` are different contracts even though they share one storage.
    pub fn name(&self) -> String {
        match self {
            Self::SemanticId => "semantic_id".to_owned(),
            Self::ContentHash => "content_hash".to_owned(),
            Self::DimensionVector => "dimension_vector".to_owned(),
            Self::QuantityValue => "quantity_value".to_owned(),
            Self::Bound => "bound".to_owned(),
            Self::IndexTuple => "index_tuple".to_owned(),
            Self::OrdinalRef { target } => format!("ordinal_ref:{target}"),
            Self::SourceSpan => "source_span".to_owned(),
            Self::Enum(name) => format!("enum:{name}"),
            Self::ExprDsl => "expr_dsl".to_owned(),
            Self::TargetPath => "target_path".to_owned(),
        }
    }
}

impl LogicalType {
    /// Whether this declared type admits exact relational key equality (blueprint §14.2).
    /// Nested quantity and floating-point payloads never acquire key semantics from storage.
    pub fn admits_exact_key(&self) -> bool {
        matches!(
            self,
            LogicalType::I64
                | LogicalType::I32
                | LogicalType::U8
                | LogicalType::U16
                | LogicalType::U32
                | LogicalType::U64
                | LogicalType::Bool
                | LogicalType::Text
                | LogicalType::Timestamp
                | LogicalType::Ext(
                    ExtensionUse::SemanticId
                        | ExtensionUse::ContentHash
                        | ExtensionUse::Enum(_)
                        | ExtensionUse::OrdinalRef { .. }
                        | ExtensionUse::IndexTuple
                )
        )
    }

    /// A list of `element`.
    pub fn list(element: Self) -> Self {
        Self::List(Box::new(element))
    }

    /// A fixed-size list of `width` elements of `element`.
    pub fn fixed_list(element: Self, width: i32) -> Self {
        Self::FixedList(Box::new(element), width)
    }

    /// A `pse.enum` column over the declared enumeration `name`.
    pub const fn enumeration(name: &'static str) -> Self {
        Self::Ext(ExtensionUse::Enum(name))
    }

    /// A `pse.semantic_id` column.
    pub const fn id() -> Self {
        Self::Ext(ExtensionUse::SemanticId)
    }

    /// A `pse.content_hash` column.
    pub const fn hash() -> Self {
        Self::Ext(ExtensionUse::ContentHash)
    }

    /// The extension use, when this type is one.
    pub const fn extension(&self) -> Option<&ExtensionUse> {
        match self {
            Self::Ext(use_) => Some(use_),
            _ => None,
        }
    }

    /// The registry logical-type name (blueprint §4.1 `schema_logical_types.name`).
    ///
    /// ```
    /// use pse_schema::model::LogicalType;
    ///
    /// assert_eq!(LogicalType::list(LogicalType::Text).name(), "list<text>");
    /// assert_eq!(LogicalType::enumeration("Namespace").name(), "enum:Namespace");
    /// ```
    pub fn name(&self) -> String {
        match self {
            Self::F64 => "f64".to_owned(),
            Self::I64 => "i64".to_owned(),
            Self::I32 => "i32".to_owned(),
            Self::U8 => "u8".to_owned(),
            Self::U16 => "u16".to_owned(),
            Self::U32 => "u32".to_owned(),
            Self::U64 => "u64".to_owned(),
            Self::Bool => "bool".to_owned(),
            Self::Text => "text".to_owned(),
            Self::Timestamp => "ts".to_owned(),
            Self::List(element) => format!("list<{}>", element.name()),
            Self::FixedList(element, width) => format!("fixed_list<{},{width}>", element.name()),
            Self::Struct(children) => {
                let rendered: Vec<String> = children
                    .iter()
                    .map(|(name, ty, nullable)| {
                        format!("{name}:{}{}", ty.name(), if *nullable { "?" } else { "" })
                    })
                    .collect();
                format!("struct{{{}}}", rendered.join(","))
            }
            Self::Ext(use_) => use_.name(),
        }
    }

    /// The declared Arrow storage.
    ///
    /// List and fixed-list children are named `item` and are non-nullable: a declared
    /// element type is what the list holds, and "a list of maybe-nothing" is a second,
    /// undeclared contract. A column that may be absent is nullable at the column.
    pub fn data_type(&self) -> DataType {
        match self {
            Self::F64 => DataType::Float64,
            Self::I64 => DataType::Int64,
            Self::I32 => DataType::Int32,
            Self::U8 => DataType::UInt8,
            Self::U16 => DataType::UInt16,
            Self::U32 => DataType::UInt32,
            Self::U64 => DataType::UInt64,
            Self::Bool => DataType::Boolean,
            Self::Text => DataType::Utf8,
            Self::Timestamp => timestamp_storage(),
            Self::List(element) => DataType::List(
                arrow_schema::Field::new_list_field(element.data_type(), false).into(),
            ),
            Self::FixedList(element, width) => DataType::FixedSizeList(
                arrow_schema::Field::new_list_field(element.data_type(), false).into(),
                *width,
            ),
            Self::Struct(children) => DataType::Struct(
                children
                    .iter()
                    .map(|(name, ty, nullable)| {
                        arrow_schema::Field::new(*name, ty.data_type(), *nullable)
                    })
                    .collect(),
            ),
            Self::Ext(use_) => use_.spec().storage(),
        }
    }

    /// Every logical type reachable from this one, this one included.
    ///
    /// The catalog needs the children as well as the parent: a `list<semantic_id>` column
    /// declares both `list<semantic_id>` and `semantic_id`, and a reader that resolves
    /// only the outer name cannot type the child field.
    pub(crate) fn walk(&self, out: &mut Vec<Self>) {
        out.push(self.clone());
        match self {
            Self::List(element) | Self::FixedList(element, _) => element.walk(out),
            Self::Struct(children) => {
                for (_, ty, _) in children {
                    ty.walk(out);
                }
            }
            _ => {}
        }
    }
}

impl fmt::Display for LogicalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

/// One `reference.schema_logical_types` row (blueprint §4.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogicalTypeRow {
    /// `named_id(REGISTRY_PACKAGE_ID, "logical_type:<name>")` (ADR-0050).
    pub id: SemanticId,
    /// The registry name, from [`LogicalType::name`].
    pub name: String,
    /// The canonical Arrow storage rendering, from [`render_data_type`].
    pub arrow_storage: String,
    /// The `ARROW:extension:name`, when this type is an extension use.
    pub extension_name: Option<&'static str>,
    /// The JSON Schema of `ARROW:extension:metadata`, when this type is an extension use.
    pub metadata_schema: Option<String>,
}

/// The canonical rendering of an Arrow storage type (blueprint §4.1 `arrow_storage`).
///
/// Written out rather than taken from `DataType`'s `Debug`, because the rendering is a
/// registry fingerprint input and §5.3 admits no `Debug` serialization on a hashing path.
/// An unsupported layout is rejected, never rendered opportunistically (§5.3 step 2).
///
/// # Errors
///
/// [`SchemaError::UnknownReference`] when the layout is not one the registry declares.
///
/// ```
/// use arrow_schema::DataType;
/// use pse_schema::model::render_data_type;
///
/// assert_eq!(render_data_type(&DataType::FixedSizeBinary(16))?, "FixedSizeBinary(16)");
/// # Ok::<(), pse_schema::SchemaError>(())
/// ```
pub fn render_data_type(data_type: &DataType) -> Result<String, SchemaError> {
    let rendered = match data_type {
        DataType::Boolean => "Boolean".to_owned(),
        DataType::Int8 => "Int8".to_owned(),
        DataType::Int16 => "Int16".to_owned(),
        DataType::Int32 => "Int32".to_owned(),
        DataType::Int64 => "Int64".to_owned(),
        DataType::UInt8 => "UInt8".to_owned(),
        DataType::UInt16 => "UInt16".to_owned(),
        DataType::UInt32 => "UInt32".to_owned(),
        DataType::UInt64 => "UInt64".to_owned(),
        DataType::Float32 => "Float32".to_owned(),
        DataType::Float64 => "Float64".to_owned(),
        DataType::Utf8 => "Utf8".to_owned(),
        DataType::Binary => "Binary".to_owned(),
        DataType::FixedSizeBinary(width) => format!("FixedSizeBinary({width})"),
        DataType::Timestamp(unit, zone) => {
            let unit = match unit {
                arrow_schema::TimeUnit::Second => "Second",
                arrow_schema::TimeUnit::Millisecond => "Millisecond",
                arrow_schema::TimeUnit::Microsecond => "Microsecond",
                arrow_schema::TimeUnit::Nanosecond => "Nanosecond",
            };
            match zone {
                Some(zone) => format!("Timestamp({unit}, \"{zone}\")"),
                None => format!("Timestamp({unit}, None)"),
            }
        }
        DataType::List(field) => format!("List<{}>", render_field(field)?),
        DataType::FixedSizeList(field, width) => {
            format!("FixedSizeList<{}, {width}>", render_field(field)?)
        }
        DataType::Struct(fields) => {
            let mut rendered = Vec::with_capacity(fields.len());
            for field in fields {
                rendered.push(render_field(field)?);
            }
            format!("Struct<{}>", rendered.join(", "))
        }
        DataType::Dictionary(key, value) => format!(
            "Dictionary({}, {})",
            render_data_type(key)?,
            render_data_type(value)?
        ),
        other => {
            return Err(SchemaError::UnknownReference {
                context: "arrow storage rendering".to_owned(),
                reference: unsupported_layout(other),
            });
        }
    };
    Ok(rendered)
}

/// A stable, non-`Debug` label for a layout the registry does not declare.
///
/// Only ever reaches an error message, never a hash.
fn unsupported_layout(data_type: &DataType) -> String {
    let tag = match data_type {
        DataType::Null => "Null",
        DataType::Float16 => "Float16",
        DataType::Date32 | DataType::Date64 => "Date",
        DataType::Time32(_) | DataType::Time64(_) => "Time",
        DataType::Duration(_) => "Duration",
        DataType::Interval(_) => "Interval",
        DataType::LargeBinary | DataType::BinaryView => "Binary (large or view)",
        DataType::LargeUtf8 | DataType::Utf8View => "Utf8 (large or view)",
        DataType::ListView(_) | DataType::LargeListView(_) => "ListView",
        DataType::LargeList(_) => "LargeList",
        DataType::Union(_, _) => "Union",
        DataType::Map(_, _) => "Map",
        DataType::Decimal32(_, _)
        | DataType::Decimal64(_, _)
        | DataType::Decimal128(_, _)
        | DataType::Decimal256(_, _) => "Decimal",
        DataType::RunEndEncoded(_, _) => "RunEndEncoded",
        _ => "an unsupported Arrow layout",
    };
    tag.to_owned()
}

/// `<name>: <type>` with a trailing `?` when the field is nullable.
///
/// # Errors
///
/// The errors of [`render_data_type`] on the field's type.
fn render_field(field: &arrow_schema::Field) -> Result<String, SchemaError> {
    Ok(format!(
        "{}: {}{}",
        field.name(),
        render_data_type(field.data_type())?,
        if field.is_nullable() { "?" } else { "" }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_uses_index_their_own_spec() {
        for (index, spec) in EXTENSION_TYPES.iter().enumerate() {
            let use_ = match index {
                0 => ExtensionUse::SemanticId,
                1 => ExtensionUse::ContentHash,
                2 => ExtensionUse::DimensionVector,
                3 => ExtensionUse::QuantityValue,
                4 => ExtensionUse::Bound,
                5 => ExtensionUse::IndexTuple,
                6 => ExtensionUse::OrdinalRef { target: "x.y" },
                7 => ExtensionUse::SourceSpan,
                8 => ExtensionUse::Enum("X"),
                9 => ExtensionUse::ExprDsl,
                _ => ExtensionUse::TargetPath,
            };
            assert_eq!(use_.extension_name(), spec.name);
        }
    }

    #[test]
    fn storage_rendering_is_structural() {
        assert_eq!(
            render_data_type(&ExtensionUse::DimensionVector.spec().storage()).unwrap(),
            "FixedSizeList<item: Struct<num: Int16, den: Int16>, 8>"
        );
        assert_eq!(
            render_data_type(&ExtensionUse::Bound.spec().storage()).unwrap(),
            "Struct<kind: Dictionary(Int8, Utf8), value: Float64?>"
        );
        assert_eq!(
            render_data_type(&LogicalType::Timestamp.data_type()).unwrap(),
            "Timestamp(Nanosecond, \"UTC\")"
        );
    }

    #[test]
    fn an_undeclared_layout_is_rejected_not_rendered() {
        let error = render_data_type(&DataType::Float16).unwrap_err();
        assert!(matches!(error, SchemaError::UnknownReference { .. }));
    }

    #[test]
    fn walk_reaches_every_child() {
        let ty = LogicalType::Struct(vec![
            ("a", LogicalType::list(LogicalType::id()), false),
            ("b", LogicalType::F64, true),
        ]);
        let mut seen = Vec::new();
        ty.walk(&mut seen);
        let names: Vec<String> = seen.iter().map(LogicalType::name).collect();
        assert_eq!(
            names,
            vec![
                "struct{a:list<semantic_id>,b:f64?}",
                "list<semantic_id>",
                "semantic_id",
                "f64"
            ]
        );
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Recursive declarations backed by one native Arrow field. Named domain facets live
//! on that field, so nested children carry the same contract as top-level fields.

use super::{ColumnRole, ExtensionUse, ForeignKey, QuantityContract};
use arrow_schema::{DataType, Field};
use std::collections::HashMap;

const EXTENSION: &str = "pse.domain.extension";
const PARAMETER: &str = "pse.domain.parameter";
const ROLE: &str = "pse.domain.role";
const DOC: &str = "pse.domain.doc";
const PER_ROW: &str = "pse.domain.per_row_quantity";
const QUANTITY: &str = "pse.domain.quantity";
const FK_RELATION: &str = "pse.domain.fk.relation";
const FK_COLUMN: &str = "pse.domain.fk.column";

/// An exact native field with named domain facets. No separate physical type tree exists.
#[derive(Clone, Debug)]
pub struct FieldContract(Field);

// Arrow deliberately omits IPC dictionary ordering from Field equality. Exact
// application declarations include it, at every depth, while retaining Arrow's
// comparison for all other field properties. No second physical tree is stored.
impl PartialEq for FieldContract {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for FieldContract {}
impl PartialOrd for FieldContract {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for FieldContract {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .cmp(&other.0)
            .then_with(|| self.dictionary_ordering().cmp(&other.dictionary_ordering()))
    }
}
impl std::hash::Hash for FieldContract {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.dictionary_ordering().hash(state);
    }
}

impl FieldContract {
    fn dictionary_ordering(&self) -> Vec<bool> {
        let mut ordering = self.0.dict_is_ordered().into_iter().collect::<Vec<_>>();
        for child in self.children() {
            ordering.extend(child.dictionary_ordering());
        }
        ordering
    }
    /// Declare any native Arrow field without restricting engine eligibility.
    pub fn from_field(field: Field) -> Self {
        Self(field)
    }
    /// The complete declaration, including nested contracts and field metadata.
    pub fn field(&self) -> &Field {
        &self.0
    }
    /// Consume the contract as a native field.
    pub fn into_field(self) -> Field {
        self.0
    }
    /// A native physical value declaration; consumers qualify supported operations separately.
    pub fn native(data_type: DataType) -> Self {
        Self(Field::new("item", data_type, false))
    }
    /// Canonical signed storage with an explicitly bounded nonnegative domain.
    pub fn nonnegative(maximum: i64) -> Self {
        Self(super::IntegerRange::nonnegative(maximum).field("item"))
    }
    /// Native physical type, including complete nested fields.
    pub fn data_type(&self) -> DataType {
        self.0.data_type().clone()
    }
    /// The field name.
    pub fn name(&self) -> &str {
        self.0.name()
    }
    /// Whether the field admits absence.
    pub fn nullable(&self) -> bool {
        self.0.is_nullable()
    }
    /// Rename a field without changing its value contract.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.0 = self.0.with_name(name);
        self
    }
    /// Declare visibility at this field, independently of its children's nullability.
    #[must_use]
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.0 = self.0.with_nullable(nullable);
        self
    }
    /// Permit absence of this field.
    #[must_use]
    pub fn optional(self) -> Self {
        self.with_nullable(true)
    }
    fn facet(mut self, key: &str, value: impl Into<String>) -> Self {
        self.0.metadata_mut().insert(key.to_owned(), value.into());
        self
    }
    fn get(&self, key: &str) -> Option<&str> {
        self.0.metadata().get(key).map(String::as_str)
    }
    /// Field role; unannotated external fields have payload meaning only.
    pub fn role(&self) -> ColumnRole {
        ColumnRole::ALL
            .into_iter()
            .find(|role| Some(role.as_str()) == self.get(ROLE))
            .unwrap_or(ColumnRole::Payload)
    }
    /// Assign a field role.
    #[must_use]
    pub fn with_role(self, role: ColumnRole) -> Self {
        self.facet(ROLE, role.as_str())
    }
    /// Set human-readable field meaning.
    #[must_use]
    pub fn with_doc(self, doc: &str) -> Self {
        self.facet(DOC, doc)
    }
    /// Human-readable field meaning.
    pub fn doc(&self) -> &str {
        self.get(DOC).unwrap_or("")
    }
    /// Quantity facet, including the predecessor per-row form until SP02 replaces its callers.
    pub fn quantity(&self) -> QuantityContract<'_> {
        if self.get(PER_ROW) == Some("true") {
            QuantityContract::PerRow
        } else {
            self.get(QUANTITY)
                .map_or(QuantityContract::None, QuantityContract::Column)
        }
    }
    /// Reference facet, at this exact field path.
    pub fn fk(&self) -> Option<ForeignKey<'_>> {
        Some(ForeignKey::new(
            self.get(FK_RELATION)?,
            self.get(FK_COLUMN)?,
        ))
    }
    /// Attach a named reference at any depth.
    #[must_use]
    pub fn with_fk(self, relation: &str, column: &str) -> Self {
        self.facet(FK_RELATION, relation).facet(FK_COLUMN, column)
    }
    /// Assign one quantity type.
    #[must_use]
    pub fn with_quantity(mut self, quantity: &str) -> Self {
        self.0.metadata_mut().remove(PER_ROW);
        self.facet(QUANTITY, quantity)
    }
    /// Assign a quantity facet obtained from another declared field.
    #[must_use]
    pub fn with_quantity_contract(mut self, quantity: QuantityContract<'_>) -> Self {
        self.0.metadata_mut().remove(QUANTITY);
        self.0.metadata_mut().remove(PER_ROW);
        match quantity {
            QuantityContract::None => self,
            QuantityContract::Column(name) => self.with_quantity(name),
            QuantityContract::PerRow => self.with_per_row_quantity(),
        }
    }
    /// Remove a reference when a native projection no longer establishes it.
    #[must_use]
    pub fn without_fk(mut self) -> Self {
        self.0.metadata_mut().remove(FK_RELATION);
        self.0.metadata_mut().remove(FK_COLUMN);
        self
    }
    /// Per-row sibling facet, replaced by typed quantities in SP02.
    #[must_use]
    pub fn with_per_row_quantity(mut self) -> Self {
        self.0.metadata_mut().remove(QUANTITY);
        self.facet(PER_ROW, "true")
    }
    /// The required quantity identity sibling.
    pub fn per_row_quantity_sibling(&self) -> String {
        format!("{}_quantity_type_id", self.name())
    }
    /// Declare a named field from an existing native value contract.
    pub fn new(name: &str, value: Self, nullable: bool, role: ColumnRole, doc: &str) -> Self {
        value
            .with_name(name)
            .with_nullable(nullable)
            .with_role(role)
            .facet(DOC, doc)
    }
    /// A required key field.
    pub fn key(name: &str, value: Self, doc: &str) -> Self {
        Self::new(name, value, false, ColumnRole::Key, doc)
    }
    /// A required reference field.
    pub fn reference(name: &str, value: Self, doc: &str) -> Self {
        Self::new(name, value, false, ColumnRole::Reference, doc)
    }
    /// A required label field.
    pub fn label(name: &str, value: Self, doc: &str) -> Self {
        Self::new(name, value, false, ColumnRole::Label, doc)
    }
    /// A required payload field.
    pub fn payload(name: &str, value: Self, doc: &str) -> Self {
        Self::new(name, value, false, ColumnRole::Payload, doc)
    }
    /// A required provenance field.
    pub fn provenance(name: &str, value: Self, doc: &str) -> Self {
        Self::new(name, value, false, ColumnRole::Provenance, doc)
    }
    /// A list of explicitly declared fields, including element name, nullability and facets.
    pub fn list(element: Self) -> Self {
        Self::native(DataType::List(element.into_field().into()))
    }
    /// A fixed-size list of explicitly declared fields.
    pub fn fixed_list(element: Self, width: i32) -> Self {
        Self::native(DataType::FixedSizeList(element.into_field().into(), width))
    }
    /// A native struct with complete child declarations.
    pub fn structure(fields: Vec<Self>) -> Self {
        Self::native(DataType::Struct(
            fields.into_iter().map(Self::into_field).collect(),
        ))
    }
    /// Use a named domain extension on its declared physical storage.
    pub fn extended(use_: ExtensionUse<'_>) -> Self {
        let field = Self::native(use_.spec().storage()).facet(EXTENSION, use_.extension_name());
        match use_ {
            ExtensionUse::Enum(name) => field.facet(PARAMETER, name),
            ExtensionUse::OrdinalRef { target } => Self(
                super::IntegerRange::NONNEGATIVE
                    .annotate(field.facet(PARAMETER, target).into_field()),
            ),
            _ => field,
        }
    }
    /// Semantic identity value.
    pub fn id() -> Self {
        Self::extended(ExtensionUse::SemanticId)
    }
    /// Content digest value.
    pub fn hash() -> Self {
        Self::extended(ExtensionUse::ContentHash)
    }
    /// Named enumeration value.
    pub fn enumeration(name: &str) -> Self {
        Self::extended(ExtensionUse::Enum(name))
    }
    /// The enum domain named by an unbound declaration or a projected native field.
    /// Resolving this name does not establish that the field or its values are valid.
    pub fn enum_name(&self) -> Option<&str> {
        Self::enum_name_of(self.field())
    }
    pub(crate) fn enum_name_of(field: &Field) -> Option<&str> {
        let metadata = field.metadata();
        if metadata.get(EXTENSION).map(String::as_str) == Some("pse.enum") {
            return metadata.get(PARAMETER).map(String::as_str);
        }
        (metadata
            .get(crate::arrow::KEY_EXTENSION_NAME)
            .map(String::as_str)
            == Some("pse.enum"))
        .then(|| {
            metadata
                .get(crate::arrow::KEY_LOGICAL_TYPE)?
                .strip_prefix("enum:")
        })
        .flatten()
    }
    /// Referenced member domains, including children owned by composite extensions.
    pub fn enum_domains(&self) -> std::collections::BTreeSet<String> {
        let mut names = self
            .enum_name()
            .map(str::to_owned)
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        for child in self.children() {
            names.extend(child.enum_domains());
        }
        names
    }
    /// The domain extension facet; arbitrary non-PSE metadata remains on the native field.
    pub fn extension(&self) -> Option<ExtensionUse<'_>> {
        Some(match self.get(EXTENSION)? {
            "pse.semantic_id" => ExtensionUse::SemanticId,
            "pse.content_hash" => ExtensionUse::ContentHash,
            "pse.dimension_vector" => ExtensionUse::DimensionVector,
            "pse.quantity_value" => ExtensionUse::QuantityValue,
            "pse.bound" => ExtensionUse::Bound,
            "pse.index_tuple" => ExtensionUse::IndexTuple,
            "pse.ordinal_ref" => ExtensionUse::OrdinalRef {
                target: self.get(PARAMETER)?,
            },
            "pse.source_span" => ExtensionUse::SourceSpan,
            "pse.enum" => ExtensionUse::Enum(self.get(PARAMETER)?),
            "pse.expr_dsl" => ExtensionUse::ExprDsl,
            "pse.target_path" => ExtensionUse::TargetPath,
            _ => return None,
        })
    }
    /// A value-type projection for comparison, excluding this field's name, absence and role.
    /// Nested fields remain exact; no second declaration is stored.
    #[must_use]
    pub fn value_type(&self) -> Self {
        let metadata: HashMap<_, _> = self
            .0
            .metadata()
            .iter()
            .filter(|(key, _)| {
                !matches!(
                    key.as_str(),
                    ROLE | DOC | QUANTITY | PER_ROW | FK_RELATION | FK_COLUMN
                )
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Self(
            self.0
                .clone()
                .with_name("item")
                .with_nullable(false)
                .with_metadata(metadata),
        )
    }
    /// Stable native type identity, with semantic extension parameters.
    ///
    /// # Errors
    /// Native serialization failures are reported to registry admission.
    pub fn type_name(&self) -> Result<String, crate::SchemaError> {
        let value = self.value_type();
        if value.0.metadata().keys().any(|key| {
            !matches!(
                key.as_str(),
                EXTENSION | PARAMETER | super::integer_range::KEY_INTEGER_RANGE
            )
        }) {
            return render_field(value.field());
        }
        if let Some(use_) = self.extension() {
            return Ok(use_.name());
        }
        Ok(match self.0.data_type() {
            DataType::Float64 => "f64".into(),
            DataType::Int64 => "i64".into(),
            DataType::Int32 => "i32".into(),
            DataType::UInt8 => "u8".into(),
            DataType::UInt16 => "u16".into(),
            DataType::UInt32 => "u32".into(),
            DataType::UInt64 => "u64".into(),
            DataType::Boolean => "bool".into(),
            DataType::Utf8 => "text".into(),
            other if other == &super::extension::timestamp_storage() => "ts".into(),
            other => render_data_type(other)?,
        })
    }
    /// Complete native declaration for registry inspection and fingerprints.
    ///
    /// # Errors
    /// Native serialization failures are reported, never replaced by a hash fallback.
    pub fn canonical_json(&self) -> Result<String, crate::SchemaError> {
        render_field(self.field())
    }
    /// Exact equality eligibility is a domain property, independent of engine eligibility.
    pub fn admits_exact_key(&self) -> bool {
        match self.extension() {
            Some(
                ExtensionUse::SemanticId
                | ExtensionUse::ContentHash
                | ExtensionUse::Enum(_)
                | ExtensionUse::OrdinalRef { .. }
                | ExtensionUse::IndexTuple,
            ) => true,
            Some(_) => false,
            None => matches!(
                self.0.data_type(),
                DataType::Boolean
                    | DataType::Int8
                    | DataType::Int16
                    | DataType::Int32
                    | DataType::Int64
                    | DataType::UInt8
                    | DataType::UInt16
                    | DataType::UInt32
                    | DataType::UInt64
                    | DataType::Utf8
                    | DataType::LargeUtf8
                    | DataType::Utf8View
                    | DataType::Timestamp(..)
            ),
        }
    }
    pub(crate) fn validate_facets(&self, context: &str) -> Result<(), crate::SchemaError> {
        let invalid = |message| crate::checks::invalid(context, message);
        for key in self
            .0
            .metadata()
            .keys()
            .filter(|key| key.starts_with("pse.domain."))
        {
            if !matches!(
                key.as_str(),
                EXTENSION | PARAMETER | ROLE | DOC | QUANTITY | PER_ROW | FK_RELATION | FK_COLUMN
            ) {
                return Err(invalid("unknown domain facet"));
            }
        }
        if self.get(EXTENSION).is_some() && self.extension().is_none() {
            return Err(invalid("unknown or incomplete domain extension"));
        }
        if self.get(PARAMETER).is_some()
            && !matches!(
                self.extension(),
                Some(ExtensionUse::Enum(_) | ExtensionUse::OrdinalRef { .. })
            )
        {
            return Err(invalid("extension parameter has no parameterized domain"));
        }
        if self
            .get(ROLE)
            .is_some_and(|role| !ColumnRole::ALL.iter().any(|known| known.as_str() == role))
        {
            return Err(invalid("unknown field role"));
        }
        if self.get(PER_ROW).is_some_and(|value| value != "true")
            || (self.get(PER_ROW).is_some() && self.get(QUANTITY).is_some())
        {
            return Err(invalid("ambiguous per-row quantity declaration"));
        }
        if self.get(FK_RELATION).is_some() != self.get(FK_COLUMN).is_some() {
            return Err(invalid("incomplete field reference"));
        }
        Ok(())
    }
    /// Every explicitly declared native child, in native order.
    pub fn children(&self) -> Vec<Self> {
        use DataType::{
            Dictionary, FixedSizeList, LargeList, LargeListView, List, ListView, Map,
            RunEndEncoded, Struct, Union,
        };
        match self.0.data_type() {
            List(f)
            | LargeList(f)
            | ListView(f)
            | LargeListView(f)
            | FixedSizeList(f, _)
            | Map(f, _) => vec![Self((**f).clone())],
            Struct(fs) => fs.iter().map(|f| Self((**f).clone())).collect(),
            Union(fs, _) => fs.iter().map(|(_, f)| Self((**f).clone())).collect(),
            Dictionary(_, value) => Self::native((**value).clone()).children(),
            RunEndEncoded(a, b) => vec![Self((**a).clone()), Self((**b).clone())],
            _ => vec![],
        }
    }
    /// Walk the native tree; extensions remain semantic leaves for domain consumers.
    pub(crate) fn walk(&self, out: &mut Vec<Self>) {
        out.push(self.clone());
        if self.extension().is_none() {
            for child in self.children() {
                child.walk(out);
            }
        }
    }
}
impl std::fmt::Display for FieldContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.extension() {
            Some(use_) => f.write_str(&use_.name()),
            None => self.0.data_type().fmt(f),
        }
    }
}

/// One `reference.schema_logical_types` row (blueprint §4.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldTypeRow {
    /// `named_id(REGISTRY_PACKAGE_ID, "logical_type:<name>")` (ADR-0050).
    pub id: pse_ids::SemanticId,
    /// The registry name, from [`FieldContract::type_name`].
    pub name: String,
    /// The canonical Arrow storage rendering, from [`render_data_type`].
    pub arrow_storage: String,
    /// The `ARROW:extension:name`, when this type is an extension use.
    pub extension_name: Option<String>,
    /// The JSON Schema of `ARROW:extension:metadata`, when this type is an extension use.
    pub metadata_schema: Option<String>,
}

/// Deterministic native Arrow serialization, including every nested field and its metadata.
///
/// # Errors
/// Returns a schema error if native serialization fails.
pub fn render_data_type(data_type: &DataType) -> Result<String, crate::SchemaError> {
    canonical_json(data_type)
}

/// Deterministic serialization of the entire native field declaration.
///
/// # Errors
/// Native serialization failures are reported to registry admission.
pub fn render_field(field: &Field) -> Result<String, crate::SchemaError> {
    canonical_json(field)
}

fn canonical_json(value: &impl serde::Serialize) -> Result<String, crate::SchemaError> {
    let mut value = serde_json::to_value(value)
        .map_err(|error| crate::checks::invalid("Arrow declaration", error.to_string()))?;
    value.sort_all_objects();
    serde_json::to_string(&value)
        .map_err(|error| crate::checks::invalid("Arrow declaration", error.to_string()))
}

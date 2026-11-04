// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owner-emitted boundary structure. This graph describes actual Rust field types;
//! it never interprets JSON Schema or replaces the scientific registry.

use std::collections::{BTreeMap, BTreeSet};

mod access;
mod array;
pub use array::{fixed_array_name, fixed_array_shape};
mod bytes;
mod float_collections;
mod float_domain;
mod float_interval;
mod generic;
pub use generic::{argument_name as projection_argument_name, optional_argument_name, sequence_argument_name, array_argument_name};
mod nominal;
pub use nominal::identity_shape;
#[cfg(test)]
mod optional_text_tests;
#[cfg(test)]
mod observation_struct_tests;
mod path;
mod public_alias_path;
mod remote;
mod standard;
mod tuple;
mod tuple_pair;
mod tuple_triple;
pub use access::{
    Adapted, AdapterError, AdapterFailure, CheckedProjectionRecord, ProjectionAdapter,
    ProjectionAlternative, ProjectionObservationRecord, ProjectionRecord, SharedAdapter, adapter_source_path,
    clone_adapter_field, validate_adapted,
};
pub use float_collections::NativeContainer;
pub use bytes::ProjectionBytes;
pub use float_collections::NominalFloat64;
pub use float_domain::Float64Domain;
pub use float_interval::Float64Interval;
pub use generic::generic_argument;
pub use path::PlatformPathBytes;

/// The shared public direction, independently of native Serde representation.
pub use crate::BoundaryDirection as Direction;

/// Primitive wire meaning. Floating point values deliberately project their exact bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scalar {
    /// Boolean value.
    Bool,
    /// Unicode text.
    String,
    /// Unsigned 32-bit value.
    U32,
    /// An octet; a vector of octets projects as binary data.
    U8,
    /// Unsigned 16-bit native value, carried in a range-checked unsigned 32-bit wire scalar.
    U16,
    /// Unsigned 64-bit value.
    U64,
    /// Signed 16-bit native value, carried in a range-checked signed 32-bit wire scalar.
    I16,
    /// Signed 32-bit value.
    I32,
    /// Signed 64-bit value.
    I64,
    /// IEEE 754 binary32 bit pattern, including signed zero and NaN payloads.
    F32Bits,
    /// IEEE 754 binary64 bit pattern, including signed zero and NaN payloads.
    F64Bits,
    /// Opaque binary data.
    Bytes,
    /// A 16-byte semantic identity, distinct from textual spelling.
    SemanticId,
    /// A 32-byte semantic content hash.
    ContentHash,
    /// Exact signed 128-bit two's-complement value in sixteen little-endian bytes.
    I128Bits,
    /// Signed 8-bit native value in a checked signed 32-bit wire scalar.
    I8,
}

/// Resolved shape, preserving optional and collection nesting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRef {
    /// Primitive representation.
    Scalar(Scalar),
    /// Named owner in the same graph.
    Named(String),
    /// Explicit absence, distinct from the contained type's default.
    Optional(Box<Self>),
    /// Ordered repeated values, including an empty collection.
    Repeated(Box<Self>),
    /// Unordered unique values, canonically ordered by their native owner.
    Set(Box<Self>),
    /// A key/value collection with explicit key and value meaning.
    Map(Box<Self>, Box<Self>),
}

/// Shared locally decidable constraints; contextual scientific checks stay with their owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalRule {
    /// Require a present value/alternative on uncontrolled ingress.
    Required,
    /// Actual standard Duration must be nonzero; general Duration still permits zero.
    PositiveDuration,
    /// Exact locally declared binary64 domain, independent of its fixed64 wire bits.
    Float64Domain(Float64Domain),
    /// Inclusive signed integer domain, preserving exact native width.
    SignedRange {
        /// Inclusive lower bound.
        min: i64,
        /// Inclusive upper bound.
        max: i64,
    },
    /// Inclusive unsigned integer range, evaluated without floating point conversion.
    UnsignedRange {
        /// Inclusive lower bound.
        min: u64,
        /// Inclusive upper bound.
        max: u64,
    },
    /// Inclusive collection/text/binary length bounds.
    Length {
        /// Inclusive lower bound.
        min: u64,
        /// Inclusive upper bound.
        max: u64,
    },
    /// Inclusive UTF-8 byte bounds, distinct from Unicode scalar-value text length.
    Utf8Bytes {
        /// Inclusive lower bound.
        min: u64,
        /// Inclusive upper bound.
        max: u64,
    },
}

impl schemars::transform::Transform for LocalRule {
    /// Numeric range declarations can also annotate intentional native JSON schemas.
    /// Other rules need their representation/parent context and refuse as a false
    /// schema when used as an isolated field transform.
    fn transform(&mut self, schema: &mut schemars::Schema) {
        match self {
            Self::UnsignedRange { min, max } => {
                schema
                    .ensure_object()
                    .insert("minimum".into(), (*min).into());
                schema
                    .ensure_object()
                    .insert("maximum".into(), (*max).into());
            }
            Self::SignedRange { min, max } => {
                schema
                    .ensure_object()
                    .insert("minimum".into(), (*min).into());
                schema
                    .ensure_object()
                    .insert("maximum".into(), (*max).into());
            }
            _ => *schema = schemars::Schema::from(false),
        }
    }
}

/// One field declared beside its actual Rust field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// Stable public field name.
    pub name: String,
    /// Explicit stable protobuf number; never derived from position.
    pub tag: u32,
    /// Actual recursively resolved Rust type.
    pub ty: TypeRef,
    /// Public direction.
    pub direction: Direction,
    /// Owner-declared local constraints.
    pub rules: Vec<LocalRule>,
    /// Exact resolved named registry facets for source links and documentation.
    /// They confer no contextual scientific admission or execution permission.
    pub named_facets: BTreeMap<String, String>,
}

impl Field {
    /// Resolve the actual declared field type.
    pub fn of<T: ProjectionType>(
        graph: &mut ProjectionGraph,
        name: &str,
        tag: u32,
        direction: Direction,
        rules: Vec<LocalRule>,
    ) -> Result<Self, ProjectionError> {
        Ok(Self {
            name: name.into(),
            tag,
            ty: T::projection(graph)?,
            direction,
            rules,
            named_facets: BTreeMap::new(),
        })
    }

    /// Retain the actual native field annotations without interpreting a published schema.
    #[must_use]
    pub fn with_named_facets(mut self, facets: BTreeMap<String, String>) -> Self {
        self.named_facets = facets;
        self
    }

    /// Select a bit representation only when it agrees with the actual integer width.
    pub fn representation(mut self, representation: Scalar) -> Result<Self, ProjectionError> {
        if !matches!(
            (&self.ty, representation),
            (TypeRef::Scalar(Scalar::U64), Scalar::F64Bits)
                | (TypeRef::Scalar(Scalar::U32), Scalar::F32Bits)
        ) {
            return Err(ProjectionError::invalid(
                &self.name,
                "bit representation does not match the actual integer field width",
            ));
        }
        self.ty = TypeRef::Scalar(representation);
        Ok(self)
    }
}

/// A tagged alternative, with its own named payload fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alternative {
    /// Stable alternative name.
    pub name: String,
    /// Stable oneof field tag.
    pub tag: u32,
    /// Empty for a unit variant; otherwise actual payload fields.
    pub fields: Vec<Field>,
}

/// A named shape's field or alternative layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Record fields.
    Record(Vec<Field>),
    /// A required tagged alternative, represented as a protobuf oneof.
    Alternatives(Vec<Alternative>),
    /// Registry-owned canonical string members, without invented numeric alternatives.
    StringEnum(Vec<String>),
}

/// Source-owned stable projection declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionRole {
    /// Reconstructible data; any contextual admission remains with its native owner.
    Data,
    /// Borrowed observations of a live owner, with no inverse native construction.
    ObservationOnly,
}

/// Source-owned stable projection declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    /// Stable public message name, independent of its Rust module.
    pub name: String,
    /// Actual Rust owner identity, used to reject name collisions.
    pub rust_type: &'static str,
    /// Nameable public Rust alias for generated adapters, when explicitly supplied.
    pub public_rust_path: Option<String>,
    /// Source file emitted by the declaration macro.
    pub source: &'static str,
    /// Owner's interpretation version, separate from package and document versions.
    pub version: u32,
    /// Public direction.
    pub direction: Direction,
    /// Removed field/alternative numbers, never available for reuse.
    pub reserved_tags: Vec<u32>,
    /// Removed field/alternative names, never available for reuse.
    pub reserved_names: Vec<String>,
    /// Actual field structure.
    pub shape: Shape,
    /// Checked foreign extraction is required; ordinary actual-owner records remain infallible.
    pub checked_record: bool,
    /// Intrinsic container facts supplied by the checked standard-container owner.
    pub native_container: Option<NativeContainer>,
    /// Native construction policy, independent of public input/output direction.
    pub construction: ConstructionRole,
    /// Original source action intent, independent of service method composition.
    pub effects: Option<crate::operation_effects::EffectFacts>,
}

/// One actual unary service boundary, resolved in the same descriptor world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Method {
    /// Stable public method name.
    pub name: String,
    /// Actual request owner.
    pub input: TypeRef,
    /// Actual response owner.
    pub output: TypeRef,
    /// Whether the response is a finite or observed server stream.
    pub server_streaming: bool,
    /// Actual operation owner's declared effects, when explicitly attached.
    pub effects: Option<crate::operation_effects::EffectFacts>,
}

/// Source-selected service and its actual request/response roots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Service {
    /// Stable public service name.
    pub name: String,
    /// Explicit unary methods.
    pub methods: Vec<Method>,
}

impl Service {
    /// Start an explicitly named service declaration.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            methods: Vec::new(),
        }
    }

    /// Resolve an actual request/response pair and retain the method's stable name.
    pub fn unary<I: ProjectionType, O: ProjectionType>(
        mut self,
        graph: &mut ProjectionGraph,
        name: &str,
    ) -> Result<Self, ProjectionError> {
        if !valid_identifier(name) || self.methods.iter().any(|method| method.name == name) {
            return Err(ProjectionError::invalid(
                &self.name,
                "invalid or duplicate method name",
            ));
        }
        let input = graph.register::<I>()?;
        let output = graph.register::<O>()?;
        if !matches!(input, TypeRef::Named(_)) || !matches!(output, TypeRef::Named(_)) {
            return Err(ProjectionError::invalid(
                &self.name,
                "service request and response must be named owners",
            ));
        }
        self.methods.push(Method {
            name: name.into(),
            input,
            output,
            server_streaming: false,
            effects: None,
        });
        Ok(self)
    }

    /// Resolve a server-streaming actual request/response pair in this descriptor world.
    pub fn server_streaming<I: ProjectionType, O: ProjectionType>(
        self,
        graph: &mut ProjectionGraph,
        name: &str,
    ) -> Result<Self, ProjectionError> {
        let mut service = self.unary::<I, O>(graph, name)?;
        if let Some(method) = service.methods.last_mut() {
            method.server_streaming = true;
        }
        Ok(service)
    }

    /// Attach original owner facts to the preceding method, without inventing defaults.
    pub fn effects(
        mut self,
        effects: crate::operation_effects::EffectFacts,
    ) -> Result<Self, ProjectionError> {
        let method = self.methods.last_mut().ok_or_else(|| {
            ProjectionError::invalid(
                &self.name,
                "effect facts require an actual preceding method",
            )
        })?;
        if method.effects.replace(effects).is_some() {
            return Err(ProjectionError::invalid(
                &self.name,
                "method effect facts were declared more than once",
            ));
        }
        Ok(self)
    }
}

/// Invalid or unsupported owner projection.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[derive(miette::Diagnostic)]
#[diagnostic(code("validation.invariant"))]
pub enum ProjectionError {
    /// A declaration cannot form an unambiguous supported boundary.
    #[error("invalid boundary projection `{owner}`: {reason}")]
    Invalid {
        /// Actual owner or field path.
        owner: String,
        /// Original refusal reason.
        reason: String,
    },
}
impl ProjectionError {
    /// Refuse an invalid owner or unsupported actual type.
    pub fn invalid(owner: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Invalid {
            owner: owner.into(),
            reason: reason.into(),
        }
    }
}

/// A locally decidable rule violation, separate from contextual or infrastructure refusals.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[derive(miette::Diagnostic)]
#[diagnostic(code("workflow.input"))]
pub enum ValidationError {
    /// A declared rule rejected the actual native value.
    #[error("boundary rule `{rule}` failed at `{path}`: {reason}")]
    Rule {
        /// Full owner/field/index path.
        path: String,
        /// Stable local rule identifier.
        rule: &'static str,
        /// Exact rejected envelope.
        reason: String,
    },
}
/// Native validation derived by the same owner declaration as protobuf rules.
pub trait ValidateProjection {
    /// Whether this actual scalar/container declaration consists of binary64 leaves.
    /// Unrelated named records never opt in by resembling a numeric container.
    const SUPPORTS_FLOAT64_DOMAIN: bool = false;
    /// Whether recursive traversal can discover any additional native predicate.
    /// The conservative default preserves validation for custom owner implementations.
    const NEEDS_RECURSIVE_VALIDATION: bool = true;
    /// Validate all local fields, preserving the original native value.
    fn validate_projection(&self) -> Result<(), ValidationError> {
        self.validate_at("")
    }
    /// Validate recursively at an already attributed field path.
    fn validate_at(&self, path: &str) -> Result<(), ValidationError>;
    /// Whether an optional value is actually present.
    fn projection_present(&self) -> bool {
        true
    }
    /// Exact unsigned scalar, where applicable.
    fn projection_unsigned(&self) -> Option<u64> {
        None
    }
    /// Exact signed scalar, where applicable.
    fn projection_signed(&self) -> Option<i64> {
        None
    }
    /// Actual binary64 scalar, where a source-owned domain rule is declared.
    fn projection_float64(&self) -> Option<f64> {
        None
    }
    /// Evaluate a declared binary64 domain across the actual scalar/container.
    fn projection_float64_domain(&self, domain: Float64Domain) -> Option<bool> {
        self.projection_float64()
            .map(|value| domain.contains(value))
    }
    /// Actual standard duration, where a source owner declares its intrinsic domain.
    fn projection_duration(&self) -> Option<std::time::Duration> {
        None
    }
    /// Logical length (Unicode scalar values for text), where applicable.
    fn projection_length(&self) -> Option<usize> {
        None
    }
    /// Actual UTF-8 byte extent, where applicable.
    fn projection_utf8_bytes(&self) -> Option<usize> {
        None
    }
}

/// Apply a declaration's local rules to the actual field and its recursive children.
pub fn validate_field<T: ValidateProjection + ?Sized>(
    value: &T,
    path: &str,
    rules: &[LocalRule],
) -> Result<(), ValidationError> {
    for rule in rules {
        // Non-presence predicates constrain a supplied optional value. Absence is
        // governed only by Required, matching protobuf optional-field validation.
        if !value.projection_present() && !matches!(rule, LocalRule::Required) {
            continue;
        }
        let failure = match rule {
            LocalRule::PositiveDuration
                if !value
                    .projection_duration()
                    .is_some_and(|value| !value.is_zero()) =>
            {
                Some((
                    "duration.positive",
                    "a positive duration is required".into(),
                ))
            }
            LocalRule::Float64Domain(domain)
                if !value
                    .projection_float64_domain(*domain)
                    .is_some_and(|valid| valid) =>
            {
                Some((
                    "float64.domain",
                    format!("expected binary64 domain {domain:?}"),
                ))
            }
            LocalRule::Required if !value.projection_present() => {
                Some(("required", "a present value is required".into()))
            }
            LocalRule::UnsignedRange { min, max }
                if !value
                    .projection_unsigned()
                    .is_some_and(|value| value >= *min && value <= *max) =>
            {
                Some((
                    "unsigned.range",
                    format!("expected inclusive range {min}..={max}"),
                ))
            }
            LocalRule::SignedRange { min, max }
                if !value
                    .projection_signed()
                    .is_some_and(|value| value >= *min && value <= *max) =>
            {
                Some((
                    "signed.range",
                    format!("expected inclusive range {min}..={max}"),
                ))
            }
            LocalRule::Length { min, max }
                if !value
                    .projection_length()
                    .and_then(|len| u64::try_from(len).ok())
                    .is_some_and(|len| len >= *min && len <= *max) =>
            {
                Some(("length", format!("expected inclusive length {min}..={max}")))
            }
            LocalRule::Utf8Bytes { min, max }
                if !value
                    .projection_utf8_bytes()
                    .and_then(|length| u64::try_from(length).ok())
                    .is_some_and(|length| length >= *min && length <= *max) =>
            {
                Some((
                    "utf8_bytes",
                    format!("UTF-8 byte extent must be in {min}..={max}"),
                ))
            }
            _ => None,
        };
        if let Some((rule, reason)) = failure {
            return Err(ValidationError::Rule {
                path: path.into(),
                rule,
                reason,
            });
        }
    }
    value.validate_at(path)
}

/// Extend a field path without losing nesting or introducing a leading separator.
pub fn field_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.into()
    } else if child.is_empty() {
        parent.into()
    } else {
        format!("{parent}.{child}")
    }
}

/// Complete recursive closure over selected actual owners.
#[derive(Clone, Debug, Default)]
pub struct ProjectionGraph {
    definitions: BTreeMap<String, Definition>,
    building: BTreeMap<String, &'static str>,
    roots: Vec<TypeRef>,
    services: BTreeMap<String, Service>,
}

impl ProjectionGraph {
    /// Resolve one selected owner and its transitive type closure.
    pub fn of<T: ProjectionType>() -> Result<Self, ProjectionError> {
        let mut graph = Self::default();
        graph.register::<T>()?;
        Ok(graph)
    }

    /// Add another selected root to this same descriptor world.
    pub fn register<T: ProjectionType>(&mut self) -> Result<TypeRef, ProjectionError> {
        let root = T::projection(self)?;
        if !self.roots.contains(&root) {
            self.roots.push(root.clone());
        }
        Ok(root)
    }

    /// All resolved owner declarations in deterministic name order.
    pub fn definitions(&self) -> &BTreeMap<String, Definition> {
        &self.definitions
    }

    /// Selected roots, rather than every reachable payload.
    pub fn roots(&self) -> &[TypeRef] {
        &self.roots
    }

    /// Services resolved into this descriptor world.
    pub fn services(&self) -> &BTreeMap<String, Service> {
        &self.services
    }

    /// Register a selected service, rejecting duplicate names and dangling owners.
    pub fn service(&mut self, service: Service) -> Result<(), ProjectionError> {
        if !valid_identifier(&service.name)
            || service.methods.is_empty()
            || self.services.contains_key(&service.name)
        {
            return Err(ProjectionError::invalid(
                &service.name,
                "invalid, empty or duplicate service",
            ));
        }
        for method in &service.methods {
            for ty in [&method.input, &method.output] {
                if !matches!(ty, TypeRef::Named(name) if self.definitions.contains_key(name)) {
                    return Err(ProjectionError::invalid(
                        &service.name,
                        "method references an owner outside this descriptor world",
                    ));
                }
            }
        }
        self.services.insert(service.name.clone(), service);
        Ok(())
    }

    /// Register an owner before walking its fields, allowing self and mutual recursion.
    #[allow(
        clippy::too_many_arguments,
        reason = "declaration metadata stays explicit at the owner"
    )]
    pub fn declare<T: ?Sized>(
        &mut self,
        name: &str,
        source: &'static str,
        version: u32,
        direction: Direction,
        reserved_tags: &[u32],
        reserved_names: &[&str],
        shape: impl FnOnce(&mut Self) -> Result<Shape, ProjectionError>,
    ) -> Result<TypeRef, ProjectionError> {
        let rust_type = std::any::type_name::<T>();
        let previous = self
            .definitions
            .get(name)
            .map(|d| d.rust_type)
            .or_else(|| self.building.get(name).copied());
        if let Some(previous) = previous {
            if previous != rust_type {
                return Err(ProjectionError::invalid(
                    name,
                    format!("public name belongs to both {previous} and {rust_type}"),
                ));
            }
            return Ok(TypeRef::Named(name.into()));
        }
        if !valid_identifier(name) || version == 0 {
            return Err(ProjectionError::invalid(
                name,
                "a protobuf identifier and nonzero interpretation version are required",
            ));
        }
        self.building.insert(name.into(), rust_type);
        let result = shape(self);
        self.building.remove(name);
        let definition = Definition {
            name: name.into(),
            rust_type,
            public_rust_path: None,
            source,
            version,
            direction,
            reserved_tags: reserved_tags.to_vec(),
            reserved_names: reserved_names.iter().map(|s| (*s).into()).collect(),
            shape: result?,
            checked_record: false,
            native_container: None,
            construction: ConstructionRole::Data,
            effects: None,
        };
        validate_definition(&definition, &self.definitions)?;
        self.definitions.insert(name.into(), definition);
        Ok(TypeRef::Named(name.into()))
    }

    /// Attach original source action effects without inferring service behavior.
    /// # Errors
    /// The owner must exist, effect facts must be coherent and repeated source
    /// registration must retain the same declaration.
    pub fn record_effects(&mut self, name: &str, effects: crate::operation_effects::EffectFacts) -> Result<(), ProjectionError> {
        let owner = self.definitions.get_mut(name).ok_or_else(|| ProjectionError::invalid(name, "action effect owner is unresolved"))?;
        if !crate::metadata::effects_coherent(effects.scope, effects.read_only, effects.destructive) {
            return Err(ProjectionError::invalid(name, "action effect scope conflicts with observation or custody release facts"));
        }
        if owner.effects.is_some_and(|previous| previous != effects) {
            return Err(ProjectionError::invalid(name, "conflicting original action effect declarations"));
        }
        owner.effects = Some(effects);
        Ok(())
    }

    /// Declare borrowed observation access at the actual native owner.
    /// # Errors
    /// The root must identify this exact record and cannot also use checked adoption.
    pub fn observation_record<T: ProjectionObservationRecord>(
        &mut self,
        root: &TypeRef,
    ) -> Result<(), ProjectionError> {
        let TypeRef::Named(name) = root else {
            return Err(ProjectionError::invalid("observation", "a named record is required"));
        };
        if self.building.get(name).is_some_and(|owner| *owner == std::any::type_name::<T>()) {
            // Recursive registration completes the same outer declaration, which
            // installs the role after its actual fields have been resolved.
            return Ok(());
        }
        let owner = self.definitions.get_mut(name)
            .ok_or_else(|| ProjectionError::invalid(name, "observation owner is unresolved"))?;
        if owner.rust_type != std::any::type_name::<T>()
            || !matches!(owner.shape, Shape::Record(_)) || owner.checked_record
        {
            return Err(ProjectionError::invalid(name, "observation access must identify its actual native record"));
        }
        owner.construction = ConstructionRole::ObservationOnly;
        Ok(())
    }

    /// Attach the checked foreign-record access contract to its actual typed carrier.
    pub fn checked_record<T>(&mut self, root: &TypeRef) -> Result<(), ProjectionError> {
        let TypeRef::Named(name) = root else {
            return Err(ProjectionError::invalid(
                "checked record",
                "a named record is required",
            ));
        };
        if self
            .building
            .get(name)
            .is_some_and(|owner| *owner == std::any::type_name::<T>())
        {
            return Ok(());
        }
        let owner = self
            .definitions
            .get_mut(name)
            .ok_or_else(|| ProjectionError::invalid(name, "checked record owner is unresolved"))?;
        if owner.rust_type != std::any::type_name::<T>() || !matches!(owner.shape, Shape::Record(_))
        {
            return Err(ProjectionError::invalid(
                name,
                "checked access must identify its actual record carrier",
            ));
        }
        if owner.construction == ConstructionRole::ObservationOnly {
            return Err(ProjectionError::invalid(name, "observation owner has no checked inverse adoption"));
        }
        owner.checked_record = true;
        Ok(())
    }

    /// Record a checked owner macro's public Rust alias; this changes no field meaning.
    pub fn public_path<T: ?Sized>(
        &mut self,
        root: &TypeRef,
        path: &str,
    ) -> Result<(), ProjectionError> {
        let TypeRef::Named(name) = root else {
            return Err(ProjectionError::invalid(
                path,
                "only a named owner has a public Rust alias",
            ));
        };
        if self
            .building
            .get(name)
            .is_some_and(|owner| *owner == std::any::type_name::<T>())
        {
            // A recursive visit returns before the outer declaration is installed. The
            // outer call records this same source-owned alias after completing its shape.
            return Ok(());
        }
        let definition = self
            .definitions
            .get_mut(name)
            .ok_or_else(|| ProjectionError::invalid(path, "public alias owner is unresolved"))?;
        if definition.rust_type != std::any::type_name::<T>() {
            return Err(ProjectionError::invalid(
                path,
                "public alias identifies another actual Rust owner",
            ));
        }
        let compact = path.replace(' ', "");
        let expanded = if let Some(relative) = compact.strip_prefix("crate::") {
            let owner_crate = definition.rust_type.split("::").next().unwrap_or_default();
            format!("{owner_crate}::{relative}")
        } else {
            compact
        };
        if definition
            .public_rust_path
            .as_ref()
            .is_some_and(|previous| previous != &expanded)
        {
            return Err(ProjectionError::invalid(
                path,
                "owner public alias was declared more than once",
            ));
        }
        definition.public_rust_path = Some(expanded);
        Ok(())
    }
}

/// Implemented by actual owner declarations and supported field types.
/// Unsupported types have no implementation and therefore fail at their declaration.
pub trait ProjectionType {
    /// Resolve the type and all recursively referenced owners.
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError>;
    /// Stable argument identity for a generic declaration, preserving native container kinds.
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(generic::argument_name(&Self::projection(graph)?))
    }
    /// Nameable native argument path, using an explicitly checked alias when supplied.
    /// # Errors
    /// An unresolved recursive owner cannot supply its public argument path yet.
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        match Self::projection(graph)? {
            TypeRef::Named(name) => {
                let owner = graph.definitions().get(&name).ok_or_else(|| {
                    ProjectionError::invalid(
                        &name,
                        "generic argument owner is still being resolved",
                    )
                })?;
                Ok(owner
                    .public_rust_path
                    .clone()
                    .unwrap_or_else(|| owner.rust_type.into()))
            }
            _ => Ok(core::any::type_name::<Self>().into()),
        }
    }
}

macro_rules! scalar_types { ($($ty:ty => $scalar:ident),* $(,)?) => { $(
    impl ProjectionType for $ty {
        fn projection(_: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
            Ok(TypeRef::Scalar(Scalar::$scalar))
        }
        fn projection_rust_path(_: &mut ProjectionGraph) -> Result<String, ProjectionError> {
            Ok(adapter_source_path::<$ty>(stringify!($ty)))
        }
        fn projection_argument_name(_: &mut ProjectionGraph) -> Result<String, ProjectionError> {
            let source = stringify!($ty).rsplit("::").next().unwrap_or_default().trim();
            let mut letters = source.chars();
            Ok(letters.next().map_or_else(String::new, |first| first.to_uppercase().collect::<String>() + letters.as_str()))
        }
    }
)* }; }
scalar_types!(bool => Bool, String => String, u8 => U8, u16 => U16, u32 => U32, u64 => U64, usize => U64, i8 => I8, i16 => I16, i32 => I32,
    i64 => I64, i128 => I128Bits, f32 => F32Bits, f64 => F64Bits);

macro_rules! inline_validation { ($($ty:ty),* $(,)?) => { $(impl ValidateProjection for $ty {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
})* }; }
inline_validation!(bool, f32);
impl ValidateProjection for i128 {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
    fn projection_length(&self) -> Option<usize> { Some(16) }
}
impl ValidateProjection for f64 {
    const SUPPORTS_FLOAT64_DOMAIN: bool = true;
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn projection_float64(&self) -> Option<f64> {
        Some(*self)
    }
}
macro_rules! signed_validation { ($($ty:ty),* $(,)?) => { $(impl ValidateProjection for $ty {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
    fn projection_signed(&self) -> Option<i64> { Some(i64::from(*self)) }
})* }; }
signed_validation!(i8, i16, i32, i64);
macro_rules! unsigned_validation { ($($ty:ty),* $(,)?) => { $(impl ValidateProjection for $ty {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> { Ok(()) }
    fn projection_unsigned(&self) -> Option<u64> { Some(u64::from(*self)) }
})* }; }
unsigned_validation!(u8, u16, u32, u64);
impl ValidateProjection for usize {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn projection_unsigned(&self) -> Option<u64> {
        u64::try_from(*self).ok()
    }
}
impl ValidateProjection for String {
    const NEEDS_RECURSIVE_VALIDATION: bool = false;
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(self.chars().count())
    }
    fn projection_utf8_bytes(&self) -> Option<usize> {
        Some(self.len())
    }
}
impl<T: ValidateProjection> ValidateProjection for Option<T> {
    const SUPPORTS_FLOAT64_DOMAIN: bool = T::SUPPORTS_FLOAT64_DOMAIN;
    const NEEDS_RECURSIVE_VALIDATION: bool = T::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        match self {
            Some(value) => value.validate_at(path),
            None => Ok(()),
        }
    }
    fn projection_present(&self) -> bool {
        self.is_some()
    }
    fn projection_duration(&self) -> Option<std::time::Duration> {
        self.as_ref()
            .and_then(ValidateProjection::projection_duration)
    }
    fn projection_unsigned(&self) -> Option<u64> {
        self.as_ref()
            .and_then(ValidateProjection::projection_unsigned)
    }
    fn projection_signed(&self) -> Option<i64> {
        self.as_ref()
            .and_then(ValidateProjection::projection_signed)
    }
    fn projection_float64(&self) -> Option<f64> {
        self.as_ref()
            .and_then(ValidateProjection::projection_float64)
    }
    fn projection_float64_domain(&self, domain: Float64Domain) -> Option<bool> {
        Self::SUPPORTS_FLOAT64_DOMAIN.then(|| {
            self.as_ref()
                .is_none_or(|value| value.projection_float64_domain(domain) == Some(true))
        })
    }
    fn projection_length(&self) -> Option<usize> {
        self.as_ref()
            .and_then(ValidateProjection::projection_length)
    }
    fn projection_utf8_bytes(&self) -> Option<usize> {
        self.as_ref()
            .and_then(ValidateProjection::projection_utf8_bytes)
    }
}
impl<T: ValidateProjection> ValidateProjection for Vec<T> {
    const SUPPORTS_FLOAT64_DOMAIN: bool = T::SUPPORTS_FLOAT64_DOMAIN;
    const NEEDS_RECURSIVE_VALIDATION: bool = T::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        if !Self::NEEDS_RECURSIVE_VALIDATION {
            return Ok(());
        }
        for (index, value) in self.iter().enumerate() {
            value.validate_at(&format!("{path}[{index}]"))?;
        }
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(self.len())
    }
    fn projection_float64_domain(&self, domain: Float64Domain) -> Option<bool> {
        Self::SUPPORTS_FLOAT64_DOMAIN.then(|| {
            self.iter()
                .all(|value| value.projection_float64_domain(domain) == Some(true))
        })
    }
}
impl<T: ValidateProjection + ?Sized> ValidateProjection for Box<T> {
    const SUPPORTS_FLOAT64_DOMAIN: bool = T::SUPPORTS_FLOAT64_DOMAIN;
    const NEEDS_RECURSIVE_VALIDATION: bool = T::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        self.as_ref().validate_at(path)
    }
    fn projection_present(&self) -> bool {
        self.as_ref().projection_present()
    }
    fn projection_duration(&self) -> Option<std::time::Duration> {
        self.as_ref().projection_duration()
    }
    fn projection_unsigned(&self) -> Option<u64> {
        self.as_ref().projection_unsigned()
    }
    fn projection_signed(&self) -> Option<i64> {
        self.as_ref().projection_signed()
    }
    fn projection_float64(&self) -> Option<f64> {
        self.as_ref().projection_float64()
    }
    fn projection_float64_domain(&self, domain: Float64Domain) -> Option<bool> {
        self.as_ref().projection_float64_domain(domain)
    }
    fn projection_length(&self) -> Option<usize> {
        self.as_ref().projection_length()
    }
    fn projection_utf8_bytes(&self) -> Option<usize> {
        self.as_ref().projection_utf8_bytes()
    }
}
impl<K: ValidateProjection + Ord, V: ValidateProjection> ValidateProjection for BTreeMap<K, V> {
    const SUPPORTS_FLOAT64_DOMAIN: bool = V::SUPPORTS_FLOAT64_DOMAIN;
    const NEEDS_RECURSIVE_VALIDATION: bool =
        K::NEEDS_RECURSIVE_VALIDATION || V::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        if !Self::NEEDS_RECURSIVE_VALIDATION {
            return Ok(());
        }
        for (index, (key, value)) in self.iter().enumerate() {
            key.validate_at(&format!("{path}[{index}].key"))?;
            value.validate_at(&format!("{path}[{index}].value"))?;
        }
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(self.len())
    }
    fn projection_float64_domain(&self, domain: Float64Domain) -> Option<bool> {
        Self::SUPPORTS_FLOAT64_DOMAIN.then(|| {
            self.values()
                .all(|value| value.projection_float64_domain(domain) == Some(true))
        })
    }
}

impl<T: ProjectionType> ProjectionType for Option<T> {
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(optional_argument_name(&T::projection_argument_name(graph)?))
    }
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        Ok(TypeRef::Optional(Box::new(T::projection(graph)?)))
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!("Option<{}>", T::projection_rust_path(graph)?))
    }
}
impl<T: ProjectionType> ProjectionType for Vec<T> {
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(sequence_argument_name(&T::projection_argument_name(graph)?))
    }
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let element = T::projection(graph)?;
        Ok(sequence_type(element))
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!(
            "std::vec::Vec<{}>",
            T::projection_rust_path(graph)?
        ))
    }
}
/// Existing native Vec projection, including its explicit byte-vector specialization.
/// Source renderers reuse this owner instead of reconstructing collection meaning.
pub fn sequence_type(element: TypeRef) -> TypeRef {
    if element == TypeRef::Scalar(Scalar::U8) { TypeRef::Scalar(Scalar::Bytes) }
    else { TypeRef::Repeated(Box::new(element)) }
}
/// Recover an element where the existing Vec declaration is already known.
pub fn sequence_element(sequence: &TypeRef) -> Option<TypeRef> {
    match sequence { TypeRef::Repeated(element) => Some(element.as_ref().clone()), TypeRef::Scalar(Scalar::Bytes) => Some(TypeRef::Scalar(Scalar::U8)), _ => None }
}

impl<T: ProjectionType> ProjectionType for Box<T> {
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!("Boxed{}", T::projection_argument_name(graph)?))
    }
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        T::projection(graph)
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!(
            "std::boxed::Box<{}>",
            T::projection_rust_path(graph)?
        ))
    }
}
impl<T: ProjectionType> ProjectionType for std::sync::Arc<[T]> {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        Vec::<T>::projection(graph)
    }
}
impl<T: ValidateProjection> ValidateProjection for std::sync::Arc<[T]> {
    const NEEDS_RECURSIVE_VALIDATION: bool = T::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        if !Self::NEEDS_RECURSIVE_VALIDATION {
            return Ok(());
        }
        for (index, value) in self.iter().enumerate() {
            value.validate_at(&format!("{path}[{index}]"))?;
        }
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<K: ProjectionType + Ord, V: ProjectionType> ProjectionType for BTreeMap<K, V> {
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!(
            "Map{}To{}",
            K::projection_argument_name(graph)?,
            V::projection_argument_name(graph)?
        ))
    }
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let key = K::projection(graph)?;
        Ok(TypeRef::Map(Box::new(key), Box::new(V::projection(graph)?)))
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!(
            "std::collections::BTreeMap<{},{}>",
            K::projection_rust_path(graph)?,
            V::projection_rust_path(graph)?
        ))
    }
}
impl<T: ProjectionType + Ord> ProjectionType for BTreeSet<T> {
    fn projection_argument_name(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!("Set{}", T::projection_argument_name(graph)?))
    }
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let element = T::projection(graph)?;
        let supported = match &element {
            TypeRef::Scalar(_) => true,
            TypeRef::Named(name) => graph
                .definitions()
                .get(name)
                .is_some_and(|owner| matches!(owner.shape, Shape::StringEnum(_))),
            _ => false,
        };
        if !supported {
            return Err(ProjectionError::invalid(
                std::any::type_name::<Self>(),
                "unique set validation requires a scalar or source-owned string vocabulary",
            ));
        }
        Ok(TypeRef::Set(Box::new(element)))
    }
    fn projection_rust_path(graph: &mut ProjectionGraph) -> Result<String, ProjectionError> {
        Ok(format!(
            "std::collections::BTreeSet<{}>",
            T::projection_rust_path(graph)?
        ))
    }
}
impl<T: ValidateProjection + Ord> ValidateProjection for BTreeSet<T> {
    const NEEDS_RECURSIVE_VALIDATION: bool = T::NEEDS_RECURSIVE_VALIDATION;
    fn validate_at(&self, path: &str) -> Result<(), ValidationError> {
        if !Self::NEEDS_RECURSIVE_VALIDATION {
            return Ok(());
        }
        for (index, value) in self.iter().enumerate() {
            value.validate_at(&format!("{path}[{index}]"))?;
        }
        Ok(())
    }
    fn projection_length(&self) -> Option<usize> {
        Some(self.len())
    }
}

fn valid_identifier(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn valid_tag(tag: u32) -> bool {
    tag != 0 && tag <= 536_870_911 && !(19_000..=19_999).contains(&tag)
}

fn validate_definition(
    definition: &Definition,
    definitions: &BTreeMap<String, Definition>,
) -> Result<(), ProjectionError> {
    let mut reserved_tags = BTreeSet::new();
    for tag in &definition.reserved_tags {
        if !valid_tag(*tag) || !reserved_tags.insert(*tag) {
            return Err(ProjectionError::invalid(
                &definition.name,
                "invalid or duplicate reserved tag",
            ));
        }
    }
    let mut reserved_names = BTreeSet::new();
    for name in &definition.reserved_names {
        if !valid_identifier(name) || !reserved_names.insert(name.as_str()) {
            return Err(ProjectionError::invalid(
                &definition.name,
                "invalid or duplicate reserved name",
            ));
        }
    }
    let fields = match &definition.shape {
        Shape::Record(fields) => fields
            .iter()
            .map(|f| (f.name.as_str(), f.tag))
            .collect::<Vec<_>>(),
        Shape::Alternatives(alternatives) => {
            if alternatives.is_empty() {
                return Err(ProjectionError::invalid(
                    &definition.name,
                    "an alternative requires at least one member",
                ));
            }
            alternatives
                .iter()
                .map(|a| (a.name.as_str(), a.tag))
                .collect()
        }
        Shape::StringEnum(members) => {
            let unique = members.iter().collect::<BTreeSet<_>>();
            if members.is_empty() || unique.len() != members.len() {
                return Err(ProjectionError::invalid(
                    &definition.name,
                    "string vocabulary is empty or contains duplicate members",
                ));
            }
            vec![("value", 1)]
        }
    };
    validate_tags(&definition.name, fields, &reserved_tags, &reserved_names)?;
    match &definition.shape {
        Shape::Record(fields) => validate_fields(&definition.name, fields, definitions)?,
        Shape::Alternatives(alternatives) => {
            for alternative in alternatives {
                let path = format!("{}.{}", definition.name, alternative.name);
                validate_tags(
                    &path,
                    alternative
                        .fields
                        .iter()
                        .map(|f| (f.name.as_str(), f.tag))
                        .collect(),
                    &BTreeSet::new(),
                    &BTreeSet::new(),
                )?;
                validate_fields(&path, &alternative.fields, definitions)?;
            }
        }
        Shape::StringEnum(_) => {}
    }
    Ok(())
}

fn validate_tags(
    owner: &str,
    fields: Vec<(&str, u32)>,
    reserved_tags: &BTreeSet<u32>,
    reserved_names: &BTreeSet<&str>,
) -> Result<(), ProjectionError> {
    let mut names = BTreeSet::new();
    let mut tags = BTreeSet::new();
    for (name, tag) in fields {
        if !valid_identifier(name)
            || !names.insert(name)
            || reserved_names.contains(name)
            || !valid_tag(tag)
            || !tags.insert(tag)
            || reserved_tags.contains(&tag)
        {
            return Err(ProjectionError::invalid(
                format!("{owner}.{name}"),
                format!("invalid, duplicate or reserved field name/tag {tag}"),
            ));
        }
    }
    Ok(())
}

fn validate_fields(
    owner: &str,
    fields: &[Field],
    definitions: &BTreeMap<String, Definition>,
) -> Result<(), ProjectionError> {
    for field in fields {
        let mut families = BTreeSet::new();
        for rule in &field.rules {
            let family = match rule {
                LocalRule::Required => 0,
                LocalRule::UnsignedRange { .. }
                | LocalRule::SignedRange { .. }
                | LocalRule::PositiveDuration
                | LocalRule::Float64Domain(_) => 1,
                LocalRule::Length { .. } | LocalRule::Utf8Bytes { .. } => 2,
            };
            if !families.insert(family) {
                return Err(ProjectionError::invalid(
                    format!("{owner}.{}", field.name),
                    "a local rule family is declared more than once; native and wire validation must share one envelope",
                ));
            }
            let valid = match rule {
                LocalRule::PositiveDuration => {
                    let ty = match &field.ty {
                        TypeRef::Optional(inner) => inner.as_ref(),
                        ty => ty,
                    };
                    matches!(ty, TypeRef::Named(name) if definitions.get(name)
                        .is_some_and(|owner| matches!(owner.native_container, Some(NativeContainer::Duration { .. }))))
                }
                LocalRule::Float64Domain(domain) => {
                    domain.interval().is_valid()
                        && float_collections::supports_float64_domain(&field.ty, definitions)
                }
                LocalRule::Utf8Bytes { min, max } => {
                    let ty = match &field.ty {
                        TypeRef::Optional(inner) => inner.as_ref(),
                        ty => ty,
                    };
                    min <= max && matches!(ty, TypeRef::Scalar(Scalar::String))
                }
                LocalRule::Required => !matches!(
                    field.ty,
                    TypeRef::Repeated(_) | TypeRef::Set(_) | TypeRef::Map(_, _)
                ),
                LocalRule::UnsignedRange { min, max } => {
                    let scalar = match &field.ty {
                        TypeRef::Optional(inner) => inner.as_ref(),
                        scalar => scalar,
                    };
                    min <= max
                        && match scalar {
                            TypeRef::Scalar(Scalar::U8) => *max <= u64::from(u8::MAX),
                            TypeRef::Scalar(Scalar::U16) => *max <= u64::from(u16::MAX),
                            TypeRef::Scalar(Scalar::U32) => *max <= u64::from(u32::MAX),
                            TypeRef::Scalar(Scalar::U64) => true,
                            _ => false,
                        }
                }
                LocalRule::SignedRange { min, max } => {
                    let scalar = match &field.ty {
                        TypeRef::Optional(inner) => inner.as_ref(),
                        scalar => scalar,
                    };
                    min <= max
                        && match scalar {
                            TypeRef::Scalar(Scalar::I8) => {
                                *min >= i64::from(i8::MIN) && *max <= i64::from(i8::MAX)
                            }
                            TypeRef::Scalar(Scalar::I16) => {
                                *min >= i64::from(i16::MIN) && *max <= i64::from(i16::MAX)
                            }
                            TypeRef::Scalar(Scalar::I32) => {
                                *min >= i64::from(i32::MIN) && *max <= i64::from(i32::MAX)
                            }
                            TypeRef::Scalar(Scalar::I64) => true,
                            _ => false,
                        }
                }
                LocalRule::Length { min, max } => {
                    min <= max
                        && matches!(
                            field.ty,
                            TypeRef::Repeated(_)
                                | TypeRef::Set(_)
                                | TypeRef::Map(_, _)
                                | TypeRef::Scalar(
                                    Scalar::String
                                        | Scalar::Bytes
                                        | Scalar::SemanticId
                                        | Scalar::ContentHash
                                        | Scalar::I128Bits
                                )
                        )
                }
            };
            if !valid {
                return Err(ProjectionError::invalid(
                    format!("{owner}.{}", field.name),
                    "local rule is malformed or unsupported for its actual field type",
                ));
            }
        }
    }
    Ok(())
}

/// Declare an actual record and its stable boundary metadata in one source declaration.
/// `construction: ObservationOnly` emits borrowed selected-field snapshots without
/// an inverse constructor; omitted construction is ordinary reconstructible data.
/// Observation `@local` custody has no initializer or implicit validation. An
/// optional `=> check:` borrows that local field under the owner's actual rule.
#[macro_export]
macro_rules! projection_struct {
    ($(#[$meta:meta])* $vis:vis struct $name:ident <$parameter:ident $(= $default:ty)?> { $($body:tt)* }) => {
        $crate::projection_generic_struct! { $(#[$meta])* $vis struct $name<$parameter $(= $default)?> { $($body)* } }
    };
    ($(#[$meta:meta])* $vis:vis struct $name:ident {
        projection: { name: $wire:expr, $(rust_path: $public:path,)? version: $version:expr, direction: $direction:expr,
            $(construction: $construction:ident,)?
            reserved_tags: [$($reserved:expr),* $(,)?], reserved_names: [$($reserved_name:expr),* $(,)?] };
        $($(#[$field_meta:meta])* $field_vis:vis $field:ident : $ty:ty => {
            tag: $tag:expr, direction: $field_direction:expr, rules: [$($rule:expr),* $(,)?] $(, representation: $representation:expr)? $(, adapter: $adapter:ty)? };)*
        $(@local {
            $($(#[$local_meta:meta])* $local_vis:vis $local:ident: $local_ty:ty $(= $local_default:expr)? $(=> check: $local_check:expr)?;)*
        })?
    }) => {
        $(#[$meta])* $vis struct $name {
            $($(#[$field_meta])* $field_vis $field: $ty,)*
            $($($(#[$local_meta])* $local_vis $local: $local_ty,)*)?
        }
        $(#[allow(unused_qualifications, reason = "the explicit public alias is retained verbatim for generated cross-crate paths")]
        const _: fn($name) -> $public = |value| value;)?
        impl $crate::boundary_projection::ProjectionType for $name {
            fn projection(graph: &mut $crate::boundary_projection::ProjectionGraph) -> ::core::result::Result<$crate::boundary_projection::TypeRef, $crate::boundary_projection::ProjectionError> {
                let root = graph.declare::<Self>($wire, file!(), $version, $direction, &[$($reserved),*], &[$($reserved_name),*], |graph| {
                    let _ = &graph;
                    Ok($crate::boundary_projection::Shape::Record(vec![$(
                        $crate::boundary_projection::Field::of::<$crate::projection_struct!(@ty $ty $(, $adapter)?)>(graph, stringify!($field), $tag, $field_direction, vec![$($rule),*])? $(.representation($representation)?)?,
                    )*]))
                })?;
                $( graph.public_path::<Self>(&root, stringify!($public))?; )?
                $crate::projection_struct!(@role graph, &root, Self $(, $construction)?);
                Ok(root)
            }
        }
        impl $crate::boundary_projection::ValidateProjection for $name {
            fn validate_at(&self, path: &str) -> ::core::result::Result<(), $crate::boundary_projection::ValidationError> {
                let _ = path;
                $( $crate::projection_struct!(@validate &self.$field,
                    &$crate::boundary_projection::field_path(path, stringify!($field)), &[$($rule),*] $(, $adapter)?)?; )*
                $($( $( ($local_check)(&self.$local,
                    &$crate::boundary_projection::field_path(path, stringify!($local)))?; )? )*)?
                Ok(())
            }
        }
        $crate::projection_struct!(@access $($construction)?; $name, $version; {
            $($field: $ty $(, $adapter)? => $tag;)*
        } @local { $($($local $(= $local_default)? $(=> check: $local_check)?;)*)? });
    };
    (@role $graph:ident, $root:expr, $owner:ty) => {};
    (@role $graph:ident, $root:expr, $owner:ty, Data) => {};
    (@role $graph:ident, $root:expr, $owner:ty, ObservationOnly) => {
        $graph.observation_record::<$owner>($root)?;
    };
    (@access ; $($body:tt)*) => { $crate::projection_struct!(@access Data; $($body)*); };
    (@access Data; $name:ident, $version:expr; {
        $($field:ident: $ty:ty $(, $adapter:ty)? => $tag:expr;)*
    } @local { $($local:ident = $local_default:expr => check: $local_check:expr;)* }) => {
        impl $crate::boundary_projection::ProjectionRecord for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),*];
            type Fields = ($($crate::projection_struct!(@ty $ty $(, $adapter)?),)*);
            fn into_projection_fields(self) -> Self::Fields { ($($crate::projection_struct!(@into self.$field $(, $adapter)?),)*) }
            fn from_projection_fields(fields: Self::Fields) -> ::core::result::Result<Self, $crate::boundary_projection::AdapterError> {
                let ($($field,)*) = fields;
                Ok(Self { $($field: $crate::projection_struct!(@from $field $(, $adapter)?),)* $($local: $local_default,)* })
            }
        }
    };
    (@access ObservationOnly; $name:ident, $version:expr; {
        $($field:ident: $ty:ty $(, $adapter:ty)? => $tag:expr;)*
    } @local { $($local:ident $(= $local_default:expr)? $(=> check: $local_check:expr)?;)* }) => {
        impl $crate::boundary_projection::ProjectionObservationRecord for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($tag, stringify!($field))),*];
            type Fields = ($($crate::projection_struct!(@ty $ty $(, $adapter)?),)*);
            fn observation_fields(&self) -> ::core::result::Result<Self::Fields, $crate::boundary_projection::AdapterError> {
                Ok(($($crate::projection_struct!(@into ::core::clone::Clone::clone(&self.$field) $(, $adapter)?),)*))
            }
        }
    };
    (@ty $ty:ty, $adapter:ty) => { $crate::boundary_projection::Adapted<$ty, $adapter> };
    (@ty $ty:ty) => { $ty };
    (@into $value:expr, $adapter:ty) => { $crate::boundary_projection::Adapted::<_, $adapter>::new($value) };
    (@into $value:expr) => { $value };
    (@from $value:expr, $adapter:ty) => { $value.into_inner() };
    (@from $value:expr) => { $value };
    (@validate $value:expr, $path:expr, $rules:expr, $adapter:ty) => {{
        $crate::boundary_projection::validate_adapted::<_, $adapter>($value, $path, $rules)
    }};
    (@validate $value:expr, $path:expr, $rules:expr) => { $crate::boundary_projection::validate_field($value, $path, $rules) };
}

/// Declare an actual enum and its explicitly tagged payload alternatives together.
#[macro_export]
macro_rules! projection_enum {
    (@serde_remote { $(#[$remote_meta:meta])* $remote_vis:vis enum $remote:ident; } $($native:tt)*) => {
        $crate::projection_enum! { $($native)* }
        $crate::projection_enum! { @remote_declaration { $(#[$remote_meta])* $remote_vis enum $remote; } $($native)* }
    };
    (@remote_declaration { $(#[$remote_meta:meta])* $remote_vis:vis enum $remote:ident; }
        $(#[$native_meta:meta])* $native_vis:vis enum $native:ident {
            projection: { $($projection:tt)* };
            $($(#[$variant_meta:meta])* $variant:ident $(( $(#[$payload_meta:meta])* $payload:ty $(=> { rules: [$($payload_rule:expr),* $(,)?] })? ))? => $tag:expr;)*
        }) => {
            $(#[$remote_meta])* $remote_vis enum $remote {
                $($(#[$variant_meta])* $variant $(( $(#[$payload_meta])* $payload ))?,)*
            }
    };
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        projection: { name: $wire:expr, $(rust_path: $public:path,)? version: $version:expr, direction: $direction:expr,
            reserved_tags: [$($reserved:expr),* $(,)?], reserved_names: [$($reserved_name:expr),* $(,)?] };
        $(@unprojected { $($(#[$local_variant_meta:meta])* $local_variant:ident;)* })?
        $($(#[$variant_meta:meta])* $variant:ident $(( $(#[$payload_meta:meta])* $payload:ty $(=> { rules: [$($payload_rule:expr),* $(,)?] })? ))? $({
            $($(#[$field_meta:meta])* $field:ident : $ty:ty => {
                tag: $tag:expr, direction: $field_direction:expr, rules: [$($rule:expr),* $(,)?] $(, representation: $representation:expr)? };)*
            $(@local { $($(#[$local_meta:meta])* $local:ident: $local_ty:ty = $local_default:expr;)* })?
        })? => $variant_tag:expr;)*
    }) => {
        $(#[$meta])* $vis enum $name { $($($(#[$local_variant_meta])* $local_variant,)*)? $($(#[$variant_meta])* $variant $(( $(#[$payload_meta])* $payload ))? $({ $($(#[$field_meta])* $field: $ty,)* $($($(#[$local_meta])* $local: $local_ty,)*)? })?,)* }
        $(#[allow(unused_qualifications, reason = "the explicit public alias is retained verbatim for generated cross-crate paths")]
        const _: fn($name) -> $public = |value| value;)?
        impl $crate::boundary_projection::ProjectionType for $name {
            fn projection(graph: &mut $crate::boundary_projection::ProjectionGraph) -> ::core::result::Result<$crate::boundary_projection::TypeRef, $crate::boundary_projection::ProjectionError> {
                let root = graph.declare::<Self>($wire, file!(), $version, $direction, &[$($reserved),*], &[$($reserved_name),*], |graph| {
                    let _ = &graph;
                    Ok($crate::boundary_projection::Shape::Alternatives(vec![$(
                        $crate::boundary_projection::Alternative { name: stringify!($variant).into(), tag: $variant_tag, fields: vec![
                            $($crate::boundary_projection::Field::of::<$payload>(graph, "value", 1, $crate::boundary_projection::Direction::Both, vec![$($($payload_rule),*)?])?,)?
                            $($($crate::boundary_projection::Field::of::<$ty>(graph, stringify!($field), $tag, $field_direction, vec![$($rule),*])? $(.representation($representation)?)?,)*)?
                        ] },
                    )*]))
                })?;
                $( graph.public_path::<Self>(&root, stringify!($public))?; )?
                Ok(root)
            }
        }
        impl $crate::boundary_projection::ValidateProjection for $name {
            fn validate_at(&self, path: &str) -> ::core::result::Result<(), $crate::boundary_projection::ValidationError> {
                let _ = path;
                $($(if matches!(self, Self::$local_variant) {
                    return Err($crate::boundary_projection::ValidationError::Rule {
                        path: path.into(), rule: "alternative.unprojected",
                        reason: concat!("native state ", stringify!($local_variant), " has no declared boundary representation").into(),
                    });
                })*)?
                $(
                    $( #[allow(irrefutable_let_patterns, reason = "the source owner may declare one alternative; validation still uses its actual payload")]
                    if let Self::$variant(value) = self {
                        let value: &$payload = value;
                        $crate::boundary_projection::validate_field(value,
                            &$crate::boundary_projection::field_path(path, stringify!($variant)), &[$($($payload_rule),*)?])?;
                    } )?
                    $( #[allow(irrefutable_let_patterns, reason = "the source owner may declare one alternative; validation still uses its actual fields")]
                    if let Self::$variant { $($field,)* .. } = self { $(
                        $crate::boundary_projection::validate_field($field,
                            &$crate::boundary_projection::field_path(&$crate::boundary_projection::field_path(path, stringify!($variant)), stringify!($field)), &[$($rule),*])?;
                    )* } )?
                )*
                Ok(())
            }
        }
        $( $crate::projection_enum!(@access $name, $version, $variant, $variant_tag; $( $payload)? $({ $( $field: $ty => $tag, )* } @local { $($($local = $local_default;)*)? })?); )*
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr;) => {
        impl $crate::boundary_projection::ProjectionAlternative<$tag> for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[];
            type Fields = ();
            #[allow(unreachable_patterns, reason = "an owner can intentionally have only one alternative")]
            fn into_projection_alternative(self) -> ::core::result::Result<Self::Fields, Self> {
                match self { Self::$variant => Ok(()), other => Err(other) }
            }
            fn from_projection_alternative((): Self::Fields) -> Self { Self::$variant }
        }
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr; $payload:ty) => {
        impl $crate::boundary_projection::ProjectionAlternative<$tag> for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "value")];
            type Fields = ($payload,);
            #[allow(unreachable_patterns, reason = "an owner can intentionally have only one alternative")]
            fn into_projection_alternative(self) -> ::core::result::Result<Self::Fields, Self> {
                match self { Self::$variant(value) => Ok((value,)), other => Err(other) }
            }
            fn from_projection_alternative((value,): Self::Fields) -> Self { Self::$variant(value) }
        }
    };
    (@access $name:ident, $version:expr, $variant:ident, $tag:expr; { $( $field:ident: $ty:ty => $field_tag:expr, )* } @local { $($local:ident = $local_default:expr;)* }) => {
        impl $crate::boundary_projection::ProjectionAlternative<$tag> for $name {
            const VERSION: u32 = $version;
            const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[$(($field_tag, stringify!($field))),*];
            type Fields = ($($ty,)*);
            #[allow(unreachable_patterns, reason = "an owner can intentionally have only one alternative")]
            fn into_projection_alternative(self) -> ::core::result::Result<Self::Fields, Self> {
                match self { Self::$variant { $($field,)* .. } => Ok(($($field,)*)), other => Err(other) }
            }
            fn from_projection_alternative(($($field,)*): Self::Fields) -> Self { Self::$variant { $($field,)* $($local: $local_default,)* } }
        }
    };
}

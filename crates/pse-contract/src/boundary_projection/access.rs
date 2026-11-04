// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed access at the actual owner, including fields private to its Rust module.

/// A source-local typed binding for a foreign native record. The implementation is
/// owned by its existing serialization declaration, rather than a second DTO.
pub trait ProjectionAdapter<T>: Sized {
    /// Versioned public projection name.
    const NAME: &'static str;
    /// Actual binding declaration source.
    const SOURCE: &'static str;
    /// Binding interpretation version.
    const VERSION: u32;
    /// Declared crossing directions, including any source reservations.
    const DIRECTION: super::Direction;
    /// Retired stable field tags.
    const RESERVED_TAGS: &'static [u32];
    /// Retired public field names.
    const RESERVED_NAMES: &'static [&'static str];
    /// Public field tuple layout, with source-owned stable tags.
    const FIELD_LAYOUT: &'static [(u32, &'static str)];
    /// Typed values extracted from the original native record.
    type Fields;
    /// Wire name; shared ownership bindings retain a distinct record identity.
    fn name() -> String {
        Self::NAME.into()
    }
    /// An explicitly checked public carrier path, when the native library uses private modules.
    fn public_path() -> Option<String> {
        None
    }
    /// Public target alias used when a library's defining module is private.
    fn native_path() -> String {
        core::any::type_name::<T>().into()
    }
    /// Public binding alias used by shared-ownership carriers.
    fn adapter_path() -> String {
        core::any::type_name::<Self>().into()
    }
    /// The same typed field declarations used by access and construction.
    fn fields(
        graph: &mut super::ProjectionGraph,
    ) -> Result<Vec<super::Field>, super::ProjectionError>;
    /// Check locally declared predicates without cloning foreign data.
    fn validate(value: &T, path: &str) -> Result<(), super::ValidationError>;
    /// Extract fields only when this exact native form has a declared encoding.
    /// # Errors
    /// Unsupported native forms refuse before their fields are copied.
    fn try_fields(value: &T) -> Result<Self::Fields, AdapterError>;
    /// Construct the actual foreign target through the source binding.
    /// # Errors
    /// Invariant or unsupported-form violations refuse rather than normalize.
    fn from_fields(fields: Self::Fields) -> Result<T, AdapterError>;
}

/// Typed field carrier for a checked foreign binding. It owns the original value;
/// it is not a serialized document or an erased intermediate value.
pub struct Adapted<T, A> {
    value: T,
    adapter: core::marker::PhantomData<fn() -> A>,
}
impl<T, A> Adapted<T, A> {
    /// Attach the source-declared adapter without interpreting or copying the value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            adapter: core::marker::PhantomData,
        }
    }
    /// Recover the original native value, preserving its nominal type.
    pub fn into_inner(self) -> T {
        self.value
    }
    /// Borrow the original native value.
    pub fn get(&self) -> &T {
        &self.value
    }
}
impl<T: Clone, A> Clone for Adapted<T, A> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}
impl<T: core::fmt::Debug, A> core::fmt::Debug for Adapted<T, A> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.debug_tuple("Adapted").field(&self.value).finish()
    }
}
impl<T, A: ProjectionAdapter<T>> super::ProjectionType for Adapted<T, A> {
    fn projection(
        graph: &mut super::ProjectionGraph,
    ) -> Result<super::TypeRef, super::ProjectionError> {
        let root = graph.declare::<Self>(
            &A::name(),
            A::SOURCE,
            A::VERSION,
            A::DIRECTION,
            A::RESERVED_TAGS,
            A::RESERVED_NAMES,
            |graph| Ok(super::Shape::Record(A::fields(graph)?)),
        )?;
        graph.checked_record::<Self>(&root)?;
        if let Some(path) = A::public_path() {
            graph.public_path::<Self>(&root, &path)?;
        }
        Ok(root)
    }
}

/// Copy an explicitly serializable source field after the binding has validated it.
/// The generic call avoids changing source declarations according to `Copy` details.
pub fn clone_adapter_field<T: Clone>(value: &T) -> T {
    value.clone()
}

/// Resolve a public alias declared beside a source-local foreign binding.
pub fn adapter_source_path<A>(path: &str) -> String {
    let owner = core::any::type_name::<A>()
        .split("::")
        .next()
        .unwrap_or_default();
    let path = path.replace(' ', "");
    path.strip_prefix("crate::")
        .map_or_else(|| path.clone(), |rest| format!("{owner}::{rest}"))
}

/// Borrow the same source binding through native shared ownership. Decoding creates
/// a fresh `Arc` containing admitted settings; it conveys no native custody evidence.
pub struct SharedAdapter<A>(core::marker::PhantomData<fn() -> A>);
impl<A> core::fmt::Debug for SharedAdapter<A> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SharedAdapter")
            .finish_non_exhaustive()
    }
}
impl<T, A: ProjectionAdapter<T>> ProjectionAdapter<std::sync::Arc<T>> for SharedAdapter<A> {
    const NAME: &'static str = A::NAME;
    const SOURCE: &'static str = A::SOURCE;
    const VERSION: u32 = A::VERSION;
    const DIRECTION: super::Direction = A::DIRECTION;
    const RESERVED_TAGS: &'static [u32] = A::RESERVED_TAGS;
    const RESERVED_NAMES: &'static [&'static str] = A::RESERVED_NAMES;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = A::FIELD_LAYOUT;
    type Fields = A::Fields;
    fn name() -> String {
        format!("Shared{}", A::name())
    }
    fn native_path() -> String {
        format!("std::sync::Arc<{}>", A::native_path())
    }
    fn adapter_path() -> String {
        format!(
            "pse_contract::boundary_projection::SharedAdapter<{}>",
            A::adapter_path()
        )
    }
    fn public_path() -> Option<String> {
        Some(format!(
            "pse_contract::boundary_projection::Adapted<{},{}>",
            <Self as ProjectionAdapter<std::sync::Arc<T>>>::native_path(),
            <Self as ProjectionAdapter<std::sync::Arc<T>>>::adapter_path()
        ))
    }
    fn fields(
        graph: &mut super::ProjectionGraph,
    ) -> Result<Vec<super::Field>, super::ProjectionError> {
        A::fields(graph)
    }
    fn validate(value: &std::sync::Arc<T>, path: &str) -> Result<(), super::ValidationError> {
        A::validate(value.as_ref(), path)
    }
    fn try_fields(value: &std::sync::Arc<T>) -> Result<Self::Fields, AdapterError> {
        A::try_fields(value.as_ref())
    }
    fn from_fields(fields: Self::Fields) -> Result<std::sync::Arc<T>, AdapterError> {
        A::from_fields(fields).map(std::sync::Arc::new)
    }
}
impl<T, A: ProjectionAdapter<T>> super::ValidateProjection for Adapted<T, A> {
    fn validate_at(&self, path: &str) -> Result<(), super::ValidationError> {
        A::validate(&self.value, path)
    }
}

/// Checked access for adapters whose foreign target includes nonserializable forms.
/// Existing infallible native record access remains its separate actual-owner contract.
pub trait CheckedProjectionRecord: Sized {
    /// Source binding interpretation.
    const VERSION: u32;
    /// Exact field tuple layout.
    const FIELD_LAYOUT: &'static [(u32, &'static str)];
    /// Source-declared typed fields.
    type Fields;
    /// # Errors
    /// Unsupported native forms must refuse rather than produce a partial record.
    fn try_into_projection_fields(self) -> Result<Self::Fields, AdapterError>;
    /// # Errors
    /// Source construction/admission predicates remain in force.
    fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError>;
}
impl<T, A: ProjectionAdapter<T>> CheckedProjectionRecord for Adapted<T, A> {
    const VERSION: u32 = A::VERSION;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = A::FIELD_LAYOUT;
    type Fields = A::Fields;
    fn try_into_projection_fields(self) -> Result<Self::Fields, AdapterError> {
        A::try_fields(&self.value)
    }
    fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError> {
        A::from_fields(fields).map(Self::new)
    }
}

/// Validate a foreign field via its declared binding without constructing a carrier.
/// # Errors
/// Any source-owned local or bound predicate refuses with its original field path.
pub fn validate_adapted<T, A: ProjectionAdapter<T>>(
    value: &T,
    path: &str,
    rules: &[super::LocalRule],
) -> Result<(), super::ValidationError> {
    struct Borrowed<'a, T, A>(&'a T, core::marker::PhantomData<fn() -> A>);
    impl<T, A: ProjectionAdapter<T>> super::ValidateProjection for Borrowed<'_, T, A> {
        fn validate_at(&self, path: &str) -> Result<(), super::ValidationError> {
            A::validate(self.0, path)
        }
    }
    super::validate_field(
        &Borrowed::<T, A>(value, core::marker::PhantomData),
        path,
        rules,
    )
}

/// A record's actual field tuple. Stable tags/names detect stale adapter layouts.
pub trait ProjectionRecord: Sized {
    /// Interpretation version co-declared by the owner.
    const VERSION: u32;
    /// Tuple layout, with explicit stable tags rather than inferred wire numbers.
    const FIELD_LAYOUT: &'static [(u32, &'static str)];
    /// Actual declared Rust field types.
    type Fields;
    /// Consume the original record without a generic intermediate copy.
    fn into_projection_fields(self) -> Self::Fields;
    /// Construct the actual native record at its defining module.
    /// # Errors
    /// The original owner refuses malformed invariant-bearing field components.
    fn from_projection_fields(fields: Self::Fields) -> Result<Self, AdapterError>;
}

/// Borrow facts from an owner without consuming its custody or exposing an inverse
/// constructor. Passive, independently decodable observations may separately use
/// `ProjectionRecord`; implementing this trait never reconstructs the live owner.
pub trait ProjectionObservationRecord {
    /// Owner-declared interpretation version.
    const VERSION: u32;
    /// Exact snapshot field layout.
    const FIELD_LAYOUT: &'static [(u32, &'static str)];
    /// Passive snapshot fields, independent of the owner's live resources.
    type Fields;
    /// Capture observations while leaving native custody with the original owner.
    /// # Errors
    /// Missing or unavailable observations retain their original typed refusal.
    fn observation_fields(&self) -> Result<Self::Fields, AdapterError>;
}

/// An actual alternative selected by its explicitly declared stable tag.
pub trait ProjectionAlternative<const TAG: u32>: Sized {
    /// Interpretation version co-declared by the owner.
    const VERSION: u32;
    /// Actual payload tuple layout.
    const FIELD_LAYOUT: &'static [(u32, &'static str)];
    /// Actual native payload field types; unit alternatives use `()`.
    type Fields;
    /// Extract this alternative, returning other alternatives unchanged.
    fn into_projection_alternative(self) -> Result<Self::Fields, Self>;
    /// Construct this actual alternative at its defining module.
    fn from_projection_alternative(fields: Self::Fields) -> Self;
}

/// Native adoption failure category, separate from contextual scientific admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterFailure {
    /// A generated adapter no longer matches its actual source owner.
    LayoutMismatch,
    /// A binary field cannot be represented by this native interpretation.
    UnknownField,
    /// An absent or unsupported alternative cannot become a native default.
    UnknownAlternative,
    /// Required exact data is absent or malformed.
    InvalidValue,
    /// A repeated entry would silently overwrite another native key.
    DuplicateKey,
}

/// Original structured failure while crossing a selected typed projection.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AdapterError {
    /// Mechanical native adoption cannot preserve the supplied wire meaning.
    #[error("boundary adoption {kind:?} at `{path}`: {reason}")]
    Refused {
        /// Complete field/alternative path.
        path: String,
        /// Typed mechanical failure category.
        kind: AdapterFailure,
        /// Original refusal detail.
        reason: String,
    },
    /// The actual owner's shared local validation rejected its value.
    #[error(transparent)]
    Validation(#[from] super::ValidationError),
}

impl AdapterError {
    /// Refuse adoption without discarding an unsupported value.
    pub fn refused(
        path: impl Into<String>,
        kind: AdapterFailure,
        reason: impl Into<String>,
    ) -> Self {
        Self::Refused {
            path: path.into(),
            kind,
            reason: reason.into(),
        }
    }
    /// Attach the containing field path to an owner-local construction refusal.
    pub fn at(self, parent: &str) -> Self {
        match self {
            Self::Refused { path, kind, reason } => Self::Refused {
                path: super::field_path(parent, &path),
                kind,
                reason,
            },
            Self::Validation(super::ValidationError::Rule { path, rule, reason }) => {
                Self::Validation(super::ValidationError::Rule {
                    path: super::field_path(parent, &path),
                    rule,
                    reason,
                })
            }
        }
    }
}



impl miette::Diagnostic for AdapterError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            Self::Refused { kind: AdapterFailure::LayoutMismatch, .. } => Some(Box::new("workflow.internal")),
            Self::Refused { .. } => Some(Box::new("workflow.input")),
            Self::Validation(error) => miette::Diagnostic::code(error),
        }
    }
    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        match self {
            Self::Validation(error) => Some(error),
            Self::Refused { .. } => None,
        }
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared diagnostic vocabulary and lossless native engine causes.

mod vocabulary;
pub use vocabulary::{DiagnosticCode, DiagnosticRule, DiagnosticStage, FailureClass, VocabularyError};

/// Source-owned attribution without a dependency on semantic identity crates.
#[derive(Clone, Debug, Default)]
pub struct DiagnosticFacts {
    /// Original semantic identities, in source order.
    pub sources: Vec<[u8; 16]>,
    /// Structured authored locations, when known.
    pub locations: Vec<DiagnosticLocation>,
    /// The checking rule; wrappers may retain a more specific child rule.
    pub rule: Option<DiagnosticRule>,
    /// Observed facts, independent of display text.
    pub observations: std::collections::BTreeMap<String, DiagnosticObservation>,
}
/// A revision-bound authored location. Absence of revision means unattributed context.
#[derive(Clone, Debug)]
pub struct DiagnosticLocation {
    /// Authored occurrence identity.
    pub source: [u8; 16],
    /// Immutable revision, when supplied by the source owner.
    pub revision: Option<[u8; 32]>,
    /// Authored path.
    pub path: String,
    /// Authored display name.
    pub name: Option<String>,
    /// UTF-8 start byte.
    pub start: Option<u32>,
    /// UTF-8 exclusive end byte.
    pub end: Option<u32>,
}
/// Low-coupling observed evidence; physical meaning uses registered identities.
#[derive(Clone, Debug)]
pub enum DiagnosticObservation {
    /// Absent evidence, distinct from a numerical value.
    Missing,
    /// A numerical observation; projection tags nonfinite values explicitly.
    Number(f64),
    /// An exact count.
    Integer(i64),
    /// Checked predicate.
    Boolean(bool),
    /// Native status or explanatory detail.
    Text(String),
    /// Ordered complete quantity/free-index operand contracts.
    Contracts(Vec<OperandContract>),
}
/// A complete registered quantity type plus its free-index obligations.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OperandContract {
    /// Immutable complete quantity-type identity.
    pub quantity: [u8; 16],
    /// Ordered bound-index, domain and entity-kind identities.
    pub indices: Vec<[[u8; 16]; 3]>,
}
impl DiagnosticFacts {
    /// Append an observation without coupling its owner to the public envelope.
    pub fn observe(&mut self, key: impl Into<String>, value: DiagnosticObservation) {
        self.observations.insert(key.into(), value);
    }
    /// Merge a wrapper's supplied facts while retaining the child's rule and evidence.
    pub fn merge(&mut self, other: Self) {
        self.sources.extend(other.sources);
        self.locations.extend(other.locations);
        if other.rule.is_some() { self.rule = other.rule; }
        self.observations.extend(other.observations);
    }
}

/// Semantic identity and aggregate structure, independent of presentation.
pub trait TypedDiagnostic: miette::Diagnostic + 'static {
    /// One declared code, or none for an empty or mixed aggregate.
    fn diagnostic_code(&self) -> Option<DiagnosticCode>;
    /// Source-owned facts; lower layers never need a model/runtime dependency.
    fn diagnostic_facts(&self) -> DiagnosticFacts { DiagnosticFacts::default() }
    /// Ordered children; an empty iterator denotes an empty failed aggregate.
    fn diagnostic_children(&self) -> Option<Box<dyn Iterator<Item = &dyn TypedDiagnostic> + '_>> {
        None
    }
    /// The class of the same semantic identity.
    fn failure_class(&self) -> Option<FailureClass> {
        self.diagnostic_code().map(DiagnosticCode::class)
    }
}

/// Borrow leaves without rendering, discarding wrappers or deduplicating occurrences.
pub fn diagnostic_leaves(error: &dyn TypedDiagnostic) -> Vec<&dyn TypedDiagnostic> {
    let mut pending = vec![error];
    let mut leaves = Vec::new();
    while let Some(error) = pending.pop() {
        if let Some(children) = error.diagnostic_children() {
            let children = children.collect::<Vec<_>>();
            pending.extend(children.into_iter().rev());
        } else {
            leaves.push(error);
        }
    }
    leaves
}

/// Implement miette's projection from typed codes, retaining domain help and sources.
/// Domain variants keep their own fields; only the projection mechanism is shared.
#[macro_export]
macro_rules! impl_diagnostic {
    ($ty:ty, code($c:ident) $code:block, forward($f:ident) $forward:block,
     help($h:ident) $help:block, related($r:ident) $related:block,
     source($s:ident) $source:block) => {
        $crate::impl_diagnostic! { $ty, code($c) $code, forward($f) $forward,
            help($h) $help, related($r) $related, source($s) $source,
            facts(_this) { $crate::DiagnosticFacts::default() }
        }
    };
    ($ty:ty, code($c:ident) $code:block, forward($f:ident) $forward:block,
     help($h:ident) $help:block, related($r:ident) $related:block,
     source($s:ident) $source:block, facts($d:ident) $facts:block) => {
        impl $ty {
            #[allow(clippy::unnecessary_wraps, reason = "a wrapper may delegate every variant while the shared projection also supports partial delegation")]
            fn diagnostic_forward(&self) -> Option<&dyn $crate::TypedDiagnostic> {
                let $f = self;
                $forward
            }
            #[allow(unreachable_patterns, clippy::unnecessary_wraps, reason = "projection macro supports both exhaustive and partially delegated variant maps")]
            fn diagnostic_own_code(&self) -> Option<$crate::DiagnosticCode> {
                let $c = self;
                $code
            }
        }
        const _: fn() = || {
            fn require_traits<T: miette::Diagnostic + std::error::Error>() {}
            require_traits::<$ty>();
        };
        #[allow(
            unreachable_patterns,
            reason = "projection macro accepts exhaustive and partially delegated variant maps"
        )]
        impl $crate::TypedDiagnostic for $ty {
            fn diagnostic_facts(&self) -> $crate::DiagnosticFacts {
                let $d = self;
                let own: $crate::DiagnosticFacts = $facts;
                let mut facts = self.diagnostic_forward().map_or_else(
                    $crate::DiagnosticFacts::default, $crate::TypedDiagnostic::diagnostic_facts);
                facts.merge(own);
                facts
            }
            fn diagnostic_code(&self) -> Option<$crate::DiagnosticCode> {
                self.diagnostic_own_code().or_else(|| {
                    self.diagnostic_forward()
                        .and_then($crate::TypedDiagnostic::diagnostic_code)
                })
            }
            fn diagnostic_children(
                &self,
            ) -> Option<Box<dyn Iterator<Item = &dyn $crate::TypedDiagnostic> + '_>> {
                let $r = self;
                let own: Option<Box<dyn Iterator<Item = &dyn $crate::TypedDiagnostic> + '_>> =
                    $related;
                own.or_else(|| {
                    if self.diagnostic_own_code().is_none() {
                        self.diagnostic_forward().map(|child| {
                            { let children: Box<dyn Iterator<Item = &dyn $crate::TypedDiagnostic>> = Box::new(std::iter::once(child)); children }
                        })
                    } else {
                        None
                    }
                })
            }
        }
        #[allow(unreachable_patterns, clippy::unnecessary_wraps, reason = "projection macro supports both exhaustive and partially delegated variant maps")]
        impl miette::Diagnostic for $ty {
            fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                $crate::TypedDiagnostic::diagnostic_code(self)
                    .map(|code| -> Box<dyn std::fmt::Display> { Box::new(code) })
            }
            fn severity(&self) -> Option<miette::Severity> {
                self.diagnostic_forward()
                    .and_then(miette::Diagnostic::severity)
            }
            fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                let $h = self;
                let own: Option<Box<dyn std::fmt::Display + 'a>> = $help;
                own.or_else(|| self.diagnostic_forward().and_then(miette::Diagnostic::help))
            }
            fn url<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
                self.diagnostic_forward().and_then(miette::Diagnostic::url)
            }
            fn source_code(&self) -> Option<&dyn miette::SourceCode> {
                self.diagnostic_forward()
                    .and_then(miette::Diagnostic::source_code)
            }
            fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
                self.diagnostic_forward()
                    .and_then(miette::Diagnostic::labels)
            }
            fn related<'a>(
                &'a self,
            ) -> Option<Box<dyn Iterator<Item = &'a dyn miette::Diagnostic> + 'a>> {
                let $r = self;
                let own: Option<Box<dyn Iterator<Item = &'a dyn $crate::TypedDiagnostic> + 'a>> =
                    $related;
                own.map(|children| {
                    { let related: Box<dyn Iterator<Item = &dyn miette::Diagnostic>> = Box::new(children.map(|child| -> &dyn miette::Diagnostic { child })); related }
                })
                .or_else(|| {
                    self.diagnostic_forward()
                        .and_then(miette::Diagnostic::related)
                })
            }
            fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
                let $s = self;
                let own: Option<&dyn miette::Diagnostic> = $source;
                own.or_else(|| {
                    self.diagnostic_forward()
                        .map(|child| -> &dyn miette::Diagnostic { child })
                })
            }
        }
    };
}

/// A synchronous core checkpoint supplied by the operation owner.
pub trait CancellationCheck {
    /// Refuse cancelled work without coupling semantic code to an async runtime.
    /// # Errors
    /// The operation has been cancelled.
    fn checkpoint(&self) -> Result<(), WorkError>;
}
/// Transient work failure; never a memoized semantic diagnostic.
#[derive(Debug, thiserror::Error)]
pub enum WorkError {
    /// Cancellation requested by the operation owner.
    #[error("cancelled")]
    Cancelled,
    /// Native resource refusal preserving the original cause.
    #[error("resource admission failed: {0}")]
    Resource(#[source] Box<dyn std::error::Error + Send + Sync>),
}
crate::impl_diagnostic! {
    WorkError,
    code(this) { Some(match this { Self::Cancelled => DiagnosticCode::RuntimeCancelled, Self::Resource(_) => DiagnosticCode::RuntimeResourceLimit }) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

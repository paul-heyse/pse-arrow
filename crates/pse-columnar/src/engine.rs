// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Borrow observations, retain causes; classification never rebuilds an error.

use crate::{DiagnosticCode, TypedDiagnostic};
use datafusion_common::{DataFusionError, diagnostic::Diagnostic as NativeDiagnostic};
use miette::Diagnostic;
use std::{error::Error, fmt, sync::Arc};

/// Origin determines responsibility for engine planning and evaluation failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PlanOrigin {
    /// A platform-generated rule plan.
    RuleCompiler,
    /// An authored query.
    Analytics,
    /// Preparation of a native numerical capability.
    NumericalPreparation,
    /// A generated kernel adapter.
    KernelUdf,
}
impl PlanOrigin {
    /// Stable registry spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RuleCompiler => "rule_compiler",
            Self::Analytics => "analytics",
            Self::KernelUdf => "kernel_udf",
            Self::NumericalPreparation => "numerical_preparation",
        }
    }
}
impl fmt::Display for PlanOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Typed platform diagnostic crossing the native External boundary.
#[derive(Debug)]
pub struct Attachment {
    code: Option<DiagnosticCode>,
    cause: Box<dyn TypedDiagnostic + Send + Sync>,
}
impl fmt::Display for Attachment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.cause.as_ref(), f)
    }
}
impl Error for Attachment {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Upcast the concrete diagnostic, not the Box<dyn Diagnostic> wrapper.
        // The latter implements Error too, but hides the original downcast target.
        Some(self.cause.as_ref())
    }
}
impl Attachment {
    /// Attach a declared code without changing the concrete original cause.
    pub fn new(code: DiagnosticCode, cause: impl TypedDiagnostic + Send + Sync + 'static) -> Self {
        Self {
            code: Some(code),
            cause: Box::new(cause),
        }
    }
    /// Original typed diagnostic (supports Error downcasts through trait upcasting).
    pub fn cause(&self) -> &(dyn Diagnostic + Send + Sync + 'static) {
        self.cause.as_ref()
    }
    /// Declared detailed code.
    pub fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        self.code.or_else(|| self.cause.diagnostic_code())
    }
}
impl Diagnostic for Attachment {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.diagnostic_code()
            .map(|code| -> Box<dyn fmt::Display> { Box::new(code) })
    }
    fn severity(&self) -> Option<miette::Severity> {
        self.cause.severity()
    }
    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.cause.help()
    }
    fn url<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.cause.url()
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.cause.source_code()
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.cause.labels()
    }
    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.cause.related()
    }
    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        Some(self.cause.as_ref())
    }
}

/// Preserve a typed platform diagnostic and every related cause at the engine boundary.
pub fn external(cause: impl TypedDiagnostic + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(Attachment {
        code: None,
        cause: Box::new(cause),
    }))
}

/// A borrowed leaf and its ordered wrappers. Repeated shared leaves remain repeated.
#[derive(Debug)]
pub struct Observation<'a> {
    /// Detailed platform code.
    pub code: DiagnosticCode,
    /// Original native leaf, never a rendered replacement.
    pub cause: &'a DataFusionError,
    /// Original domain leaf when the native error wraps a platform diagnostic.
    pub domain_cause: Option<&'a dyn TypedDiagnostic>,
    /// Outer-to-inner native context.
    pub contexts: Vec<&'a str>,
    /// Original native diagnostics, including labels and suggestions.
    pub diagnostics: Vec<&'a NativeDiagnostic>,
    route: Vec<RouteStep>,
    domain_route: Option<Vec<usize>>,
}

#[derive(Clone, Debug)]
enum RouteStep {
    Child(usize),
    NativeSource,
}

/// Find the next native error across Arrow, I/O or user-defined source wrappers.
/// Typed domain attachments are leaves: their declared semantic code wins.
fn native_source(error: &DataFusionError) -> Option<&DataFusionError> {
    if matches!(error, DataFusionError::External(source) if source.is::<Attachment>()) {
        return None;
    }
    let mut source = error.source();
    while let Some(cause) = source {
        if let Some(native) = cause.downcast_ref::<DataFusionError>() {
            return Some(native);
        }
        if let Some(native) = cause.downcast_ref::<Arc<DataFusionError>>() {
            return Some(native.as_ref());
        }
        source = cause.source();
    }
    None
}

/// Walk Context, Diagnostic, Shared and Collection at every depth.
/// An empty collection has no observations but remains an error when attached.
pub fn observe(error: &DataFusionError, origin: PlanOrigin) -> Vec<Observation<'_>> {
    let mut result = Vec::new();
    let mut pending = vec![(error, Vec::new(), Vec::new(), Vec::new())];
    while let Some((node, mut route, mut contexts, mut diagnostics)) = pending.pop() {
        match node {
            DataFusionError::Collection(children) => {
                for (index, child) in children.iter().enumerate().rev() {
                    let mut child_route = route.clone();
                    child_route.push(RouteStep::Child(index));
                    pending.push((child, child_route, contexts.clone(), diagnostics.clone()));
                }
            }
            DataFusionError::Context(context, inner) => {
                route.push(RouteStep::Child(0));
                contexts.push(context.as_str());
                pending.push((inner, route, contexts, diagnostics));
            }
            DataFusionError::Diagnostic(diagnostic, inner) => {
                route.push(RouteStep::Child(0));
                diagnostics.push(diagnostic.as_ref());
                pending.push((inner, route, contexts, diagnostics));
            }
            DataFusionError::Shared(inner) => {
                route.push(RouteStep::Child(0));
                pending.push((inner, route, contexts, diagnostics));
            }
            DataFusionError::External(source) if source.is::<Attachment>() => {
                if let Some(attachment) = source.downcast_ref::<Attachment>() {
                    let mut domains: Vec<(&dyn TypedDiagnostic, Vec<usize>)> =
                        vec![(attachment, vec![])];
                    while let Some((domain, domain_route)) = domains.pop() {
                        if let Some(children) = domain.diagnostic_children() {
                            let children = children.enumerate().collect::<Vec<_>>();
                            for (index, child) in children.into_iter().rev() {
                                let mut next = domain_route.clone();
                                next.push(index);
                                domains.push((child, next));
                            }
                        } else {
                            result.push(Observation {
                                code: domain
                                    .diagnostic_code()
                                    .unwrap_or(DiagnosticCode::InternalInvariant),
                                cause: node,
                                domain_cause: Some(domain),
                                contexts: contexts.clone(),
                                diagnostics: diagnostics.clone(),
                                route: route.clone(),
                                domain_route: Some(domain_route),
                            });
                        }
                    }
                }
            }
            leaf => {
                if let Some(inner) = native_source(leaf) {
                    route.push(RouteStep::NativeSource);
                    pending.push((inner, route, contexts, diagnostics));
                } else {
                    result.push(Observation {
                        code: leaf_code(leaf, origin),
                        cause: leaf,
                        domain_cause: None,
                        contexts,
                        diagnostics,
                        route,
                        domain_route: None,
                    });
                }
            }
        }
    }
    result
}

fn leaf_code(error: &DataFusionError, origin: PlanOrigin) -> DiagnosticCode {
    match error {
        DataFusionError::External(source) => {
            source
                .downcast_ref::<Attachment>()
                .map_or(DiagnosticCode::InternalInvariant, |value| {
                    value
                        .diagnostic_code()
                        .unwrap_or(DiagnosticCode::InternalInvariant)
                })
        }
        DataFusionError::ResourcesExhausted(_) => DiagnosticCode::RuntimeResourceLimit,
        DataFusionError::Configuration(_) => DiagnosticCode::ConfigInvalid,
        DataFusionError::Plan(_) | DataFusionError::SchemaError(..)
            if origin == PlanOrigin::Analytics =>
        {
            DiagnosticCode::UserModel
        }
        DataFusionError::Plan(_)
        | DataFusionError::SchemaError(..)
        | DataFusionError::NotImplemented(_)
            if origin == PlanOrigin::NumericalPreparation =>
        {
            DiagnosticCode::CapabilityBackend
        }
        DataFusionError::Execution(_) if origin == PlanOrigin::KernelUdf => {
            DiagnosticCode::SolveEvaluationError
        }
        DataFusionError::Execution(_)
        | DataFusionError::ArrowError(..)
        | DataFusionError::IoError(_) => DiagnosticCode::RuntimeInfrastructure,
        DataFusionError::ParquetError(_) | DataFusionError::ObjectStore(_) => {
            DiagnosticCode::RuntimeInfrastructure
        }
        _ => DiagnosticCode::InternalInvariant,
    }
}

/// One classified leaf retaining ownership of the complete native tree.
#[derive(Clone, Debug)]
pub struct Failure {
    source: Arc<DataFusionError>,
    route: Vec<RouteStep>,
    domain_route: Option<Vec<usize>>,
    code: DiagnosticCode,
}
impl Failure {
    /// Detailed code chosen without consuming or copying the cause.
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        self.code
    }
    /// The exact original leaf.
    pub fn cause(&self) -> &DataFusionError {
        let mut current = self.source.as_ref();
        for step in &self.route {
            if let RouteStep::NativeSource = step {
                current = native_source(current).unwrap_or(current);
                continue;
            }
            current = match (current, step) {
                (DataFusionError::Collection(children), RouteStep::Child(index)) => {
                    &children[*index]
                }
                (DataFusionError::Context(_, inner) | DataFusionError::Diagnostic(_, inner), _) => {
                    inner
                }
                (DataFusionError::Shared(inner), _) => inner,
                _ => current,
            };
        }
        current
    }
    /// Concrete domain leaf when the native leaf wraps a typed platform aggregate.
    pub fn typed_cause(&self) -> Option<&(dyn TypedDiagnostic + 'static)> {
        let DataFusionError::External(source) = self.cause() else {
            return None;
        };
        let mut current: &(dyn TypedDiagnostic + 'static) = source.downcast_ref::<Attachment>()?;
        for &index in self.domain_route.as_deref()? {
            current = current.diagnostic_children()?.nth(index)?;
        }
        Some(current)
    }
    /// Exact concrete platform leaf, or the native leaf for an engine error.
    pub fn leaf_cause(&self) -> &(dyn Error + 'static) {
        let cause: &(dyn Error + 'static) = self.typed_cause().map_or(self.cause(), |cause| cause);
        cause.downcast_ref::<Self>().map_or(cause, Self::leaf_cause)
    }
    fn attachment(&self) -> Option<&dyn Diagnostic> {
        self.typed_cause().map(|cause| -> &dyn Diagnostic { cause })
    }
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.leaf_cause(), f)
    }
}
impl Error for Failure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.leaf_cause())
    }
}
impl Diagnostic for Failure {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.code))
    }
    fn severity(&self) -> Option<miette::Severity> {
        self.attachment().and_then(Diagnostic::severity)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.attachment().and_then(Diagnostic::help)
    }
    fn url<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.attachment().and_then(Diagnostic::url)
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.attachment().and_then(Diagnostic::source_code)
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.attachment().and_then(Diagnostic::labels)
    }
    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.attachment().and_then(Diagnostic::related)
    }
    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.attachment().map(|value| -> &dyn Diagnostic { value })
    }
}

impl TypedDiagnostic for Failure {
    fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        Some(self.code)
    }
}
impl TypedDiagnostic for Attachment {
    fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        self.diagnostic_code()
    }
    fn diagnostic_children(&self) -> Option<Box<dyn Iterator<Item = &dyn TypedDiagnostic> + '_>> {
        self.code
            .is_none()
            .then(|| -> Box<dyn Iterator<Item = &dyn TypedDiagnostic>> {
                {
                    let cause: &dyn TypedDiagnostic = self.cause.as_ref();
                    Box::new(std::iter::once(cause))
                }
            })
    }
}

/// Engine-facing error independent of catalog, schema, relations and IDs.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{source}")]
pub struct EngineError {
    #[source]
    source: Arc<DataFusionError>,
    origin: PlanOrigin,
    failures: Vec<Failure>,
    resource_keys: Vec<&'static str>,
}
impl EngineError {
    /// Retain an already shared native error without changing its ownership.
    pub fn from_shared(source: Arc<DataFusionError>, origin: PlanOrigin) -> Self {
        let failures = observe(&source, origin)
            .into_iter()
            .map(|observation| Failure {
                source: Arc::clone(&source),
                route: observation.route,
                domain_route: observation.domain_route,
                code: observation.code,
            })
            .collect();
        Self {
            source,
            origin,
            failures,
            resource_keys: Vec::new(),
        }
    }
    /// Original native tree, including empty collections and all wrapper diagnostics.
    pub fn native(&self) -> &Arc<DataFusionError> {
        &self.source
    }
    /// Borrow the original tree's leaves and wrappers.
    pub fn observations(&self) -> Vec<Observation<'_>> {
        observe(&self.source, self.origin)
    }
    /// Classified leaves in original order and multiplicity.
    pub fn failures(&self) -> &[Failure] {
        &self.failures
    }
    /// Annotate a known bound policy; this cannot change classification.
    #[must_use]
    pub fn with_resource_key(mut self, key: &'static str) -> Self {
        if !self.resource_keys.contains(&key) {
            self.resource_keys.push(key);
        }
        self
    }
    /// Bound resource policy hints; native message text remains in the original cause.
    pub fn resource_keys(&self) -> &[&'static str] {
        &self.resource_keys
    }
}
impl Diagnostic for EngineError {
    fn severity(&self) -> Option<miette::Severity> {
        match self.failures.as_slice() {
            [one] => one.severity(),
            _ => None,
        }
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self.failures.as_slice() {
            [one] => one.source_code(),
            _ => None,
        }
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match self.failures.as_slice() {
            [one] => one.labels(),
            _ => None,
        }
    }
    fn url<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        match self.failures.as_slice() {
            [one] => one.url(),
            _ => None,
        }
    }
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        TypedDiagnostic::diagnostic_code(self)
            .map(|code| -> Box<dyn fmt::Display> { Box::new(code) })
    }
    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        Some(Box::new(
            self.failures
                .iter()
                .map(|failure| -> &dyn Diagnostic { failure }),
        ))
    }
    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self.failures.as_slice() {
            [one] => Some(one),
            _ => None,
        }
    }
    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        if self
            .failures
            .iter()
            .any(|value| value.code == DiagnosticCode::RuntimeResourceLimit)
            && !self.resource_keys.is_empty()
        {
            return Some(Box::new(format!(
                "adjust one of: {}",
                self.resource_keys.join(", ")
            )));
        }
        match self.failures.as_slice() {
            [one] => one.help(),
            _ => None,
        }
    }
}

impl TypedDiagnostic for EngineError {
    fn diagnostic_code(&self) -> Option<DiagnosticCode> {
        let code = self.failures.first()?.code;
        self.failures
            .iter()
            .all(|failure| failure.code == code)
            .then_some(code)
    }
    fn diagnostic_children(&self) -> Option<Box<dyn Iterator<Item = &dyn TypedDiagnostic> + '_>> {
        Some(Box::new(
            self.failures
                .iter()
                .map(|failure| -> &dyn TypedDiagnostic { failure }),
        ))
    }
}

/// Attach a native failure. Empty collections remain failed results with zero leaves.
pub fn classify(error: DataFusionError, origin: PlanOrigin) -> EngineError {
    EngineError::from_shared(Arc::new(error), origin)
}

impl From<EngineError> for DataFusionError {
    fn from(cause: EngineError) -> Self {
        external(cause)
    }
}

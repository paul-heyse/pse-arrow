// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Preserve original operation failures through adapter and native drain boundaries.

type Error = Box<dyn std::error::Error + Send + Sync>;
pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

/// Original operation and terminal native-drain failures remain independently inspectable.
#[derive(Debug, thiserror::Error)]
pub enum ProjectOperationError {
    /// The original owner refused or failed the requested action.
    #[error("{source}")]
    Action {
        /// Original typed owner failure; available for source inspection and downcasting.
        #[source]
        source: Error,
    },
    /// The requested action completed, but its original native lanes failed to drain.
    #[error("terminal native drain failed: {source}")]
    NativeDrain {
        /// Original native lane failure.
        #[source]
        source: Box<crate::math::MathRuntimeError>,
    },
    /// Neither the original action failure nor the drain failure replaces the other.
    #[error("{action}; terminal native drain also failed: {drain}")]
    ActionAndDrain {
        /// Original action failure.
        #[source]
        action: Error,
        /// Original native drain failure.
        drain: Box<crate::math::MathRuntimeError>,
    },
}

fn original<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a dyn pse_diagnostics::TypedDiagnostic> {
    if let Some(error) = error.downcast_ref::<ProjectOperationError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<crate::workflow::WorkflowError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<std::sync::Arc<crate::workflow::WorkflowError>>() {
        return Some(error.as_ref());
    }
    if let Some(error) = error.downcast_ref::<crate::RuntimeError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<pse_operations::canonical::CanonicalError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<pse_admission::authoring_driver::DriverError>() {
        return Some(error);
    }
    if let Some(error) = error.downcast_ref::<pse_schema::error::SchemaError>() {
        return Some(error);
    }
    None
}

pse_diagnostics::impl_diagnostic! {
    ProjectOperationError,
    code(this) { match this { Self::Action { source } | Self::ActionAndDrain { action: source, .. } if original(source.as_ref()).is_none() => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure), _ => None } },
    forward(this) { match this { Self::Action { source } | Self::ActionAndDrain { action: source, .. } => original(source.as_ref()), Self::NativeDrain { source } => Some(source.as_ref()) } },
    help(_this) { None },
    related(this) { match this { Self::ActionAndDrain { action, drain } => { let mut errors: Vec<&dyn pse_diagnostics::TypedDiagnostic> = vec![drain.as_ref()]; if let Some(action) = original(action.as_ref()) { errors.push(action); } Some(Box::new(errors.into_iter())) }, _ => None } },
    source(_this) { None }
}

impl ProjectOperationError {
    pub(super) fn action(source: Error) -> Self {
        Self::Action { source }
    }
}

pub(super) fn settle_action<T>(
    action: Result<T>,
    drain: std::result::Result<(), crate::math::MathRuntimeError>,
) -> std::result::Result<T, ProjectOperationError> {
    match (action, drain) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(source), Ok(())) => Err(ProjectOperationError::Action { source }),
        (Ok(_), Err(source)) => Err(ProjectOperationError::NativeDrain {
            source: Box::new(source),
        }),
        (Err(action), Err(drain)) => Err(ProjectOperationError::ActionAndDrain {
            action,
            drain: Box::new(drain),
        }),
    }
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("{context}: {source}")]
#[diagnostic(code(runtime::infrastructure))]
struct ContextFailure {
    context: String,
    #[source]
    source: Error,
}

pub(super) trait Context<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
}
impl<T, E: Into<Error>> Context<T> for std::result::Result<T, E> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|source| -> Error {
            Box::new(ContextFailure {
                context: context.into(),
                source: source.into(),
            })
        })
    }
}
impl<T> Context<T> for Option<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| context.into().into())
    }
}
macro_rules! ensure {
    ($condition:expr, $($message:tt)*) => {
        if !$condition { return Err(format!($($message)*).into()); }
    };
}
pub(super) use ensure;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_project_action_and_drain_keep_both_original_failures() {
        let outcome = settle_action::<()>(
            Err(Box::new(std::io::Error::other("original action"))),
            Err(crate::math::MathRuntimeError::Panic("original lane".into())),
        );
        assert!(matches!(outcome,
            Err(ProjectOperationError::ActionAndDrain { action, drain })
                if action.downcast_ref::<std::io::Error>().is_some()
                    && matches!(drain.as_ref(), crate::math::MathRuntimeError::Panic(reason) if reason == "original lane")
        ));
    }

    #[test]
    fn completed_project_action_keeps_original_failed_native_drain_source() {
        let error = settle_action(
            Ok(()),
            Err(crate::math::MathRuntimeError::Panic(
                "original drain".into(),
            )),
        )
        .unwrap_err();
        let source = std::error::Error::source(&error).unwrap();
        assert!(matches!(
            source.downcast_ref::<crate::math::MathRuntimeError>(),
            Some(crate::math::MathRuntimeError::Panic(reason)) if reason == "original drain"
        ));
        assert_eq!(
            pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
            pse_diagnostics::TypedDiagnostic::diagnostic_code(
                source
                    .downcast_ref::<crate::math::MathRuntimeError>()
                    .unwrap()
            )
        );
    }
}

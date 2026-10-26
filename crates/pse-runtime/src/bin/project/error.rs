// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Standard-library error boundary for the standalone product operator.

pub(super) type Error = Box<dyn std::error::Error + Send + Sync>;
pub(super) type Result<T, E = Error> = std::result::Result<T, E>;

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

/// Preserve the original action and terminal drain failures independently.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("{action}; terminal connection drain also failed: {drain}")]
#[diagnostic(code(runtime::infrastructure))]
struct ActionDrainFailure {
    #[source]
    action: Error,
    drain: Error,
}

pub(super) fn settle_action<T>(action: Result<T>, drain: Result<()>) -> Result<T> {
    match (action, drain) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(action), Err(drain)) => Err(Box::new(ActionDrainFailure { action, drain })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_and_terminal_drain_failures_retain_both_original_causes() -> Result<()> {
        let outcome = settle_action::<()>(
            Err("original output allowance exhausted".into()),
            Err("original protection release refused".into()),
        );
        let error = outcome.err().ok_or("both failures must remain failures")?;
        let causes = error
            .downcast_ref::<ActionDrainFailure>()
            .ok_or("both typed causes must be retained")?;
        assert_eq!(
            causes.action.to_string(),
            "original output allowance exhausted"
        );
        assert_eq!(
            causes.drain.to_string(),
            "original protection release refused"
        );
        Ok(())
    }
}

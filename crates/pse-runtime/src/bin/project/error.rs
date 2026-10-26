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

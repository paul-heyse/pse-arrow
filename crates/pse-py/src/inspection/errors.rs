// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use miette::Diagnostic;
use pyo3::{exceptions::PyException, prelude::*};

// This declaration drives both the exception's runtime attributes and the narrow
// stub supplement for PyO3's native-exception introspection gap under abi3-py311.
macro_rules! inspection_exception {
    ($name:ident, $base:ty, { $($field:ident: $ty:ty),* $(,)? }) => {
        pyo3::create_exception!(_native, $name, $base);
        struct Details { $( $field: $ty, )* }
        impl Details {
            fn attach(self, value: &Bound<'_, pyo3::types::PyAny>) -> PyResult<()> {
                $( value.setattr(stringify!($field), self.$field)?; )*
                Ok(())
            }
        }
    };
}
inspection_exception!(InspectionError, PyException, {
    code: Option<String>,
    related: Vec<(Option<String>, String)>,
});

pub(super) fn diagnostic(py: Python<'_>, error: &dyn Diagnostic) -> PyErr {
    let result = InspectionError::new_err(message(error));
    let value = result.value(py);
    let code = error.code().map(|code| code.to_string());
    let related = error
        .related()
        .into_iter()
        .flatten()
        .map(|related| {
            (
                related.code().map(|code| code.to_string()),
                message(related),
            )
        })
        .collect::<Vec<_>>();
    if let Err(error) = (Details { code, related }).attach(value.as_any()) {
        return error;
    }
    result
}

pub(super) fn message(error: &dyn Diagnostic) -> String {
    let mut message = error
        .code()
        .map_or_else(|| error.to_string(), |code| format!("[{code}] {error}"));
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }
    message
}

pub(super) fn invalid(reason: &str) -> pse_catalog::CatalogError {
    pse_catalog::CatalogError::ConfigInvalid {
        key: "pse.inspection".to_owned(),
        reason: reason.to_owned(),
    }
}

pub(super) fn closed() -> pse_catalog::CatalogError {
    pse_catalog::CatalogError::Admission {
        path: "inspection handle".to_owned(),
        reason: "handle is closed".to_owned(),
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Mechanical projection of the native cache observation declaration.
use pyo3::prelude::*;
macro_rules! report {
    ($( $(#[$meta:meta])* $field:ident: $ty:ty, )*) => {
        /// Native cache accounting; unavailable observations stay None.
        #[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
        #[derive(Clone, Debug)]
        pub(crate) struct CacheReport { $( $(#[$meta])* $field: $ty, )* }
        impl From<pse_engine::cache_service::CacheReport> for CacheReport {
            fn from(value: pse_engine::cache_service::CacheReport) -> Self {
                Self { $( $field: value.$field, )* }
            }
        }
    };
}
pse_engine::cache_report_fields!(report);

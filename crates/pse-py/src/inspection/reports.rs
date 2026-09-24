// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Immutable named observations with native field names and units.
use super::tuple::Tuple;
use pyo3::prelude::*;

/// A literal native table reference; components never undergo SQL normalization.
#[pyclass(frozen, skip_from_py_object, get_all, eq, hash, module = "pse._native")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TableName {
    catalog: String,
    schema: String,
    table: String,
}
impl From<(String, String, String)> for TableName {
    fn from((catalog, schema, table): (String, String, String)) -> Self {
        Self {
            catalog,
            schema,
            table,
        }
    }
}

/// A current native memory consumer and its accounted reservation.
#[pyclass(frozen, skip_from_py_object, get_all, module = "pse._native")]
#[derive(Clone, Debug)]
pub(crate) struct ResourceConsumer {
    name: String,
    reserved_bytes: usize,
}

/// Native pool counters and independent process observations.
#[pyclass(frozen, module = "pse._native")]
#[derive(Debug)]
pub(crate) struct ResourceReport(pse_runtime::ResourceReport);
impl From<pse_runtime::ResourceReport> for ResourceReport {
    fn from(value: pse_runtime::ResourceReport) -> Self {
        Self(value)
    }
}
macro_rules! report {
    ($( $(#[$meta:meta])* $field:ident: $ty:ty),*; consumers: $consumers:ident; caches: $caches:ident;) => {
        #[pymethods]
        impl ResourceReport {
            $( $(#[$meta])* #[getter] fn $field(&self) -> $ty { self.0.$field } )*
            #[getter]
            fn $consumers(&self) -> Tuple<ResourceConsumer> {
                Tuple(self.0.$consumers.iter().map(|(name, reserved_bytes)| ResourceConsumer {
                    name: name.clone(), reserved_bytes: *reserved_bytes,
                }).collect())
            }
            #[getter]
            fn $caches(&self) -> Tuple<super::CacheReport> {
                Tuple(self.0.$caches.iter().cloned().map(super::CacheReport::from).collect())
            }
        }
    };
}
pse_runtime::resource_report_fields!(report);

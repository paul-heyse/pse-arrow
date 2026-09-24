// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit factories for the eleven PSE extensions, using the shared validator.

use datafusion::arrow::array::Array;
use datafusion::arrow::datatypes::{DataType, Field};
use datafusion::arrow::util::display::{ArrayFormatter, FormatOptions};
use datafusion::common::{
    DataFusionError, Result,
    types::{DFExtensionType, DFExtensionTypeRef},
};
use datafusion::logical_expr::registry::{
    ExtensionTypeRegistration, ExtensionTypeRegistry, ExtensionTypeRegistryRef,
    MemoryExtensionTypeRegistry,
};
use pse_schema::{Registry, model::EXTENSION_TYPES};
use std::sync::Arc;

/// Install canonical Arrow factories and every declared PSE factory without replacement.
/// # Errors
/// A duplicate registration or malformed storage/metadata contract.
pub fn build(registry: &Arc<Registry>) -> Result<ExtensionTypeRegistryRef> {
    let types = MemoryExtensionTypeRegistry::new_with_canonical_extension_types();
    for spec in EXTENSION_TYPES {
        let registry = Arc::clone(registry);
        let name = spec.name;
        let registration = ExtensionTypeRegistration::new_arc(name, move |storage, metadata| {
            let mut field = Field::new("value", storage.clone(), true);
            let mut attributes = std::collections::HashMap::new();
            attributes.insert(
                pse_schema::arrow::KEY_EXTENSION_NAME.to_owned(),
                name.to_owned(),
            );
            if let Some(metadata) = metadata {
                attributes.insert(
                    pse_schema::arrow::KEY_EXTENSION_METADATA.to_owned(),
                    metadata.to_owned(),
                );
            }
            field = field.with_metadata(attributes);
            pse_relations::validate::validate_field(&registry, &field).map_err(|errors| {
                DataFusionError::Plan(format!("invalid extension {name}: {errors:?}"))
            })?;
            let extension: DFExtensionTypeRef = Arc::new(PlatformExtension { field });
            Ok(extension)
        });
        if types
            .add_extension_type_registration(registration)?
            .is_some()
        {
            return Err(DataFusionError::Configuration(format!(
                "duplicate extension {name}"
            )));
        }
    }
    Ok(Arc::new(types))
}
#[derive(Debug)]
struct PlatformExtension {
    field: Field,
}
impl DFExtensionType for PlatformExtension {
    fn storage_type(&self) -> DataType {
        self.field.data_type().clone()
    }
    fn serialize_metadata(&self) -> Option<String> {
        self.field
            .metadata()
            .get(pse_schema::arrow::KEY_EXTENSION_METADATA)
            .cloned()
    }
    fn create_array_formatter<'fmt>(
        &self,
        array: &'fmt dyn Array,
        options: &FormatOptions<'fmt>,
    ) -> Result<Option<ArrayFormatter<'fmt>>> {
        pse_relations::ext::create_owned_formatter(array, options, self.field.clone())
            .map_err(DataFusionError::from)
    }
}

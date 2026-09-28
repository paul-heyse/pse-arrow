// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared value domains of native floating-point fields. The storage admits NaN and
//! both infinities; a column whose meaning excludes them says so here, once, and every
//! target renders the guard in its own dialect (the DataFusion validator and the
//! PostgreSQL CHECK of the operational store).

use arrow_schema::{DataType, Field};

use crate::SchemaError;

/// `finite` or `not_nan` on a Float64 field.
pub const KEY_FLOAT_DOMAIN: &str = "pse.semantic.float_domain";

/// Which non-ordinary values a floating-point column refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloatDomain {
    /// No NaN and no infinity.
    Finite,
    /// No NaN; an infinity is a meaningful value (an unbounded dual bound, say).
    NotNan,
}

impl FloatDomain {
    /// The canonical metadata spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finite => "finite",
            Self::NotNan => "not_nan",
        }
    }

    /// Annotate a field. Registry admission checks its storage.
    pub fn annotate(self, mut field: Field) -> Field {
        field
            .metadata_mut()
            .insert(KEY_FLOAT_DOMAIN.into(), self.as_str().into());
        field
    }

    /// Read and validate the declared domain.
    ///
    /// # Errors
    /// An unknown spelling or a non-Float64 storage.
    pub fn from_field(field: &Field) -> Result<Option<Self>, SchemaError> {
        let Some(text) = field.metadata().get(KEY_FLOAT_DOMAIN) else {
            return Ok(None);
        };
        let domain = match text.as_str() {
            "finite" => Self::Finite,
            "not_nan" => Self::NotNan,
            _ => {
                return Err(crate::checks::invalid(
                    field.name(),
                    "unknown floating-point domain",
                ));
            }
        };
        if field.data_type() != &DataType::Float64 {
            return Err(crate::checks::invalid(
                field.name(),
                "a floating-point domain requires Float64 storage",
            ));
        }
        Ok(Some(domain))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_domains_round_trip_and_require_float_storage() {
        for domain in [FloatDomain::Finite, FloatDomain::NotNan] {
            let field = domain.annotate(Field::new("x", DataType::Float64, true));
            assert_eq!(FloatDomain::from_field(&field).unwrap(), Some(domain));
            let wrong = domain.annotate(Field::new("x", DataType::Int64, true));
            assert!(FloatDomain::from_field(&wrong).is_err());
        }
        assert_eq!(
            FloatDomain::from_field(&Field::new("x", DataType::Float64, true)).unwrap(),
            None
        );
    }
}

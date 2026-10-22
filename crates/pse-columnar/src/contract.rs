// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared native columnar field paths and finite allocation envelopes.

use std::fmt;
use crate::error::CanonError;

/// A position in a schema: zero-based registry column and child ordinals in ASCII decimal
/// without leading zeros, joined by `/`.
///
/// The empty path denotes the schema itself. `0/2` is the third child of the first column.
/// Ordinals rather than names, because a column rename must not change a field path, and
/// `/` rather than `.` because a struct field name may contain a dot.
///
/// ```
/// use pse_columnar::FieldPath;
///
/// assert_eq!(FieldPath::root().as_str(), "");
/// assert_eq!(FieldPath::root().child(0).child(2).as_str(), "0/2");
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldPath(String);

impl FieldPath {
    /// The schema itself.
    pub fn root() -> Self {
        Self(String::new())
    }

    /// The `ordinal`-th child of this path.
    #[must_use]
    pub fn child(&self, ordinal: usize) -> Self {
        if self.0.is_empty() {
            Self(ordinal.to_string())
        } else {
            Self(format!("{}/{ordinal}", self.0))
        }
    }

    /// The rendered ordinal path used by native columnar diagnostics.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The selected resource envelope of a native columnar operation (blueprint §14.3).
///
/// A stated bound rather than an implicit one: allocation preflight uses checked arithmetic against
/// these numbers and reserves before allocating, so exceeding one is
/// `runtime.resource_limit` and never a partial publication.
///
/// ```
/// use pse_columnar::Envelope;
///
/// assert_eq!(Envelope::DEFAULT.max_rows, 100_000_000);
/// assert!(Envelope::new(1_000, 1 << 20).is_ok());
/// // Deployment settings remain bounded by the native representation.
/// assert!(Envelope::new(200_000_000, 1 << 30).is_ok());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Envelope {
    /// Rows in the complete relation.
    pub max_rows: u64,
    /// Bytes of normalized value, offset and validity buffers.
    pub max_normalized_bytes: u64,
}

impl Envelope {
    /// Selected default: 100 million rows and 32 GiB of normalized buffers on 64-bit
    /// hosts. This is a configurable resource policy, not a measured workload promise.
    pub const DEFAULT: Self = Self {
        max_rows: 100_000_000,
        max_normalized_bytes: if isize::BITS >= 64 {
            32 << 30
        } else {
            isize::MAX as u64
        },
    };

    /// Select a deployment envelope within actual row-index and addressability limits.
    /// Zero admits only empty data. Variable-width offsets retain their own checked
    /// representation limits; selecting a larger policy never widens an Arrow offset.
    ///
    /// # Errors
    ///
    /// [`CanonError::Envelope`] when a policy exceeds native `u32` row indices or
    /// addressable buffer sizes. No allocation is performed by this constructor.
    pub fn new(max_rows: u64, max_normalized_bytes: u64) -> Result<Self, CanonError> {
        let row_limit = u64::from(u32::MAX).min(isize::MAX as u64);
        if max_rows > row_limit {
            return Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Rows,
                limit: row_limit,
                actual: max_rows,
            });
        }
        if max_normalized_bytes > isize::MAX as u64 {
            return Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Bytes,
                limit: isize::MAX as u64,
                actual: max_normalized_bytes,
            });
        }
        Ok(Self {
            max_rows,
            max_normalized_bytes,
        })
    }
}

impl Default for Envelope {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_path_is_ordinals_not_names() {
        assert_eq!(FieldPath::root().as_str(), "");
        assert_eq!(FieldPath::root().to_string(), "");
        assert_eq!(FieldPath::root().child(0).as_str(), "0");
        assert_eq!(FieldPath::root().child(0).child(2).as_str(), "0/2");
        assert_eq!(FieldPath::root().child(10).child(0).as_str(), "10/0");

        // Diagnostic ordering uses the rendered ordinal paths.
        let mut paths = [
            FieldPath::root().child(1),
            FieldPath::root(),
            FieldPath::root().child(0).child(2),
        ];
        paths.sort();
        assert_eq!(
            paths.iter().map(FieldPath::as_str).collect::<Vec<_>>(),
            vec!["", "0/2", "1"]
        );
    }

    #[test]
    fn envelope_policy_is_configurable_with_checked_representation_limits() {
        assert_eq!(Envelope::default(), Envelope::DEFAULT);
        assert_eq!(
            Envelope::new(1_000, 1 << 20).ok(),
            Some(Envelope {
                max_rows: 1_000,
                max_normalized_bytes: 1 << 20
            })
        );
        assert!(matches!(
            Envelope::new(u64::from(u32::MAX) + 1, 0),
            Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Rows,
                ..
            })
        ));
        assert!(matches!(
            Envelope::new(0, isize::MAX as u64 + 1),
            Err(CanonError::Envelope {
                what: crate::error::EnvelopeBound::Bytes,
                ..
            })
        ));
        assert!(Envelope::new(200_000_000, 1 << 30).is_ok());
        assert!(Envelope::new(0, 0).is_ok());
    }
}

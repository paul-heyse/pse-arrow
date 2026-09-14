// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Where an artifact lives in the store, and what a ref may be called
//! (blueprint §20.1).
//!
//! Every path is computed here and nowhere else. A path assembled at a call site is a
//! second declaration of the layout, and the two disagree on the day one of them gains a
//! component — which is exactly the failure §20.1's "verify the existing object against
//! the intended manifest member" is meant to catch and would then be unable to, because
//! it would be looking in the other place.
//!
//! Paths are content-addressed: the name of an object is a claim about its bytes, never
//! a claim about its meaning. `relations/<ns>/<name>@<v>/<encoding_checksum>.arrow`
//! carries the relation's identity in the directory and the *encoding*'s identity in the
//! file name, so two encodings of one logical relation are siblings and neither can
//! overwrite the other (ADR-0045).

use object_store::path::Path;
use pse_ids::{ContentHash, EncodingChecksum, SchemaVersion, SemanticId};

use crate::error::CatalogError;

/// The directory holding mutable refs.
const DIR_REFS: &str = "refs";
/// The directory holding encoded manifests.
const DIR_MANIFESTS: &str = "manifests";
/// The directory holding relation artifacts.
const DIR_RELATIONS: &str = "relations";
/// The directory holding noncanonical plan and diagnostic evidence (ADR-0044).
const DIR_EVIDENCE: &str = "evidence";
/// The directory holding exact authored source bytes.
const DIR_DOCUMENTS: &str = "documents";
/// The directory holding the stage-key sidecar, which is never snapshot membership.
const DIR_STAGES: &str = "stages";
/// Complete durable change-set envelopes, outside semantic membership.
const DIR_CHANGES: &str = "changes";

/// The suffix of the JSON-encoded objects.
const EXT_JSON: &str = "json";

pub(super) fn local_temporary_name(process: u32, ordinal: u64) -> String {
    format!(".pse-control-{process}-{ordinal}.pending")
}

/// Recognize only this protocol's unreachable temporary siblings after interruption.
pub(super) fn is_local_temporary(name: &str) -> bool {
    name.strip_prefix(".pse-control-")
        .and_then(|name| name.strip_suffix(".pending"))
        .and_then(|name| name.split_once('-'))
        .is_some_and(|(process, ordinal)| {
            process
                .parse::<u32>()
                .is_ok_and(|value| value.to_string() == process)
                && ordinal
                    .parse::<u64>()
                    .is_ok_and(|value| value.to_string() == ordinal)
        })
}

/// The store directories a snapshot member may never be published under (§5.3 step 7).
///
/// Refs are mutable, manifests contain the membership being computed, and stages are
/// pass-attempt bookkeeping. A member in any of these directories would make a snapshot ID
/// depend on something the snapshot contains, which is the self-reference §5.3 forbids.
pub const SIDECAR_DIRECTORIES: [&str; 4] = [DIR_REFS, DIR_MANIFESTS, DIR_STAGES, DIR_CHANGES];

/// The maximum length of a [`RefName`], including the first character.
const REF_NAME_MAX: usize = 64;

/// A mutable alias for a snapshot: `head`, `main`, `case.base` (blueprint §20.1).
///
/// The accepted shape is `^[a-z0-9][a-z0-9._-]{0,63}$`, checked by hand rather than by a
/// regular expression: one function that answers "is this a ref name?" is cheaper to read
/// than a dependency, and a ref name is also a path segment, so an unchecked one could
/// escape `refs/` entirely.
///
/// ```
/// use pse_catalog::RefName;
///
/// assert!(RefName::parse("head").is_ok());
/// assert!(RefName::parse("case.base-1").is_ok());
/// // No traversal, no upper case, no leading punctuation.
/// assert!(RefName::parse("../escape").is_err());
/// assert!(RefName::parse("HEAD").is_err());
/// assert!(RefName::parse(".hidden").is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefName(String);

impl RefName {
    /// Reads a ref name, or says which character refused it.
    ///
    /// # Errors
    ///
    /// [`CatalogError::ConfigInvalid`] naming the rule that failed. A ref name arrives
    /// from a caller's configuration, so this is `config::invalid` and not an internal
    /// invariant.
    pub fn parse(text: &str) -> Result<Self, CatalogError> {
        let refused = |reason: &str| CatalogError::ConfigInvalid {
            key: format!("ref name `{text}`"),
            reason: reason.to_owned(),
        };

        let mut characters = text.chars();
        let Some(first) = characters.next() else {
            return Err(refused("a ref name is not empty"));
        };
        if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
            return Err(refused(
                "a ref name starts with a lowercase letter or a digit",
            ));
        }
        for character in characters {
            let accepted = character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-');
            if !accepted {
                return Err(refused(
                    "a ref name continues with lowercase letters, digits, `.`, `_` or `-`",
                ));
            }
        }
        if text.chars().count() > REF_NAME_MAX {
            return Err(refused("a ref name is at most 64 characters"));
        }
        Ok(Self(text.to_owned()))
    }

    /// The name as written, which is also its path segment under `refs/`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RefName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A physical encoding of a relation (blueprint §20.1, §20.2).
///
/// Arrow IPC files are the hot path for compiled and runtime relations; Parquet is the
/// durable form for authored, reference and runtime history. Both are encodings of the
/// same logical relation and carry the same `logical_hash` and different
/// `encoding_checksum`s — which is the whole of ADR-0045 in one sentence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EncodingFormat {
    /// A finished Arrow IPC **file** (not a stream).
    ArrowIpcFile,
    /// A Parquet file.
    Parquet,
}

impl EncodingFormat {
    /// Every format, in declaration order; the parse side reads this rather than
    /// repeating the spellings.
    pub const ALL: [Self; 2] = [Self::ArrowIpcFile, Self::Parquet];

    /// The spelling the manifest stores.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ArrowIpcFile => "arrow_ipc_file",
            Self::Parquet => "parquet",
        }
    }

    /// The file extension, without the dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::ArrowIpcFile => "arrow",
            Self::Parquet => "parquet",
        }
    }

    /// Reads the spelling [`Self::as_str`] writes.
    ///
    /// # Errors
    ///
    /// [`CatalogError::UnknownVersion`] for any other spelling: §20.5 rejects an unknown
    /// encoding rather than guessing which decoder to reach for.
    pub fn parse(text: &str) -> Result<Self, CatalogError> {
        Self::ALL
            .into_iter()
            .find(|format| format.as_str() == text)
            .ok_or_else(|| CatalogError::UnknownVersion {
                field: "format".to_owned(),
                value: text.to_owned(),
            })
    }
}

impl std::fmt::Display for EncodingFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `refs/<name>.json` — the one mutable object in the store.
#[must_use]
pub fn ref_path(name: &RefName) -> Path {
    Path::from(format!("{DIR_REFS}/{name}.{EXT_JSON}"))
}

/// `manifests/<manifest_checksum>.json` — immutable; the snapshot ID is a field inside,
/// never the object's own name (blueprint §20.1).
#[must_use]
pub fn manifest_path(checksum: &EncodingChecksum) -> Path {
    Path::from(format!(
        "{DIR_MANIFESTS}/{}.{EXT_JSON}",
        checksum.content_hash().to_hex()
    ))
}

/// `relations/<namespace>/<name>@<version>/<encoding_checksum>.<ext>`.
#[must_use]
pub fn relation_path(
    namespace: &str,
    name: &str,
    version: SchemaVersion,
    checksum: &EncodingChecksum,
    format: EncodingFormat,
) -> Path {
    Path::from(format!(
        "{DIR_RELATIONS}/{namespace}/{name}@{version}/{}.{}",
        checksum.content_hash().to_hex(),
        format.extension()
    ))
}

/// `evidence/<encoding_checksum>` — noncanonical plan and diagnostic encodings
/// (ADR-0044). No extension: the codec is a manifest field, not a file suffix.
#[must_use]
pub fn evidence_path(checksum: &EncodingChecksum) -> Path {
    Path::from(format!(
        "{DIR_EVIDENCE}/{}",
        checksum.content_hash().to_hex()
    ))
}

/// `documents/<document_id>/<encoding_checksum>` — the exact authored source bytes.
#[must_use]
pub fn document_path(document_id: SemanticId, checksum: &EncodingChecksum) -> Path {
    Path::from(format!(
        "{DIR_DOCUMENTS}/{}/{}",
        document_id.to_hex(),
        checksum.content_hash().to_hex()
    ))
}

/// `stages/<stage_key>.json` — the stage-key sidecar (blueprint §5.3 step 7, ADR-0041).
///
/// A sidecar, never a member: it records which snapshot a stage key resolved to, so a
/// snapshot that contained it would contain a statement about itself.
#[must_use]
pub fn stage_path(key: &ContentHash) -> Path {
    Path::from(format!("{DIR_STAGES}/{}.{EXT_JSON}", key.to_hex()))
}

/// `changes/<encoding_checksum>.json`, the exact complete operation/source receipt.
#[must_use]
pub fn change_set_path(checksum: &EncodingChecksum) -> Path {
    Path::from(format!(
        "{DIR_CHANGES}/{}.{EXT_JSON}",
        checksum.content_hash().to_hex()
    ))
}

/// Whether `path` lies under one of the [`SIDECAR_DIRECTORIES`].
#[must_use]
pub fn is_sidecar_path(path: &str) -> bool {
    SIDECAR_DIRECTORIES
        .iter()
        .any(|directory| path.starts_with(&format!("{directory}/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checksum(byte: u8) -> EncodingChecksum {
        EncodingChecksum(ContentHash::from_bytes([byte; 32]))
    }

    fn hex(byte: u8) -> String {
        ContentHash::from_bytes([byte; 32]).to_hex()
    }

    #[test]
    fn accepted_ref_names() {
        for name in [
            "head",
            "main",
            "case.base",
            "a",
            "0",
            "a-b_c.d",
            &"a".repeat(64),
        ] {
            assert!(
                RefName::parse(name).is_ok(),
                "`{name}` should be a ref name"
            );
        }
    }

    #[test]
    fn refused_ref_names() {
        for name in [
            "",
            "HEAD",
            "-head",
            ".head",
            "_head",
            "head/child",
            "head name",
            "héad",
            "head!",
            "..",
            &"a".repeat(65),
        ] {
            assert!(
                RefName::parse(name).is_err(),
                "`{name}` should not be a ref name"
            );
        }
    }

    #[test]
    fn a_ref_name_renders_as_written() {
        let Ok(name) = RefName::parse("head") else {
            panic!("`head` is a ref name");
        };
        assert_eq!(name.as_str(), "head");
        assert_eq!(name.to_string(), "head");
    }

    #[test]
    fn the_layout_renders_exactly_as_section_20_1_declares_it() {
        let Ok(name) = RefName::parse("head") else {
            panic!("`head` is a ref name");
        };
        assert_eq!(ref_path(&name).as_ref(), "refs/head.json");

        assert_eq!(
            manifest_path(&checksum(0xab)).as_ref(),
            format!("manifests/{}.json", hex(0xab))
        );

        assert_eq!(
            relation_path(
                "compiled",
                "math_expr_nodes",
                SchemaVersion(1),
                &checksum(0x01),
                EncodingFormat::ArrowIpcFile,
            )
            .as_ref(),
            format!("relations/compiled/math_expr_nodes@1/{}.arrow", hex(0x01))
        );
        assert_eq!(
            relation_path(
                "authored",
                "stoichiometry",
                SchemaVersion(7),
                &checksum(0x02),
                EncodingFormat::Parquet,
            )
            .as_ref(),
            format!("relations/authored/stoichiometry@7/{}.parquet", hex(0x02))
        );

        assert_eq!(
            evidence_path(&checksum(0x03)).as_ref(),
            format!("evidence/{}", hex(0x03))
        );

        assert_eq!(
            document_path(SemanticId::from_bytes([0x04; 16]), &checksum(0x05)).as_ref(),
            format!("documents/{}/{}", "04".repeat(16), hex(0x05))
        );

        assert_eq!(
            stage_path(&ContentHash::from_bytes([0x06; 32])).as_ref(),
            format!("stages/{}.json", hex(0x06))
        );
    }

    #[test]
    fn the_at_sign_survives_object_store_path_encoding() {
        // `object_store` percent-encodes a set of characters AWS and GCS advise against;
        // `@` is not one of them, so the `name@version` segment is stored literally.
        let rendered = relation_path(
            "compiled",
            "math_equations",
            SchemaVersion(2),
            &checksum(0x07),
            EncodingFormat::ArrowIpcFile,
        );
        assert!(rendered.as_ref().contains("math_equations@2"));
    }

    #[test]
    fn the_formats_spell_themselves_and_parse_back() {
        assert_eq!(EncodingFormat::ArrowIpcFile.as_str(), "arrow_ipc_file");
        assert_eq!(EncodingFormat::ArrowIpcFile.extension(), "arrow");
        assert_eq!(EncodingFormat::Parquet.to_string(), "parquet");
        assert_eq!(EncodingFormat::Parquet.extension(), "parquet");
        for format in EncodingFormat::ALL {
            assert_eq!(EncodingFormat::parse(format.as_str()).ok(), Some(format));
        }
        assert!(matches!(
            EncodingFormat::parse("feather"),
            Err(CatalogError::UnknownVersion { .. })
        ));
    }

    #[test]
    fn the_sidecar_directories_are_recognised() {
        assert!(is_sidecar_path("refs/head.json"));
        assert!(is_sidecar_path("manifests/aa.json"));
        assert!(is_sidecar_path("stages/aa.json"));
        assert!(!is_sidecar_path("relations/compiled/x@1/aa.arrow"));
        assert!(!is_sidecar_path("evidence/aa"));
        assert!(!is_sidecar_path("references/aa"));
    }
}

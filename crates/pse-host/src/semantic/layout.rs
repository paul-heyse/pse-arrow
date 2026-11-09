// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Guarded association of fallible HIR layout with an original opaque compiler artifact.
use super::Answer;
use crate::compiler_investigation::artifact::{Kind, Observation};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;

/// Association of recorded premises, separate from numerical ABI comparison.
#[derive(Debug, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutAssociationState {
    /// Every selected recorded guard agrees; compiler layout remains opaque.
    CompatiblePremises,
    /// At least one recorded guard disagrees.
    Refused,
    /// Required matching premises were not recorded by one of the providers.
    Unknown,
}
/// An association never interprets rustc debug text or compares type spellings.
#[derive(Debug, Serialize, JsonSchema)]
pub struct LayoutAssociation {
    /// Original retained compiler acquisition.
    pub compiler_invocation: crate::envelope::CliInvocationId,
    /// Original terminal-bound compiler context identity.
    pub compiler_context_sha256: Option<String>,
    /// Retained selected-unit/source/compiler premises, not current-workspace guesses.
    pub compiler_context: crate::compiler_investigation::artifact::SelectedContext,
    /// Exact original indexed files with byte-preserving paths and digests.
    pub compiler_files: Vec<crate::compiler_investigation::artifact::ArtifactFile>,
    /// Guard outcome, not an ABI equality claim.
    pub state: LayoutAssociationState,
    /// Which independent observed guards agree.
    pub matched: Vec<String>,
    /// Mismatches that prevent association.
    pub refusals: Vec<String>,
    /// Missing or opaque premises; never silently treated as equal.
    pub unknown: Vec<String>,
    /// Always absent: native compiler layout debug output is retained as opaque evidence.
    pub numerical_equality: Option<bool>,
}
/// Compose already acquired evidence without Cargo, compilation or display-name matching.
pub fn associate_layout(native: &Answer, compiler: &Observation) -> LayoutAssociation {
    let mut result = LayoutAssociation {
        compiler_invocation: compiler.original_invocation.clone(),
        compiler_context_sha256: compiler
            .index
            .as_ref()
            .map(|index| index.context_sha256.clone()),
        compiler_context: crate::compiler_investigation::artifact::selected_context(compiler),
        compiler_files: compiler
            .index
            .as_ref()
            .map(|index| {
                index
                    .files
                    .iter()
                    .map(
                        |file| crate::compiler_investigation::artifact::ArtifactFile {
                            path: file.path.clone(),
                            bytes: file.bytes,
                            sha256: file.sha256.clone(),
                        },
                    )
                    .collect()
            })
            .unwrap_or_default(),
        state: LayoutAssociationState::Unknown,
        matched: Vec::new(),
        refusals: Vec::new(),
        unknown: Vec::new(),
        numerical_equality: None,
    };
    if compiler.context.kind != Kind::TypeSizes {
        result
            .refusals
            .push("Selected compiler artifact is not a type-sizes acquisition.".into());
    }
    if compiler.original_outcome != crate::envelope::CommandOutcome::Success
        || compiler.index.is_none()
    {
        result.refusals.push(
            "Compiler acquisition did not publish a successful completed artifact index.".into(),
        );
    }
    let source = compiler
        .context
        .inputs
        .iter()
        .find(|input| Path::new(&input.source) == native.source.path());
    match source {
        Some(source) if source.sha256 == native.source.source_sha256() => result
            .matched
            .push("Exact selected source path and captured bytes agree.".into()),
        Some(_) => result
            .refusals
            .push("Selected source bytes differ from the retained compiler input.".into()),
        None => result
            .unknown
            .push("Compiler acquisition did not retain this selected native source file.".into()),
    }
    let facts = native.native_types.as_ref();
    if facts.and_then(|facts| facts.layout.as_ref()).is_none() {
        result
            .unknown
            .push("The native answer has no explicit fallible layout observation.".into());
    }
    if let Some(context) = facts.and_then(|facts| facts.context.as_ref()) {
        let roots = compiler
            .context
            .admitted
            .units
            .iter()
            .filter(|unit| unit.root)
            .collect::<Vec<_>>();
        if let [root] = roots.as_slice() {
            if Path::new(&root.source_path) != context.crate_root {
                result
                    .refusals
                    .push("Compiler root and native crate root differ.".into());
            } else {
                result
                    .matched
                    .push("Actual native crate root matches the selected compiler root.".into());
            }
            if root.features != context.features {
                result
                    .refusals
                    .push("Actual effective feature sets differ.".into());
            } else {
                result
                    .matched
                    .push("Actual effective feature sets agree.".into());
            }
            match (&root.platform,&context.target) {
                (Some(compiler),Some(native)) if compiler==native=>result.matched.push("Explicit actual targets agree.".into()),
                (Some(_),Some(_))=>result.refusals.push("Actual targets differ.".into()),
                _=>result.unknown.push("Select an explicit compiler target and a unique native target profile to compare placement.".into()),
            }
            if context.cfg.iter().any(|cfg| cfg == "test") != (root.mode == "test") {
                result
                    .refusals
                    .push("Native cfg(test) and compiler unit mode differ.".into());
            }
        } else {
            result
                .unknown
                .push("Compiler acquisition has no unique selected root.".into());
        }
        if context.compiler_identity.is_empty() {
            result
                .unknown
                .push("Native compiler identity is unavailable.".into());
        } else if context.compiler_identity != compiler.context.rustc_identity {
            result
                .refusals
                .push("Actual compiler identities differ.".into());
        } else {
            result
                .matched
                .push("Actual compiler verbose identities agree.".into());
        }
        // Opaque compiler layout artifacts do not report structured edition/cfg premises.
        result.unknown.push("Opaque compiler layout artifacts do not expose structured cfg/edition or per-type identity; no numerical ABI comparison is available.".into());
    } else {
        result
            .unknown
            .push("Native type context is unavailable.".into());
    }
    result.state = if !result.refusals.is_empty() {
        LayoutAssociationState::Refused
    } else if !result.unknown.is_empty() {
        LayoutAssociationState::Unknown
    } else {
        LayoutAssociationState::CompatiblePremises
    };
    result
}

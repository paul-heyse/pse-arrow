// SPDX-License-Identifier: MIT OR Apache-2.0
//! Presentation of owned, pinned compiler observations; never a textual MIR parser.
use super::{artifact, invalid};
use crate::{
    HostError,
    code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb,
    envelope::{CliInvocationId, Completeness, Envelope, NextAction},
    paths::CheckoutState,
};
use buffa::Message as _;
use schemars::JsonSchema;
use serde::Serialize;
use std::{num::NonZeroUsize, path::Path};

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CaptureMode {
    Shared,
    UniqueShared,
    Mutable,
    Moved,
    Used,
}
/// Coordinates in normalized compiler source. They do not assert current authored correspondence.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Source {
    pub file: String,
    pub start: u32,
    pub end: u32,
    pub macro_expanded: bool,
    /// Retained compiler-verified original file; absent in historical reports.
    pub source_file: Option<u32>,
    /// Original raw UTF-8 offsets, separate from normalized compiler offsets.
    pub original_start: Option<u32>,
    pub original_end: Option<u32>,
}
/// One exact original file retained within the digest-bound compiler facts artifact.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct SourceFile {
    pub id: u32,
    pub path: crate::investigation::NativePath,
    pub display_name: String,
    pub original_sha256: String,
    pub normalized_sha256: String,
}
fn source(value: Option<pb::CompilerSourceLocation>) -> Option<Source> {
    value.map(|s| Source {
        file: s.file,
        start: s.start,
        end: s.end,
        macro_expanded: s.macro_expanded,
        source_file: s.source_file,
        original_start: s.original_start,
        original_end: s.original_end,
    })
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Capture {
    pub place_display: String,
    pub source_type_display: String,
    pub stored_type_display: Option<String>,
    pub mode: CaptureMode,
    pub source: Option<Source>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct SavedField {
    pub index: u32,
    pub type_display: String,
    pub source_display: String,
    pub source: Option<Source>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct SuspensionVariant {
    pub index: u32,
    pub fields: Vec<u32>,
    pub source_display: String,
    pub source: Option<Source>,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Terminator {
    Drop,
    Call,
    Assert,
    Return,
    UnwindResume,
    UnwindTerminate,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Unwind {
    NoEdge,
    Continue,
    Unreachable,
    Cleanup { block: u32 },
    Terminate,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Initialization {
    pub may_initialized: bool,
    pub may_uninitialized: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TerminationCause {
    Abi,
    InCleanup,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Callee {
    pub abi: String,
    pub declaration_path: Option<String>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Cleanup {
    pub phase: String,
    pub block: u32,
    pub terminator: Terminator,
    pub local: Option<u32>,
    pub place_display: String,
    pub normal_successors: Vec<u32>,
    pub unwind: Unwind,
    pub source_display: String,
    pub source: Option<Source>,
    pub is_cleanup: bool,
    pub initialization: Option<Initialization>,
    pub termination_cause: Option<TerminationCause>,
    pub callee: Option<Callee>,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Body {
    pub path: String,
    pub captures: Vec<Capture>,
    pub saved_fields: Vec<SavedField>,
    pub variants: Vec<SuspensionVariant>,
    pub cleanup: Vec<Cleanup>,
    pub source: Option<Source>,
    pub unavailable: Vec<String>,
    pub abi: String,
    pub panic_strategy: String,
    pub coroutine: bool,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct Report {
    pub original_invocation: CliInvocationId,
    pub compiler: String,
    /// Historical unit/source/feature/compiler inputs. No current-world equality is implied.
    pub context: artifact::SelectedContext,
    pub bodies: Vec<Body>,
    pub omitted_bodies: usize,
    pub fact_focus: artifact::FactFocus,
    pub source_files: Vec<SourceFile>,
    pub cfg: Vec<String>,
    pub edition: Option<String>,
    pub target: Option<String>,
}
/// A bounded read of source bytes retained by the original compiler, never the current path.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct SourceRead {
    pub original_invocation: CliInvocationId,
    pub source_file: u32,
    pub path: crate::investigation::NativePath,
    pub original_sha256: String,
    pub original_bytes: u64,
    pub byte_range: std::ops::Range<u64>,
    pub content: crate::artifacts::Content,
    pub next_offset: Option<u64>,
}
fn retained_facts(original: &artifact::Observation) -> Result<pb::CompilerFacts, HostError> {
    if original.context.kind != artifact::Kind::Facts {
        return Err(invalid(
            "original acquisition did not select typed compiler observations",
        ));
    }
    let file = original
        .index
        .as_ref()
        .and_then(|index| {
            index.files.iter().find(|file| {
                Path::new(&file.path)
                    .file_name()
                    .is_some_and(|name| name == "facts.binpb")
            })
        })
        .ok_or_else(|| invalid("published compiler facts file is absent"))?;
    let bytes = crate::test_inventory::read_private(Path::new(&file.path))?;
    if super::digest(&bytes) != file.sha256 {
        return Err(invalid("original compiler facts digest changed"));
    }
    validate_acquisition(&bytes, &original.context)
}
pub(crate) fn read_source(
    root: &Path,
    state: &CheckoutState,
    selector: &str,
    source_file: u32,
    offset: u64,
    length: std::num::NonZeroU64,
) -> Result<Envelope<SourceRead>, HostError> {
    use base64::Engine as _;
    let original = artifact::observe(root, state, selector)?;
    let facts = retained_facts(&original.payload)?;
    let file = facts
        .source_files
        .into_iter()
        .find(|file| file.id == source_file)
        .ok_or_else(|| {
            invalid(
                "original compiler source file is absent; historical facts cannot reacquire source",
            )
        })?;
    let total = file.original_source.len() as u64;
    if offset > total {
        return Err(invalid("compiler source offset exceeds retained bytes"));
    }
    let end = offset.saturating_add(length.get()).min(total);
    let selected = &file.original_source[offset as usize..end as usize];
    let content = std::str::from_utf8(selected)
        .map(|text| crate::artifacts::Content::Utf8(text.into()))
        .unwrap_or_else(|_| {
            crate::artifacts::Content::Base64(
                base64::engine::general_purpose::STANDARD.encode(selected),
            )
        });
    let path = {
        use std::os::unix::ffi::OsStringExt as _;
        std::path::PathBuf::from(std::ffi::OsString::from_vec(file.local_path.to_vec())).into()
    };
    let mut response = Envelope::response(
        "inspect compiler-source",
        "bounded original compiler-verified bytes; no current source read",
        SourceRead {
            original_invocation: original.payload.original_invocation,
            source_file,
            path,
            original_sha256: file.original_sha256,
            original_bytes: total,
            byte_range: offset..end,
            content,
            next_offset: (end < total).then_some(end),
        },
    );
    response.providers = original.providers;
    response.artifacts = original.artifacts;
    if let Some(root) = root.to_str().filter(|_| end < total) {
        response.next.push(NextAction::Command {
            argv: vec![
                "pse".into(),
                "-C".into(),
                root.into(),
                "inspect".into(),
                "compiler-source".into(),
                response.payload.original_invocation.to_string(),
                source_file.to_string(),
                "--offset".into(),
                end.to_string(),
                "--length".into(),
                length.to_string(),
            ],
            reason: "continue the exact retained compiler source bytes".into(),
        });
    }
    Ok(response)
}
/// Reject unsupported enum values rather than manufacturing a native conclusion.
fn project(body: pb::CompilerBodyFact) -> Result<Body, HostError> {
    let captures = body
        .captures
        .into_iter()
        .map(|capture| {
            use pb::compiler_capture_fact::Mode;
            let mode = match capture.mode.as_known() {
                Some(Mode::SHARED) => CaptureMode::Shared,
                Some(Mode::UNIQUE_SHARED) => CaptureMode::UniqueShared,
                Some(Mode::MUTABLE) => CaptureMode::Mutable,
                Some(Mode::MOVED) => CaptureMode::Moved,
                Some(Mode::USED) => CaptureMode::Used,
                _ => return Err(invalid("unsupported compiler capture mode")),
            };
            Ok(Capture {
                place_display: capture.place_display,
                source_type_display: capture.source_type_display,
                stored_type_display: capture.stored_type_display,
                mode,
                source: source(capture.source.into_option()),
            })
        })
        .collect::<Result<_, HostError>>()?;
    let cleanup = body
        .cleanup
        .into_iter()
        .map(|fact| {
            use pb::compiler_cleanup_fact::{Kind, Unwind as NativeUnwind};
            let terminator = match fact.kind.as_known() {
                Some(Kind::DROP) => Terminator::Drop,
                Some(Kind::CALL) => Terminator::Call,
                Some(Kind::ASSERT) => Terminator::Assert,
                Some(Kind::RETURN) => Terminator::Return,
                Some(Kind::UNWIND_RESUME) => Terminator::UnwindResume,
                Some(Kind::UNWIND_TERMINATE) => Terminator::UnwindTerminate,
                _ => return Err(invalid("unsupported compiler terminator kind")),
            };
            let unwind = match (fact.unwind.as_known(), fact.cleanup_block) {
                (Some(NativeUnwind::NO_UNWIND_EDGE), None) => Unwind::NoEdge,
                (Some(NativeUnwind::CONTINUE), None) => Unwind::Continue,
                (Some(NativeUnwind::UNREACHABLE), None) => Unwind::Unreachable,
                (Some(NativeUnwind::TERMINATE), None) => Unwind::Terminate,
                (Some(NativeUnwind::CLEANUP), Some(block)) => Unwind::Cleanup { block },
                _ => return Err(invalid("invalid or unsupported compiler unwind edge")),
            };
            let termination_cause = fact
                .termination_cause
                .map(|value| match value.as_known() {
                    Some(pb::compiler_cleanup_fact::TerminationCause::ABI) => {
                        Ok(TerminationCause::Abi)
                    }
                    Some(pb::compiler_cleanup_fact::TerminationCause::IN_CLEANUP) => {
                        Ok(TerminationCause::InCleanup)
                    }
                    _ => Err(invalid("unsupported compiler termination cause")),
                })
                .transpose()?;
            Ok(Cleanup {
                phase: fact.phase,
                block: fact.block,
                terminator,
                local: fact.local,
                place_display: fact.place_display,
                normal_successors: fact.normal_successors,
                unwind,
                source_display: fact.source_display,
                source: source(fact.source.into_option()),
                is_cleanup: fact.is_cleanup,
                initialization: fact.initialization.into_option().map(|i| Initialization {
                    may_initialized: i.may_initialized,
                    may_uninitialized: i.may_uninitialized,
                }),
                termination_cause,
                callee: fact.callee.into_option().map(|callee| Callee {
                    abi: callee.abi,
                    declaration_path: callee.declaration_path,
                }),
            })
        })
        .collect::<Result<_, HostError>>()?;
    Ok(Body {
        path: body.path,
        captures,
        cleanup,
        source: source(body.source.into_option()),
        unavailable: body.unavailable,
        abi: body.abi,
        panic_strategy: body.panic_strategy,
        coroutine: body.coroutine,
        saved_fields: body
            .saved_fields
            .into_iter()
            .map(|f| SavedField {
                index: f.index,
                type_display: f.type_display,
                source_display: f.source_display,
                source: source(f.source.into_option()),
            })
            .collect(),
        variants: body
            .variants
            .into_iter()
            .map(|v| SuspensionVariant {
                index: v.index,
                fields: v.fields,
                source_display: v.source_display,
                source: source(v.source.into_option()),
            })
            .collect(),
    })
}
pub(super) fn decode(bytes: &[u8]) -> Result<pb::CompilerFacts, HostError> {
    let facts = pb::CompilerFacts::decode_from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    if facts.compiler.is_empty() || facts.bodies.is_empty() {
        return Err(invalid(
            "compiler observations have no matched body/compiler identity",
        ));
    }
    let mut source_ids = std::collections::BTreeMap::new();
    for file in &facts.source_files {
        if source_ids.insert(file.id, file).is_some()
            || file.local_path.is_empty()
            || super::digest(&file.original_source) != file.original_sha256
            || file.normalized_sha256.is_empty()
            || std::str::from_utf8(&file.original_source).is_err()
        {
            return Err(invalid(
                "invalid compiler-verified original source identity",
            ));
        }
    }
    let validate_source = |source: &pb::CompilerSourceLocation| -> Result<(), HostError> {
        match (
            source.source_file,
            source.original_start,
            source.original_end,
        ) {
            (None, None, None) => Ok(()),
            (Some(id), Some(start), Some(end)) => {
                let file = source_ids
                    .get(&id)
                    .ok_or_else(|| invalid("compiler source file reference absent"))?;
                let text = std::str::from_utf8(&file.original_source)
                    .map_err(|_| invalid("compiler source not UTF-8"))?;
                if start > end || text.get(start as usize..end as usize).is_none() {
                    return Err(invalid("compiler original range is outside verified bytes"));
                }
                Ok(())
            }
            _ => Err(invalid("partial compiler original source binding")),
        }
    };
    let mut paths = std::collections::BTreeSet::new();
    for body in &facts.bodies {
        for source in body
            .source
            .as_option()
            .into_iter()
            .chain(
                body.captures
                    .iter()
                    .filter_map(|value| value.source.as_option()),
            )
            .chain(
                body.saved_fields
                    .iter()
                    .filter_map(|value| value.source.as_option()),
            )
            .chain(
                body.variants
                    .iter()
                    .filter_map(|value| value.source.as_option()),
            )
            .chain(
                body.cleanup
                    .iter()
                    .filter_map(|value| value.source.as_option()),
            )
        {
            validate_source(source)?;
        }
        if body.path.is_empty() || !paths.insert(&body.path) {
            return Err(invalid("compiler body identity is absent or duplicated"));
        }
        let fields = body
            .saved_fields
            .iter()
            .map(|f| f.index)
            .collect::<std::collections::BTreeSet<_>>();
        if fields.len() != body.saved_fields.len()
            || body
                .variants
                .iter()
                .any(|v| v.fields.iter().any(|field| !fields.contains(field)))
        {
            return Err(invalid(
                "suspension variant refers to an absent or duplicated saved field",
            ));
        }
        project(body.clone())?;
    }
    Ok(facts)
}
pub(super) fn validate_acquisition(
    bytes: &[u8],
    context: &artifact::Context,
) -> Result<pb::CompilerFacts, HostError> {
    let facts = decode(bytes)?;
    let selected = context
        .body
        .as_deref()
        .ok_or_else(|| invalid("original compiler body selection is absent"))?;
    if !facts.bodies.iter().any(|body| body.path == selected)
        || facts
            .bodies
            .iter()
            .any(|body| body.path != selected && !body.path.starts_with(&format!("{selected}::")))
    {
        return Err(invalid(
            "compiler facts do not match the exact selected owning body and its nested bodies",
        ));
    }
    if context.rustc_identity.lines().next() != Some(format!("rustc {}", facts.compiler).as_str()) {
        return Err(invalid(
            "compiler fact producer identity differs from the selected compiler",
        ));
    }
    Ok(facts)
}
/// Read only a published, digest-bound original bundle, with whole-body output budgeting.
pub(crate) fn observe(
    root: &Path,
    state: &CheckoutState,
    selector: &str,
    body: Option<&str>,
    budget: NonZeroUsize,
    focus: artifact::FactFocus,
) -> Result<Envelope<Report>, HostError> {
    observe_selected(root, state, selector, Selection::Body(body), budget, focus)
}
enum Selection<'a> {
    Body(Option<&'a str>),
    Source {
        ordinal: usize,
        source: Option<&'a crate::semantic::NativeLocation>,
    },
}
/// Select the original facts artifact and nominate bodies by verified original coordinates.
/// Nomination bounds presentation; the lifetime consumer independently checks unit correspondence.
pub(crate) fn observe_for_source(
    root: &Path,
    state: &CheckoutState,
    selector: &str,
    ordinal: usize,
    source: Option<&crate::semantic::NativeLocation>,
    budget: NonZeroUsize,
) -> Result<Envelope<Report>, HostError> {
    observe_selected(
        root,
        state,
        selector,
        Selection::Source { ordinal, source },
        budget,
        artifact::FactFocus::All,
    )
}
fn observe_selected(
    root: &Path,
    state: &CheckoutState,
    selector: &str,
    selection: Selection<'_>,
    budget: NonZeroUsize,
    focus: artifact::FactFocus,
) -> Result<Envelope<Report>, HostError> {
    let (body, ordinal, selected_source) = match selection {
        Selection::Body(body) => (body, None, None),
        Selection::Source { ordinal, source } => (None, Some(ordinal), Some(source)),
    };
    let original = artifact::observe(root, state, selector)?;
    if original.payload.context.kind != artifact::Kind::Facts {
        return Err(invalid(
            "original acquisition did not select typed compiler observations",
        ));
    }
    let index =
        original.payload.index.as_ref().ok_or_else(|| {
            invalid("original compiler observations were not successfully published")
        })?;
    let file = index
        .files
        .iter()
        .find(|f| {
            Path::new(&f.path)
                .file_name()
                .is_some_and(|n| n == "facts.binpb")
        })
        .ok_or_else(|| invalid("published compiler facts file is absent"))?;
    if ordinal.is_some_and(|ordinal| {
        index
            .files
            .get(ordinal)
            .is_none_or(|selected| selected.path != file.path)
    }) {
        return Err(invalid(
            "lifetime evidence selector must address the original facts.binpb artifact",
        ));
    }
    let facts = retained_facts(&original.payload)?;
    let acquired = original
        .payload
        .context
        .fact_focus
        .ok_or_else(|| invalid("original compiler fact demand is absent"))?;
    if focus != artifact::FactFocus::All && !acquired.includes(focus) {
        return Err(invalid(
            "requested fact family was not acquired; select a compatible original acquisition or explicitly generate new evidence",
        ));
    }
    let focus = if focus == artifact::FactFocus::All {
        acquired
    } else {
        focus
    };
    if body.is_some_and(|selected| !facts.bodies.iter().any(|body| body.path == selected)) {
        return Err(invalid(
            "requested body is not in the original acquired body population",
        ));
    }
    let mut bodies = Vec::new();
    let mut used = 0;
    let mut omitted = 0;
    for native in facts.bodies {
        if body.is_some_and(|selected| selected != native.path) {
            continue;
        }
        if let Some(selected) = selected_source {
            let nominated = selected.is_some_and(|selected| {
                native.source.as_option().is_some_and(|source| {
                    !source.macro_expanded
                        && source.original_start.map(|v| v as usize) == Some(selected.start)
                        && source.original_end.map(|v| v as usize) == Some(selected.end)
                        && source
                            .source_file
                            .and_then(|id| facts.source_files.iter().find(|file| file.id == id))
                            .is_some_and(|file| {
                                use std::os::unix::ffi::OsStrExt as _;
                                file.local_path.as_slice() == selected.path.as_os_str().as_bytes()
                            })
                })
            });
            if !nominated {
                omitted += 1;
                continue;
            }
        }
        let mut projected = project(native)?;
        if !focus.includes(artifact::FactFocus::Captures) {
            projected.captures.clear();
        }
        if !focus.includes(artifact::FactFocus::Suspension) {
            projected.saved_fields.clear();
            projected.variants.clear();
        }
        if !focus.includes(artifact::FactFocus::Cleanup) {
            projected.cleanup.clear();
        }
        let bytes = serde_json::to_vec(&projected)
            .map_err(std::io::Error::other)?
            .len();
        if used + bytes > budget.get() {
            omitted += 1;
        } else {
            used += bytes;
            bodies.push(projected);
        }
    }
    let selected_context = artifact::selected_context(&original.payload);
    let mut response = Envelope::response(
        "inspect compiler-facts",
        "owned original compiler observations; no compilation or current-world inference",
        Report {
            original_invocation: original.payload.original_invocation,
            compiler: facts.compiler,
            context: selected_context,
            bodies,
            omitted_bodies: omitted,
            fact_focus: focus,
            source_files: facts
                .source_files
                .into_iter()
                .map(|file| SourceFile {
                    id: file.id,
                    path: {
                        use std::os::unix::ffi::OsStringExt as _;
                        std::path::PathBuf::from(std::ffi::OsString::from_vec(
                            file.local_path.to_vec(),
                        ))
                        .into()
                    },
                    display_name: file.display_name,
                    original_sha256: file.original_sha256,
                    normalized_sha256: file.normalized_sha256,
                })
                .collect(),
            cfg: facts.cfg,
            edition: facts.edition,
            target: facts.target,
        },
    );
    response.providers = original.providers;
    for provider in &mut response.providers {
        // Original context bytes remain separately addressed. Repeating its transitive
        // compiler-unit graph in an opaque provider string defeats the selected projection.
        provider.context = Some(
            serde_json::json!({
                "original_invocation": response.payload.original_invocation,
                "context": response.payload.context.full_context,
            })
            .to_string(),
        );
    }
    response.artifacts = original.artifacts;
    response.next = original.next;
    if let Some(root) = root.to_str() {
        for file in &response.payload.source_files {
            response.next.push(NextAction::Command{argv:vec!["pse".into(),"-C".into(),root.into(),"inspect".into(),"compiler-source".into(),response.payload.original_invocation.to_string(),file.id.to_string()],reason:"read bounded compiler-verified original source bytes, even if the working file changed".into()});
        }
    }
    response.observation.scope = "original_selected_compiler_bodies".into();
    response.observation.limitations.push("Saved fields describe conservative compiler storage, not dynamically live resources or Send proof. Cleanup edges describe phase-specific static control flow, not execution or resource release. Initialization flags are conservative native may-analyses at exact move paths before analysis-phase drops; absent flags or both false establish no initialization or reachability conclusion. Source displays may be synthetic; no unique current-source correspondence is asserted. Loan facts are not extracted.".into());
    if omitted > 0 {
        response.observation.completeness = Completeness::Partial;
        response.observation.limitations.push("Whole compiler bodies omitted by output byte budget; increase --max-bytes or select --body/--fact-focus.".into());
    }
    if response
        .payload
        .bodies
        .iter()
        .any(|body| !body.unavailable.is_empty())
    {
        response.observation.completeness = Completeness::Partial;
        response.observation.limitations.push("At least one selected compiler query was unavailable; empty fields do not establish absence for that phase.".into());
    }
    if let Some(root) = root.to_str() {
        for body in &response.payload.bodies {
            response.next.push(NextAction::Command {argv:vec!["pse".into(),"-C".into(),root.into(),"inspect".into(),"compiler-facts".into(),response.payload.original_invocation.to_string(),"--body".into(),body.path.clone(),"--max-bytes".into(),budget.get().saturating_mul(2).to_string(),"--fact-focus".into(),focus.as_str().into()],reason:"focus on this exact original compiler body; provenance and whole-body boundaries remain guarded".into()});
        }
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiler_original_source_binding_retains_raw_utf8_and_refuses_tampering_or_partial_ranges() {
        let original = "\u{feff}pub fn target(){let text=\"é\";}\r\n";
        let mut facts = pb::CompilerFacts {
            compiler: "pinned".into(),
            source_files: vec![pb::CompilerSourceFile {
                id: 7,
                local_path: b"/fixture/lib.rs".to_vec(),
                display_name: "remapped/lib.rs".into(),
                original_source: original.as_bytes().to_vec(),
                original_sha256: super::super::digest(original.as_bytes()),
                normalized_sha256: super::super::digest(
                    "pub fn target(){let text=\"é\";}\n".as_bytes(),
                ),
                ..Default::default()
            }],
            bodies: vec![pb::CompilerBodyFact {
                path: "target".into(),
                source: Some(pb::CompilerSourceLocation {
                    file: "remapped/lib.rs".into(),
                    start: 0,
                    end: 29,
                    source_file: Some(7),
                    original_start: Some(3),
                    original_end: Some(original.len() as u32),
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(decode(&facts.encode_to_vec()).is_ok());
        facts.source_files[0].original_sha256 = "tampered".into();
        assert!(decode(&facts.encode_to_vec()).is_err());
        facts.source_files[0].original_sha256 = super::super::digest(original.as_bytes());
        facts.bodies[0]
            .source
            .as_option_mut()
            .unwrap()
            .original_start = Some(1);
        assert!(
            decode(&facts.encode_to_vec()).is_err(),
            "raw offsets must respect UTF-8 boundaries"
        );
        facts.bodies[0]
            .source
            .as_option_mut()
            .unwrap()
            .original_start = None;
        assert!(
            decode(&facts.encode_to_vec()).is_err(),
            "partial new bindings must fail closed"
        );
        let source = facts.bodies[0].source.as_option_mut().unwrap();
        source.source_file = None;
        source.original_end = None;
        facts.source_files.clear();
        assert!(
            decode(&facts.encode_to_vec()).is_ok(),
            "historical reports remain readable without asserting a join"
        );
    }
    #[test]
    fn compiler_fact_decoder_refuses_unknown_modes_dangling_saved_fields_and_empty_acquisition() {
        let mut facts = pb::CompilerFacts {
            compiler: "pinned-test-compiler".into(),
            ..Default::default()
        };
        assert!(decode(&facts.encode_to_vec()).is_err());
        facts.bodies.push(pb::CompilerBodyFact {
            path: "selected::closure".into(),
            captures: vec![pb::CompilerCaptureFact {
                place_display: "pair.0".into(),
                mode: 999.into(),
                ..Default::default()
            }],
            ..Default::default()
        });
        assert!(decode(&facts.encode_to_vec()).is_err());
        facts.bodies[0].captures[0].mode = pb::compiler_capture_fact::Mode::SHARED.into();
        let valid = decode(&facts.encode_to_vec()).unwrap();
        assert!(matches!(
            project(valid.bodies.into_iter().next().unwrap())
                .unwrap()
                .captures[0]
                .mode,
            CaptureMode::Shared
        ));
        facts.bodies[0]
            .variants
            .push(pb::CompilerSuspensionVariant {
                index: 3,
                fields: vec![7],
                ..Default::default()
            });
        assert!(decode(&facts.encode_to_vec()).is_err());
    }
    #[test]
    fn compiler_cleanup_projection_preserves_normal_unwind_and_phase_distinctions() {
        let mut body = pb::CompilerBodyFact {
            path: "partial".into(),
            ..Default::default()
        };
        body.cleanup.push(pb::CompilerCleanupFact {
            phase: "Runtime(PostCleanup)".into(),
            block: 2,
            kind: pb::compiler_cleanup_fact::Kind::DROP.into(),
            local: Some(0),
            normal_successors: vec![3],
            unwind: pb::compiler_cleanup_fact::Unwind::CLEANUP.into(),
            cleanup_block: Some(5),
            ..Default::default()
        });
        let projected = project(body.clone()).unwrap();
        assert_eq!(projected.cleanup[0].local, Some(0));
        assert!(matches!(projected.cleanup[0].terminator, Terminator::Drop));
        assert!(matches!(
            projected.cleanup[0].unwind,
            Unwind::Cleanup { block: 5 }
        ));
        assert_eq!(projected.cleanup[0].normal_successors, vec![3]);
        body.cleanup[0].unwind = pb::compiler_cleanup_fact::Unwind::CONTINUE.into();
        assert!(
            project(body).is_err(),
            "a cleanup target must not survive a continue edge"
        );
    }
}

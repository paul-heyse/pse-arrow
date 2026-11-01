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
}
fn source(value: Option<pb::CompilerSourceLocation>) -> Option<Source> {
    value.map(|s| Source {
        file: s.file,
        start: s.start,
        end: s.end,
        macro_expanded: s.macro_expanded,
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
    let mut paths = std::collections::BTreeSet::new();
    for body in &facts.bodies {
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
    let bytes = crate::test_inventory::read_private(Path::new(&file.path))?;
    if super::digest(&bytes) != file.sha256 {
        return Err(invalid("original compiler facts digest changed"));
    }
    let facts = validate_acquisition(&bytes, &original.payload.context)?;
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
        },
    );
    response.providers = original.providers;
    response.artifacts = original.artifacts;
    response.next = original.next;
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

// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit compiler-stage wrapper. Ordinary host construction never links these crates.
#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_span;

use buffa::Message as _;
use pse_host::code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb;
use pse_host::compiler_investigation::artifact::FactFocus;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::{
    mir,
    ty::{self, TyCtxt},
};
use sha2::{Digest as _, Sha256};
use std::{cell::RefCell, collections::BTreeMap, os::unix::ffi::OsStrExt as _};
use std::{io::Write as _, os::unix::fs::OpenOptionsExt as _, path::PathBuf, process::ExitCode};

struct Observe {
    selector: String,
    output: PathBuf,
    focus: FactFocus,
}
impl Callbacks for Observe {
    fn after_expansion<'tcx>(&mut self, _: &interface::Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        let mut selected = Vec::new();
        let sources = RefCell::new(BTreeMap::new());
        for def in tcx.hir_body_owners() {
            let path = tcx.def_path_str(def);
            if path != self.selector && !path.starts_with(&format!("{}::", self.selector)) {
                continue;
            }
            let supported = matches!(
                tcx.def_kind(def),
                rustc_hir::def::DefKind::Fn
                    | rustc_hir::def::DefKind::AssocFn
                    | rustc_hir::def::DefKind::Closure
                    | rustc_hir::def::DefKind::SyntheticCoroutineBody
            );
            let coroutine = supported && tcx.is_coroutine(def.to_def_id());
            let abi = if matches!(
                tcx.def_kind(def),
                rustc_hir::def::DefKind::Fn | rustc_hir::def::DefKind::AssocFn
            ) {
                tcx.fn_sig(def)
                    .instantiate_identity()
                    .skip_binder()
                    .abi()
                    .name()
                    .to_owned()
            } else {
                String::new()
            };
            let hir_id = tcx.local_def_id_to_hir_id(def);
            // def_span deliberately selects diagnostic headers. The complete HIR node
            // is the source interval of a body; synthetic owners have no authored interval.
            let body_source = if matches!(tcx.hir_node(hir_id), rustc_hir::Node::Synthetic) {
                None
            } else {
                source(tcx, tcx.hir_span_with_body(hir_id), &sources)
            };
            let mut body = pb::CompilerBodyFact {
                path,
                source: body_source.into(),
                coroutine,
                abi,
                panic_strategy: format!("{:?}", tcx.sess.panic_strategy()),
                ..Default::default()
            };
            if !supported {
                body.unavailable.push("typed compiler body queries support functions, closures and coroutines; const/static bodies require their CTFE query route".into());
                selected.push((def, supported, body));
                continue;
            }
            if self.focus.includes(FactFocus::Captures) && tcx.is_closure_like(def.to_def_id()) {
                let closure_ty = tcx.type_of(def).instantiate_identity().skip_normalization();
                let upvars = match closure_ty.kind() {
                    ty::Closure(_, args) => Some(args.as_closure().tupled_upvars_ty()),
                    ty::Coroutine(_, args) => Some(args.as_coroutine().tupled_upvars_ty()),
                    ty::CoroutineClosure(_, args) => {
                        Some(args.as_coroutine_closure().tupled_upvars_ty())
                    }
                    _ => None,
                };
                let stored = upvars.and_then(|ty| {
                    if let ty::Tuple(fields) = ty.kind() {
                        Some(fields)
                    } else {
                        None
                    }
                });
                for (index, capture) in tcx.closure_captures(def).iter().enumerate() {
                    use pb::compiler_capture_fact::Mode;
                    let mode = match capture.info.capture_kind {
                        ty::UpvarCapture::ByValue => Mode::MOVED,
                        ty::UpvarCapture::ByUse => Mode::USED,
                        ty::UpvarCapture::ByRef(ty::BorrowKind::Immutable) => Mode::SHARED,
                        ty::UpvarCapture::ByRef(ty::BorrowKind::UniqueImmutable) => {
                            Mode::UNIQUE_SHARED
                        }
                        ty::UpvarCapture::ByRef(ty::BorrowKind::Mutable) => Mode::MUTABLE,
                    };
                    body.captures.push(pb::CompilerCaptureFact {
                        place_display: capture.to_string(tcx),
                        mode: mode.into(),
                        source_type_display: capture.place.ty().to_string(),
                        stored_type_display: stored
                            .and_then(|fields| fields.get(index))
                            .map(ToString::to_string),
                        source: source(tcx, capture.get_capture_kind_span(tcx), &sources).into(),
                        ..Default::default()
                    });
                }
            }
            // Copy early observations before later queries consume their bodies.
            if self.focus.includes(FactFocus::Cleanup) {
                let early = tcx.mir_promoted(def).0;
                if !early.is_stolen() {
                    cleanup(tcx, &early.borrow(), &mut body.cleanup, &sources);
                } else {
                    body.unavailable
                        .push("analysis MIR was already consumed".into());
                }
            }
            selected.push((def, supported, body));
        }
        // Complete each earlier phase across all selected bodies before a later query can
        // consume a nested body's MIR while optimizing its parent.
        if self.focus.includes(FactFocus::Cleanup) {
            for (def, supported, body) in &mut selected {
                if !*supported {
                    continue;
                }
                let lowered = tcx.mir_drops_elaborated_and_const_checked(*def);
                if !lowered.is_stolen() {
                    cleanup(tcx, &lowered.borrow(), &mut body.cleanup, &sources);
                } else {
                    body.unavailable
                        .push("drop-elaborated MIR was already consumed".into());
                }
            }
        }
        if self.focus.includes(FactFocus::Suspension) {
            for (def, _, body) in &mut selected {
                if body.coroutine {
                    if let Some(layout) = tcx.mir_coroutine_witnesses(def.to_def_id()) {
                        for (index, field) in layout.field_tys.iter().enumerate() {
                            body.saved_fields.push(pb::CompilerSavedField {
                                index: index as u32,
                                type_display: field.ty.to_string(),
                                source_display: format!("{:?}", field.source_info.span),
                                source: source(tcx, field.source_info.span, &sources).into(),
                                ..Default::default()
                            });
                        }
                        for (index, fields) in layout.variant_fields.iter().enumerate() {
                            let location = layout.variant_source_info.iter().nth(index);
                            body.variants.push(pb::CompilerSuspensionVariant {
                                index: index as u32,
                                fields: fields.iter().map(|field| field.index() as u32).collect(),
                                source_display: location
                                    .map(|source| format!("{:?}", source.span))
                                    .unwrap_or_default(),
                                source: location
                                    .and_then(|location| source(tcx, location.span, &sources))
                                    .into(),
                                ..Default::default()
                            });
                        }
                    } else {
                        body.unavailable
                            .push("coroutine witness layout was unavailable".into());
                    }
                }
            }
        }
        if self.focus.includes(FactFocus::Cleanup) {
            for (def, supported, body) in &mut selected {
                if !*supported {
                    continue;
                }
                cleanup(tcx, tcx.optimized_mir(*def), &mut body.cleanup, &sources);
            }
        }
        let bodies = selected.into_iter().map(|(_, _, body)| body).collect();
        let facts = pb::CompilerFacts {
            compiler: rustc_interface::util::rustc_version_str()
                .unwrap_or("unknown")
                .into(),
            bodies,
            source_files: sources.into_inner().into_values().collect(),
            cfg: tcx
                .sess
                .config
                .iter()
                .map(|(key, value)| match value {
                    Some(value) => format!("{}={:?}", key, value.as_str()),
                    None => key.to_string(),
                })
                .collect(),
            edition: Some(tcx.sess.edition().to_string()),
            target: Some(tcx.sess.opts.target_triple.to_string()),
            ..Default::default()
        };
        // This file remains provisional until the outer owner verifies successful compilation.
        let retained = (|| -> std::io::Result<()> {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&self.output)?;
            file.write_all(&facts.encode_to_vec())?;
            file.sync_all()
        })();
        if let Err(error) = retained {
            tcx.dcx()
                .err(format!("cannot retain compiler observations: {error}"));
        }
        Compilation::Continue
    }
}
type SourceCatalog = RefCell<BTreeMap<u32, pb::CompilerSourceFile>>;
fn source_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn source(
    tcx: TyCtxt<'_>,
    span: rustc_span::Span,
    sources: &SourceCatalog,
) -> Option<pb::CompilerSourceLocation> {
    if span.is_dummy() {
        return None;
    }
    let map = tcx.sess.source_map();
    let lo = map.lookup_byte_offset(span.lo());
    let hi = map.lookup_byte_offset(span.hi());
    if lo.sf.start_pos != hi.sf.start_pos {
        return None;
    }
    let key = lo.sf.start_pos.0;
    if !sources.borrow().contains_key(&key)
        && let rustc_span::FileName::Real(name) = &lo.sf.name
        && let Some(path) = name.local_path()
    {
        let Ok(path) = path.canonicalize() else {
            return Some(pb::CompilerSourceLocation {
                file: lo.sf.name.prefer_remapped_unconditionally().to_string(),
                start: lo.pos.0,
                end: hi.pos.0,
                macro_expanded: span.from_expansion(),
                ..Default::default()
            });
        };
        if let Ok(original) = std::fs::read_to_string(&path)
            && lo.sf.src_hash.matches(&original)
            && let Some(normalized) = &lo.sf.src
        {
            sources.borrow_mut().insert(
                key,
                pb::CompilerSourceFile {
                    id: key,
                    local_path: path.as_os_str().as_bytes().to_vec(),
                    display_name: lo.sf.name.prefer_remapped_unconditionally().to_string(),
                    original_sha256: source_sha256(original.as_bytes()),
                    normalized_sha256: source_sha256(normalized.as_bytes()),
                    original_source: original.into_bytes(),
                    ..Default::default()
                },
            );
        }
    }
    let bound = sources.borrow().contains_key(&key);
    Some(pb::CompilerSourceLocation {
        file: lo.sf.name.prefer_remapped_unconditionally().to_string(),
        start: lo.pos.0,
        end: hi.pos.0,
        macro_expanded: span.from_expansion(),
        source_file: bound.then_some(key),
        original_start: bound.then(|| lo.sf.original_relative_byte_pos(span.lo()).0),
        original_end: bound.then(|| lo.sf.original_relative_byte_pos(span.hi()).0),
        ..Default::default()
    })
}
fn cleanup<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    output: &mut Vec<pb::CompilerCleanupFact>,
    sources: &SourceCatalog,
) {
    use pb::compiler_cleanup_fact::{Kind, Unwind};
    use rustc_mir_dataflow::{
        Analysis as _,
        impls::{MaybeInitializedPlaces, MaybeUninitializedPlaces},
        move_paths::{LookupResult, MoveData},
    };
    // Query the compiler's existing move-path analyses before lowering consumes this body.
    // No parent-path approximation or state inferred from pretty-printed MIR is exported.
    let moves = matches!(body.phase, mir::MirPhase::Analysis(_))
        .then(|| MoveData::gather_moves(body, tcx, |_| true));
    let mut initialized = moves.as_ref().map(|moves| {
        MaybeInitializedPlaces::new(tcx, body, moves)
            .iterate_to_fixpoint(tcx, body, None)
            .into_results_cursor(body)
    });
    let mut uninitialized = moves.as_ref().map(|moves| {
        MaybeUninitializedPlaces::new(tcx, body, moves)
            .iterate_to_fixpoint(tcx, body, None)
            .into_results_cursor(body)
    });
    for (block, data) in body.basic_blocks.iter_enumerated() {
        let Some(term) = &data.terminator else {
            continue;
        };
        let (kind, place, edge) = match &term.kind {
            mir::TerminatorKind::Drop { place, unwind, .. } => {
                (Kind::DROP, Some(*place), Some(*unwind))
            }
            mir::TerminatorKind::Call { unwind, .. } => (Kind::CALL, None, Some(*unwind)),
            mir::TerminatorKind::Assert { unwind, .. } => (Kind::ASSERT, None, Some(*unwind)),
            mir::TerminatorKind::Return => (Kind::RETURN, None, None),
            mir::TerminatorKind::UnwindResume => (Kind::UNWIND_RESUME, None, None),
            mir::TerminatorKind::UnwindTerminate(_) => (Kind::UNWIND_TERMINATE, None, None),
            _ => continue,
        };
        let (unwind, cleanup_block) = match edge {
            None => (Unwind::NO_UNWIND_EDGE, None),
            Some(mir::UnwindAction::Continue) => (Unwind::CONTINUE, None),
            Some(mir::UnwindAction::Unreachable) => (Unwind::UNREACHABLE, None),
            Some(mir::UnwindAction::Terminate(_)) => (Unwind::TERMINATE, None),
            Some(mir::UnwindAction::Cleanup(target)) => {
                (Unwind::CLEANUP, Some(target.index() as u32))
            }
        };
        let termination_reason = match &term.kind {
            mir::TerminatorKind::UnwindTerminate(reason) => Some(*reason),
            _ => {
                if let Some(mir::UnwindAction::Terminate(reason)) = edge {
                    Some(reason)
                } else {
                    None
                }
            }
        };
        let termination_cause = termination_reason.map(|reason| match reason {
            mir::UnwindTerminateReason::Abi => {
                pb::compiler_cleanup_fact::TerminationCause::ABI.into()
            }
            mir::UnwindTerminateReason::InCleanup => {
                pb::compiler_cleanup_fact::TerminationCause::IN_CLEANUP.into()
            }
        });
        let callee = if let mir::TerminatorKind::Call { func, .. } = &term.kind {
            let callable = func.ty(&body.local_decls, tcx);
            if matches!(callable.kind(), ty::FnDef(..) | ty::FnPtr(..)) {
                Some(pb::CompilerCallee {
                    abi: callable.fn_sig(tcx).skip_binder().abi().name().to_owned(),
                    declaration_path: if let ty::FnDef(def, _) = callable.kind() {
                        Some(tcx.def_path_str(*def))
                    } else {
                        None
                    },
                    ..Default::default()
                })
            } else {
                None
            }
        } else {
            None
        };
        let initialization = match (&moves, &mut initialized, &mut uninitialized, place) {
            (Some(moves), Some(initialized), Some(uninitialized), Some(place)) => {
                match moves.rev_lookup.find(place.as_ref()) {
                    LookupResult::Exact(path) => {
                        let location = mir::Location {
                            block,
                            statement_index: data.statements.len(),
                        };
                        initialized.seek_before_primary_effect(location);
                        uninitialized.seek_before_primary_effect(location);
                        Some(pb::CompilerInitialization {
                            may_initialized: initialized.get().contains(path),
                            may_uninitialized: uninitialized.get().contains(path),
                            ..Default::default()
                        })
                    }
                    LookupResult::Parent(_) => None,
                }
            }
            _ => None,
        };
        output.push(pb::CompilerCleanupFact {
            phase: format!("{:?}", body.phase),
            block: block.index() as u32,
            kind: kind.into(),
            local: place.map(|p| p.local.index() as u32),
            place_display: place.map(|p| format!("{p:?}")).unwrap_or_default(),
            normal_successors: term
                .successors()
                .filter(|b| Some(b.index() as u32) != cleanup_block)
                .map(|b| b.index() as u32)
                .collect(),
            unwind: unwind.into(),
            cleanup_block,
            source_display: format!("{:?}", term.source_info.span),
            source: source(tcx, term.source_info.span, sources).into(),
            is_cleanup: data.is_cleanup,
            initialization: initialization.into(),
            termination_cause,
            callee: callee.into(),
            ..Default::default()
        });
    }
}
fn main() -> std::io::Result<ExitCode> {
    let mut incoming = std::env::args_os();
    incoming.next();
    // Only Cargo's workspace-wrapper protocol is supported. Never delegate to ourselves.
    let Some(compiler) = incoming.next() else {
        writeln!(
            std::io::stderr().lock(),
            "compiler observation wrapper requires the original compiler executable"
        )?;
        return Ok(ExitCode::from(125));
    };
    if compiler.as_encoded_bytes().starts_with(b"-") {
        writeln!(
            std::io::stderr().lock(),
            "compiler observation wrapper requires Cargo's compiler executable argument"
        )?;
        return Ok(ExitCode::from(125));
    }
    let args = std::iter::once(compiler)
        .chain(incoming)
        .collect::<Vec<_>>();
    let selected = std::env::var_os("PSE_COMPILER_FACTS_ROOT")
        .and_then(|root| std::fs::canonicalize(root).ok());
    let matches_root = selected.as_ref().is_some_and(|root| {
        args.iter()
            .skip(1)
            .any(|arg| std::fs::canonicalize(arg).ok().as_ref() == Some(root))
    });
    if !matches_root {
        return Ok(std::process::Command::new(&args[0])
            .args(&args[1..])
            .status()
            .map(|status| ExitCode::from(status.code().unwrap_or(125) as u8))
            .unwrap_or(ExitCode::from(125)));
    }
    let Ok(args) = args
        .into_iter()
        .map(std::ffi::OsString::into_string)
        .collect::<Result<Vec<_>, _>>()
    else {
        writeln!(
            std::io::stderr().lock(),
            "selected rustc driver argument interface requires Unicode; native bytes were not converted lossily"
        )?;
        return Ok(ExitCode::from(125));
    };
    let Some(output) = std::env::var_os("PSE_COMPILER_FACTS_OUTPUT") else {
        return Ok(ExitCode::from(125));
    };
    let Ok(selector) = std::env::var("PSE_COMPILER_FACTS_BODY") else {
        return Ok(ExitCode::from(125));
    };
    let focus = match std::env::var("PSE_COMPILER_FACTS_FOCUS").as_deref() {
        Ok("all") => FactFocus::All,
        Ok("captures") => FactFocus::Captures,
        Ok("suspension") => FactFocus::Suspension,
        Ok("cleanup") => FactFocus::Cleanup,
        _ => return Ok(ExitCode::from(125)),
    };
    let mut callback = Observe {
        selector,
        output: output.into(),
        focus,
    };
    Ok(rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(&args, &mut callback)
    }))
}

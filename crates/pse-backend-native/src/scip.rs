// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "direct pinned SCIP 10.0.2 C boundary: one owned instance per attempt, checked return codes and contained callbacks"
)]
//! Direct SCIP 10.0.2 adapter over raw `scip-sys` (ADR-0105). One instance per attempt is
//! created, used and freed on the owning worker, on every path including unwinding; no
//! SCIP type leaves this module.
//!
//! The export maps the neutral program node for node onto SCIP expressions: columns and
//! auxiliaries become variables with their closed boxes, affine functions become linear
//! constraints and every other function a nonlinear constraint on its bounds. A nonlinear
//! objective is exported through an epigraph variable. The native model is read back and
//! evaluated against [`FactorableProgram::evaluate`] before any claim transfers.
use crate::{
    LimitKind, NativeStatus, OracleContract, ProblemError, Variable,
    execution::ScipSettings as Settings,
    solve::{
        Assurance, Backend, BoundSource, Candidate, CandidateKind, Compatibility, Controls, Event,
        Execution, GlobalEvidence, Metric, NativeTermination, OptionValue, Options, PrimalSource,
        Progress, ResolvedAccuracy, SolveIntent, SolveReport, Termination, WarmPayload,
        WarmStart,
    },
};
use pse_math::{
    binding::ObjectiveSense,
    factorable::{FactorableProgram, Fidelity, Node},
    normalization::Normalization,
};
use pse_model::generated::enums::ModelingVariableDomain;
use scip_sys as ffi;
use std::{
    ffi::{CStr, CString, c_char},
    ptr::{self, NonNull},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

/// SCIP release the bindings were generated from; the linked library must be exactly it.
pub const VERSION: (i32, i32, i32) = (10, 0, 2);
/// Header API version. SCIP has no run-time API query, so it is checked at build time.
pub const API_VERSION: u32 = 156;
const _: () = assert!(
    ffi::SCIP_VERSION_MAJOR == 10 && ffi::SCIP_VERSION_MINOR == 0 && ffi::SCIP_VERSION_PATCH == 2
);
const _: () = assert!(ffi::SCIP_APIVERSION == API_VERSION);
// `SCIP_Real` is `double`, `SCIP_Longint` is 64-bit and every index and count is a C `int`.
const _: unsafe extern "C" fn(*mut ffi::SCIP) -> f64 = ffi::SCIPinfinity;
const _: unsafe extern "C" fn(*mut ffi::SCIP) -> i64 = ffi::SCIPgetNTotalNodes;
const _: unsafe extern "C" fn(*mut ffi::SCIP) -> i32 = ffi::SCIPgetNVars;
const _: () = assert!(size_of::<std::ffi::c_int>() == 4);

/// Relative deviation within which the native model reads back as the exported program.
pub const READBACK_TOLERANCE: f64 = 1e-9;
/// SCIP_EVENTTYPE_* constants of `type_event.h`. They use `UINT64_C`, which bindgen does
/// not evaluate, so they are restated from the 10.0.2 header.
mod event {
    pub(super) const PRESOLVEROUND: u64 = 0x40000;
    pub(super) const NODEFOCUSED: u64 = 0x80000;
    /// `NODEFEASIBLE | NODEINFEASIBLE | NODEBRANCHED`.
    pub(super) const NODESOLVED: u64 = 0x100000 | 0x200000 | 0x400000;
    pub(super) const DUALBOUNDIMPROVED: u64 = 0x1000000;
    pub(super) const LPSOLVED: u64 = 0x4000000;
    pub(super) const BESTSOLFOUND: u64 = 0x10000000;
    pub(super) const SOLVING: u64 =
        NODEFOCUSED | NODESOLVED | LPSOLVED | BESTSOLFOUND | DUALBOUNDIMPROVED;
}

/// Linked library facts, checked at run time against the bindings (ADR-0105 §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Abi {
    /// `SCIPmajorVersion`, `SCIPminorVersion` and `SCIPtechVersion` of the linked library.
    pub library: (i32, i32, i32),
    /// Header API version of the bindings.
    pub api: u32,
}
/// Check the linked library against the bindings.
///
/// # Errors
/// The linked SCIP differs from the 10.0.2 headers the bindings were generated from.
pub fn abi() -> Result<Abi, ProblemError> {
    // SAFETY: a query of static library data with no arguments.
    let major = unsafe { ffi::SCIPmajorVersion() };
    // SAFETY: as above.
    let minor = unsafe { ffi::SCIPminorVersion() };
    // SAFETY: as above.
    let tech = unsafe { ffi::SCIPtechVersion() };
    let library = (major, minor, tech);
    if library != VERSION {
        return Err(ProblemError::Unsupported(format!(
            "linked SCIP {major}.{minor}.{tech} differs from the 10.0.2 bindings"
        )));
    }
    Ok(Abi {
        library,
        api: API_VERSION,
    })
}

fn retcode_name(code: ffi::SCIP_RETCODE) -> &'static str {
    match code {
        ffi::SCIP_Retcode_SCIP_OKAY => "SCIP_OKAY",
        ffi::SCIP_Retcode_SCIP_ERROR => "SCIP_ERROR",
        ffi::SCIP_Retcode_SCIP_NOMEMORY => "SCIP_NOMEMORY",
        ffi::SCIP_Retcode_SCIP_READERROR => "SCIP_READERROR",
        ffi::SCIP_Retcode_SCIP_WRITEERROR => "SCIP_WRITEERROR",
        ffi::SCIP_Retcode_SCIP_NOFILE => "SCIP_NOFILE",
        ffi::SCIP_Retcode_SCIP_FILECREATEERROR => "SCIP_FILECREATEERROR",
        ffi::SCIP_Retcode_SCIP_LPERROR => "SCIP_LPERROR",
        ffi::SCIP_Retcode_SCIP_NOPROBLEM => "SCIP_NOPROBLEM",
        ffi::SCIP_Retcode_SCIP_INVALIDCALL => "SCIP_INVALIDCALL",
        ffi::SCIP_Retcode_SCIP_INVALIDDATA => "SCIP_INVALIDDATA",
        ffi::SCIP_Retcode_SCIP_INVALIDRESULT => "SCIP_INVALIDRESULT",
        ffi::SCIP_Retcode_SCIP_PLUGINNOTFOUND => "SCIP_PLUGINNOTFOUND",
        ffi::SCIP_Retcode_SCIP_PARAMETERUNKNOWN => "SCIP_PARAMETERUNKNOWN",
        ffi::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE => "SCIP_PARAMETERWRONGTYPE",
        ffi::SCIP_Retcode_SCIP_PARAMETERWRONGVAL => "SCIP_PARAMETERWRONGVAL",
        ffi::SCIP_Retcode_SCIP_KEYALREADYEXISTING => "SCIP_KEYALREADYEXISTING",
        ffi::SCIP_Retcode_SCIP_MAXDEPTHLEVEL => "SCIP_MAXDEPTHLEVEL",
        ffi::SCIP_Retcode_SCIP_BRANCHERROR => "SCIP_BRANCHERROR",
        ffi::SCIP_Retcode_SCIP_NOTIMPLEMENTED => "SCIP_NOTIMPLEMENTED",
        _ => "SCIP_UNKNOWN_RETCODE",
    }
}
/// Classify a native return code, keeping its identity.
fn check(code: ffi::SCIP_RETCODE, operation: &str) -> Result<(), ProblemError> {
    if code == ffi::SCIP_Retcode_SCIP_OKAY {
        return Ok(());
    }
    let status = NativeStatus {
        backend: Backend::Scip,
        code: i64::from(code),
        name: retcode_name(code).into(),
    };
    Err(match code {
        ffi::SCIP_Retcode_SCIP_NOMEMORY => ProblemError::Limit {
            kind: LimitKind::Memory,
            detail: format!("{operation} [{status}]"),
        },
        ffi::SCIP_Retcode_SCIP_PARAMETERUNKNOWN
        | ffi::SCIP_Retcode_SCIP_PARAMETERWRONGTYPE
        | ffi::SCIP_Retcode_SCIP_PARAMETERWRONGVAL
        | ffi::SCIP_Retcode_SCIP_INVALIDDATA => {
            ProblemError::Contract(format!("{operation} [{status}]"))
        }
        ffi::SCIP_Retcode_SCIP_INVALIDCALL | ffi::SCIP_Retcode_SCIP_NOPROBLEM => {
            ProblemError::Internal(format!("{operation} [{status}]"))
        }
        _ => ProblemError::Numerical {
            status: Some(status),
            detail: operation.into(),
        },
    })
}
/// One checked native call whose arguments are pointers owned by a live instance on its
/// owning thread, or caller storage that outlives the call.
macro_rules! native {
    ($operation:literal, $call:expr) => {{
        // SAFETY: SCIP is called on the owning worker with pointers owned by the live
        // instance, or with caller storage that outlives the call.
        let code = unsafe { $call };
        check(code, $operation)
    }};
}
fn cstring(s: &str) -> Result<CString, ProblemError> {
    CString::new(s).map_err(|_| ProblemError::Contract("NUL in SCIP name or option".into()))
}

/// Every raw `SCIPgetStatus` value of SCIP 10.0.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// `SCIP_STATUS_UNKNOWN`.
    Unknown,
    /// `SCIP_STATUS_OPTIMAL`.
    Optimal,
    /// `SCIP_STATUS_INFEASIBLE`.
    Infeasible,
    /// `SCIP_STATUS_UNBOUNDED`.
    Unbounded,
    /// `SCIP_STATUS_INFORUNBD`.
    InfeasibleOrUnbounded,
    /// `SCIP_STATUS_USERINTERRUPT`.
    UserInterrupt,
    /// `SCIP_STATUS_TERMINATE`.
    Terminate,
    /// `SCIP_STATUS_NODELIMIT`.
    NodeLimit,
    /// `SCIP_STATUS_TOTALNODELIMIT`.
    TotalNodeLimit,
    /// `SCIP_STATUS_STALLNODELIMIT`.
    StallNodeLimit,
    /// `SCIP_STATUS_TIMELIMIT`.
    TimeLimit,
    /// `SCIP_STATUS_MEMLIMIT`.
    MemoryLimit,
    /// `SCIP_STATUS_GAPLIMIT`.
    GapLimit,
    /// `SCIP_STATUS_PRIMALLIMIT`.
    PrimalLimit,
    /// `SCIP_STATUS_DUALLIMIT`.
    DualLimit,
    /// `SCIP_STATUS_SOLLIMIT`.
    SolutionLimit,
    /// `SCIP_STATUS_BESTSOLLIMIT`.
    BestSolutionLimit,
    /// `SCIP_STATUS_RESTARTLIMIT`.
    RestartLimit,
}
impl Status {
    /// Every status, in raw-code order.
    pub const ALL: [Self; 18] = [
        Self::Unknown,
        Self::Optimal,
        Self::Infeasible,
        Self::Unbounded,
        Self::InfeasibleOrUnbounded,
        Self::UserInterrupt,
        Self::Terminate,
        Self::NodeLimit,
        Self::TotalNodeLimit,
        Self::StallNodeLimit,
        Self::TimeLimit,
        Self::MemoryLimit,
        Self::GapLimit,
        Self::PrimalLimit,
        Self::DualLimit,
        Self::SolutionLimit,
        Self::BestSolutionLimit,
        Self::RestartLimit,
    ];
    /// The raw native value.
    pub const fn raw(self) -> ffi::SCIP_STATUS {
        match self {
            Self::Unknown => ffi::SCIP_Status_SCIP_STATUS_UNKNOWN,
            Self::Optimal => ffi::SCIP_Status_SCIP_STATUS_OPTIMAL,
            Self::Infeasible => ffi::SCIP_Status_SCIP_STATUS_INFEASIBLE,
            Self::Unbounded => ffi::SCIP_Status_SCIP_STATUS_UNBOUNDED,
            Self::InfeasibleOrUnbounded => ffi::SCIP_Status_SCIP_STATUS_INFORUNBD,
            Self::UserInterrupt => ffi::SCIP_Status_SCIP_STATUS_USERINTERRUPT,
            Self::Terminate => ffi::SCIP_Status_SCIP_STATUS_TERMINATE,
            Self::NodeLimit => ffi::SCIP_Status_SCIP_STATUS_NODELIMIT,
            Self::TotalNodeLimit => ffi::SCIP_Status_SCIP_STATUS_TOTALNODELIMIT,
            Self::StallNodeLimit => ffi::SCIP_Status_SCIP_STATUS_STALLNODELIMIT,
            Self::TimeLimit => ffi::SCIP_Status_SCIP_STATUS_TIMELIMIT,
            Self::MemoryLimit => ffi::SCIP_Status_SCIP_STATUS_MEMLIMIT,
            Self::GapLimit => ffi::SCIP_Status_SCIP_STATUS_GAPLIMIT,
            Self::PrimalLimit => ffi::SCIP_Status_SCIP_STATUS_PRIMALLIMIT,
            Self::DualLimit => ffi::SCIP_Status_SCIP_STATUS_DUALLIMIT,
            Self::SolutionLimit => ffi::SCIP_Status_SCIP_STATUS_SOLLIMIT,
            Self::BestSolutionLimit => ffi::SCIP_Status_SCIP_STATUS_BESTSOLLIMIT,
            Self::RestartLimit => ffi::SCIP_Status_SCIP_STATUS_RESTARTLIMIT,
        }
    }
    /// The status of a raw value; `None` outside the 10.0.2 ABI.
    pub fn from_raw(raw: ffi::SCIP_STATUS) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.raw() == raw)
    }
    /// Native symbolic name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "SCIP_STATUS_UNKNOWN",
            Self::Optimal => "SCIP_STATUS_OPTIMAL",
            Self::Infeasible => "SCIP_STATUS_INFEASIBLE",
            Self::Unbounded => "SCIP_STATUS_UNBOUNDED",
            Self::InfeasibleOrUnbounded => "SCIP_STATUS_INFORUNBD",
            Self::UserInterrupt => "SCIP_STATUS_USERINTERRUPT",
            Self::Terminate => "SCIP_STATUS_TERMINATE",
            Self::NodeLimit => "SCIP_STATUS_NODELIMIT",
            Self::TotalNodeLimit => "SCIP_STATUS_TOTALNODELIMIT",
            Self::StallNodeLimit => "SCIP_STATUS_STALLNODELIMIT",
            Self::TimeLimit => "SCIP_STATUS_TIMELIMIT",
            Self::MemoryLimit => "SCIP_STATUS_MEMLIMIT",
            Self::GapLimit => "SCIP_STATUS_GAPLIMIT",
            Self::PrimalLimit => "SCIP_STATUS_PRIMALLIMIT",
            Self::DualLimit => "SCIP_STATUS_DUALLIMIT",
            Self::SolutionLimit => "SCIP_STATUS_SOLLIMIT",
            Self::BestSolutionLimit => "SCIP_STATUS_BESTSOLLIMIT",
            Self::RestartLimit => "SCIP_STATUS_RESTARTLIMIT",
        }
    }
}
/// The one status map: raw status to the shared category and the native assurance, which
/// original-coordinate qualification then constrains. The requested gap limit is the
/// successful stop; every other limit leaves the bound unestablished.
pub fn termination(status: Status) -> NativeTermination {
    let (category, assurance) = match status {
        Status::Optimal | Status::GapLimit => (Termination::Success, Assurance::GlobalBound),
        Status::Infeasible => (Termination::Infeasible, Assurance::ProvenInfeasible),
        Status::Unbounded => (Termination::Unbounded, Assurance::None),
        Status::InfeasibleOrUnbounded => (Termination::InfeasibleOrUnbounded, Assurance::None),
        Status::UserInterrupt | Status::Terminate => (Termination::Cancelled, Assurance::None),
        Status::NodeLimit
        | Status::TotalNodeLimit
        | Status::StallNodeLimit
        | Status::RestartLimit => (Termination::Limit, Assurance::None),
        Status::TimeLimit => (Termination::TimeLimit, Assurance::None),
        Status::MemoryLimit => (Termination::ResourceExhausted, Assurance::None),
        Status::PrimalLimit | Status::DualLimit => (Termination::ObjectiveLimit, Assurance::None),
        Status::SolutionLimit | Status::BestSolutionLimit => {
            (Termination::SolutionLimit, Assurance::None)
        }
        Status::Unknown => (Termination::Inconclusive, Assurance::None),
    };
    NativeTermination {
        code: i64::from(status.raw()),
        name: status.name().into(),
        message: None,
        category,
        assurance,
    }
}

/// What the event handler observes during one solve; owned by its instance.
#[derive(Debug)]
struct Watch {
    cancel: Arc<AtomicBool>,
    progress: Arc<Progress>,
    started: Instant,
    events: u64,
    interrupted: bool,
    failed: bool,
}
fn watch<'a>(eventhdlr: *mut ffi::SCIP_EVENTHDLR) -> Option<&'a mut Watch> {
    // SAFETY: SCIP returns the data pointer this module registered for the handler.
    let data = unsafe { ffi::SCIPeventhdlrGetData(eventhdlr) };
    // SAFETY: the pointer is the instance's boxed `Watch`, freed only after `SCIPfree`,
    // and SCIP calls handlers sequentially on the owning thread.
    unsafe { data.cast::<Watch>().as_mut() }
}
/// Contain a panic in a native callback as a SCIP error.
fn contained(work: impl FnOnce() -> ffi::SCIP_RETCODE) -> ffi::SCIP_RETCODE {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
        .unwrap_or(ffi::SCIP_Retcode_SCIP_ERROR)
}
unsafe extern "C" fn watch_exec(
    scip: *mut ffi::SCIP,
    eventhdlr: *mut ffi::SCIP_EVENTHDLR,
    event: *mut ffi::SCIP_EVENT,
    _data: *mut ffi::SCIP_EVENTDATA,
) -> ffi::SCIP_RETCODE {
    contained(|| {
        let Some(watch) = watch(eventhdlr) else {
            return ffi::SCIP_Retcode_SCIP_INVALIDDATA;
        };
        watch.events = watch.events.saturating_add(1);
        // SAFETY: SCIP passes the live event being processed.
        let kind = unsafe { ffi::SCIPeventGetType(event) };
        if kind & (event::BESTSOLFOUND | event::DUALBOUNDIMPROVED) != 0 {
            // SAFETY: bound queries are valid in every stage that issues these events.
            let primal = unsafe { ffi::SCIPgetPrimalbound(scip) };
            // SAFETY: as above.
            let dual = unsafe { ffi::SCIPgetDualbound(scip) };
            watch.progress.push(Event {
                phase: "scip.bound".into(),
                elapsed: watch.started.elapsed(),
                values: [
                    ("primal_bound".into(), Metric::Real(primal)),
                    ("dual_bound".into(), Metric::Real(dual)),
                ]
                .into(),
            });
        }
        if watch.cancel.load(Ordering::Acquire) && !watch.interrupted {
            watch.interrupted = true;
            // SAFETY: interruption is admitted in every stage that issues these events.
            let code = unsafe { ffi::SCIPinterruptSolve(scip) };
            watch.failed |= code != ffi::SCIP_Retcode_SCIP_OKAY;
            return code;
        }
        ffi::SCIP_Retcode_SCIP_OKAY
    })
}
unsafe extern "C" fn watch_init(
    scip: *mut ffi::SCIP,
    eventhdlr: *mut ffi::SCIP_EVENTHDLR,
) -> ffi::SCIP_RETCODE {
    // SAFETY: SCIP calls the handler's init after transforming the problem, where global
    // event catching is admitted.
    contained(|| unsafe {
        ffi::SCIPcatchEvent(
            scip,
            event::PRESOLVEROUND,
            eventhdlr,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    })
}
unsafe extern "C" fn watch_exit(
    scip: *mut ffi::SCIP,
    eventhdlr: *mut ffi::SCIP_EVENTHDLR,
) -> ffi::SCIP_RETCODE {
    // SAFETY: drops the catch made in `watch_init`; -1 lets SCIP find its filter position.
    contained(|| unsafe {
        ffi::SCIPdropEvent(scip, event::PRESOLVEROUND, eventhdlr, ptr::null_mut(), -1)
    })
}
unsafe extern "C" fn watch_initsol(
    scip: *mut ffi::SCIP,
    eventhdlr: *mut ffi::SCIP_EVENTHDLR,
) -> ffi::SCIP_RETCODE {
    // SAFETY: SCIP calls initsol before branch-and-bound, where node events are caught.
    contained(|| unsafe {
        ffi::SCIPcatchEvent(
            scip,
            event::SOLVING,
            eventhdlr,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    })
}
unsafe extern "C" fn watch_exitsol(
    scip: *mut ffi::SCIP,
    eventhdlr: *mut ffi::SCIP_EVENTHDLR,
) -> ffi::SCIP_RETCODE {
    // SAFETY: drops the catch made in `watch_initsol`.
    contained(|| unsafe {
        ffi::SCIPdropEvent(scip, event::SOLVING, eventhdlr, ptr::null_mut(), -1)
    })
}

/// One SCIP instance and everything it owns, released and freed on drop, including
/// during unwinding.
pub(crate) struct Instance {
    scip: NonNull<ffi::SCIP>,
    vars: Vec<*mut ffi::SCIP_VAR>,
    conss: Vec<*mut ffi::SCIP_CONS>,
    watch: NonNull<Watch>,
    infinity: f64,
}
impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScipInstance")
            .field("vars", &self.vars.len())
            .field("conss", &self.conss.len())
            .finish_non_exhaustive()
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        let scip = self.scip.as_ptr();
        for var in &mut self.vars {
            // SAFETY: each held variable carries one reference this instance captured.
            let _ = unsafe { ffi::SCIPreleaseVar(scip, var) };
        }
        for cons in &mut self.conss {
            // SAFETY: each held constraint carries one reference this instance captured.
            let _ = unsafe { ffi::SCIPreleaseCons(scip, cons) };
        }
        let mut raw = scip;
        // SAFETY: the instance is freed once, on its owning thread, after its references.
        let _ = unsafe { ffi::SCIPfree(&mut raw) };
        // SAFETY: SCIP no longer references the handler data after `SCIPfree`.
        drop(unsafe { Box::from_raw(self.watch.as_ptr()) });
    }
}
impl Instance {
    /// A quiet instance with default plugins, the cancellation handler and an empty problem.
    pub(crate) fn new(execution: &Execution) -> Result<Self, ProblemError> {
        let mut raw = ptr::null_mut();
        native!("SCIPcreate", ffi::SCIPcreate(&mut raw))?;
        let scip = NonNull::new(raw).ok_or_else(|| ProblemError::memory("SCIPcreate"))?;
        let watch = Box::new(Watch {
            cancel: execution.cancel.clone(),
            progress: execution.progress.clone(),
            started: execution.started,
            events: 0,
            interrupted: false,
            failed: false,
        });
        let mut instance = Self {
            scip,
            vars: vec![],
            conss: vec![],
            watch: NonNull::from(Box::leak(watch)),
            infinity: 0.0,
        };
        let s = instance.scip.as_ptr();
        native!(
            "SCIPincludeDefaultPlugins",
            ffi::SCIPincludeDefaultPlugins(s)
        )?;
        // SAFETY: silences the instance's own message handler.
        unsafe { ffi::SCIPsetMessagehdlrQuiet(s, 1) };
        let name = cstring("pse_watch")?;
        let description = cstring("attempt cancellation and bound progress")?;
        let mut hdlr = ptr::null_mut();
        native!(
            "SCIPincludeEventhdlrBasic",
            ffi::SCIPincludeEventhdlrBasic(
                s,
                &mut hdlr,
                name.as_ptr(),
                description.as_ptr(),
                Some(watch_exec),
                instance.watch.as_ptr().cast(),
            )
        )?;
        native!(
            "SCIPsetEventhdlrInit",
            ffi::SCIPsetEventhdlrInit(s, hdlr, Some(watch_init))
        )?;
        native!(
            "SCIPsetEventhdlrExit",
            ffi::SCIPsetEventhdlrExit(s, hdlr, Some(watch_exit))
        )?;
        native!(
            "SCIPsetEventhdlrInitsol",
            ffi::SCIPsetEventhdlrInitsol(s, hdlr, Some(watch_initsol))
        )?;
        native!(
            "SCIPsetEventhdlrExitsol",
            ffi::SCIPsetEventhdlrExitsol(s, hdlr, Some(watch_exitsol))
        )?;
        let problem = cstring("pse_factorable")?;
        native!(
            "SCIPcreateProbBasic",
            ffi::SCIPcreateProbBasic(s, problem.as_ptr())
        )?;
        // SAFETY: a value query on the live instance.
        instance.infinity = unsafe { ffi::SCIPinfinity(s) };
        Ok(instance)
    }
    fn ptr(&self) -> *mut ffi::SCIP {
        self.scip.as_ptr()
    }
    fn observed(&self) -> &Watch {
        // SAFETY: the handler data outlives the instance's borrows and is written only
        // inside `SCIPsolve`, which has returned whenever this is read.
        unsafe { self.watch.as_ref() }
    }
    fn native(&self, x: f64) -> Result<f64, ProblemError> {
        if x.is_nan() || x.is_finite() && x.abs() >= self.infinity {
            return Err(ProblemError::Unsupported(
                "finite value reaches SCIP's infinity threshold".into(),
            ));
        }
        Ok(x.clamp(-self.infinity, self.infinity))
    }
    fn finite(&self, x: f64) -> Option<f64> {
        (x.is_finite() && x.abs() < self.infinity).then_some(x)
    }
    fn set_bool(&self, key: &str, value: bool) -> Result<(), ProblemError> {
        let k = cstring(key)?;
        native!(
            "SCIPsetBoolParam",
            ffi::SCIPsetBoolParam(self.ptr(), k.as_ptr(), u32::from(value))
        )
    }
    fn set_int(&self, key: &str, value: i32) -> Result<(), ProblemError> {
        let k = cstring(key)?;
        native!(
            "SCIPsetIntParam",
            ffi::SCIPsetIntParam(self.ptr(), k.as_ptr(), value)
        )
    }
    fn set_longint(&self, key: &str, value: i64) -> Result<(), ProblemError> {
        let k = cstring(key)?;
        native!(
            "SCIPsetLongintParam",
            ffi::SCIPsetLongintParam(self.ptr(), k.as_ptr(), value)
        )
    }
    fn set_real(&self, key: &str, value: f64) -> Result<(), ProblemError> {
        let k = cstring(key)?;
        native!(
            "SCIPsetRealParam",
            ffi::SCIPsetRealParam(self.ptr(), k.as_ptr(), value)
        )
    }
    fn set_string(&self, key: &str, value: &str) -> Result<(), ProblemError> {
        let k = cstring(key)?;
        let v = cstring(value)?;
        native!(
            "SCIPsetStringParam",
            ffi::SCIPsetStringParam(self.ptr(), k.as_ptr(), v.as_ptr())
        )
    }
    /// Read one native parameter back in its native type.
    pub(crate) fn option(
        &self,
        key: &str,
        kind: &OptionValue,
    ) -> Result<OptionValue, ProblemError> {
        let k = cstring(key)?;
        let s = self.ptr();
        Ok(match kind {
            OptionValue::Bool(_) => {
                let mut v = 0;
                native!(
                    "SCIPgetBoolParam",
                    ffi::SCIPgetBoolParam(s, k.as_ptr(), &mut v)
                )?;
                OptionValue::Bool(v != 0)
            }
            OptionValue::Integer(_) => {
                let mut v = 0;
                native!(
                    "SCIPgetIntParam",
                    ffi::SCIPgetIntParam(s, k.as_ptr(), &mut v)
                )?;
                OptionValue::Integer(v)
            }
            OptionValue::Real(_) => {
                let mut v = 0.0;
                native!(
                    "SCIPgetRealParam",
                    ffi::SCIPgetRealParam(s, k.as_ptr(), &mut v)
                )?;
                OptionValue::Real(v)
            }
            OptionValue::Text(_) => {
                let mut v: *mut c_char = ptr::null_mut();
                native!(
                    "SCIPgetStringParam",
                    ffi::SCIPgetStringParam(s, k.as_ptr(), &mut v)
                )?;
                if v.is_null() {
                    return Err(ProblemError::Internal(format!("SCIP {key} has no value")));
                }
                // SAFETY: SCIP returns its own NUL-terminated parameter storage, valid
                // until the parameter changes; it is copied at once.
                let text = unsafe { CStr::from_ptr(v) };
                OptionValue::Text(text.to_string_lossy().into_owned())
            }
        })
    }
}

/// Reserved native options: set from typed settings and shared controls, never by
/// free-form options. Concurrent and parallel solving need admitted permits (Plan 22 G7).
const RESERVED: [&str; 12] = [
    "misc/catchctrlc",
    "limits/time",
    "limits/memory",
    "limits/gap",
    "limits/absgap",
    "limits/totalnodes",
    "numerics/feastol",
    "randomization/randomseedshift",
    "lp/threads",
    "nlpi/ipopt/linear_solver",
    "nlpi/ipopt/hsllib",
    "nlpi/ipopt/pardisolib",
];
const RESERVED_PREFIXES: [&str; 2] = ["parallel/", "concurrent/"];
/// The native feasibility tolerance derived from the resolved normalized budget, within
/// SCIP's numerically safe range.
fn feasibility(accuracy: &ResolvedAccuracy) -> f64 {
    accuracy.feasibility.clamp(1e-9, 1e-6)
}
/// Reserved options from typed settings and controls, then admitted free-form options;
/// returns the effective values read back from the instance.
pub(crate) fn configure(
    instance: &Instance,
    settings: &Settings,
    controls: &Controls,
    accuracy: &ResolvedAccuracy,
    execution: &Execution,
    gap_absolute: f64,
) -> Result<Options, ProblemError> {
    settings.admit()?;
    if let Some(key) = controls.options.keys().find(|k| {
        RESERVED.contains(&k.as_str()) || RESERVED_PREFIXES.iter().any(|p| k.starts_with(p))
    }) {
        return Err(ProblemError::Contract(format!(
            "native option {key} conflicts with typed controls"
        )));
    }
    let memory = execution.memory.ok_or_else(|| {
        ProblemError::Contract("SCIP needs the worker's admitted foreign allowance".into())
    })?;
    let remaining = execution
        .time_limit
        .saturating_sub(execution.started.elapsed())
        .as_secs_f64();
    let mut reserved: Vec<(&str, OptionValue)> = vec![
        ("misc/catchctrlc", OptionValue::Bool(false)),
        ("limits/time", OptionValue::Real(remaining)),
        (
            "limits/memory",
            OptionValue::Real((memory / (1 << 20)).max(1) as f64),
        ),
        (
            "limits/gap",
            OptionValue::Real(accuracy.mip_relative_gap),
        ),
        ("limits/absgap", OptionValue::Real(gap_absolute)),
        ("numerics/feastol", OptionValue::Real(feasibility(accuracy))),
        (
            "randomization/randomseedshift",
            OptionValue::Integer(i32::from(settings.seed)),
        ),
        ("lp/threads", OptionValue::Integer(1)),
        (
            "nlpi/ipopt/linear_solver",
            OptionValue::Text(settings.nlp_linear_solver.as_str().into()),
        ),
    ];
    for (key, value) in &reserved {
        set(instance, key, value)?;
    }
    let nodes = settings.nodes.map_or(-1, i64::from);
    instance.set_longint("limits/totalnodes", nodes)?;
    instance.set_int("display/verblevel", 0)?;
    for (key, value) in &controls.options {
        set(instance, key, value)?;
    }
    reserved.extend(
        controls
            .options
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone())),
    );
    let mut effective = Options::new();
    for (key, kind) in &reserved {
        effective.insert((*key).into(), instance.option(key, kind)?);
    }
    effective.insert(
        "limits/totalnodes".into(),
        OptionValue::Text(nodes.to_string()),
    );
    Ok(effective)
}
fn set(instance: &Instance, key: &str, value: &OptionValue) -> Result<(), ProblemError> {
    match value {
        OptionValue::Bool(v) => instance.set_bool(key, *v),
        OptionValue::Integer(v) => instance.set_int(key, *v),
        OptionValue::Real(v) => instance.set_real(key, *v),
        OptionValue::Text(v) => instance.set_string(key, v),
    }
}

/// One exported constraint, retained for readback.
#[derive(Debug)]
struct Exported {
    cons: *mut ffi::SCIP_CONS,
    node: pse_math::factorable::NodeId,
    lower: f64,
    upper: f64,
    /// Linear constraints hold the constant their sides were shifted by; nonlinear
    /// constraints hold an expression evaluated whole.
    linear: Option<f64>,
    /// The epigraph `f - z` of a nonlinear objective.
    epigraph: bool,
}
/// The native model of one plan.
#[derive(Debug)]
pub(crate) struct Export {
    /// Variables of the columns, then the auxiliaries.
    coordinates: Vec<*mut ffi::SCIP_VAR>,
    epigraph: Option<*mut ffi::SCIP_VAR>,
    constraints: Vec<Exported>,
    /// Affine objective form, exported through variable objective coefficients.
    objective: Option<pse_math::factorable::NodeId>,
    /// Constraint counts: linear and nonlinear.
    pub linear: usize,
    pub nonlinear: usize,
}
fn vartype(domain: ModelingVariableDomain) -> ffi::SCIP_VARTYPE {
    match domain {
        ModelingVariableDomain::Binary => ffi::SCIP_Vartype_SCIP_VARTYPE_BINARY,
        ModelingVariableDomain::Integer | ModelingVariableDomain::Semiinteger => {
            ffi::SCIP_Vartype_SCIP_VARTYPE_INTEGER
        }
        ModelingVariableDomain::Continuous | ModelingVariableDomain::Semicontinuous => {
            ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS
        }
    }
}
impl Instance {
    fn variable(
        &mut self,
        name: &str,
        (lower, upper): (f64, f64),
        objective: f64,
        kind: ffi::SCIP_VARTYPE,
    ) -> Result<*mut ffi::SCIP_VAR, ProblemError> {
        let n = cstring(name)?;
        let (lower, upper) = (self.native(lower)?, self.native(upper)?);
        let mut var = ptr::null_mut();
        native!(
            "SCIPcreateVarBasic",
            ffi::SCIPcreateVarBasic(
                self.ptr(),
                &mut var,
                n.as_ptr(),
                lower,
                upper,
                objective,
                kind
            )
        )?;
        self.vars.push(var);
        native!("SCIPaddVar", ffi::SCIPaddVar(self.ptr(), var))?;
        Ok(var)
    }
    fn hold(&mut self, cons: *mut ffi::SCIP_CONS) -> Result<(), ProblemError> {
        self.conss.push(cons);
        native!("SCIPaddCons", ffi::SCIPaddCons(self.ptr(), cons))
    }
}
/// Releases created expressions; constraints hold their own references. It never
/// outlives the instance that created the expressions.
struct Expressions {
    scip: *mut ffi::SCIP,
    nodes: Vec<*mut ffi::SCIP_EXPR>,
}
impl Drop for Expressions {
    fn drop(&mut self) {
        for expr in &mut self.nodes {
            if !expr.is_null() {
                // SAFETY: each created expression carries one reference owned here, and
                // the instance outlives this guard.
                let _ = unsafe { ffi::SCIPreleaseExpr(self.scip, expr) };
            }
        }
    }
}
/// Map every needed node onto its SCIP expression, children first.
fn expressions(
    instance: &Instance,
    program: &FactorableProgram,
    coordinates: &[*mut ffi::SCIP_VAR],
    needed: &[bool],
) -> Result<Expressions, ProblemError> {
    let s = instance.ptr();
    let columns = program.variables.len();
    let mut out = Expressions {
        scip: s,
        nodes: vec![ptr::null_mut(); program.nodes.len()],
    };
    for (i, node) in program.nodes.iter().enumerate() {
        if !needed[i] {
            continue;
        }
        let mut expr = ptr::null_mut();
        let child = |c: &usize| out.nodes[*c];
        match node {
            Node::Var(c) => native!(
                "SCIPcreateExprVar",
                ffi::SCIPcreateExprVar(s, &mut expr, coordinates[*c], None, ptr::null_mut())
            )?,
            Node::Aux(k) => native!(
                "SCIPcreateExprVar",
                ffi::SCIPcreateExprVar(
                    s,
                    &mut expr,
                    coordinates[columns + k],
                    None,
                    ptr::null_mut()
                )
            )?,
            Node::Const(c) => native!(
                "SCIPcreateExprValue",
                ffi::SCIPcreateExprValue(s, &mut expr, c.value(), None, ptr::null_mut())
            )?,
            Node::Sum(c) => {
                let mut children: Vec<_> = c.iter().map(child).collect();
                let mut ones = vec![1.0; children.len()];
                let n = i32::try_from(children.len())
                    .map_err(|_| ProblemError::Unsupported("SCIP sum arity".into()))?;
                native!(
                    "SCIPcreateExprSum",
                    ffi::SCIPcreateExprSum(
                        s,
                        &mut expr,
                        n,
                        children.as_mut_ptr(),
                        ones.as_mut_ptr(),
                        0.0,
                        None,
                        ptr::null_mut()
                    )
                )?;
            }
            Node::Product(c) => {
                let mut children: Vec<_> = c.iter().map(child).collect();
                let n = i32::try_from(children.len())
                    .map_err(|_| ProblemError::Unsupported("SCIP product arity".into()))?;
                native!(
                    "SCIPcreateExprProduct",
                    ffi::SCIPcreateExprProduct(
                        s,
                        &mut expr,
                        n,
                        children.as_mut_ptr(),
                        1.0,
                        None,
                        ptr::null_mut()
                    )
                )?;
            }
            Node::Pow { base, exponent } => native!(
                "SCIPcreateExprPow",
                ffi::SCIPcreateExprPow(
                    s,
                    &mut expr,
                    child(base),
                    exponent.value(),
                    None,
                    ptr::null_mut()
                )
            )?,
            Node::Exp(c) => native!(
                "SCIPcreateExprExp",
                ffi::SCIPcreateExprExp(s, &mut expr, child(c), None, ptr::null_mut())
            )?,
            Node::Log(c) => native!(
                "SCIPcreateExprLog",
                ffi::SCIPcreateExprLog(s, &mut expr, child(c), None, ptr::null_mut())
            )?,
            Node::Abs(c) => native!(
                "SCIPcreateExprAbs",
                ffi::SCIPcreateExprAbs(s, &mut expr, child(c), None, ptr::null_mut())
            )?,
            Node::Sin(c) => native!(
                "SCIPcreateExprSin",
                ffi::SCIPcreateExprSin(s, &mut expr, child(c), None, ptr::null_mut())
            )?,
            Node::Cos(c) => native!(
                "SCIPcreateExprCos",
                ffi::SCIPcreateExprCos(s, &mut expr, child(c), None, ptr::null_mut())
            )?,
        }
        out.nodes[i] = expr;
    }
    Ok(out)
}
/// Export a plan: variables with their boxes and domains, linear and nonlinear
/// constraints, and the objective (coefficients when affine, an epigraph otherwise).
pub(crate) fn export(
    instance: &mut Instance,
    plan: &crate::execution::factorable::Plan<'_>,
) -> Result<Export, ProblemError> {
    let program = plan.program;
    let columns = program.variables.len();
    let affine_objective = plan
        .objective
        .and_then(|(node, _)| plan.affine[node].as_ref().map(|f| (node, f)));
    let objective_coefficient = |index: usize| {
        affine_objective.map_or(0.0, |(_, f)| {
            f.terms
                .iter()
                .filter(|(j, _)| *j == index)
                .map(|(_, c)| *c)
                .sum()
        })
    };
    let mut coordinates = Vec::with_capacity(plan.boxes.len());
    for (i, v) in program.variables.iter().enumerate() {
        coordinates.push(instance.variable(
            &format!("x{i}"),
            plan.boxes[i],
            objective_coefficient(i),
            vartype(v.domain),
        )?);
    }
    for k in 0..program.auxiliaries.len() {
        coordinates.push(instance.variable(
            &format!("a{k}"),
            plan.boxes[columns + k],
            objective_coefficient(columns + k),
            ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS,
        )?);
    }
    if let Some((_, sense)) = plan.objective {
        let sense = match sense {
            ObjectiveSense::Minimize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MINIMIZE,
            ObjectiveSense::Maximize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MAXIMIZE,
        };
        native!(
            "SCIPsetObjsense",
            ffi::SCIPsetObjsense(instance.ptr(), sense)
        )?;
    }
    if let Some((_, f)) = affine_objective
        && f.constant != 0.0
    {
        native!(
            "SCIPaddOrigObjoffset",
            ffi::SCIPaddOrigObjoffset(instance.ptr(), f.constant)
        )?;
    }
    let epigraph = match (plan.objective, affine_objective) {
        (Some(_), None) => Some(instance.variable(
            "objective",
            (f64::NEG_INFINITY, f64::INFINITY),
            1.0,
            ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS,
        )?),
        _ => None,
    };
    // Nodes under a nonlinear root, marked in one reverse pass.
    let mut needed = vec![false; program.nodes.len()];
    for c in &plan.constraints {
        if plan.affine[c.node].is_none() {
            needed[c.node] = true;
        }
    }
    let nonlinear_objective = plan
        .objective
        .filter(|(node, _)| plan.affine[*node].is_none());
    if let Some((node, _)) = nonlinear_objective {
        needed[node] = true;
    }
    for i in (0..program.nodes.len()).rev() {
        if needed[i] {
            for c in crate::execution::factorable::children(&program.nodes[i]) {
                needed[*c] = true;
            }
        }
    }
    let exprs = expressions(instance, program, &coordinates, &needed)?;
    let mut constraints = Vec::with_capacity(plan.constraints.len() + 1);
    let (mut linear, mut nonlinear) = (0, 0);
    for (k, c) in plan.constraints.iter().enumerate() {
        let name = cstring(&format!("c{k}"))?;
        let mut cons = ptr::null_mut();
        let exported = if let Some(form) = &plan.affine[c.node] {
            let mut vars: Vec<_> = form.terms.iter().map(|(j, _)| coordinates[*j]).collect();
            let mut vals: Vec<_> = form.terms.iter().map(|(_, v)| *v).collect();
            let n = i32::try_from(vars.len())
                .map_err(|_| ProblemError::Unsupported("SCIP linear arity".into()))?;
            let lhs = instance.native(c.lower - form.constant)?;
            let rhs = instance.native(c.upper - form.constant)?;
            native!(
                "SCIPcreateConsBasicLinear",
                ffi::SCIPcreateConsBasicLinear(
                    instance.ptr(),
                    &mut cons,
                    name.as_ptr(),
                    n,
                    vars.as_mut_ptr(),
                    vals.as_mut_ptr(),
                    lhs,
                    rhs
                )
            )?;
            linear += 1;
            Some(form.constant)
        } else {
            let lhs = instance.native(c.lower)?;
            let rhs = instance.native(c.upper)?;
            native!(
                "SCIPcreateConsBasicNonlinear",
                ffi::SCIPcreateConsBasicNonlinear(
                    instance.ptr(),
                    &mut cons,
                    name.as_ptr(),
                    exprs.nodes[c.node],
                    lhs,
                    rhs
                )
            )?;
            nonlinear += 1;
            None
        };
        instance.hold(cons)?;
        constraints.push(Exported {
            cons,
            node: c.node,
            lower: c.lower,
            upper: c.upper,
            linear: exported,
            epigraph: false,
        });
    }
    if let (Some((node, sense)), Some(z)) = (nonlinear_objective, epigraph) {
        // min f: f - z <= 0; max f: f - z >= 0.
        let mut z_expr = ptr::null_mut();
        native!(
            "SCIPcreateExprVar",
            ffi::SCIPcreateExprVar(instance.ptr(), &mut z_expr, z, None, ptr::null_mut())
        )?;
        let mut terms = [exprs.nodes[node], z_expr];
        let mut coefficients = [1.0, -1.0];
        let mut expr = ptr::null_mut();
        let created = native!(
            "SCIPcreateExprSum",
            ffi::SCIPcreateExprSum(
                instance.ptr(),
                &mut expr,
                2,
                terms.as_mut_ptr(),
                coefficients.as_mut_ptr(),
                0.0,
                None,
                ptr::null_mut()
            )
        );
        // SAFETY: releases the variable expression's creation reference; the sum holds one.
        let _ = unsafe { ffi::SCIPreleaseExpr(instance.ptr(), &mut z_expr) };
        created?;
        let (lhs, rhs) = match sense {
            ObjectiveSense::Minimize => (-instance.infinity, 0.0),
            ObjectiveSense::Maximize => (0.0, instance.infinity),
        };
        let name = cstring("objective")?;
        let mut cons = ptr::null_mut();
        let created = native!(
            "SCIPcreateConsBasicNonlinear",
            ffi::SCIPcreateConsBasicNonlinear(
                instance.ptr(),
                &mut cons,
                name.as_ptr(),
                expr,
                lhs,
                rhs
            )
        );
        // SAFETY: releases the epigraph expression's creation reference.
        let _ = unsafe { ffi::SCIPreleaseExpr(instance.ptr(), &mut expr) };
        created?;
        instance.hold(cons)?;
        nonlinear += 1;
        constraints.push(Exported {
            cons,
            node,
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
            linear: None,
            epigraph: true,
        });
    }
    drop(exprs);
    Ok(Export {
        coordinates,
        epigraph,
        constraints,
        objective: affine_objective.map(|(node, _)| node),
        linear,
        nonlinear,
    })
}

/// An original-space SCIP solution owned by this module.
struct Solution<'i> {
    instance: &'i Instance,
    sol: *mut ffi::SCIP_SOL,
}
impl Drop for Solution<'_> {
    fn drop(&mut self) {
        if !self.sol.is_null() {
            // SAFETY: the solution was created on this instance and is freed once.
            let _ = unsafe { ffi::SCIPfreeSol(self.instance.ptr(), &mut self.sol) };
        }
    }
}
impl<'i> Solution<'i> {
    fn new(instance: &'i Instance, partial: bool) -> Result<Self, ProblemError> {
        let mut sol = ptr::null_mut();
        if partial {
            native!(
                "SCIPcreatePartialSol",
                ffi::SCIPcreatePartialSol(instance.ptr(), &mut sol, ptr::null_mut())
            )?;
        } else {
            native!(
                "SCIPcreateOrigSol",
                ffi::SCIPcreateOrigSol(instance.ptr(), &mut sol, ptr::null_mut())
            )?;
        }
        Ok(Self { instance, sol })
    }
    fn set(&self, var: *mut ffi::SCIP_VAR, value: f64) -> Result<(), ProblemError> {
        native!(
            "SCIPsetSolVal",
            ffi::SCIPsetSolVal(self.instance.ptr(), self.sol, var, value)
        )
    }
}
/// Largest relative deviation between the native model and the neutral program at one
/// point: sides and function values of every exported constraint, and the objective. An
/// undefined value on both sides agrees; on one side only it is an infinite deviation.
pub(crate) fn readback(
    instance: &Instance,
    export: &Export,
    program: &FactorableProgram,
    point: &[f64],
    auxiliary: &[f64],
) -> Result<f64, ProblemError> {
    let values = program.evaluate(point, auxiliary)?;
    let solution = Solution::new(instance, false)?;
    for (var, value) in export.coordinates.iter().zip(point.iter().chain(auxiliary)) {
        solution.set(*var, *value)?;
    }
    if let Some(z) = export.epigraph {
        solution.set(z, 0.0)?;
    }
    let s = instance.ptr();
    let invalid = |v: f64| !v.is_finite() || v.abs() >= ffi::SCIP_INVALID;
    let deviation = |native: f64, neutral: f64| match (invalid(native), invalid(neutral)) {
        (true, true) => 0.0,
        (false, false) => (native - neutral).abs() / 1.0_f64.max(native.abs()).max(neutral.abs()),
        _ => f64::INFINITY,
    };
    let side = |native: f64, neutral: f64| {
        if native.abs() >= instance.infinity && !neutral.is_finite() {
            0.0
        } else {
            deviation(native, neutral)
        }
    };
    let mut worst: f64 = 0.0;
    for c in &export.constraints {
        let neutral = values[c.node];
        if let Some(constant) = c.linear {
            // SAFETY: the constraint is a live linear constraint held by the instance.
            let n = unsafe { ffi::SCIPgetNVarsLinear(s, c.cons) };
            // SAFETY: as above; the arrays have `n` entries owned by the constraint.
            let vars = unsafe { ffi::SCIPgetVarsLinear(s, c.cons) };
            // SAFETY: as above.
            let vals = unsafe { ffi::SCIPgetValsLinear(s, c.cons) };
            let n = usize::try_from(n).map_err(|_| ProblemError::internal("SCIP arity"))?;
            let mut activity = constant;
            if n > 0 {
                if vars.is_null() || vals.is_null() {
                    return Err(ProblemError::internal("SCIP linear readback"));
                }
                // SAFETY: SCIP owns `n` initialized entries at each pointer.
                let vars = unsafe { std::slice::from_raw_parts(vars, n) };
                // SAFETY: as above.
                let vals = unsafe { std::slice::from_raw_parts(vals, n) };
                for (var, coefficient) in vars.iter().zip(vals) {
                    // SAFETY: a value query of a held variable in a solution of this instance.
                    activity += coefficient * unsafe { ffi::SCIPgetSolVal(s, solution.sol, *var) };
                }
            }
            // SAFETY: side queries of a live linear constraint.
            let lhs = unsafe { ffi::SCIPgetLhsLinear(s, c.cons) };
            // SAFETY: as above.
            let rhs = unsafe { ffi::SCIPgetRhsLinear(s, c.cons) };
            worst = worst
                .max(deviation(activity, neutral))
                .max(side(lhs + constant, c.lower))
                .max(side(rhs + constant, c.upper));
        } else {
            // SAFETY: the constraint is a live nonlinear constraint held by the instance.
            let expr = unsafe { ffi::SCIPgetExprNonlinear(c.cons) };
            native!("SCIPevalExpr", ffi::SCIPevalExpr(s, expr, solution.sol, 0))?;
            // SAFETY: reads the value SCIP just stored in the live expression.
            let native = unsafe { ffi::SCIPexprGetEvalValue(expr) };
            worst = worst.max(deviation(native, neutral));
            if !c.epigraph {
                // SAFETY: side queries of a live nonlinear constraint.
                let lhs = unsafe { ffi::SCIPgetLhsNonlinear(c.cons) };
                // SAFETY: as above.
                let rhs = unsafe { ffi::SCIPgetRhsNonlinear(c.cons) };
                worst = worst.max(side(lhs, c.lower)).max(side(rhs, c.upper));
            }
        }
    }
    if let Some(node) = export.objective {
        // SAFETY: a value query of the live instance's objective offset.
        let mut native = unsafe { ffi::SCIPgetOrigObjoffset(s) };
        for (var, value) in export.coordinates.iter().zip(point.iter().chain(auxiliary)) {
            // SAFETY: an objective-coefficient query of a held variable.
            native += unsafe { ffi::SCIPvarGetObj(*var) } * value;
        }
        worst = worst.max(deviation(native, values[node]));
    }
    Ok(worst)
}
/// Submit a primal incumbent in program columns. A program with auxiliaries or an
/// epigraph gets a partial solution, completed by SCIP; returns whether SCIP stored it.
fn inject(
    instance: &Instance,
    export: &Export,
    program: &FactorableProgram,
    primal: &[f64],
) -> Result<bool, ProblemError> {
    if primal.len() != program.variables.len() || primal.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::Contract(
            "SCIP incumbent dimensions/values".into(),
        ));
    }
    let partial = !program.auxiliaries.is_empty() || export.epigraph.is_some();
    let mut solution = Solution::new(instance, partial)?;
    for (var, value) in export.coordinates.iter().zip(primal) {
        solution.set(*var, *value)?;
    }
    let mut stored = 0;
    let added = native!(
        "SCIPaddSolFree",
        ffi::SCIPaddSolFree(instance.ptr(), &mut solution.sol, &mut stored)
    );
    // `SCIPaddSolFree` frees the solution and clears the pointer, success or not.
    solution.sol = ptr::null_mut();
    added?;
    Ok(stored != 0)
}

/// One SCIP attempt over an admitted factorable program.
#[derive(Debug)]
pub(crate) struct Request<'a> {
    pub program: &'a FactorableProgram,
    pub initial: &'a [f64],
    pub intent: SolveIntent,
    pub normalization: &'a Normalization,
    pub settings: &'a Settings,
    pub controls: &'a Controls,
    pub accuracy: &'a ResolvedAccuracy,
    pub execution: &'a Execution,
    pub warm: Option<&'a WarmStart>,
    pub compatibility: &'a Compatibility,
}
/// Export, read back, inject, solve and report in original coordinates. Qualification
/// belongs to the factorable runner.
///
/// # Errors
/// ABI, export, option or native failures before a report exists.
pub(crate) fn solve(r: &Request<'_>) -> Result<SolveReport, ProblemError> {
    r.controls.validate()?;
    let abi = abi()?;
    let plan = crate::execution::factorable::plan(r.program, r.intent).map_err(|refusals| {
        let reasons: Vec<String> = refusals.iter().map(ToString::to_string).collect();
        ProblemError::Unsupported(format!("factorable export refused: {}", reasons.join("; ")))
    })?;
    let program = r.program;
    let columns = program.variables.len();
    if r.initial.len() != columns {
        return Err(ProblemError::Contract("SCIP start dimensions".into()));
    }
    // Normalized absolute gap budgets convert to original objective units.
    let gap_absolute = r.accuracy.mip_absolute_gap * r.normalization.objective;
    let mut instance = Instance::new(r.execution)?;
    let options = configure(
        &instance,
        r.settings,
        r.controls,
        r.accuracy,
        r.execution,
        gap_absolute,
    )?;
    let export = export(&mut instance, &plan)?;
    // Readback at the start, clamped into the box, with auxiliaries inside theirs.
    let clamp = |x: f64, (l, u): (f64, f64)| {
        let x = if x.is_finite() { x } else { 0.0 };
        x.max(l).min(u)
    };
    let start: Vec<f64> = r
        .initial
        .iter()
        .zip(&plan.boxes)
        .map(|(x, b)| clamp(*x, *b))
        .collect();
    let auxiliary: Vec<f64> = plan.boxes[columns..]
        .iter()
        .map(|b| clamp(0.5 * (b.0 + b.1), *b))
        .collect();
    let deviation = readback(&instance, &export, program, &start, &auxiliary)?;
    let submitted = match r.warm {
        Some(seed) => {
            seed.validate(r.compatibility)?;
            let WarmPayload::Nlp { primal, .. } = &seed.payload else {
                return Err(ProblemError::Unsupported(
                    "SCIP consumes a primal incumbent only".into(),
                ));
            };
            Some(inject(&instance, &export, program, primal)?)
        }
        None => None,
    };
    native!("SCIPsolve", ffi::SCIPsolve(instance.ptr()))?;
    let s = instance.ptr();
    // SAFETY: a status query after `SCIPsolve` returned.
    let raw = unsafe { ffi::SCIPgetStatus(s) };
    let status = Status::from_raw(raw).ok_or_else(|| {
        ProblemError::Internal(format!("SCIP status {raw} outside the 10.0.2 ABI"))
    })?;
    // SAFETY: bound and statistic queries after `SCIPsolve` returned.
    let primal_bound = instance.finite(unsafe { ffi::SCIPgetPrimalbound(s) });
    // SAFETY: as above.
    let dual_bound = instance.finite(unsafe { ffi::SCIPgetDualbound(s) });
    // SAFETY: as above.
    let gap = instance.finite(unsafe { ffi::SCIPgetGap(s) });
    // SAFETY: as above.
    let nodes = unsafe { ffi::SCIPgetNTotalNodes(s) };
    // SAFETY: as above.
    let best = unsafe { ffi::SCIPgetBestSol(s) };
    let candidate = if best.is_null() {
        None
    } else {
        let primal: Vec<f64> = export.coordinates[..columns]
            .iter()
            // SAFETY: value queries of held variables in SCIP's best solution.
            .map(|var| unsafe { ffi::SCIPgetSolVal(s, best, *var) })
            .collect();
        // SAFETY: the original objective of SCIP's best solution.
        let objective = unsafe { ffi::SCIPgetSolOrigObj(s, best) };
        Some(Candidate {
            kind: CandidateKind::FeasiblePoint,
            primal,
            objective: plan.objective.and(instance.finite(objective)),
            row_dual: None,
            bound_dual: None,
            reduced_costs: None,
            slacks: None,
        })
    };
    let watch = instance.observed();
    let contract = OracleContract {
        identity: program.structure,
        variables: program
            .variables
            .iter()
            .zip(&plan.boxes)
            .map(|(v, (lower, upper))| Variable {
                id: v.id,
                lower: *lower,
                upper: *upper,
            })
            .collect(),
        rows: program.rows.iter().map(|r| r.id).collect(),
        derivatives: pse_kernels::DerivativeOrder::Value,
        smoothness: pse_kernels::DerivativeOrder::Value,
    };
    let mut report = SolveReport::new(Backend::Scip, &contract, termination(status), r.execution);
    let readback = deviation <= READBACK_TOLERANCE;
    report.evidence.start_submitted = submitted.is_some();
    report.evidence.global = Some(GlobalEvidence {
        fidelity: plan.fidelity,
        domain: plan.domain,
        sense: plan
            .objective
            .map_or(ObjectiveSense::Minimize, |(_, sense)| sense),
        feasibility: feasibility(r.accuracy),
        gap_relative: r.accuracy.mip_relative_gap,
        gap_absolute,
        dual_bound: plan.objective.and(dual_bound),
        primal_bound: plan.objective.and(primal_bound),
        gap: plan.objective.and(gap),
        nodes,
        readback,
        dual: if plan.fidelity == Fidelity::Exact {
            BoundSource::ExactExport
        } else {
            BoundSource::RelaxedExport
        },
        primal: PrimalSource::Backend,
        infeasible: status == Status::Infeasible,
    });
    let metrics = &mut report.metrics;
    metrics.insert("scip.status".into(), Metric::Integer(i64::from(raw)));
    metrics.insert("scip.nodes".into(), Metric::Integer(nodes));
    for (key, value) in [
        ("scip.primal_bound", primal_bound),
        ("scip.dual_bound", dual_bound),
        ("scip.gap", gap),
    ] {
        metrics.insert(
            key.into(),
            value.map_or(
                Metric::Unavailable(crate::solve::UnavailableReason::NotApplicable),
                Metric::Real,
            ),
        );
    }
    metrics.insert(
        "scip.events".into(),
        Metric::Integer(i64::try_from(watch.events).unwrap_or(i64::MAX)),
    );
    metrics.insert("scip.interrupted".into(), Metric::Bool(watch.interrupted));
    metrics.insert("scip.callback_failed".into(), Metric::Bool(watch.failed));
    metrics.insert(
        "export.fidelity".into(),
        Metric::Text(
            match plan.fidelity {
                Fidelity::Exact => "exact",
                Fidelity::Relaxed => "relaxed",
                Fidelity::Unavailable => "unavailable",
            }
            .into(),
        ),
    );
    metrics.insert(
        "export.constraints.linear".into(),
        Metric::Integer(i64::try_from(export.linear).unwrap_or(i64::MAX)),
    );
    metrics.insert(
        "export.constraints.nonlinear".into(),
        Metric::Integer(i64::try_from(export.nonlinear).unwrap_or(i64::MAX)),
    );
    metrics.insert("export.readback.deviation".into(), Metric::Real(deviation));
    if let Some(stored) = submitted {
        metrics.insert("scip.incumbent.stored".into(), Metric::Bool(stored));
    }
    report.options = options;
    report.provenance.extend([
        (
            "scip.version".into(),
            format!("{}.{}.{}", abi.library.0, abi.library.1, abi.library.2),
        ),
        ("scip.api".into(), abi.api.to_string()),
        (
            "scip.objective".into(),
            "authored sense; nonlinear objectives through an epigraph variable".into(),
        ),
    ]);
    report.warm_start = candidate.as_ref().map(|c| WarmStart {
        origin: None,
        compatibility: r.compatibility.clone(),
        payload: WarmPayload::Nlp {
            primal: c.primal.clone(),
            bounds: None,
            rows: None,
        },
    });
    report.candidate = candidate;
    drop(instance);
    Ok(report)
}

#[cfg(test)]
pub(crate) mod testing {
    //! Native access for the adapter's unit tests.
    use super::*;
    /// Solve the exported plan directly, without the adapter's report.
    pub(crate) fn raw_status(
        program: &FactorableProgram,
        intent: SolveIntent,
        execution: &Execution,
    ) -> Result<(Status, u64, bool), ProblemError> {
        let plan = crate::execution::factorable::plan(program, intent)
            .map_err(|r| ProblemError::Unsupported(format!("{r:?}")))?;
        let mut instance = Instance::new(execution)?;
        configure(
            &instance,
            &Settings::default(),
            &Controls::default(),
            &ResolvedAccuracy::nominal(),
            execution,
            1e-6,
        )?;
        export(&mut instance, &plan)?;
        native!("SCIPsolve", ffi::SCIPsolve(instance.ptr()))?;
        // SAFETY: a status query after `SCIPsolve` returned.
        let raw = unsafe { ffi::SCIPgetStatus(instance.ptr()) };
        let status = Status::from_raw(raw).ok_or_else(|| ProblemError::internal("status"))?;
        let watch = instance.observed();
        Ok((status, watch.events, watch.interrupted))
    }
    /// Export and read back at each point, returning the worst deviation.
    pub(crate) fn readback_at(
        program: &FactorableProgram,
        intent: SolveIntent,
        points: &[(Vec<f64>, Vec<f64>)],
    ) -> Result<(f64, usize, usize), ProblemError> {
        let plan = crate::execution::factorable::plan(program, intent)
            .map_err(|r| ProblemError::Unsupported(format!("{r:?}")))?;
        let execution = Execution::new(Arc::default(), &Controls::default());
        let mut instance = Instance::new(&execution)?;
        let export = export(&mut instance, &plan)?;
        let mut worst: f64 = 0.0;
        for (x, a) in points {
            worst = worst.max(readback(&instance, &export, program, x, a)?);
        }
        Ok((worst, export.linear, export.nonlinear))
    }
    /// Export `exported` and read it back against `checked`, a program of the same shape:
    /// the negative control of readback.
    pub(crate) fn readback_against(
        exported: &FactorableProgram,
        checked: &FactorableProgram,
        points: &[(Vec<f64>, Vec<f64>)],
    ) -> Result<f64, ProblemError> {
        let plan = crate::execution::factorable::plan(exported, SolveIntent::FeasiblePoint)
            .map_err(|r| ProblemError::Unsupported(format!("{r:?}")))?;
        let execution = Execution::new(Arc::default(), &Controls::default());
        let mut instance = Instance::new(&execution)?;
        let export = export(&mut instance, &plan)?;
        let mut worst: f64 = 0.0;
        for (x, a) in points {
            worst = worst.max(readback(&instance, &export, checked, x, a)?);
        }
        Ok(worst)
    }
    /// The effective native options after configuration.
    pub(crate) fn configured(
        settings: &Settings,
        controls: &Controls,
        accuracy: &ResolvedAccuracy,
    ) -> Result<Options, ProblemError> {
        let mut execution = Execution::new(Arc::default(), controls);
        execution.memory = Some(256 << 20);
        let instance = Instance::new(&execution)?;
        configure(&instance, settings, controls, accuracy, &execution, 1e-6)
    }
}

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
//! constraints and every other function a nonlinear constraint on its bounds. SCIP 10 has
//! no semicontinuous variable type, so a semi column arrives lowered by the plan's
//! `semi(indicator)` transformation: its binary indicator is a variable after the
//! auxiliaries and its links are linear constraints. A nonlinear objective is exported
//! through an epigraph variable. The native model is read back and evaluated against
//! [`FactorableProgram::evaluate`] before any claim transfers.
use crate::execution::factorable::{Affine, Expression, Plan, Semi};
use crate::{
    LimitKind, NativeStatus, OracleContract, ProblemError, Variable,
    execution::ScipSettings as Settings,
    solve::{
        Assurance, Backend, BoundSource, Candidate, CandidateKind, CaptureThrottle, Compatibility,
        Controls, Event, Execution, GlobalEvidence, GlobalRecord, Iis, IisMember, IncumbentEvent,
        Metric, NativeTermination, OptionValue, Options, PoolSolution, PrimalSource, Progress,
        ResolvedAccuracy, SolveIntent, SolveReport, Termination, UnavailableReason, WarmPayload,
        WarmStart,
    },
};
use pse_ids::{ContentHash, FramedHasher};
use pse_math::{
    binding::ObjectiveSense,
    factorable::{FactorableProgram, Fidelity, Node},
    normalization::Normalization,
};
use pse_model::generated::enums::ModelingVariableDomain;
use scip_sys as ffi;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{CStr, CString, c_char},
    ptr::{self, NonNull},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
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

/// What the event handler observes during one solve; owned by its instance. The handler
/// is copied into sub-SCIPs, concurrent solvers and the IIS sub-problem with the same
/// data, which those poll from their own threads, so every shared field is atomic or
/// locked. `cancel`, `progress`, `started` and `objective` change only between solves.
#[derive(Debug)]
struct Watch {
    cancel: Arc<AtomicBool>,
    progress: Arc<Progress>,
    started: Instant,
    events: AtomicU64,
    interrupted: AtomicBool,
    failed: AtomicBool,
    /// Incumbent and bound reporting of a program with an objective; only the owning
    /// SCIP's handler reads it.
    objective: Option<Objective>,
}
/// How the owning SCIP's incumbents and bounds are reported in original units: the owning
/// SCIP (copies share the handler data), the column variables of the export, the
/// objective constant held outside SCIP, SCIP's infinity and the capture throttle of
/// solution vectors.
#[derive(Debug)]
struct Objective {
    owner: *mut ffi::SCIP,
    columns: Vec<*mut ffi::SCIP_VAR>,
    offset: f64,
    infinity: f64,
    throttle: CaptureThrottle,
}
impl Objective {
    /// A finite native value in original units, the offset applied.
    fn original(&self, x: f64) -> Option<f64> {
        (x.is_finite() && x.abs() < self.infinity).then_some(x + self.offset)
    }
    /// A finite native gap.
    fn gap(&self, gap: f64) -> Option<f64> {
        (gap.is_finite() && gap.abs() < self.infinity).then_some(gap)
    }
    /// Whether `scip` is the owning SCIP in a stage that admits every query of
    /// [`Self::incumbent`] and [`Self::bound`]; SCIP aborts the process on a query in
    /// another stage.
    fn admitted(&self, scip: *mut ffi::SCIP) -> bool {
        // SAFETY: a stage query of the live owning SCIP.
        scip == self.owner
            && matches!(
                unsafe { ffi::SCIPgetStage(scip) },
                ffi::SCIP_Stage_SCIP_STAGE_INITSOLVE
                    | ffi::SCIP_Stage_SCIP_STAGE_SOLVING
                    | ffi::SCIP_Stage_SCIP_STAGE_SOLVED
            )
    }
    /// Report the owning SCIP's best solution as an incumbent, with its solution vector
    /// when `capture` admits one; nothing when there is none.
    fn incumbent(&self, scip: *mut ffi::SCIP, watch: &Watch, capture: impl FnOnce() -> bool) {
        if !self.admitted(scip) {
            return;
        }
        // SAFETY: the stage admits the queries; the solution belongs to this SCIP, and the
        // column variables are held by its instance.
        let best = unsafe { ffi::SCIPgetBestSol(scip) };
        if best.is_null() {
            return;
        }
        // SAFETY: as above.
        let (objective, dual_bound, gap, nodes, seconds, primal) = unsafe {
            let primal = capture().then(|| {
                self.columns
                    .iter()
                    .map(|var| ffi::SCIPgetSolVal(scip, best, *var))
                    .collect::<Vec<f64>>()
            });
            (
                ffi::SCIPgetSolOrigObj(scip, best),
                ffi::SCIPgetDualbound(scip),
                ffi::SCIPgetGap(scip),
                ffi::SCIPgetNTotalNodes(scip),
                ffi::SCIPgetSolvingTime(scip),
                primal,
            )
        };
        let Some(objective) = self.original(objective) else {
            return;
        };
        watch.progress.push(Event {
            phase: "scip.incumbent".into(),
            elapsed: watch.started.elapsed(),
            values: BTreeMap::new(),
            incumbent: Some(IncumbentEvent {
                objective,
                dual_bound: self.original(dual_bound),
                gap: self.gap(gap),
                nodes,
                seconds,
                primal: primal.filter(|p| p.iter().all(|v| v.is_finite())),
            }),
        });
    }
    /// Report an improved dual bound, with the primal bound and gap it leaves.
    fn bound(&self, scip: *mut ffi::SCIP, watch: &Watch) {
        if !self.admitted(scip) {
            return;
        }
        // SAFETY: the stage admits the bound queries.
        let (primal, dual, gap) = unsafe {
            (
                ffi::SCIPgetPrimalbound(scip),
                ffi::SCIPgetDualbound(scip),
                ffi::SCIPgetGap(scip),
            )
        };
        let metric = |v: Option<f64>| {
            v.map_or(
                Metric::Unavailable(UnavailableReason::NotApplicable),
                Metric::Real,
            )
        };
        watch.progress.push(Event {
            phase: "scip.bound".into(),
            elapsed: watch.started.elapsed(),
            values: [
                ("primal_bound".into(), metric(self.original(primal))),
                ("dual_bound".into(), metric(self.original(dual))),
                ("gap".into(), metric(self.gap(gap))),
            ]
            .into(),
            incumbent: None,
        });
    }
}
fn watch<'a>(eventhdlr: *mut ffi::SCIP_EVENTHDLR) -> Option<&'a Watch> {
    // SAFETY: SCIP returns the data pointer this module registered for the handler.
    let data = unsafe { ffi::SCIPeventhdlrGetData(eventhdlr) };
    // SAFETY: the pointer is the instance's boxed `Watch`, freed only after `SCIPfree`,
    // which frees every copy of the handler first; shared fields are atomic.
    unsafe { data.cast::<Watch>().as_ref() }
}
/// Contain a panic in a native callback as a SCIP error.
fn contained(work: impl FnOnce() -> ffi::SCIP_RETCODE) -> ffi::SCIP_RETCODE {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
        .unwrap_or(ffi::SCIP_Retcode_SCIP_ERROR)
}
/// Interrupt the calling SCIP when the attempt was cancelled; each SCIP once.
fn poll(scip: *mut ffi::SCIP, watch: &Watch) -> ffi::SCIP_RETCODE {
    if !watch.cancel.load(Ordering::Acquire) {
        return ffi::SCIP_Retcode_SCIP_OKAY;
    }
    watch.interrupted.store(true, Ordering::Release);
    // SAFETY: interruption is admitted in every stage that issues the caught events,
    // and repeated interruption of one SCIP is idempotent.
    let code = unsafe { ffi::SCIPinterruptSolve(scip) };
    if code != ffi::SCIP_Retcode_SCIP_OKAY {
        watch.failed.store(true, Ordering::Release);
    }
    code
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
        watch.events.fetch_add(1, Ordering::Relaxed);
        // SAFETY: SCIP passes the live event being processed.
        let kind = unsafe { ffi::SCIPeventGetType(event) };
        if let Some(objective) = &watch.objective {
            // A new best solution is an incumbent (its bounds travel with it); an improved
            // dual bound alone is a bound event. A deferred solution capture is taken at
            // the first event after the throttle interval.
            if kind & event::BESTSOLFOUND != 0 {
                objective.incumbent(scip, watch, || objective.throttle.admit());
            } else {
                if kind & event::DUALBOUNDIMPROVED != 0 {
                    objective.bound(scip, watch);
                }
                if objective.throttle.deferred() {
                    objective.incumbent(scip, watch, || true);
                }
            }
        }
        poll(scip, watch)
    })
}
/// The handler of a copy (sub-SCIP, concurrent solver, IIS sub-problem): cancellation
/// only, since a copy's bounds are not the attempt's bounds.
unsafe extern "C" fn watch_copy_exec(
    scip: *mut ffi::SCIP,
    eventhdlr: *mut ffi::SCIP_EVENTHDLR,
    _event: *mut ffi::SCIP_EVENT,
    _data: *mut ffi::SCIP_EVENTDATA,
) -> ffi::SCIP_RETCODE {
    contained(|| match watch(eventhdlr) {
        Some(watch) => poll(scip, watch),
        None => ffi::SCIP_Retcode_SCIP_INVALIDDATA,
    })
}
/// An event execution callback.
type EventExec = Option<
    unsafe extern "C" fn(
        *mut ffi::SCIP,
        *mut ffi::SCIP_EVENTHDLR,
        *mut ffi::SCIP_EVENT,
        *mut ffi::SCIP_EVENTDATA,
    ) -> ffi::SCIP_RETCODE,
>;
/// Include the handler with `exec` and `data` in `scip`, with every lifecycle callback.
fn include_watch(
    scip: *mut ffi::SCIP,
    exec: EventExec,
    data: *mut ffi::SCIP_EVENTHDLRDATA,
) -> Result<(), ProblemError> {
    let name = cstring("pse_watch")?;
    let description = cstring("attempt cancellation, incumbents and bound progress")?;
    let mut hdlr = ptr::null_mut();
    native!(
        "SCIPincludeEventhdlrBasic",
        ffi::SCIPincludeEventhdlrBasic(
            scip,
            &mut hdlr,
            name.as_ptr(),
            description.as_ptr(),
            exec,
            data
        )
    )?;
    native!(
        "SCIPsetEventhdlrCopy",
        ffi::SCIPsetEventhdlrCopy(scip, hdlr, Some(watch_copy))
    )?;
    native!(
        "SCIPsetEventhdlrInit",
        ffi::SCIPsetEventhdlrInit(scip, hdlr, Some(watch_init))
    )?;
    native!(
        "SCIPsetEventhdlrExit",
        ffi::SCIPsetEventhdlrExit(scip, hdlr, Some(watch_exit))
    )?;
    native!(
        "SCIPsetEventhdlrInitsol",
        ffi::SCIPsetEventhdlrInitsol(scip, hdlr, Some(watch_initsol))
    )?;
    native!(
        "SCIPsetEventhdlrExitsol",
        ffi::SCIPsetEventhdlrExitsol(scip, hdlr, Some(watch_exitsol))
    )
}
unsafe extern "C" fn watch_copy(
    scip: *mut ffi::SCIP,
    eventhdlr: *mut ffi::SCIP_EVENTHDLR,
) -> ffi::SCIP_RETCODE {
    contained(|| {
        // SAFETY: SCIP passes the source handler, whose data is the instance's `Watch`.
        let data = unsafe { ffi::SCIPeventhdlrGetData(eventhdlr) };
        match include_watch(scip, Some(watch_copy_exec), data) {
            Ok(()) => ffi::SCIP_Retcode_SCIP_OKAY,
            Err(_) => ffi::SCIP_Retcode_SCIP_ERROR,
        }
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
    contained(|| {
        // SAFETY: SCIP calls initsol before branch-and-bound, where node events are caught.
        let code = unsafe {
            ffi::SCIPcatchEvent(
                scip,
                event::SOLVING,
                eventhdlr,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        // A solution the owning SCIP already holds (a submitted start, or one found in
        // presolve) is the search's first incumbent.
        if code == ffi::SCIP_Retcode_SCIP_OKAY
            && let Some(watch) = watch(eventhdlr)
            && let Some(objective) = &watch.objective
        {
            objective.incumbent(scip, watch, || objective.throttle.admit());
        }
        code
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
    modes: Modes,
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
/// Modes fixed before the problem exists (SCIP stage `INIT`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Modes {
    /// Exact rational solving (`SCIPenableExactSolving`).
    pub exact: bool,
    /// Reoptimization over a sequence (`SCIPenableReoptimization`).
    pub reoptimize: bool,
}
impl Instance {
    /// A quiet instance with default plugins, the cancellation handler, the requested
    /// modes and an empty problem.
    pub(crate) fn new(execution: &Execution, modes: Modes) -> Result<Self, ProblemError> {
        let mut raw = ptr::null_mut();
        native!("SCIPcreate", ffi::SCIPcreate(&mut raw))?;
        let scip = NonNull::new(raw).ok_or_else(|| ProblemError::memory("SCIPcreate"))?;
        let watch = Box::new(Watch {
            cancel: execution.cancel.clone(),
            progress: execution.progress.clone(),
            started: execution.started,
            events: AtomicU64::new(0),
            interrupted: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            objective: None,
        });
        let mut instance = Self {
            scip,
            vars: vec![],
            conss: vec![],
            watch: NonNull::from(Box::leak(watch)),
            infinity: 0.0,
            modes,
        };
        let s = instance.scip.as_ptr();
        native!(
            "SCIPincludeDefaultPlugins",
            ffi::SCIPincludeDefaultPlugins(s)
        )?;
        // SAFETY: silences the instance's own message handler.
        unsafe { ffi::SCIPsetMessagehdlrQuiet(s, 1) };
        include_watch(s, Some(watch_exec), instance.watch.as_ptr().cast())?;
        // Exact solving is admitted only in stage INIT, before the problem exists.
        if modes.exact {
            native!("SCIPenableExactSolving", ffi::SCIPenableExactSolving(s, 1))?;
        }
        let problem = cstring("pse_factorable")?;
        native!(
            "SCIPcreateProbBasic",
            ffi::SCIPcreateProbBasic(s, problem.as_ptr())
        )?;
        if modes.reoptimize {
            native!(
                "SCIPenableReoptimization",
                ffi::SCIPenableReoptimization(s, 1)
            )?;
        }
        // SAFETY: a value query on the live instance.
        instance.infinity = unsafe { ffi::SCIPinfinity(s) };
        Ok(instance)
    }
    fn ptr(&self) -> *mut ffi::SCIP {
        self.scip.as_ptr()
    }
    fn observed(&self) -> &Watch {
        // SAFETY: the handler data outlives the instance's borrows; outside a solve no
        // copy of the handler runs.
        unsafe { self.watch.as_ref() }
    }
    /// Attach the next attempt's cancellation, progress and clock to a retained
    /// instance, between solves.
    fn rewatch(&mut self, execution: &Execution) {
        // SAFETY: no solve is running, so no handler holds the data; the box is owned
        // exclusively by this instance.
        let watch = unsafe { self.watch.as_mut() };
        watch.cancel = execution.cancel.clone();
        watch.progress = execution.progress.clone();
        watch.started = execution.started;
        watch.events.store(0, Ordering::Relaxed);
        watch.interrupted.store(false, Ordering::Relaxed);
        watch.failed.store(false, Ordering::Relaxed);
    }
    /// Report the next solve's incumbents and bounds for a program with an objective:
    /// solutions in the export's column variables, objective values with the offset the
    /// export holds outside SCIP. Called between solves.
    fn report_incumbents(&mut self, export: &Export, columns: usize, objective: bool) {
        let reported = objective.then(|| Objective {
            owner: self.ptr(),
            columns: export.coordinates[..columns].to_vec(),
            offset: export.offset,
            infinity: self.infinity,
            throttle: CaptureThrottle::new(),
        });
        // SAFETY: no solve is running, so no handler holds the data; the box is owned
        // exclusively by this instance.
        unsafe { self.watch.as_mut() }.objective = reported;
    }
    /// Report the incumbent whose solution capture the throttle still defers, once the
    /// solve returned.
    fn flush_incumbent(&self) {
        let watch = self.observed();
        if let Some(objective) = &watch.objective
            && objective.throttle.outstanding()
        {
            objective.incumbent(self.ptr(), watch, || true);
        }
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
const RESERVED_PREFIXES: [&str; 5] = [
    "parallel/",
    "concurrent/",
    "iis/",
    "reoptimization/",
    "exact/",
];
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
    settings.admit(controls.threads)?;
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
        ("limits/gap", OptionValue::Real(accuracy.mip_relative_gap)),
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
    // Concurrent solving is deterministic, with the admitted permits as its threads.
    if controls.threads > 1 {
        let threads = i32::try_from(controls.threads)
            .map_err(|_| ProblemError::Unsupported("SCIP concurrent thread count".into()))?;
        reserved.extend([
            ("parallel/mode", OptionValue::Integer(1)),
            ("parallel/maxnthreads", OptionValue::Integer(threads)),
            ("parallel/minnthreads", OptionValue::Integer(threads)),
        ]);
    }
    // Two SCIP 10.0.2 behaviours are avoided. With bound removal, its post-processing
    // deletes any linear constraint whose single-use variables lost both bounds, which
    // can leave a feasible "subsystem"; so bounds stay in the subsystem. After the
    // greedy finder's additive phase, constraints it re-added carry an extra use, the
    // deletion phase skips them as ineligible and still reports the result irreducible;
    // so the finder runs the deletion filter alone.
    if settings.iis {
        reserved.extend([
            ("iis/irreducible", OptionValue::Bool(true)),
            ("iis/removebounds", OptionValue::Bool(false)),
            ("iis/removeunusedvars", OptionValue::Bool(true)),
            ("iis/greedy/additive", OptionValue::Bool(false)),
            ("iis/silent", OptionValue::Bool(true)),
        ]);
    }
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

/// How an exported constraint is represented natively, for readback.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    /// A linear constraint whose sides were shifted by the affine constant.
    Linear { constant: f64 },
    /// An exact rational linear constraint (exact mode), shifted like `Linear`.
    ExactLinear { constant: f64 },
    /// A nonlinear constraint on the expression, evaluated whole.
    Nonlinear,
    /// The epigraph `f - z` of a nonlinear objective.
    Epigraph,
    /// One side of a linear row under an indicator, `sign·(a·x) <= rhs`: `+1` holds the
    /// upper side and `-1` the lower side.
    Indicator { constant: f64, sign: f64 },
}
/// One exported constraint, retained for readback and IIS attribution.
#[derive(Debug)]
struct Exported {
    cons: *mut ffi::SCIP_CONS,
    expression: Expression,
    lower: f64,
    upper: f64,
    kind: Kind,
}
/// The native model of one plan.
#[derive(Debug)]
pub(crate) struct Export {
    /// Variables of the columns, then the auxiliaries, then the indicators of lowered semi
    /// columns.
    coordinates: Vec<*mut ffi::SCIP_VAR>,
    /// Lowered semi columns, whose indicators close the coordinates.
    semi: Vec<Semi>,
    epigraph: Option<*mut ffi::SCIP_VAR>,
    /// Exported functions; the constraint named `c{k}` belongs to plan function `k`.
    constraints: Vec<Exported>,
    /// Affine objective form, exported through variable objective coefficients.
    objective: Option<pse_math::factorable::NodeId>,
    /// Affine objective constant held outside SCIP (reoptimization changes coefficients
    /// only); zero when SCIP holds it as its objective offset.
    offset: f64,
    /// Constraint counts: linear, nonlinear and native.
    pub linear: usize,
    pub nonlinear: usize,
    pub native: usize,
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
/// A SCIP rational, freed on drop. Every finite binary64 value is an exact dyadic
/// rational, so conversion from `f64` is exact.
struct Rational(*mut ffi::SCIP_RATIONAL);
impl Rational {
    fn new(value: f64) -> Result<Self, ProblemError> {
        let mut raw = ptr::null_mut();
        native!("SCIPrationalCreate", ffi::SCIPrationalCreate(&mut raw))?;
        if raw.is_null() {
            return Err(ProblemError::memory("SCIPrationalCreate"));
        }
        let r = Self(raw);
        // SAFETY: a live rational owned by `r`; infinities map to SCIP's infinite rationals.
        unsafe {
            if value == f64::INFINITY {
                ffi::SCIPrationalSetInfinity(r.0);
            } else if value == f64::NEG_INFINITY {
                ffi::SCIPrationalSetNegInfinity(r.0);
            } else {
                ffi::SCIPrationalSetReal(r.0, value);
            }
        }
        Ok(r)
    }
    fn text(&self) -> String {
        let mut buffer: Vec<c_char> = vec![0; 256];
        // SAFETY: SCIP writes at most `len` bytes including the terminator into the buffer.
        let written = unsafe { ffi::SCIPrationalToString(self.0, buffer.as_mut_ptr(), 256) };
        if written < 0 {
            return String::new();
        }
        // SAFETY: the buffer is NUL-terminated within its length.
        unsafe { CStr::from_ptr(buffer.as_ptr()) }
            .to_string_lossy()
            .into_owned()
    }
}
impl Drop for Rational {
    fn drop(&mut self) {
        // SAFETY: the rational was created by `SCIPrationalCreate` and is freed once.
        unsafe { ffi::SCIPrationalFree(&mut self.0) };
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
        if self.modes.exact {
            let rational = |v: f64| {
                Rational::new(if v.abs() >= self.infinity {
                    v.signum() * f64::INFINITY
                } else {
                    v
                })
            };
            let (l, u, c) = (rational(lower)?, rational(upper)?, rational(objective)?);
            native!(
                "SCIPaddVarExactData",
                ffi::SCIPaddVarExactData(self.ptr(), var, l.0, u.0, c.0)
            )?;
        }
        native!("SCIPaddVar", ffi::SCIPaddVar(self.ptr(), var))?;
        Ok(var)
    }
    fn hold(&mut self, cons: *mut ffi::SCIP_CONS) -> Result<(), ProblemError> {
        self.conss.push(cons);
        native!("SCIPaddCons", ffi::SCIPaddCons(self.ptr(), cons))
    }
    /// The variable, or its negation, as a logic literal.
    fn literal(
        &self,
        var: *mut ffi::SCIP_VAR,
        negated: bool,
    ) -> Result<*mut ffi::SCIP_VAR, ProblemError> {
        if !negated {
            return Ok(var);
        }
        let mut negation = ptr::null_mut();
        native!(
            "SCIPgetNegatedVar",
            ffi::SCIPgetNegatedVar(self.ptr(), var, &mut negation)
        )?;
        Ok(negation)
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
fn arity(n: usize) -> Result<i32, ProblemError> {
    i32::try_from(n).map_err(|_| ProblemError::Unsupported("SCIP constraint arity".into()))
}
/// The coefficients of an affine form over the combined coordinates.
fn coefficients(form: &Affine, coordinates: usize) -> Vec<f64> {
    let mut out = vec![0.0; coordinates];
    for (j, c) in &form.terms {
        out[*j] += c;
    }
    out
}
/// Export a plan: variables with their boxes and domains (semi columns lowered to binary
/// indicators and linear links), linear, nonlinear, conditional and native constraints,
/// and the objective (coefficients when affine, an epigraph otherwise). Exact mode admits
/// only linear programs without native forms; a reoptimization session only affine
/// objectives and constraints.
pub(crate) fn export(instance: &mut Instance, plan: &Plan<'_>) -> Result<Export, ProblemError> {
    let program = plan.program;
    let columns = program.variables.len();
    let modes = instance.modes;
    let conditional = plan.constraints.iter().any(|c| c.condition.is_some());
    if modes.exact
        && (plan.nonlinear()
            || !program.native.is_empty()
            || conditional
            || !program.auxiliaries.is_empty())
    {
        return Err(ProblemError::Unsupported(
            "SCIP exact solving admits linear programs without native forms or auxiliaries".into(),
        ));
    }
    if modes.reoptimize && plan.nonlinear() {
        return Err(ProblemError::Unsupported(
            "SCIP reoptimization admits linear constraints and a linear objective".into(),
        ));
    }
    let affine_objective = plan
        .objective
        .and_then(|(node, _)| plan.affine[node].as_ref().map(|f| (node, f)));
    let objective = affine_objective
        .map(|(_, f)| coefficients(f, plan.boxes.len()))
        .unwrap_or_default();
    let mut coordinates = Vec::with_capacity(plan.boxes.len());
    for (i, v) in program.variables.iter().enumerate() {
        coordinates.push(instance.variable(
            &format!("x{i}"),
            plan.boxes[i],
            objective.get(i).copied().unwrap_or(0.0),
            vartype(v.domain),
        )?);
    }
    for k in 0..program.auxiliaries.len() {
        coordinates.push(instance.variable(
            &format!("a{k}"),
            plan.boxes[columns + k],
            objective.get(columns + k).copied().unwrap_or(0.0),
            ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS,
        )?);
    }
    for j in 0..plan.semi.len() {
        coordinates.push(instance.variable(
            &format!("z{j}"),
            plan.boxes[plan.indicator(j)],
            0.0,
            ffi::SCIP_Vartype_SCIP_VARTYPE_BINARY,
        )?);
    }
    if let Some((_, sense)) = plan.objective {
        native!(
            "SCIPsetObjsense",
            ffi::SCIPsetObjsense(instance.ptr(), objsense(sense))
        )?;
    }
    let mut offset = 0.0;
    if let Some((_, f)) = affine_objective
        && f.constant != 0.0
    {
        if modes.reoptimize {
            offset = f.constant;
        } else {
            native!(
                "SCIPaddOrigObjoffset",
                ffi::SCIPaddOrigObjoffset(instance.ptr(), f.constant)
            )?;
        }
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
        if let Expression::Node(node) = c.expression
            && plan.affine[node].is_none()
        {
            needed[node] = true;
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
        let condition = c
            .condition
            .map(|c| instance.literal(coordinates[c.column], !c.active))
            .transpose()?;
        // A function without an affine form is a program node; links are affine.
        let node = || match c.expression {
            Expression::Node(node) => Ok(exprs.nodes[node]),
            Expression::Link(_) => Err(ProblemError::internal("SCIP export of a link")),
        };
        match (plan.form(c.expression), condition) {
            (Some(form), None) => {
                let mut vars: Vec<_> = form.terms.iter().map(|(j, _)| coordinates[*j]).collect();
                let mut cons = ptr::null_mut();
                let n = arity(vars.len())?;
                let lhs = instance.native(c.lower - form.constant)?;
                let rhs = instance.native(c.upper - form.constant)?;
                let kind = if modes.exact {
                    let mut owned = Vec::with_capacity(form.terms.len());
                    for (_, v) in &form.terms {
                        owned.push(Rational::new(*v)?);
                    }
                    let mut vals: Vec<_> = owned.iter().map(|r| r.0).collect();
                    let (l, u) = (
                        Rational::new(c.lower - form.constant)?,
                        Rational::new(c.upper - form.constant)?,
                    );
                    native!(
                        "SCIPcreateConsBasicExactLinear",
                        ffi::SCIPcreateConsBasicExactLinear(
                            instance.ptr(),
                            &mut cons,
                            name.as_ptr(),
                            n,
                            vars.as_mut_ptr(),
                            vals.as_mut_ptr(),
                            l.0,
                            u.0
                        )
                    )?;
                    Kind::ExactLinear {
                        constant: form.constant,
                    }
                } else {
                    let mut vals: Vec<_> = form.terms.iter().map(|(_, v)| *v).collect();
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
                    Kind::Linear {
                        constant: form.constant,
                    }
                };
                instance.hold(cons)?;
                linear += 1;
                constraints.push(Exported {
                    cons,
                    expression: c.expression,
                    lower: c.lower,
                    upper: c.upper,
                    kind,
                });
            }
            (Some(form), Some(binvar)) => {
                // `a·x + c <= u` and `-(a·x) <= -(l - c)`, each while the literal holds.
                for (sign, side) in [(1.0, c.upper), (-1.0, c.lower)] {
                    if !side.is_finite() {
                        continue;
                    }
                    let mut vars: Vec<_> =
                        form.terms.iter().map(|(j, _)| coordinates[*j]).collect();
                    let mut vals: Vec<_> = form.terms.iter().map(|(_, v)| sign * v).collect();
                    let rhs = instance.native(sign * (side - form.constant))?;
                    let mut cons = ptr::null_mut();
                    native!(
                        "SCIPcreateConsBasicIndicator",
                        ffi::SCIPcreateConsBasicIndicator(
                            instance.ptr(),
                            &mut cons,
                            name.as_ptr(),
                            binvar,
                            arity(vars.len())?,
                            vars.as_mut_ptr(),
                            vals.as_mut_ptr(),
                            rhs
                        )
                    )?;
                    instance.hold(cons)?;
                    linear += 1;
                    constraints.push(Exported {
                        cons,
                        expression: c.expression,
                        lower: c.lower,
                        upper: c.upper,
                        kind: Kind::Indicator {
                            constant: form.constant,
                            sign,
                        },
                    });
                }
            }
            (None, None) => {
                let lhs = instance.native(c.lower)?;
                let rhs = instance.native(c.upper)?;
                let mut cons = ptr::null_mut();
                native!(
                    "SCIPcreateConsBasicNonlinear",
                    ffi::SCIPcreateConsBasicNonlinear(
                        instance.ptr(),
                        &mut cons,
                        name.as_ptr(),
                        node()?,
                        lhs,
                        rhs
                    )
                )?;
                instance.hold(cons)?;
                nonlinear += 1;
                constraints.push(Exported {
                    cons,
                    expression: c.expression,
                    lower: c.lower,
                    upper: c.upper,
                    kind: Kind::Nonlinear,
                });
            }
            (None, Some(binvar)) => {
                // A nonlinear row under an indicator is lifted exactly, as SCIP lifts a
                // linear one: f − s ≤ u and f + t ≥ l with slacks s, t ≥ 0 that the
                // literal forces to zero. SCIP 10.0.2's superindicator over a nonlinear
                // constraint crashes during solving, so it is not used.
                for (upper, side) in [(true, c.upper), (false, c.lower)] {
                    if !side.is_finite() {
                        continue;
                    }
                    let suffix = if upper { "u" } else { "l" };
                    let slack = instance.variable(
                        &format!("s{k}{suffix}"),
                        (0.0, f64::INFINITY),
                        0.0,
                        ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS,
                    )?;
                    let mut slack_expr = ptr::null_mut();
                    native!(
                        "SCIPcreateExprVar",
                        ffi::SCIPcreateExprVar(
                            instance.ptr(),
                            &mut slack_expr,
                            slack,
                            None,
                            ptr::null_mut()
                        )
                    )?;
                    let mut terms = [node()?, slack_expr];
                    let mut weights = [1.0, if upper { -1.0 } else { 1.0 }];
                    let mut expr = ptr::null_mut();
                    let created = native!(
                        "SCIPcreateExprSum",
                        ffi::SCIPcreateExprSum(
                            instance.ptr(),
                            &mut expr,
                            2,
                            terms.as_mut_ptr(),
                            weights.as_mut_ptr(),
                            0.0,
                            None,
                            ptr::null_mut()
                        )
                    );
                    // SAFETY: releases the slack expression's creation reference; the sum
                    // holds one.
                    let _ = unsafe { ffi::SCIPreleaseExpr(instance.ptr(), &mut slack_expr) };
                    created?;
                    let (lhs, rhs) = if upper {
                        (-instance.infinity, instance.native(side)?)
                    } else {
                        (instance.native(side)?, instance.infinity)
                    };
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
                    // SAFETY: releases the lifted expression's creation reference.
                    let _ = unsafe { ffi::SCIPreleaseExpr(instance.ptr(), &mut expr) };
                    created?;
                    instance.hold(cons)?;
                    nonlinear += 1;
                    // Read back at zero slack: the lifted function is the row's.
                    constraints.push(Exported {
                        cons,
                        expression: c.expression,
                        lower: if upper { f64::NEG_INFINITY } else { c.lower },
                        upper: if upper { c.upper } else { f64::INFINITY },
                        kind: Kind::Nonlinear,
                    });
                    let mut vars = [slack];
                    let mut vals = [1.0];
                    let mut switch = ptr::null_mut();
                    native!(
                        "SCIPcreateConsBasicIndicator",
                        ffi::SCIPcreateConsBasicIndicator(
                            instance.ptr(),
                            &mut switch,
                            name.as_ptr(),
                            binvar,
                            1,
                            vars.as_mut_ptr(),
                            vals.as_mut_ptr(),
                            0.0
                        )
                    )?;
                    instance.hold(switch)?;
                    linear += 1;
                }
            }
        }
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
            expression: Expression::Node(node),
            lower: f64::NEG_INFINITY,
            upper: f64::INFINITY,
            kind: Kind::Epigraph,
        });
    }
    drop(exprs);
    let native = export_native(instance, plan, &coordinates)?;
    Ok(Export {
        coordinates,
        semi: plan.semi.clone(),
        epigraph,
        constraints,
        objective: affine_objective.map(|(node, _)| node),
        offset,
        linear,
        nonlinear,
        native,
    })
}
fn objsense(sense: ObjectiveSense) -> ffi::SCIP_OBJSENSE {
    match sense {
        ObjectiveSense::Minimize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MINIMIZE,
        ObjectiveSense::Maximize => ffi::SCIP_Objsense_SCIP_OBJSENSE_MAXIMIZE,
    }
}
/// SOS1/SOS2, and/or/xor and cardinality constraints, named `n{k}` after the native
/// ordinal. A fixed operand becomes a fixed variable.
fn export_native(
    instance: &mut Instance,
    plan: &Plan<'_>,
    coordinates: &[*mut ffi::SCIP_VAR],
) -> Result<usize, ProblemError> {
    use pse_math::factorable::{NativeOperand, ProjectedNative};
    use pse_model::generated::enums::NativeConstraintForm as F;
    let program = plan.program;
    let mut fixed = 0_usize;
    let mut var = |instance: &mut Instance, o: &NativeOperand, binary: bool| match o {
        NativeOperand::Column(c) => Ok(coordinates[*c]),
        NativeOperand::Fixed(v) => {
            fixed += 1;
            instance.variable(
                &format!("f{fixed}"),
                (*v, *v),
                0.0,
                if binary {
                    ffi::SCIP_Vartype_SCIP_VARTYPE_BINARY
                } else {
                    ffi::SCIP_Vartype_SCIP_VARTYPE_CONTINUOUS
                },
            )
        }
    };
    for k in &plan.native {
        let name = cstring(&format!("n{k}"))?;
        let mut cons = ptr::null_mut();
        match &program.native[*k] {
            ProjectedNative::Indicator { .. } => continue,
            ProjectedNative::Sos { form, members } => {
                let mut vars = members
                    .iter()
                    .map(|(o, _)| var(instance, o, false))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut weights: Vec<f64> = members.iter().map(|(_, w)| *w).collect();
                let n = arity(vars.len())?;
                if *form == F::Sos2 {
                    native!(
                        "SCIPcreateConsBasicSOS2",
                        ffi::SCIPcreateConsBasicSOS2(
                            instance.ptr(),
                            &mut cons,
                            name.as_ptr(),
                            n,
                            vars.as_mut_ptr(),
                            weights.as_mut_ptr()
                        )
                    )?;
                } else {
                    native!(
                        "SCIPcreateConsBasicSOS1",
                        ffi::SCIPcreateConsBasicSOS1(
                            instance.ptr(),
                            &mut cons,
                            name.as_ptr(),
                            n,
                            vars.as_mut_ptr(),
                            weights.as_mut_ptr()
                        )
                    )?;
                }
            }
            ProjectedNative::Cardinality { members, bound } => {
                let mut vars = members
                    .iter()
                    .map(|o| var(instance, o, false))
                    .collect::<Result<Vec<_>, _>>()?;
                let cardinality = i32::try_from(*bound)
                    .map_err(|_| ProblemError::Unsupported("SCIP cardinality bound".into()))?;
                // Explicit weights: SCIP 10.0.2 copies a cardinality constraint into
                // sub-SCIPs by duplicating its weights, which are null when none are given.
                let mut weights: Vec<f64> = (1..=vars.len()).map(|w| w as f64).collect();
                native!(
                    "SCIPcreateConsBasicCardinality",
                    ffi::SCIPcreateConsBasicCardinality(
                        instance.ptr(),
                        &mut cons,
                        name.as_ptr(),
                        arity(vars.len())?,
                        vars.as_mut_ptr(),
                        cardinality,
                        ptr::null_mut(),
                        weights.as_mut_ptr()
                    )
                )?;
            }
            ProjectedNative::Logic {
                form,
                resultant,
                operands,
            } => {
                let resultant = var(instance, resultant, true)?;
                let mut vars = Vec::with_capacity(operands.len() + 1);
                for (o, negated) in operands {
                    let v = var(instance, o, true)?;
                    vars.push(instance.literal(v, *negated)?);
                }
                match form {
                    F::And => native!(
                        "SCIPcreateConsBasicAnd",
                        ffi::SCIPcreateConsBasicAnd(
                            instance.ptr(),
                            &mut cons,
                            name.as_ptr(),
                            resultant,
                            arity(vars.len())?,
                            vars.as_mut_ptr()
                        )
                    )?,
                    F::Or => native!(
                        "SCIPcreateConsBasicOr",
                        ffi::SCIPcreateConsBasicOr(
                            instance.ptr(),
                            &mut cons,
                            name.as_ptr(),
                            resultant,
                            arity(vars.len())?,
                            vars.as_mut_ptr()
                        )
                    )?,
                    _ => {
                        // z = a xor b as the parity constraint z xor a xor b = 0.
                        vars.insert(0, resultant);
                        native!(
                            "SCIPcreateConsBasicXor",
                            ffi::SCIPcreateConsBasicXor(
                                instance.ptr(),
                                &mut cons,
                                name.as_ptr(),
                                0,
                                arity(vars.len())?,
                                vars.as_mut_ptr()
                            )
                        )?;
                    }
                }
            }
        }
        instance.hold(cons)?;
    }
    Ok(plan.native.len())
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
/// Activity of `n` terms at the point, skipping `skip` (an indicator's slack variable).
///
/// # Safety
/// `vars` and `vals` hold `n` initialized entries owned by a live constraint.
unsafe fn activity(
    point: &BTreeMap<*mut ffi::SCIP_VAR, f64>,
    vars: *mut *mut ffi::SCIP_VAR,
    vals: impl Fn(usize) -> f64,
    n: i32,
    skip: *mut ffi::SCIP_VAR,
) -> Result<f64, ProblemError> {
    let n = usize::try_from(n).map_err(|_| ProblemError::internal("SCIP arity"))?;
    if n == 0 {
        return Ok(0.0);
    }
    if vars.is_null() {
        return Err(ProblemError::internal("SCIP linear readback"));
    }
    // SAFETY: the caller guarantees `n` initialized entries.
    let vars = unsafe { std::slice::from_raw_parts(vars, n) };
    let mut total = 0.0;
    for (i, var) in vars.iter().enumerate() {
        if *var == skip {
            continue;
        }
        let value = point
            .get(var)
            .ok_or_else(|| ProblemError::internal("SCIP readback of an unexported variable"))?;
        total += vals(i) * value;
    }
    Ok(total)
}
/// Largest relative deviation between the native model and the neutral program of `plan`
/// at one point, with each lowered semi column's indicator at its nearest branch: sides
/// and function values of every exported constraint, and the objective. An undefined
/// value on both sides agrees; on one side only it is an infinite deviation.
pub(crate) fn readback(
    instance: &Instance,
    export: &Export,
    plan: &Plan<'_>,
    point: &[f64],
    auxiliary: &[f64],
) -> Result<f64, ProblemError> {
    let values = plan.program.evaluate(point, auxiliary)?;
    let coordinates = plan.coordinates(point, auxiliary);
    let solution = Solution::new(instance, false)?;
    let mut at: BTreeMap<*mut ffi::SCIP_VAR, f64> = BTreeMap::new();
    for (var, value) in export.coordinates.iter().zip(&coordinates) {
        solution.set(*var, *value)?;
        at.insert(*var, *value);
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
    let nonlinear = |cons: *mut ffi::SCIP_CONS| -> Result<(f64, f64, f64), ProblemError> {
        // SAFETY: the constraint is a live nonlinear constraint held by the instance.
        let expr = unsafe { ffi::SCIPgetExprNonlinear(cons) };
        native!("SCIPevalExpr", ffi::SCIPevalExpr(s, expr, solution.sol, 0))?;
        // SAFETY: reads the value SCIP just stored in the live expression; side queries
        // of the same live constraint.
        Ok(unsafe {
            (
                ffi::SCIPexprGetEvalValue(expr),
                ffi::SCIPgetLhsNonlinear(cons),
                ffi::SCIPgetRhsNonlinear(cons),
            )
        })
    };
    let mut worst: f64 = 0.0;
    for c in &export.constraints {
        let neutral = plan.value(c.expression, &values, &coordinates);
        match c.kind {
            Kind::Linear { constant } => {
                // SAFETY: a live linear constraint held by the instance; its arrays have
                // the reported length.
                let (a, lhs, rhs) = unsafe {
                    let n = ffi::SCIPgetNVarsLinear(s, c.cons);
                    let vals = ffi::SCIPgetValsLinear(s, c.cons);
                    (
                        activity(
                            &at,
                            ffi::SCIPgetVarsLinear(s, c.cons),
                            |i| *vals.add(i),
                            n,
                            ptr::null_mut(),
                        )?,
                        ffi::SCIPgetLhsLinear(s, c.cons),
                        ffi::SCIPgetRhsLinear(s, c.cons),
                    )
                };
                worst = worst
                    .max(deviation(a + constant, neutral))
                    .max(side(lhs + constant, c.lower))
                    .max(side(rhs + constant, c.upper));
            }
            Kind::ExactLinear { constant } => {
                // SAFETY: a live exact linear constraint held by the instance; its arrays
                // have the reported length and its rationals stay live.
                let (a, lhs, rhs) = unsafe {
                    let n = ffi::SCIPgetNVarsExactLinear(s, c.cons);
                    let vals = ffi::SCIPgetValsExactLinear(s, c.cons);
                    (
                        activity(
                            &at,
                            ffi::SCIPgetVarsExactLinear(s, c.cons),
                            |i| ffi::SCIPrationalGetReal(*vals.add(i)),
                            n,
                            ptr::null_mut(),
                        )?,
                        ffi::SCIPrationalGetReal(ffi::SCIPgetLhsExactLinear(s, c.cons)),
                        ffi::SCIPrationalGetReal(ffi::SCIPgetRhsExactLinear(s, c.cons)),
                    )
                };
                worst = worst
                    .max(deviation(a + constant, neutral))
                    .max(side(lhs + constant, c.lower))
                    .max(side(rhs + constant, c.upper));
            }
            Kind::Indicator { constant, sign } => {
                // SAFETY: a live indicator constraint; its linear constraint and slack
                // variable are owned by it, and their arrays have the reported length.
                let (a, rhs) = unsafe {
                    let lin = ffi::SCIPgetLinearConsIndicator(c.cons);
                    if lin.is_null() {
                        return Err(ProblemError::internal("SCIP indicator readback"));
                    }
                    let slack = ffi::SCIPgetSlackVarIndicator(c.cons);
                    let n = ffi::SCIPgetNVarsLinear(s, lin);
                    let vals = ffi::SCIPgetValsLinear(s, lin);
                    (
                        activity(
                            &at,
                            ffi::SCIPgetVarsLinear(s, lin),
                            |i| *vals.add(i),
                            n,
                            slack,
                        )?,
                        ffi::SCIPgetRhsLinear(s, lin),
                    )
                };
                let (bound, expected) = if sign > 0.0 {
                    (rhs + constant, c.upper)
                } else {
                    (constant - rhs, c.lower)
                };
                worst = worst
                    .max(deviation(sign * a + constant, neutral))
                    .max(side(bound, expected));
            }
            Kind::Nonlinear => {
                let (native, lhs, rhs) = nonlinear(c.cons)?;
                worst = worst
                    .max(deviation(native, neutral))
                    .max(side(lhs, c.lower))
                    .max(side(rhs, c.upper));
            }
            Kind::Epigraph => {
                let (native, ..) = nonlinear(c.cons)?;
                worst = worst.max(deviation(native, neutral));
            }
        }
    }
    if let Some(node) = export.objective {
        // SAFETY: a value query of the live instance's objective offset.
        let mut native = unsafe { ffi::SCIPgetOrigObjoffset(s) } + export.offset;
        for (var, value) in export.coordinates.iter().zip(&coordinates) {
            // SAFETY: an objective-coefficient query of a held variable.
            native += unsafe { ffi::SCIPvarGetObj(*var) } * value;
        }
        worst = worst.max(deviation(native, values[node]));
    }
    Ok(worst)
}
/// Submit a primal incumbent in program columns, each lowered semi column's indicator at
/// its nearest branch. A program with auxiliaries or an epigraph gets a partial solution,
/// completed by SCIP; returns whether SCIP stored it.
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
    let first = export.coordinates.len() - export.semi.len();
    for (s, var) in export.semi.iter().zip(&export.coordinates[first..]) {
        solution.set(*var, f64::from(u8::from(s.active(primal[s.column]))))?;
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

/// Identity of the constraint system a reoptimization session was built for: every
/// exported function's affine form, sides and condition, the box, the domains and the
/// native forms. Only the objective may change between the steps of one session.
fn system(plan: &Plan<'_>) -> Result<ContentHash, ProblemError> {
    let mut h = FramedHasher::new(pse_ids::Frame::ScipReoptimizationSystemV1);
    h.hash(&plan.domain);
    h.u64(plan.constraints.len() as u64);
    for c in &plan.constraints {
        let form = plan.form(c.expression).ok_or_else(|| {
            ProblemError::Unsupported("SCIP reoptimization admits linear constraints only".into())
        })?;
        h.u64(pse_ids::canonical_f64_bits(c.lower))
            .u64(pse_ids::canonical_f64_bits(c.upper))
            .u64(pse_ids::canonical_f64_bits(form.constant))
            .u64(form.terms.len() as u64);
        for (j, v) in &form.terms {
            h.u64(*j as u64).u64(pse_ids::canonical_f64_bits(*v));
        }
        match c.condition {
            Some(k) => h.bool(true).u64(k.column as u64).bool(k.active),
            None => h.bool(false),
        };
    }
    h.u64(plan.native.len() as u64);
    for k in &plan.native {
        h.u64(*k as u64);
    }
    Ok(h.finish_hash())
}
/// A reoptimization session retained across the steps of a finite MIP sequence.
#[derive(Debug)]
struct Session {
    instance: Instance,
    export: Export,
    system: ContentHash,
    options: Options,
}
impl Session {
    /// Install the step's objective in the retained problem (`SCIPchgReoptObjective`).
    fn reoptimize(
        &mut self,
        plan: &Plan<'_>,
        execution: &Execution,
    ) -> Result<(), ProblemError> {
        let s = self.instance.ptr();
        native!("SCIPfreeReoptSolve", ffi::SCIPfreeReoptSolve(s))?;
        let (sense, form) = match plan.objective {
            Some((node, sense)) => (
                sense,
                plan.affine[node].as_ref().ok_or_else(|| {
                    ProblemError::Unsupported(
                        "SCIP reoptimization admits a linear objective only".into(),
                    )
                })?,
            ),
            None => {
                return Err(ProblemError::Unsupported(
                    "SCIP reoptimization needs an objective".into(),
                ));
            }
        };
        let mut vars = self.export.coordinates.clone();
        let mut coefs = coefficients(form, vars.len());
        native!(
            "SCIPchgReoptObjective",
            ffi::SCIPchgReoptObjective(
                s,
                objsense(sense),
                vars.as_mut_ptr(),
                coefs.as_mut_ptr(),
                arity(vars.len())?
            )
        )?;
        self.export.offset = form.constant;
        self.export.objective = plan.objective.map(|(node, _)| node);
        self.instance.rewatch(execution);
        let remaining = execution
            .time_limit
            .saturating_sub(execution.started.elapsed())
            .as_secs_f64();
        self.instance.set_real("limits/time", remaining)?;
        self.options.insert(
            "limits/time".into(),
            self.instance
                .option("limits/time", &OptionValue::Real(0.0))?,
        );
        Ok(())
    }
}
/// Build an instance, configure it and export the plan.
fn build(
    r: &Request<'_>,
    plan: &Plan<'_>,
    gap_absolute: f64,
) -> Result<Session, ProblemError> {
    let mut instance = Instance::new(
        r.execution,
        Modes {
            exact: r.settings.exact,
            reoptimize: r.settings.reoptimize,
        },
    )?;
    let options = configure(
        &instance,
        r.settings,
        r.controls,
        r.accuracy,
        r.execution,
        gap_absolute,
    )?;
    let export = export(&mut instance, plan)?;
    let system = if r.settings.reoptimize {
        system(plan)?
    } else {
        ContentHash::from_bytes([0; 32])
    };
    Ok(Session {
        instance,
        export,
        system,
        options,
    })
}
/// The infeasible subsystem SCIP's IIS finders leave in their sub-problem, attributed to
/// exported functions, native forms and declared bounds by native name. Constraints are
/// minimized; the declared bounds of the variables the kept constraints use stay members.
fn iis(
    session: &Session,
    plan: &Plan<'_>,
    execution: &Execution,
) -> Result<Option<Iis>, ProblemError> {
    use crate::execution::factorable::Origin;
    let program = plan.program;
    let instance = &session.instance;
    let remaining = execution
        .time_limit
        .saturating_sub(execution.started.elapsed())
        .as_secs_f64();
    instance.set_real("iis/time", remaining)?;
    native!("SCIPgenerateIIS", ffi::SCIPgenerateIIS(instance.ptr()))?;
    // SAFETY: the IIS storage of the live instance.
    let storage = unsafe { ffi::SCIPgetIIS(instance.ptr()) };
    if storage.is_null() {
        return Ok(None);
    }
    // SAFETY: flag and sub-problem queries of the live IIS storage.
    let (infeasible, irreducible, sub) = unsafe {
        (
            ffi::SCIPiisIsSubscipInfeasible(storage) != 0,
            ffi::SCIPiisIsSubscipIrreducible(storage) != 0,
            ffi::SCIPiisGetSubscip(storage),
        )
    };
    if !infeasible || sub.is_null() {
        return Ok(None);
    }
    let name = |raw: *const c_char| -> String {
        if raw.is_null() {
            return String::new();
        }
        // SAFETY: SCIP returns NUL-terminated names owned by the live sub-problem.
        unsafe { CStr::from_ptr(raw) }
            .to_string_lossy()
            .into_owned()
    };
    let ordinal = |text: &str, prefix: char| -> Option<usize> {
        text.strip_prefix(prefix).and_then(|n| n.parse().ok())
    };
    let mut members = BTreeSet::new();
    // SAFETY: the sub-problem's original constraints, with the reported count.
    let conss = unsafe {
        let n = usize::try_from(ffi::SCIPgetNOrigConss(sub)).unwrap_or(0);
        let conss = ffi::SCIPgetOrigConss(sub);
        if n == 0 || conss.is_null() {
            &[][..]
        } else {
            std::slice::from_raw_parts(conss, n)
        }
    };
    for cons in conss {
        // SAFETY: a live constraint of the sub-problem.
        let text = name(unsafe { ffi::SCIPconsGetName(*cons) });
        if let Some(k) = ordinal(&text, 'c') {
            let Some(f) = plan.constraints.get(k) else {
                continue;
            };
            members.insert(match f.origin {
                Origin::Row(r) => IisMember::Row(program.rows[r].id),
                Origin::Obligation(o, _) => IisMember::Obligation {
                    instance: program.obligations[o].instance,
                    source: program.obligations[o].source,
                },
                Origin::Residual(b, k) => IisMember::Residual {
                    instance: program.implicit[b].instance,
                    ordinal: k,
                },
                Origin::ImplicitBound(b, k) => IisMember::ImplicitBound {
                    instance: program.implicit[b].instance,
                    ordinal: k,
                },
                // A link is its semi column's domain: `x − u·z ≤ 0` carries the upper
                // bound and `x − l·z ≥ 0` the active lower bound.
                Origin::SemiLink { semi, upper } => {
                    let id = program.variables[plan.semi[semi].column].id;
                    if upper {
                        IisMember::VariableUpper(id)
                    } else {
                        IisMember::VariableLower(id)
                    }
                }
            });
        } else if let Some(k) = ordinal(&text, 'n') {
            members.insert(IisMember::Native(k));
        }
    }
    // SAFETY: the sub-problem's original variables, with the reported count.
    let vars = unsafe {
        let n = usize::try_from(ffi::SCIPgetNOrigVars(sub)).unwrap_or(0);
        let vars = ffi::SCIPgetOrigVars(sub);
        if n == 0 || vars.is_null() {
            &[][..]
        } else {
            std::slice::from_raw_parts(vars, n)
        }
    };
    // SAFETY: an infinity query of the live sub-problem.
    let infinity = unsafe { ffi::SCIPinfinity(sub) };
    let columns = program.variables.len();
    for var in vars {
        // SAFETY: name and original-bound queries of a live sub-problem variable.
        let (text, lower, upper) = unsafe {
            (
                name(ffi::SCIPvarGetName(*var)),
                ffi::SCIPvarGetLbOriginal(*var),
                ffi::SCIPvarGetUbOriginal(*var),
            )
        };
        let (low, high) = if let Some(i) = ordinal(&text, 'x').filter(|i| *i < columns) {
            let id = program.variables[i].id;
            (IisMember::VariableLower(id), IisMember::VariableUpper(id))
        } else if let Some(k) = ordinal(&text, 'a') {
            (IisMember::AuxiliaryLower(k), IisMember::AuxiliaryUpper(k))
        } else {
            continue;
        };
        if lower > -infinity {
            members.insert(low);
        }
        if upper < infinity {
            members.insert(high);
        }
    }
    Ok(Some(Iis {
        irreducible,
        members: members.into_iter().collect(),
    }))
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
/// belongs to the factorable runner. A reoptimization session is retained on the owning
/// worker across a finite sequence; every other attempt builds and frees its instance.
///
/// # Errors
/// ABI, export, option or native failures before a report exists.
pub(crate) fn solve(
    r: &Request<'_>,
    retained: &mut crate::execution::Retained,
) -> Result<SolveReport, ProblemError> {
    r.controls.validate()?;
    r.settings.admit(r.controls.threads)?;
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
    let mut local = None;
    let (session, reoptimized) = if r.settings.reoptimize {
        let system = system(&plan)?;
        retained.session(
            Backend::Scip,
            r.controls.reuse,
            |held: &mut Session| {
                if held.system != system {
                    return Ok(false);
                }
                held.reoptimize(&plan, r.execution)?;
                Ok(true)
            },
            || build(r, &plan, gap_absolute),
        )?
    } else {
        // Nothing native is retained across attempts.
        retained.clear();
        (local.insert(build(r, &plan, gap_absolute)?), false)
    };
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
    let auxiliary: Vec<f64> = plan.boxes[columns..plan.indicator(0)]
        .iter()
        .map(|b| clamp(0.5 * (b.0 + b.1), *b))
        .collect();
    let deviation = readback(&session.instance, &session.export, &plan, &start, &auxiliary)?;
    let submitted = match r.warm {
        // Exact solving certifies its own solutions; a floating-point seed is not given.
        Some(_) if r.settings.exact => None,
        Some(seed) => {
            seed.validate(r.compatibility)?;
            let WarmPayload::Nlp { primal, .. } = &seed.payload else {
                return Err(ProblemError::Unsupported(
                    "SCIP consumes a primal incumbent only".into(),
                ));
            };
            Some(inject(&session.instance, &session.export, program, primal)?)
        }
        None => None,
    };
    session
        .instance
        .report_incumbents(&session.export, columns, plan.objective.is_some());
    let s = session.instance.ptr();
    if r.controls.threads > 1 {
        native!("SCIPsolveConcurrent", ffi::SCIPsolveConcurrent(s))?;
    } else {
        native!("SCIPsolve", ffi::SCIPsolve(s))?;
    }
    session.instance.flush_incumbent();
    // SAFETY: a status query after the solve returned.
    let raw = unsafe { ffi::SCIPgetStatus(s) };
    let status = Status::from_raw(raw).ok_or_else(|| {
        ProblemError::Internal(format!("SCIP status {raw} outside the 10.0.2 ABI"))
    })?;
    let instance = &session.instance;
    let export = &session.export;
    let offset = export.offset;
    // SAFETY: bound and statistic queries after the solve returned.
    let primal_bound = instance
        .finite(unsafe { ffi::SCIPgetPrimalbound(s) })
        .map(|v| v + offset);
    // SAFETY: as above.
    let dual_bound = instance
        .finite(unsafe { ffi::SCIPgetDualbound(s) })
        .map(|v| v + offset);
    // SAFETY: as above.
    let gap = instance.finite(unsafe { ffi::SCIPgetGap(s) });
    // SAFETY: as above.
    let nodes = unsafe { ffi::SCIPgetNTotalNodes(s) };
    // SAFETY: as above.
    let best = unsafe { ffi::SCIPgetBestSol(s) };
    let values = |sol: *mut ffi::SCIP_SOL| -> Vec<f64> {
        export.coordinates[..columns]
            .iter()
            // SAFETY: value queries of held variables in a stored solution of this instance.
            .map(|var| unsafe { ffi::SCIPgetSolVal(s, sol, *var) })
            .collect()
    };
    // SAFETY: the original objective of a stored solution.
    let objective_of = |sol| instance.finite(unsafe { ffi::SCIPgetSolOrigObj(s, sol) } + offset);
    let candidate = (!best.is_null()).then(|| Candidate {
        kind: CandidateKind::FeasiblePoint,
        primal: values(best),
        objective: plan.objective.and(objective_of(best)),
        row_dual: None,
        bound_dual: None,
        reduced_costs: None,
        slacks: None,
    });
    // SAFETY: a mode query of the live instance.
    let exact = r.settings.exact
        && unsafe { ffi::SCIPisExact(s) } != 0
        && matches!(status, Status::Optimal | Status::Infeasible);
    let exact_objective = if exact && !best.is_null() && plan.objective.is_some() {
        let rational = Rational::new(0.0)?;
        // SAFETY: the exact original objective of SCIP's best solution.
        unsafe { ffi::SCIPgetSolOrigObjExact(s, best, rational.0) };
        Some(rational.text())
    } else {
        None
    };
    let pool = if r.settings.pool > 0 {
        // SAFETY: the stored solutions, best first, with the reported count.
        let stored = unsafe {
            let n = usize::try_from(ffi::SCIPgetNSols(s)).unwrap_or(0);
            let sols = ffi::SCIPgetSols(s);
            if n == 0 || sols.is_null() {
                &[][..]
            } else {
                std::slice::from_raw_parts(sols, n)
            }
        };
        stored
            .iter()
            .take(usize::from(r.settings.pool))
            .enumerate()
            .map(|(rank, sol)| PoolSolution {
                rank,
                primal: values(*sol),
                objective: plan.objective.and(objective_of(*sol)),
                feasible: None,
            })
            .collect()
    } else {
        vec![]
    };
    let subsystem = if r.settings.iis && status == Status::Infeasible {
        iis(session, &plan, r.execution)?
    } else {
        None
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
    report.evidence.reused_native_state = reoptimized;
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
        exact,
    });
    report.global = Some(Arc::new(GlobalRecord {
        boxes: plan.boxes[..plan.indicator(0)].to_vec(),
        pool,
        iis: subsystem,
        exact_objective,
        reoptimized,
        transformations: plan.transformations(),
    }));
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
                Metric::Unavailable(UnavailableReason::NotApplicable),
                Metric::Real,
            ),
        );
    }
    metrics.insert(
        "scip.events".into(),
        Metric::Integer(i64::try_from(watch.events.load(Ordering::Acquire)).unwrap_or(i64::MAX)),
    );
    metrics.insert(
        "scip.interrupted".into(),
        Metric::Bool(watch.interrupted.load(Ordering::Acquire)),
    );
    metrics.insert(
        "scip.callback_failed".into(),
        Metric::Bool(watch.failed.load(Ordering::Acquire)),
    );
    metrics.insert(
        "scip.threads".into(),
        Metric::Integer(i64::try_from(r.controls.threads).unwrap_or(i64::MAX)),
    );
    metrics.insert("scip.exact".into(), Metric::Bool(exact));
    // The conditions every global claim holds under (ADR-0106 §9-§10): tolerances, the
    // identity of the declared box and the export fidelity below.
    for (key, value) in [
        ("global.feasibility", feasibility(r.accuracy)),
        ("global.gap_relative", r.accuracy.mip_relative_gap),
        ("global.gap_absolute", gap_absolute),
    ] {
        metrics.insert(key.into(), Metric::Real(value));
    }
    metrics.insert(
        "global.domain".into(),
        Metric::Text(plan.domain.to_string()),
    );
    metrics.insert("scip.reoptimized".into(), Metric::Bool(reoptimized));
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
    for (key, count) in [
        ("export.constraints.linear", export.linear),
        ("export.constraints.nonlinear", export.nonlinear),
        ("export.constraints.native", export.native),
    ] {
        metrics.insert(
            key.into(),
            Metric::Integer(i64::try_from(count).unwrap_or(i64::MAX)),
        );
    }
    metrics.insert("export.readback.deviation".into(), Metric::Real(deviation));
    if let Some(stored) = submitted {
        metrics.insert("scip.incumbent.stored".into(), Metric::Bool(stored));
    }
    report.options = session.options.clone();
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
        payload: WarmPayload::primal(c.primal.clone()),
    });
    report.candidate = candidate;
    drop(local);
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
        let mut instance = Instance::new(execution, Modes::default())?;
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
        Ok((
            status,
            watch.events.load(Ordering::Acquire),
            watch.interrupted.load(Ordering::Acquire),
        ))
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
        let mut instance = Instance::new(&execution, Modes::default())?;
        let export = export(&mut instance, &plan)?;
        let mut worst: f64 = 0.0;
        for (x, a) in points {
            worst = worst.max(readback(&instance, &export, &plan, x, a)?);
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
        let reference = crate::execution::factorable::plan(checked, SolveIntent::FeasiblePoint)
            .map_err(|r| ProblemError::Unsupported(format!("{r:?}")))?;
        let execution = Execution::new(Arc::default(), &Controls::default());
        let mut instance = Instance::new(&execution, Modes::default())?;
        let export = export(&mut instance, &plan)?;
        let mut worst: f64 = 0.0;
        for (x, a) in points {
            worst = worst.max(readback(&instance, &export, &reference, x, a)?);
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
        let instance = Instance::new(&execution, Modes::default())?;
        configure(&instance, settings, controls, accuracy, &execution, 1e-6)
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected copies at actual IDAS returns, never at an ordinary statistics read.
use super::*;
use crate::solve::{Event, EventSource, Metric};
use pse_diagnostics::observation::{OwnerReservation, Producer, ProducerRole};
use std::collections::BTreeMap;

const FAMILY: &str = "dynamic_local_error.v1";
const CONTRIBUTORS: usize = 8;

pub(super) struct ReturnCheckpoint<'a> {
    pub(super) generation: u64,
    pub(super) flag: i32,
    pub(super) returned_time: f64,
    pub(super) suppress_alg: bool,
    pub(super) phase: &'a str,
}

struct Scratch {
    context: ffi::SUNContext,
    weights: ffi::N_Vector,
    errors: ffi::N_Vector,
    _reservation: OwnerReservation,
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.weights.is_null() {
            // SAFETY: this unattached vector belongs exclusively to this owner.
            unsafe {
                ffi::N_VDestroy(self.weights);
            }
        }
        if !self.errors.is_null() {
            // SAFETY: this distinct unattached vector is released exactly once.
            unsafe {
                ffi::N_VDestroy(self.errors);
            }
        }
        if !self.context.is_null() {
            // SAFETY: both vector consumers have been freed before their context.
            unsafe {
                ffi::SUNContext_Free(&raw mut self.context);
            }
        }
    }
}

pub(super) struct LocalError {
    producer: Producer,
    scratch: Option<Scratch>,
    /// Every actual native return advances the watermark, including failed returns.
    /// A later interpolation-only success cannot revive a failed step's estimate.
    seen: Option<(u64, c_long)>,
    return_ordinal: u64,
}
impl LocalError {
    /// Optional diagnostic allowance is independent of numerical admission. Refusal
    /// is observable but cannot turn a scientifically admitted solve into a failure.
    pub(super) fn prepare(n: usize) -> Option<Self> {
        let observation = pse_diagnostics::observation::current()?;
        let tap = observation.record_tap()?;
        if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tap.selected(FAMILY)))
            .unwrap_or(false)
        {
            return None;
        }
        let producer = observation.producer(ProducerRole::NativeCallback, "idas.local_error");
        if !producer.detail_selected(FAMILY) {
            return None;
        }
        producer.activate_record_family(FAMILY);
        // Two N-entry native vectors plus their opaque allocation headers. The
        // consumed 7.9.0 vector/ops/content payloads occupy 1008 bytes on this ABI;
        // 4096 admits those headers, a private context/logger/handler and headroom.
        // No Rust vector or second full copy is retained. Ranking uses stack storage.
        let scratch = n
            .checked_mul(2 * size_of::<f64>())
            .and_then(|bytes| bytes.checked_add(4096))
            .and_then(|bytes| producer.reserve_owner_bytes(bytes))
            .and_then(|reservation| {
                let n = index(n).ok()?;
                let mut scratch = Scratch {
                    context: std::ptr::null_mut(),
                    weights: std::ptr::null_mut(),
                    errors: std::ptr::null_mut(),
                    _reservation: reservation,
                };
                // A failed optional vector allocation must not set the scientific
                // context's last_err. This context owns diagnostic allocations only.
                // SAFETY: the local output is writable; no MPI is used by serial copies.
                if unsafe { ffi::SUNContext_Create(ffi::comm_no_mpi(), &raw mut scratch.context) }
                    != 0
                {
                    return None;
                }
                // SAFETY: scratch retains its private context through destruction.
                // Both serial vectors have the exact contract state extent.
                scratch.weights = unsafe { ffi::N_VNew_Serial(n, scratch.context) };
                // SAFETY: the same private context owns the matching second vector.
                scratch.errors = unsafe { ffi::N_VNew_Serial(n, scratch.context) };
                (!scratch.weights.is_null() && !scratch.errors.is_null()).then_some(scratch)
            });
        if scratch.is_none() {
            observation.record_source_unavailable(FAMILY);
        }
        Some(Self {
            producer,
            scratch,
            seen: None,
            return_ordinal: 0,
        })
    }

    /// The caller owns live initialized IDAS memory, on its solving thread, just
    /// after an actual IDASolve/IDASolveF return. `suppress_alg` is the setting the
    /// performing owner actually installed, not inferred from the differential map.
    pub(super) unsafe fn returned(
        &mut self,
        mem: *mut c_void,
        contract: &Contract,
        checkpoint: ReturnCheckpoint<'_>,
        execution: &Execution,
    ) {
        let captured = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // SAFETY: this boundary forwards the caller's live owner-thread handle,
            // contract and actual return without changing any native input or status.
            unsafe {
                self.capture(mem, contract, checkpoint, execution);
            }
        }));
        if captured.is_err() {
            // A selected diagnostic failure must not replace the native outcome.
            // This marking contains user sink failures itself and preserves loss.
            self.producer.context().record_source_unavailable(FAMILY);
        }
    }

    unsafe fn capture(
        &mut self,
        mem: *mut c_void,
        contract: &Contract,
        checkpoint: ReturnCheckpoint<'_>,
        execution: &Execution,
    ) {
        let ReturnCheckpoint {
            generation,
            flag,
            returned_time,
            suppress_alg,
            phase,
        } = checkpoint;
        if !self.producer.detail_selected(FAMILY) {
            // Ended selection releases both admitted copies immediately. Selection
            // starting later requires a newly prepared attempt; no lazy allocation.
            self.scratch = None;
            return;
        }
        self.return_ordinal = self.return_ordinal.saturating_add(1);
        macro_rules! output {
            ($get:ident, $type:ty) => {{
                let mut value: $type = Default::default();
                // SAFETY: caller retains live initialized memory and each exact-type
                // scalar output is local; these optional getters initiate no work.
                (unsafe { ffi::$get(mem, &raw mut value) } == 0).then_some(value)
            }};
        }
        let steps = output!(IDAGetNumSteps, c_long);
        let previous = self.seen;
        if let Some(steps) = steps {
            self.seen = Some((generation, steps));
        }
        let n = contract.states.len();
        let advanced = steps.is_some_and(|steps| steps > 0);
        let fresh = steps.is_some_and(|steps| {
            steps > 0 && previous.is_none_or(|(epoch, old)| epoch != generation || steps > old)
        });
        let reason = if flag < 0 {
            Some("negative_native_return")
        } else if !advanced {
            Some("no_accepted_internal_step")
        } else if !fresh {
            Some("no_new_accepted_internal_step")
        } else if n == 0 || contract.differential.len() != n {
            Some("state_mapping_unavailable")
        } else if self.scratch.is_none() {
            Some("selected_scratch_unavailable")
        } else {
            None
        };
        let mut values = BTreeMap::from([
            ("backend".into(), Metric::Text("idas".into())),
            ("native_generation".into(), ordinal(generation)),
            ("native_return_ordinal".into(), ordinal(self.return_ordinal)),
            (
                "native_return_flag".into(),
                Metric::Integer(i64::from(flag)),
            ),
            // Negative returns need not initialize the caller's time output. The
            // independently queried native internal time retains its own meaning.
            (
                "returned_time".into(),
                if flag >= 0 {
                    Metric::Real(returned_time)
                } else {
                    Metric::Unavailable(crate::solve::UnavailableReason::NotComputed)
                },
            ),
            (
                "contract_identity".into(),
                Metric::Text(contract.identity.to_string()),
            ),
            (
                "population".into(),
                Metric::Text("state_local_error_only_not_global_physical_error".into()),
            ),
            ("suppress_algebraic".into(), Metric::Bool(suppress_alg)),
            ("norm_denominator".into(), ordinal(n as u64)),
            ("contributors_retained".into(), Metric::Integer(0)),
            ("contributors_omitted".into(), ordinal(n as u64)),
            (
                "tolerance_scale_usage".into(),
                Metric::Text("descriptive_only".into()),
            ),
            ("record_kind".into(), Metric::Text("summary".into())),
        ]);
        for (name, value) in [
            (
                "accepted_steps",
                steps.map(|v| Metric::Integer(long_counter(v))),
            ),
            (
                "internal_time",
                output!(IDAGetCurrentTime, f64).map(Metric::Real),
            ),
            (
                "last_step",
                advanced
                    .then(|| output!(IDAGetLastStep, f64))
                    .flatten()
                    .map(Metric::Real),
            ),
            (
                "last_order",
                advanced
                    .then(|| output!(IDAGetLastOrder, c_int))
                    .flatten()
                    .map(|v| Metric::Integer(i64::from(v))),
            ),
            // This descriptive scalar does not establish validity of either vector.
            (
                "tolerance_scale_factor",
                output!(IDAGetTolScaleFactor, f64).map(Metric::Real),
            ),
        ] {
            values.insert(
                name.into(),
                value.unwrap_or(Metric::Unavailable(
                    crate::solve::UnavailableReason::NotComputed,
                )),
            );
        }
        let mut contributors = [None; CONTRIBUTORS];
        let reason = reason.or_else(|| {
            let scratch = self.scratch.as_ref()?;
            // SUNDIALS 7.9.0 idas_io.c copies ida_ewt/ida_ee without checking
            // stage validity. Only the actual return/step/freshness guards above
            // authorize calling them; getter success alone never does.
            // SAFETY: guards establish a fresh nonnegative return; vectors are
            // admitted, dimensionally matched and exclusively owned through this call.
            let weights_status = unsafe { ffi::IDAGetErrWeights(mem, scratch.weights) };
            // SAFETY: the same fresh native return authorizes this second copy into
            // an exclusively owned vector with the exact state extent.
            let errors_status = unsafe { ffi::IDAGetEstLocalErrors(mem, scratch.errors) };
            if (weights_status, errors_status) != (0, 0) {
                return Some("native_vector_getter_failed");
            }
            // SAFETY: allocated serial vectors have exactly n initialized entries
            // after the successful copies. Borrowed slices do not allocate or escape.
            let weights = unsafe { ffi::N_VGetArrayPointer(scratch.weights) };
            // SAFETY: the second initialized serial vector remains live throughout.
            let errors = unsafe { ffi::N_VGetArrayPointer(scratch.errors) };
            // SAFETY: successful copy initialized exactly n entries; the slice is local.
            let weights = unsafe { std::slice::from_raw_parts(weights, n) };
            // SAFETY: the second successful copy initialized the matching n entries.
            let errors = unsafe { std::slice::from_raw_parts(errors, n) };
            contributors = project(&mut values, contract, weights, errors, suppress_alg);
            None
        });
        values.insert("source_available".into(), Metric::Bool(reason.is_none()));
        values.insert(
            "availability".into(),
            Metric::Text(reason.unwrap_or("available").into()),
        );
        let emit = |values| {
            execution.progress.push_from(Event {
            source: Some(EventSource::new(
                "dynamic_local_error", Some("contract_state_order"),
                Some("sqrt(sum(included_i*(estimated_error_i*weight_i)^2)/all_state_count)"),
            ).with_role(ProducerRole::NativeCallback)),
            phase: phase.into(),
            elapsed: execution.started.elapsed(),
            values,
            incumbent: None,
        }, &self.producer)
        };
        for (slot, entry) in contributors.iter().flatten().enumerate() {
            let mut contributor = contributor_fields(contract, *entry, slot, n);
            contributor.insert("native_generation".into(), ordinal(generation));
            contributor.insert("native_return_ordinal".into(), ordinal(self.return_ordinal));
            contributor.insert("accepted_steps".into(), values["accepted_steps"].clone());
            contributor.insert("suppress_algebraic".into(), Metric::Bool(suppress_alg));
            emit(contributor);
        }
        // Summary last keeps live latest progress meaningful. Every issued part has
        // the same native return identity, so retained loss cannot invent completeness.
        emit(values);
    }
}

fn ordinal(value: u64) -> Metric {
    i64::try_from(value).map_or_else(|_| Metric::Text(value.to_string()), Metric::Integer)
}

#[derive(Clone, Copy)]
struct Contribution {
    index: usize,
    weight: f64,
    error: f64,
    weighted: f64,
    included: bool,
}

fn project(
    values: &mut BTreeMap<String, Metric>,
    contract: &Contract,
    weights: &[f64],
    errors: &[f64],
    suppress_alg: bool,
) -> [Option<Contribution>; CONTRIBUTORS] {
    let mut top: [Option<Contribution>; CONTRIBUTORS] = [None; CONTRIBUTORS];
    let (mut sum, mut nonfinite, mut included_total) = (0., 0_u64, 0_u64);
    for (index, (&weight, &error)) in weights.iter().zip(errors).enumerate() {
        let weighted = weight * error;
        let included = !suppress_alg || contract.differential[index];
        if !weight.is_finite() || !error.is_finite() || !weighted.is_finite() {
            nonfinite += 1;
        }
        if included {
            included_total += 1;
            sum += weighted * weighted;
        }
        let candidate = Contribution {
            index,
            weight,
            error,
            weighted,
            included,
        };
        // Suppressed entries have zero error-test contribution. Nonfinite entries
        // stay visible at the front instead of silently dropping from the ranking.
        let rank = |entry: Contribution| {
            if !entry.weight.is_finite() || !entry.error.is_finite() || !entry.weighted.is_finite()
            {
                f64::INFINITY
            } else if entry.included {
                entry.weighted.abs()
            } else {
                0.
            }
        };
        if let Some(slot) = top
            .iter()
            .position(|entry| entry.is_none_or(|entry| rank(candidate) > rank(entry)))
        {
            top[slot..].rotate_right(1);
            top[slot] = Some(candidate);
        }
    }
    let retained = top.iter().flatten().count();
    values.insert("included_states".into(), ordinal(included_total));
    values.insert("nonfinite_states".into(), ordinal(nonfinite));
    values.insert("contributors_retained".into(), ordinal(retained as u64));
    values.insert(
        "contributors_omitted".into(),
        ordinal((weights.len() - retained) as u64),
    );
    let wrms = (sum / weights.len() as f64).sqrt();
    values.insert("weighted_rms".into(), Metric::Real(wrms));
    values.insert("weighted_rms_finite".into(), Metric::Bool(wrms.is_finite()));
    top
}

fn contributor_fields(
    contract: &Contract,
    entry: Contribution,
    slot: usize,
    n: usize,
) -> BTreeMap<String, Metric> {
    BTreeMap::from([
        ("record_kind".into(), Metric::Text("contributor".into())),
        ("contributor_ordinal".into(), ordinal(slot as u64)),
        ("state_index".into(), ordinal(entry.index as u64)),
        (
            "state_identity".into(),
            Metric::Text(contract.states[entry.index].to_string()),
        ),
        (
            "differential".into(),
            Metric::Bool(contract.differential[entry.index]),
        ),
        (
            "included_in_error_test".into(),
            Metric::Bool(entry.included),
        ),
        ("weight".into(), Metric::Real(entry.weight)),
        ("estimated_local_error".into(), Metric::Real(entry.error)),
        ("weighted_local_error".into(), Metric::Real(entry.weighted)),
        (
            "squared_norm_contribution".into(),
            Metric::Real(if entry.included {
                entry.weighted.powi(2) / n as f64
            } else {
                0.
            }),
        ),
        ("norm_denominator".into(), ordinal(n as u64)),
    ])
}

#[cfg(test)]
mod tests {
    use super::super::checkpoint_owner_tests::NativeForward;
    use super::*;
    use pse_diagnostics::observation::{
        CoherentRecord, ObservationHub, OwnerBudget, Policy, RecordTap,
    };
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct Tap {
        selected: std::sync::atomic::AtomicBool,
        budget: OwnerBudget,
        records: Mutex<Vec<CoherentRecord>>,
        unavailable: std::sync::atomic::AtomicUsize,
    }
    impl Tap {
        fn new(selected: bool) -> Self {
            Self {
                selected: std::sync::atomic::AtomicBool::new(selected),
                budget: OwnerBudget::new(8192),
                records: Mutex::default(),
                unavailable: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }
    impl RecordTap for Tap {
        fn selected(&self, family: &str) -> bool {
            family == FAMILY && self.selected.load(std::sync::atomic::Ordering::Relaxed)
        }
        fn reserve_owner_bytes(&self, bytes: usize) -> Option<OwnerReservation> {
            self.budget.reserve(bytes)
        }
        fn observe(&self, record: &CoherentRecord) {
            self.records.lock().unwrap().push(record.clone());
        }
        fn source_unavailable(&self, family: &str) {
            assert_eq!(family, FAMILY);
            self.unavailable
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    fn contract() -> Contract {
        let id = |n: u8| SemanticId::from_bytes([n; 16]);
        Contract {
            identity: ContentHash::from_bytes([55; 32]),
            states: vec![id(7), id(8)],
            differential: vec![true, false],
            // Parameter ordering deliberately differs from state ordering.
            parameters: vec![id(9)],
            outputs: vec![],
            events: vec![vec![]],
            signs: vec![],
            derivatives: pse_kernels::DerivativeOrder::First,
            quadratures: vec![],
            balances: vec![],
        }
    }
    unsafe extern "C" fn multiscale(
        _time: f64,
        y: ffi::N_Vector,
        yp: ffi::N_Vector,
        residual: ffi::N_Vector,
        _data: *mut c_void,
    ) -> i32 {
        // SAFETY: the fixture's owned two-entry vectors match this callback.
        let y = unsafe { ffi::N_VGetArrayPointer(y) };
        // SAFETY: the fixture supplies the matching live two-entry rate vector.
        let yp = unsafe { ffi::N_VGetArrayPointer(yp) };
        // SAFETY: IDAS supplies an exclusive writable two-entry residual vector.
        let residual = unsafe { ffi::N_VGetArrayPointer(residual) };
        // SAFETY: state is initialized with exactly two entries through this callback.
        let y = unsafe { std::slice::from_raw_parts(y, 2) };
        // SAFETY: the matching rate storage is initialized and borrowed only here.
        let yp = unsafe { std::slice::from_raw_parts(yp, 2) };
        // SAFETY: residual storage is exclusively writable until this callback returns.
        let r = unsafe { std::slice::from_raw_parts_mut(residual, 2) };
        r[0] = yp[0] + y[0];
        r[1] = y[1] - 1000. * y[0];
        0
    }
    fn execution() -> Execution {
        Execution::new(Cancellation::default(), &crate::solve::Controls::default())
    }
    fn current(execution: &Execution) -> BTreeMap<String, Metric> {
        execution.progress.current().unwrap().values
    }
    /// Uses the same owner-return helper as Session, immediately after the actual
    /// fixture solve. It does not call a reader or re-evaluate the physical model.
    fn capture(
        local: &mut LocalError,
        native: &NativeForward,
        execution: &Execution,
        generation: u64,
        flag: i32,
        time: f64,
        suppressed: bool,
    ) -> BTreeMap<String, Metric> {
        // SAFETY: fixture owns this live initialized memory and its context/scratch
        // on this thread, and supplies the actual solve return or pre-solve control.
        unsafe {
            local.returned(
                native.memory,
                &contract(),
                ReturnCheckpoint {
                    generation,
                    flag,
                    returned_time: time,
                    suppress_alg: suppressed,
                    phase: "idas.forward.local_error",
                },
                execution,
            );
        }
        current(execution)
    }

    #[test]
    fn local_error_native_masked_wrms_and_freshness() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("idas.local_error.control");
        let tap = Arc::new(Tap::new(true));
        let context = operation.context().with_record_tap(tap.clone());
        let _entered = context.enter();
        let mut native = NativeForward::new_with(100, &[1., 1000.], &[-1., -1000.], multiscale);
        // SAFETY: matching two-entry vectors and live fixture context. IDASetId
        // copies the identities, so the local id vector can be freed afterwards.
        let id = unsafe { ffi::N_VNew_Serial(2, native.context) };
        assert!(!id.is_null());
        // SAFETY: this private serial mask has exactly two writable entries.
        unsafe {
            write(id, &[1., 0.]);
        }
        // SAFETY: live native memory copies the dimensionally matched identities.
        assert_eq!(unsafe { ffi::IDASetId(native.memory, id) }, 0);
        // SAFETY: the valid id vector was installed before enabling its mask.
        assert_eq!(unsafe { ffi::IDASetSuppressAlg(native.memory, 1) }, 0);
        let mut local = LocalError::prepare(2).unwrap();
        let execution = execution();
        let before = capture(&mut local, &native, &execution, 11, 0, 0., true);
        assert_eq!(
            before["availability"],
            Metric::Text("no_accepted_internal_step".into())
        );
        assert!(!before.contains_key("contributor.0.state_identity"));
        let (flag, time, state) = native.solve(0.01, ffi::IDA_NORMAL);
        assert!(flag >= 0);
        assert!((state - (-time).exp()).abs() < 0.001);
        let accepted = capture(&mut local, &native, &execution, 11, flag, time, true);
        assert_eq!(accepted["source_available"], Metric::Bool(true));
        assert_eq!(accepted["norm_denominator"], Metric::Integer(2));
        assert_eq!(accepted["included_states"], Metric::Integer(1));
        let field = |record: &CoherentRecord, name: &str| {
            record
                .values
                .iter()
                .find(|field| field.name == name)
                .unwrap()
                .value
                .clone()
        };
        let records = tap.records.lock().unwrap();
        let contributors: Vec<_> = records
            .iter()
            .filter(|record| {
                record.values.iter().any(|field| {
                    field.name == "record_kind" && field.value == Value::Text("contributor".into())
                })
            })
            .collect();
        assert_eq!(contributors.len(), 2);
        use pse_diagnostics::observation::Value;
        assert_eq!(
            field(contributors[0], "state_identity"),
            Value::Text(contract().states[0].to_string())
        );
        assert_eq!(
            field(contributors[1], "included_in_error_test"),
            Value::Boolean(false)
        );
        assert_eq!(
            field(contributors[1], "squared_norm_contribution"),
            Value::Number(0.)
        );
        assert_eq!(
            field(contributors[0], "native_return_ordinal"),
            Value::Integer(2)
        );
        drop(records);
        let scratch = local.scratch.as_ref().unwrap();
        assert_ne!(scratch.context, native.context);
        // SAFETY: live copied vectors and exactly the mask installed in native IDAS.
        let expected = unsafe { ffi::N_VWrmsNormMask(scratch.errors, scratch.weights, id) };
        assert_eq!(accepted["weighted_rms"], Metric::Real(expected));
        let internal = match accepted["internal_time"] {
            Metric::Real(v) => v,
            _ => panic!(),
        };
        assert!(internal >= time);
        // Repeat NORMAL at the already returned sample: no internal step is taken.
        let (flag, time, _) = native.solve(time, ffi::IDA_NORMAL);
        assert!(flag >= 0);
        let repeated = capture(&mut local, &native, &execution, 11, flag, time, true);
        assert_eq!(
            repeated["availability"],
            Metric::Text("no_new_accepted_internal_step".into())
        );
        assert!(!repeated.contains_key("weighted_rms"));
        // An actual invalid-task return is negative even while getters can still
        // copy earlier vectors. Those copies cannot authorize a failed observation.
        let (flag, time, _) = native.solve(0.02, -999);
        assert!(flag < 0);
        let failed = capture(&mut local, &native, &execution, 11, flag, time, true);
        assert_eq!(
            failed["availability"],
            Metric::Text("negative_native_return".into())
        );
        assert!(matches!(failed["returned_time"], Metric::Unavailable(_)));
        assert!(!failed.contains_key("contributor.0.state_identity"));
        assert!(failed.contains_key("tolerance_scale_factor"));
        // Reinitialization resets native counts, and the new generation cannot
        // expose a stale vector even if statistics supplies a synthetic zero flag.
        // SAFETY: both fixture state entries are exclusively owned and writable.
        unsafe {
            write(native.state, &[1., 1000.]);
        }
        // SAFETY: the matching private rate vector also has two writable entries.
        unsafe {
            write(native.rate, &[-1., -1000.]);
        }
        assert_eq!(
            // SAFETY: live IDAS memory receives its own dimensionally matched vectors.
            unsafe { ffi::IDAReInit(native.memory, 0., native.state, native.rate) },
            0
        );
        // SAFETY: the fixture owns initialized adjoint memory through this reset.
        assert_eq!(unsafe { ffi::IDAAdjReInit(native.memory) }, 0);
        let reinit = capture(&mut local, &native, &execution, 12, 0, 0., true);
        assert_eq!(
            reinit["availability"],
            Metric::Text("no_accepted_internal_step".into())
        );
        let (flag, time, _) = native.solve(0.01, ffi::IDA_NORMAL);
        assert!(flag >= 0);
        let restarted = capture(&mut local, &native, &execution, 12, flag, time, true);
        assert_eq!(restarted["source_available"], Metric::Bool(true));
        assert_eq!(restarted["native_generation"], Metric::Integer(12));
        assert!(
            tap.records
                .lock()
                .unwrap()
                .iter()
                .any(|record| record.family == FAMILY)
        );
        drop(local);
        // SAFETY: mask vector is a separately owned test allocation and is freed
        // before the fixture context, after the final native norm query.
        unsafe {
            ffi::N_VDestroy(id);
        }
    }

    #[test]
    fn local_error_off_refusal_and_selection_release_preserve_native_output() {
        fn run(selected: bool, allowance: usize) -> (i32, f64, f64) {
            let hub = ObservationHub::new(Policy {
                enabled: false,
                ..Policy::default()
            });
            let operation = hub.start("idas.local_error.off_on");
            let tap = Arc::new(Tap {
                budget: OwnerBudget::new(allowance),
                ..Tap::new(selected)
            });
            let context = operation.context().with_record_tap(tap.clone());
            let _entered = context.enter();
            let mut native = NativeForward::new_with(100, &[1., 1000.], &[-1., -1000.], multiscale);
            let mut local = LocalError::prepare(2);
            assert_eq!(local.is_some(), selected);
            let result = native.solve(0.1, ffi::IDA_NORMAL);
            if let Some(local) = &mut local {
                let execution = execution();
                let value = capture(local, &native, &execution, 1, result.0, result.1, false);
                assert_eq!(value["source_available"], Metric::Bool(allowance > 0));
                // A selection ending during this attempt releases native scratch.
                tap.selected
                    .store(false, std::sync::atomic::Ordering::Relaxed);
                capture(local, &native, &execution, 1, result.0, result.1, false);
                assert!(local.scratch.is_none());
            } else {
                assert!(tap.records.lock().unwrap().is_empty());
            }
            drop(local);
            result
        }
        assert_eq!(run(false, 8192), run(true, 8192));
        assert_eq!(run(false, 8192), run(true, 0));
    }

    #[test]
    fn local_error_bounded_projection_tracks_omissions_nonfinite_and_mask() {
        let mut contract = contract();
        contract.states = (0..12).map(|n| SemanticId::from_bytes([n; 16])).collect();
        contract.differential = vec![true; 12];
        contract.differential[11] = false;
        let mut errors = [1.; 12];
        errors[10] = f64::NAN;
        errors[11] = 1000.;
        let mut values = BTreeMap::new();
        let top = project(&mut values, &contract, &[2.; 12], &errors, true);
        assert_eq!(values["contributors_retained"], Metric::Integer(8));
        assert_eq!(values["contributors_omitted"], Metric::Integer(4));
        assert_eq!(values["nonfinite_states"], Metric::Integer(1));
        assert_eq!(values["weighted_rms_finite"], Metric::Bool(false));
        assert_eq!(top[0].unwrap().index, 10);
    }

    #[test]
    fn local_error_optional_progress_failure_preserves_native_return() {
        #[derive(Debug)]
        struct FailingProgress;
        impl crate::solve::ProgressTap for FailingProgress {
            fn observe(&self, _: &Event) {
                panic!("selected local-error progress control");
            }
        }
        fn run(fail: bool) -> (i32, f64, f64) {
            let hub = ObservationHub::new(Policy::default());
            let operation = hub.start("idas.local_error.failure");
            let tap = Arc::new(Tap::new(true));
            let context = operation.context().with_record_tap(tap.clone());
            let _entered = context.enter();
            let mut native = NativeForward::new_with(100, &[1., 1000.], &[-1., -1000.], multiscale);
            let mut local = LocalError::prepare(2).unwrap();
            let mut execution = execution();
            if fail {
                execution.progress = Arc::new(Progress::tapped(0, Arc::new(FailingProgress)));
            }
            let result = native.solve(0.1, ffi::IDA_NORMAL);
            assert!(result.0 >= 0);
            // SAFETY: this fixture owns the live handle on this thread and forwards
            // its actual return, with the matching prepared state coordinates.
            unsafe {
                local.returned(
                    native.memory,
                    &contract(),
                    ReturnCheckpoint {
                        generation: 1,
                        flag: result.0,
                        returned_time: result.1,
                        suppress_alg: false,
                        phase: "idas.forward.local_error",
                    },
                    &execution,
                );
            }
            assert_eq!(
                tap.unavailable.load(std::sync::atomic::Ordering::Relaxed),
                usize::from(fail)
            );
            drop(local);
            result
        }
        assert_eq!(run(false), run(true));
    }
}

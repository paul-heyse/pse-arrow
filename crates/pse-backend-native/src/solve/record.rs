// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable native attempt facts retained before a later attempt supersedes its report.
use super::*;
use pse_model::diagnostic::Observation;
use std::mem::{align_of, size_of};

/// Actual submitted source-coordinate seed, distinct from an output or native checkpoint.
#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubmittedStartRecord {
    /// Original predecessor attempt selection.
    pub previous_attempt: Option<usize>,
    /// Full actual source-coordinate seed with explicit nonfinite observations.
    pub seed: Option<WarmStartSnapshot>,
    /// Full partial coordinate start, retaining exact IEEE bits including signed zero.
    pub sparse_seed: Option<BTreeMap<SemanticId, u64>>,
    /// Complete transformations actually applied before API submission.
    pub transformations: Vec<SeedTransformation>,
    /// Actual API submission; native internal use remains separately observed.
    pub submitted: bool,
}
/// Native-owned compact attempt record; scientific workflow position belongs to its caller.
/// No factor, actor, oracle, candidate or complete SolveReport is retained here.
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeAttemptRecord")]
pub struct AttemptRecord {
    /// Actual executed adapter.
    pub backend: Backend,
    /// Complete original variable correspondence recorded by the adapter's report.
    pub variables: Vec<SemanticId>,
    /// Complete original row correspondence recorded by the adapter's report.
    pub rows: Vec<SemanticId>,
    /// Complete actual adapter options, separate from queried defaults.
    #[serde(serialize_with = "super::serialize_option_observations")]
    #[schemars(with = "BTreeMap<String, Observation>")]
    pub options: Options,
    /// Native defaults only when supplied by a getter.
    #[serde(serialize_with = "super::serialize_option_observations")]
    #[schemars(with = "BTreeMap<String, Observation>")]
    pub native_defaults: Options,
    /// Actual owner-emitted build/runtime/numerical provenance, without parsing.
    pub provenance: BTreeMap<String, String>,
    /// Actual native status and the adapter's conservative interpretation.
    pub termination: NativeTermination,
    /// Executing-owner work counters; absent values remain unknown.
    pub work: WorkEvidence,
    /// Exact attempt-local abandonment, distinct from task cancellation.
    pub abandoned: Option<pse_model::generated::enums::NumericalAttemptObservation>,
    /// Whether the adapter submitted a start, even if no complete source receipt exists.
    pub start_submitted: bool,
    /// Actual compatible native state reuse, not merely a requested reuse policy.
    pub reused_native_state: bool,
    /// Actual submitted source-coordinate start and transformations.
    pub start: Option<SubmittedStartRecord>,
    /// Shared bounded native counters, phase, submitted snapshots and stop fidelity.
    pub observations: Option<Arc<NativeObservations>>,
    /// Shared bounded raw map history, preserving omissions, failures and sample roles.
    pub raw_map: Option<Arc<RawMapHistory>>,
    /// The caller's pre-admitted record allocation, retained through every Arc consumer.
    #[serde(skip)]
    #[schemars(skip)]
    _owner: Arc<dyn pse_math::AllocationOwner>,
}

struct Extent(usize);
impl Extent {
    fn add(&mut self, bytes: usize) -> Option<()> {
        self.0 = self.0.checked_add(bytes)?;
        Some(())
    }
    fn vector<T>(&mut self, capacity: usize) -> Option<()> {
        self.add(capacity.checked_mul(size_of::<T>())?)
    }
    fn arc<T>(&mut self) -> Option<()> {
        self.add(
            size_of::<T>()
                .checked_add(2 * size_of::<usize>())?
                .checked_add(align_of::<T>())?,
        )
    }
    fn tree<K, V>(&mut self, length: usize) -> Option<()> {
        // One complete pinned Rust BTree internal node per entry, plus a possibly
        // allocated empty root, bounds sparse leaf occupancy and every split.
        let alignment = align_of::<K>()
            .max(align_of::<V>())
            .max(align_of::<usize>());
        let node = size_of::<K>()
            .checked_add(size_of::<V>())?
            .checked_mul(11)?
            .checked_add(size_of::<usize>().checked_mul(13)?)?
            .checked_add(size_of::<u16>().checked_mul(2)?)?
            .checked_add(alignment.checked_mul(7)?)?;
        self.add(length.checked_add(1)?.checked_mul(node)?)
    }
    fn options(&mut self, options: &Options) -> Option<()> {
        self.tree::<String, OptionValue>(options.len())?;
        for (key, value) in options {
            self.add(key.capacity())?;
            if let OptionValue::Text(value) = value {
                self.add(value.capacity())?;
            }
        }
        Some(())
    }
    fn seed(&mut self, seed: &WarmStart) -> Option<()> {
        // snapshot() creates diagnostic observations rather than copying f64s.
        match &seed.payload {
            WarmPayload::Root(primal) => self.vector::<Observation>(primal.len())?,
            WarmPayload::Nlp {
                primal,
                bounds,
                rows,
                barrier: _,
                working,
            } => {
                self.vector::<Observation>(primal.len())?;
                if let Some((lower, upper)) = bounds {
                    self.vector::<Observation>(lower.len())?;
                    self.vector::<Observation>(upper.len())?;
                }
                if let Some(rows) = rows {
                    self.vector::<Observation>(rows.len())?;
                }
                if let Some(working) = working {
                    #[cfg(feature = "pounce")]
                    {
                        self.vector::<BoundActivity>(working.active.bounds.len())?;
                        self.vector::<RowActivity>(working.active.constraints.len())?;
                    }
                    #[cfg(not(feature = "pounce"))]
                    let _ = working;
                }
            }
            WarmPayload::Highs {
                primal,
                dual,
                basis,
            } => {
                if let Some(primal) = primal {
                    self.vector::<Observation>(primal.len())?;
                }
                if let Some((columns, rows)) = dual {
                    self.vector::<Observation>(columns.len())?;
                    self.vector::<Observation>(rows.len())?;
                }
                if let Some(basis) = basis {
                    self.vector::<i32>(basis.columns.capacity())?;
                    self.vector::<i32>(basis.rows.capacity())?;
                }
            }
        }
        Some(())
    }
    fn sample(&mut self, sample: &RawMapSample) -> Option<()> {
        self.arc::<RawMapSample>()?;
        self.vector::<RawMapCoordinate>(sample.values.capacity())
    }
    fn report(&mut self, report: &SolveReport) -> Option<()> {
        self.arc::<AttemptRecord>()?;
        self.vector::<SemanticId>(report.variables.capacity())?;
        self.vector::<SemanticId>(report.rows.capacity())?;
        self.options(&report.options)?;
        self.options(&report.native_defaults)?;
        self.tree::<String, String>(report.provenance.len())?;
        for (key, value) in &report.provenance {
            self.add(key.capacity())?;
            self.add(value.capacity())?;
        }
        self.add(report.termination.name.capacity())?;
        if let Some(message) = &report.termination.message {
            self.add(message.capacity())?;
        }
        if let Some(start) = &report.start_receipt {
            self.vector::<SeedTransformation>(start.transformations.capacity())?;
            for transform in &start.transformations {
                if let SeedTransformation::Presolve(value) = transform {
                    self.vector::<crate::presolve::Pass>(value.passes.capacity())?;
                }
            }
            if let Some(sparse) = &start.sparse_seed {
                self.tree::<SemanticId, u64>(sparse.len())?;
            }
            if let Some(seed) = &start.seed {
                self.seed(seed)?;
            }
        }
        // These payloads are Arc-cloned, but this record outlives the report's
        // owner. Charge their actual retained capacities under the new owner.
        if let Some(native) = &report.evidence.native_observations {
            self.arc::<NativeObservations>()?;
            if let Some(current) = &native.current {
                self.arc::<NativeSnapshot>()?;
                self.vector::<NativeCoordinate>(current.values.capacity())?;
            }
        }
        if let Some(history) = &report.evidence.raw_map {
            self.arc::<RawMapHistory>()?;
            self.vector::<Arc<RawMapSample>>(history.samples.capacity())?;
            for sample in &history.samples {
                self.sample(sample)?;
            }
            for (last, already) in [
                (history.last_successful.as_ref(), None),
                (
                    history.last_evaluation.as_ref(),
                    history.last_successful.as_ref(),
                ),
            ] {
                if let Some(last) = last
                    && !already.is_some_and(|sample| Arc::ptr_eq(sample, last))
                    && !history
                        .samples
                        .iter()
                        .any(|sample| Arc::ptr_eq(sample, last))
                {
                    self.sample(last)?;
                }
            }
        }
        Some(())
    }
}
impl AttemptRecord {
    /// Conservative retained allocation before snapshotting or cloning any payload.
    /// Scans borrowed source capacities only; it allocates no vectors or serialization buffers.
    /// Shared diagnostic payloads are included because their report owner may be released.
    pub fn retained_extent(report: &SolveReport) -> Result<usize, ProblemError> {
        let mut extent = Extent(0);
        extent
            .report(report)
            .ok_or_else(|| ProblemError::memory("native attempt record extent overflow"))?;
        Ok(extent.0)
    }
    /// Capture only after the caller reserves at least retained_extent(report).
    /// AllocationOwner is an opaque lifetime anchor and exposes no allowance getter;
    /// the caller owns the reservation contract. This operation rechecks arithmetic
    /// before allocating and keeps that supplied owner alive through all consumers.
    pub fn capture(
        report: &SolveReport,
        owner: Arc<dyn pse_math::AllocationOwner>,
    ) -> Result<Arc<Self>, ProblemError> {
        Self::retained_extent(report)?;
        Ok(Arc::new(Self {
            backend: report.backend,
            variables: report.variables.clone(),
            rows: report.rows.clone(),
            options: report.options.clone(),
            native_defaults: report.native_defaults.clone(),
            provenance: report.provenance.clone(),
            termination: report.termination.clone(),
            work: report.evidence.work,
            abandoned: report.evidence.abandoned,
            start_submitted: report.evidence.start_submitted,
            reused_native_state: report.evidence.reused_native_state,
            start: report
                .start_receipt
                .as_ref()
                .map(|start| SubmittedStartRecord {
                    previous_attempt: start.previous_attempt,
                    seed: start.seed.as_ref().map(WarmStart::snapshot),
                    sparse_seed: start.sparse_seed.as_ref().map(|seed| {
                        seed.iter()
                            .map(|(id, value)| (*id, value.to_bits()))
                            .collect()
                    }),
                    transformations: start.transformations.clone(),
                    submitted: start.submitted,
                }),
            observations: report.evidence.native_observations.clone(),
            raw_map: report.evidence.raw_map.clone(),
            _owner: owner,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn report() -> SolveReport {
        let execution = Execution::new(
            Arc::default(),
            &Controls {
                history: 0,
                ..Controls::default()
            },
        );
        SolveReport::new(
            Backend::Kinsol,
            &crate::solver_tests::contract(),
            NativeTermination {
                code: -13,
                name: "KIN_SYSFUNC_FAIL".into(),
                message: Some("actual map failure".into()),
                category: Termination::Evaluation,
                assurance: Assurance::None,
            },
            &execution,
        )
    }
    #[test]
    fn attempt_record_preserves_full_submitted_start_bits_and_nonfinite_observations() {
        let mut report = report();
        report.start_receipt = Some(StartReceipt {
            previous_attempt: Some(3),
            seed: Some(WarmStart {
                origin: None,
                compatibility: crate::solver_tests::stamp(Backend::Ipopt),
                payload: WarmPayload::Nlp {
                    primal: vec![-0., f64::NAN],
                    bounds: Some((vec![1., 2.], vec![3., 4.])),
                    rows: Some(vec![f64::INFINITY]),
                    barrier: Some(0.25),
                    working: None,
                },
            }),
            sparse_seed: Some(BTreeMap::from([(SemanticId::NIL, -0.)])),
            transformations: vec![SeedTransformation::Presolve(PresolveSeedTransformation {
                transformation: ContentHash::from_bytes([7; 32]),
                passes: vec![crate::presolve::Pass::Fbbt],
            })],
            submitted: false,
        });
        report.evidence.work.iterations = None;
        report.evidence.work.evaluations = Some(7);
        report.options = BTreeMap::from([
            ("infinite".into(), OptionValue::Real(f64::INFINITY)),
            ("finite".into(), OptionValue::Real(-0.0)),
            ("text".into(), OptionValue::Text("native".into())),
            ("integer".into(), OptionValue::Integer(7)),
            ("boolean".into(), OptionValue::Bool(true)),
        ]);
        report
            .native_defaults
            .insert("nan".into(), OptionValue::Real(f64::NAN));
        let record = AttemptRecord::capture(&report, Arc::new(())).unwrap();
        let start = record.start.as_ref().unwrap();
        assert_eq!(start.previous_attempt, Some(3));
        assert_eq!(
            start.sparse_seed.as_ref().unwrap()[&SemanticId::NIL],
            (-0_f64).to_bits()
        );
        assert!(!start.submitted);
        assert_eq!(record.work.evaluations, Some(7));
        assert_eq!(record.work.iterations, None);
        assert_eq!(record.termination.code, -13);
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(
            json["start"]["seed"]["payload"]["primal"][1]["kind"],
            "nonfinite"
        );
        assert_eq!(
            json["start"]["seed"]["payload"]["row_duals"][0]["value"],
            "positive_infinity"
        );
        assert_eq!(json["options"]["infinite"]["value"], "positive_infinity");
        assert_eq!(json["native_defaults"]["nan"]["value"], "nan");
        for (name, kind) in [
            ("finite", "real"),
            ("text", "text"),
            ("integer", "integer"),
            ("boolean", "boolean"),
        ] {
            let observed: Observation =
                serde_json::from_value(json["options"][name].clone()).unwrap();
            assert_eq!(serde_json::to_value(observed).unwrap()["kind"], kind);
        }
        assert!(json.get("_owner").is_none());
        let schema = serde_json::to_value(schemars::schema_for!(AttemptRecord)).unwrap();
        assert_eq!(schema["additionalProperties"], false);
    }
    #[test]
    fn attempt_record_extent_includes_basis_only_seed_and_all_variable_payloads_before_capture() {
        let mut report = report();
        let original = AttemptRecord::retained_extent(&report).unwrap();
        report.start_receipt = Some(StartReceipt {
            previous_attempt: None,
            seed: Some(WarmStart {
                origin: None,
                compatibility: crate::solver_tests::stamp(Backend::Highs),
                payload: WarmPayload::Highs {
                    primal: None,
                    dual: None,
                    basis: Some(Basis {
                        columns: vec![1; 4096],
                        rows: vec![2; 2048],
                    }),
                },
            }),
            sparse_seed: None,
            transformations: vec![],
            submitted: true,
        });
        report
            .options
            .insert("actual.text".into(), OptionValue::Text("q".repeat(8192)));
        report
            .native_defaults
            .insert("native.text".into(), OptionValue::Text("d".repeat(4096)));
        report
            .provenance
            .insert("native.build".into(), "p".repeat(2048));
        let extent = AttemptRecord::retained_extent(&report).unwrap();
        assert!(extent >= original + 6144 * size_of::<i32>() + 8192 + 4096 + 2048);
        let record = AttemptRecord::capture(&report, Arc::new(())).unwrap();
        let WarmPayloadSnapshot::Highs {
            basis: Some(basis), ..
        } = &record
            .start
            .as_ref()
            .unwrap()
            .seed
            .as_ref()
            .unwrap()
            .payload
        else {
            panic!("basis-only seed");
        };
        assert_eq!(basis.columns.len(), 4096);
        assert_eq!(basis.rows.len(), 2048);
        let mut bound = Extent(usize::MAX);
        assert!(bound.vector::<u64>(1).is_none());
    }
    #[test]
    fn attempt_record_retains_raw_arc_payload_and_precharged_owner_after_report_drop() {
        let mut report = report();
        let mut capture = RawMapCapture::new(
            &crate::solver_tests::contract(),
            ContentHash::from_bytes([1; 32]),
            0,
            true,
            &[0.5],
        )
        .unwrap();
        capture.record(
            &crate::solver_tests::contract(),
            &[1.],
            Some(&[2.]),
            RawMapRole::FixedPointEvaluation,
            None,
        );
        capture.record(
            &crate::solver_tests::contract(),
            &[2.],
            None,
            RawMapRole::FixedPointEvaluation,
            Some(RawMapFailure::Trial),
        );
        let before = AttemptRecord::retained_extent(&report).unwrap();
        report.evidence.raw_map = Some(capture.finish());
        let extent = AttemptRecord::retained_extent(&report).unwrap();
        assert!(extent > before + 2 * size_of::<RawMapCoordinate>());
        let owner = Arc::new(());
        let weak = Arc::downgrade(&owner);
        let record = AttemptRecord::capture(&report, owner).unwrap();
        assert!(Arc::ptr_eq(
            record.raw_map.as_ref().unwrap(),
            report.evidence.raw_map.as_ref().unwrap()
        ));
        drop(report);
        assert!(weak.upgrade().is_some());
        let retained = record.clone();
        drop(record);
        let raw = retained.raw_map.as_ref().unwrap();
        assert_eq!(raw.omitted, 2);
        assert_eq!(
            raw.last_successful.as_ref().unwrap().values[0].residual,
            Some(1.)
        );
        assert!(matches!(
            raw.last_evaluation.as_ref().unwrap().failure,
            Some(RawMapFailure::Trial)
        ));
        drop(retained);
        assert!(weak.upgrade().is_none());
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fresh-process symbolic continuation of actual authored caloric recipes.
use super::*;
use pse_ids::roles::SemanticBodyHash;
use pse_model::generated::enums::ModelingValidityLayer;
use pse_modeling::specialize::root_instance;
use std::sync::{Mutex, atomic::AtomicUsize};

const MODE: &str = "PSE_B06_CALORIC_MODE";
const FILE: &str = "PSE_B06_CALORIC_FILE";
const CHILD: &str =
    "workspace::modeling::executable::symbolic_replay_tests::typed_replay_caloric_child";
type Records = Vec<(SemanticBodyHash, Vec<u8>)>;
#[derive(Debug)]
struct Recipes {
    quantities: Arc<QuantityRegistry>,
    records: Mutex<BTreeMap<SemanticBodyHash, Vec<u8>>>,
    receiving: bool,
    reads: AtomicUsize,
}
impl ModelingBodyRetention for Recipes {
    fn generation(&self) -> u64 {
        0
    }
    #[allow(
        unsafe_code,
        reason = "the fresh-process test forwards exact records captured from its actual compiler producer"
    )]
    fn get(
        &self,
        key: SemanticBodyHash,
    ) -> std::result::Result<Option<Arc<AdmittedBody>>, MathError> {
        if !self.receiving {
            return Ok(None);
        }
        let records = self.records.lock().unwrap();
        let payload = records.get(&key).ok_or_else(|| {
            MathError::Contract(
                "receiver requested a body outside the original recipe population".into(),
            )
        })?;
        // SAFETY: this test's producer writes the actual compiler-issued bytes to
        // its private parent-owned file; no arbitrary production publication is minted.
        let permit = unsafe {
            pse_ids::scientific_replay::QualifiedScientificRecipe::from_admitted_record_fixture(
                payload.clone(),
                crate::typed_math::portable_payload_hash(payload),
                ContentHash::from_bytes([121; 32]),
                ContentHash::from_bytes([122; 32]),
            )
        }
        .map_err(|error| MathError::Contract(error.to_string()))?;
        let body = crate::typed_math::reconstruct_portable(
            &permit,
            &self.quantities,
            &Arc::new(AtomicBool::new(false)),
        )?;
        if body.semantic_identity() != Some(key) {
            return Err(MathError::Contract(
                "restored recipe changed its original semantic identity".into(),
            ));
        }
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(Some(Arc::new(body)))
    }
    fn retain(
        &self,
        _: u64,
        key: SemanticBodyHash,
        body: Arc<AdmittedBody>,
    ) -> std::result::Result<Arc<AdmittedBody>, MathError> {
        if self.receiving {
            return Err(MathError::Contract(
                "symbolic receiver unexpectedly readmitted source mathematics".into(),
            ));
        }
        self.records
            .lock()
            .unwrap()
            .insert(key, body.portable_payload()?);
        Ok(body)
    }
}
fn source() -> String {
    format!(
        r#"{}
package p {{use correlations @"1.0.0";
def Root {{var T:Temperature;var reference:Temperature;var h:DeltaH;var s:DeltaS;
eq heat:correlations.dippr100_dh_value(reference,T,75.3{{J/(mol*K)}},0.01{{J/(mol*K^2)}},1e-5{{J/(mol*K^3)}},1e-8{{J/(mol*K^4)}},1e-11{{J/(mol*K^5)}})==h;
eq entropy:correlations.dippr100_ds_value(reference,T,75.3{{J/(mol*K)}},0.01{{J/(mol*K^2)}},1e-5{{J/(mol*K^3)}},1e-8{{J/(mol*K^4)}},1e-11{{J/(mol*K^5)}})==s;
}} }}"#,
        crate::authored_transfer_tests::reference_sources()
    )
}
fn bindings() -> ModelingCaseBindings {
    ModelingCaseBindings {
        values: [
            ("T".into(), 320.0),
            ("reference".into(), 300.0),
            ("h".into(), 0.0),
            ("s".into(), 0.0),
        ]
        .into(),
        variables: [
            (
                "T".into(),
                ModelingVariableState {
                    lower: Some(Some(280.0)),
                    upper: Some(Some(400.0)),
                    ..Default::default()
                },
            ),
            (
                "reference".into(),
                ModelingVariableState {
                    fixed: Some(true),
                    ..Default::default()
                },
            ),
        ]
        .into(),
        ..Default::default()
    }
}
fn workspace(
    records: Records,
    receiving: bool,
) -> (CompilerWorkspace, DeclarationId, Arc<Recipes>) {
    let context = crate::authored_transfer_tests::context();
    let recipes = Arc::new(Recipes {
        quantities: context.quantities.clone(),
        records: Mutex::new(records.into_iter().collect()),
        receiving,
        reads: AtomicUsize::new(0),
    });
    let mut workspace = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
    workspace.attach_body_retention(recipes.clone()).unwrap();
    let rows = crate::authored_transfer_tests::rows(&source());
    let root = crate::authored_transfer_tests::root(&rows, "p", "Root");
    workspace
        .publish_modeling(rows, PhysicalScope::default())
        .unwrap();
    (workspace, root, recipes)
}
fn cause(error: &MathError) -> &MathError {
    if let MathError::Instance { cause: inner, .. } = error {
        cause(inner)
    } else {
        error
    }
}

#[test]
fn typed_replay_caloric_child() {
    let Ok(mode) = std::env::var(MODE) else {
        return;
    };
    let file = std::path::PathBuf::from(std::env::var_os(FILE).unwrap());
    assert!(
        pse_math::context().is_err(),
        "a child must begin without the producer's symbolic pool"
    );
    let records = if mode == "receive" {
        assert!(
            std::fs::metadata(&file).unwrap().len()
                < 4 * pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES as u64
        );
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap()
    } else {
        Vec::new()
    };
    let (mut workspace, root, recipes) = workspace(records, mode == "receive");
    let cancel = Arc::new(AtomicBool::new(false));
    let (model, base, values, _) = workspace
        .prepare_modeling_case_cancellable(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
            &bindings(),
            DerivativeOrder::Value,
            Profile::default(),
            cancel.clone(),
        )
        .unwrap();
    assert_eq!(base.plan.order(), DerivativeOrder::Value);
    assert!(matches!(
        base.presolve.class_status,
        pse_math::presolve::ClassStatus::Unassessed
    ));
    if mode == "produce" {
        let records = recipes
            .records
            .lock()
            .unwrap()
            .iter()
            .map(|(key, payload)| (*key, payload.clone()))
            .collect::<Records>();
        assert!(!records.is_empty());
        let bytes = serde_json::to_vec(&records).unwrap();
        use std::io::Write;
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(file)
            .unwrap();
        output.write_all(&bytes).unwrap();
        return;
    }
    assert_eq!(mode, "receive");
    assert!(
        recipes.reads.load(Ordering::SeqCst) > 0,
        "actual compiler acquisition must use qualified recipe replay"
    );
    // No numerical derivative or coefficient product existed in the producer.
    let class = base.prepare_class(&values, &cancel).unwrap();
    assert!(
        matches!(
            class.presolve.class_status,
            pse_math::presolve::ClassStatus::RuledOut(
                pse_math::presolve::ClassWitness::NonAffineRow { .. }
            )
        ),
        "{:?}",
        class.presolve.class_status
    );
    assert!(class.coefficients.is_none());
    let second = base
        .prepare_order(DerivativeOrder::Second, &cancel)
        .unwrap();
    assert_eq!(second.plan.order(), DerivativeOrder::Second);
    let scope = pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None);
    let assembly = Arc::new(
        second
            .plan
            .compile(Optimization::default(), EvaluationLimits::default(), &scope)
            .unwrap(),
    );
    let temperature = model.model.paths["T"];
    let column = second
        .plan
        .columns()
        .iter()
        .position(|id| *id == temperature)
        .unwrap();
    let rows = ["heat", "entropy"].map(|name| {
        model
            .model
            .equations
            .iter()
            .find(|row| row.lineage.path.ends_with(name))
            .unwrap()
            .id
    });
    let positions = rows.map(|id| {
        assembly
            .structure()
            .rows()
            .iter()
            .position(|row| row.id == id)
            .unwrap()
    });
    let mut worker = assembly.worker_scoped(pse_kernels::ProviderBindings::new(), scope.clone());
    let t = 320.0_f64;
    let cp = 75.3 + 0.01 * t + 1e-5 * t * t + 1e-8 * t * t * t + 1e-11 * t * t * t * t;
    let cp_first = 0.01 + 2e-5 * t + 3e-8 * t * t + 4e-11 * t * t * t;
    let jacobian = worker.jacobian(&values).unwrap();
    for (row, expected) in positions.into_iter().zip([cp, cp / t]) {
        let actual = jacobian.get(row, column).copied().unwrap_or(0.0);
        assert!(
            (actual / expected - 1.0).abs() < 2e-12,
            "{actual} != {expected}"
        );
    }
    for (row, expected) in positions
        .into_iter()
        .zip([cp_first, cp_first / t - cp / (t * t)])
    {
        let mut weights = vec![0.0; assembly.structure().rows().len()];
        weights[row] = 1.0;
        let actual = worker
            .hessian(&values, 0.0, &weights)
            .unwrap()
            .get(column, column)
            .copied()
            .unwrap_or(0.0);
        assert!(
            (actual / expected - 1.0).abs() < 2e-12,
            "{actual} != {expected}"
        );
    }
    // A second current view removes the contextual box; the original authored
    // function validity must still reject zero, independently of that box.
    let mut unbounded = bindings();
    let state = unbounded.variables.get_mut("T").unwrap();
    state.lower = Some(None);
    state.upper = Some(None);
    let (_, unbounded, mut invalid, _) = workspace
        .prepare_modeling_case_cancellable(
            root,
            root_instance(root),
            Bindings::default(),
            Limits::default(),
            &unbounded,
            DerivativeOrder::Value,
            Profile::default(),
            cancel.clone(),
        )
        .unwrap();
    invalid.scalars.insert(temperature, 0.0);
    let assembly = Arc::new(
        unbounded
            .plan
            .compile(Optimization::default(), EvaluationLimits::default(), &scope)
            .unwrap(),
    );
    let error = assembly
        .worker_scoped(pse_kernels::ProviderBindings::new(), scope)
        .constraints(&invalid)
        .unwrap_err();
    assert!(
        matches!(cause(&error),MathError::Validity(lineage) if lineage.layer==ModelingValidityLayer::Form
        && ["db3d88c1114d4e48917fe6e0e714cfcc","da740711232f4efdaec92047a55244b8"].contains(&lineage.source.to_string().as_str())),
        "{error:?}"
    );
}

#[test]
fn qualified_caloric_replay_requests_new_derivative_and_class_after_producer_exit() {
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let path = std::env::temp_dir().join(format!("pse-b06-caloric-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let scratch = Scratch(path);
    let file = scratch.0.join("recipes.json");
    for mode in ["produce", "receive"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env(MODE, mode)
            .env(FILE, &file)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{mode}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

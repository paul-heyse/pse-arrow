// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual authored-to-resolved numerical controls; no source-AST compatibility fixture.
use super::{AdmittedBody, ProviderCall, Request, SelectedProviders, prepare_selected_providers};
use crate::workspace::{CompileError, CompilerContext};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::{
    MathError, guarded::CompiledBody, jets::EvaluationLimits, library::Optimization,
    typed::BodyLimits,
};
use pse_modeling::{
    Bindings, Limits, PhysicalScope, SpecializedModel, TypeContext,
    resolved::{RegionOutput, RegionRequest, SelectedRegion},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
};

/// Actual immutable owners retained by a test's consuming admission, not claimed support.
#[derive(Debug)]
pub(crate) struct DirectFixture {
    pub(crate) context: CompilerContext,
    pub(crate) model: SpecializedModel,
    pub(crate) region: SelectedRegion,
    pub(crate) providers: SelectedProviders,
    definition: SemanticId,
    physical: ContentHash,
    structure: ContentHash,
}
/// The public authored/check/specialize path issues every root and physical product.
/// `members` selects actual declared numerical members in the required output order.
pub(crate) fn fixture(
    context: CompilerContext,
    text: &str,
    members: &[&str],
) -> Result<DirectFixture, CompileError> {
    let rows = crate::authored_transfer_tests::rows(text);
    let root = crate::authored_transfer_tests::root(&rows, "direct", "Root");
    let scope = PhysicalScope::default();
    let types = TypeContext {
        admissions: None,
        formula_authority: None,
        quantities: &context.quantities,
        preconditions: context.preconditions.as_ref(),
        scope: &scope,
    };
    let checked = pse_modeling::check(&rows, &types)?;
    let model = pse_modeling::specialize(
        &checked,
        root,
        pse_modeling::specialize::root_instance(root),
        &Bindings {
            demand: members.iter().map(|member| (*member).to_owned()).collect(),
            ..Bindings::default()
        },
        Limits::default(),
    )?;
    let contracts = model.function_contracts(&checked);
    let outputs = members
        .iter()
        .map(|name| {
            let suffix = format!(".{name}");
            let selected = model
                .symbols
                .values()
                .filter(|symbol| symbol.lineage.path.ends_with(&suffix))
                .collect::<Vec<_>>();
            if selected.len() != 1 {
                return Err(CompileError::Missing(format!(
                    "exact fixture member {name}"
                )));
            }
            Ok(RegionOutput::Value(model.symbol_root(selected[0].id)?))
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    let region = model.prepare_region(
        RegionRequest {
            outputs: &outputs,
            validity: &[],
        },
        &contracts.context(),
    )?;
    let providers =
        prepare_selected_providers(
            &region.syntax,
            &region.support,
            &mut |name| {
                context.providers.get(name).cloned().ok_or_else(|| {
                    MathError::Contract(format!("unregistered fixture provider {name}"))
                })
            },
            &BTreeMap::new(),
            &contracts.context(),
        )?;
    let structure = region.syntax.identity()?;
    let mut physical = FramedHasher::new(pse_ids::Frame::MathPhysicalPremisesV1);
    region.support.frame(&mut physical);
    providers.frame(&mut physical);
    let physical = physical.finish_hash();
    Ok(DirectFixture {
        context,
        model,
        region,
        providers,
        definition: root.as_id(),
        physical,
        structure,
    })
}
impl DirectFixture {
    pub(crate) fn admit(&self, cancel: &Arc<AtomicBool>) -> Result<AdmittedBody, MathError> {
        pse_math::initialize()?;
        Request {
            definition: self.definition,
            syntax: &self.region.syntax,
            inputs: &self.region.inputs,
            support: &self.region.support,
            providers: &self.providers,
            physical: self.physical,
            structure: self.structure,
            limits: BodyLimits::default(),
        }
        .admit(
            &self.context.quantities,
            self.context.preconditions.as_ref(),
            cancel,
        )
    }
    fn compile(
        &self,
        admitted: &AdmittedBody,
        order: DerivativeOrder,
        optimization: Optimization,
    ) -> Result<Arc<CompiledBody>, MathError> {
        let cancel = pse_diagnostics::Cancellation::default();
        admitted
            .math()
            .compile(
                &(0..self.region.syntax.outputs.len()).collect::<Vec<_>>(),
                &(0..self.region.inputs.len()).collect::<Vec<_>>(),
                order,
                optimization,
                EvaluationLimits::default(),
                &pse_kernels::ExecutionScope::new(cancel, None),
            )
            .map(Arc::new)
    }
}
fn scalar(expression: &str) -> Result<DirectFixture, CompileError> {
    fixture(
        crate::authored_transfer_tests::context(),
        &format!("package direct {{ def Root {{ var x:Scalar; let out:Scalar={expression}; }} }}"),
        &["out"],
    )
}

#[test]
fn direct_powers_preserve_integral_degree_bound() {
    for expression in ["x^1025", "x^(-1025)"] {
        let owner = scalar(expression).unwrap();
        let error = owner.admit(&Arc::new(AtomicBool::new(false))).unwrap_err();
        assert!(
            error.to_string().contains("integral power degree"),
            "{error}"
        );
    }
    let owner = scalar("x^1024").unwrap();
    owner.admit(&Arc::new(AtomicBool::new(false))).unwrap();
}
#[test]
fn authored_power_literals_preserve_exact_facts_and_analytic_jets() {
    for (expression, x, value, first, second) in [
        ("x^2", 3.0, 9.0, 6.0, 2.0),
        ("x^-1", 2.0, 0.5, -0.25, 0.25),
        ("x^(1/2)", 4.0, 2.0, 0.25, -0.03125),
        ("x^(-1/2)", 4.0, 0.5, -0.0625, 0.0234375),
        ("x^(2/4)", 4.0, 2.0, 0.25, -0.03125),
    ] {
        let owner = scalar(expression).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let admitted = owner.admit(&cancel).unwrap();
        let artifact = owner
            .compile(&admitted, DerivativeOrder::Second, Optimization::default())
            .unwrap();
        let jet = artifact
            .worker()
            .evaluate(
                &[x],
                DerivativeOrder::Second,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel,
            )
            .unwrap();
        for (actual, expected) in [
            (jet.values[0], value),
            (jet.jacobian[0], first),
            (jet.hessians[0], second),
        ] {
            assert!(
                (actual - expected).abs() < 1e-12,
                "{expression}: {actual} != {expected}"
            );
        }
    }
    for expression in ["x^-1", "x^(1/2)"] {
        let owner = scalar(expression).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let admitted = owner.admit(&cancel).unwrap();
        let artifact = owner
            .compile(&admitted, DerivativeOrder::Second, Optimization::default())
            .unwrap();
        assert!(
            artifact
                .worker()
                .evaluate(
                    &[0.0],
                    DerivativeOrder::Second,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel
                )
                .is_err()
        );
    }
}
#[test]
fn source_compilation_preserves_obligations_erased_by_cas() {
    for expression in ["x/x", "exp(log(x))", "log(1/x)"] {
        let owner = scalar(expression).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let admitted = owner.admit(&cancel).unwrap();
        let artifact = owner
            .compile(&admitted, DerivativeOrder::Second, Optimization::default())
            .unwrap();
        let mut worker = artifact.worker();
        assert!(
            worker
                .evaluate(
                    &[0.0],
                    DerivativeOrder::Value,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel
                )
                .is_err()
        );
        assert!(
            worker
                .evaluate(
                    &[2.0],
                    DerivativeOrder::Value,
                    &mut pse_kernels::ProviderBindings::new(),
                    &cancel
                )
                .is_ok()
        );
    }
}
#[test]
fn typed_source_identity_excludes_whitespace_but_refreshes_original_occurrences() {
    let text = "package direct {def Root {var x:Scalar;let out:Scalar=x+1;}}";
    let first = fixture(crate::authored_transfer_tests::context(), text, &["out"]).unwrap();
    let shifted = fixture(
        crate::authored_transfer_tests::context(),
        &format!("\n\n{text}"),
        &["out"],
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    assert_eq!(
        first.admit(&cancel).unwrap().spec(),
        shifted.admit(&cancel).unwrap().spec()
    );
    assert_ne!(first.region.occurrences, shifted.region.occurrences);
    assert_eq!(
        first.model.symbols.keys().collect::<Vec<_>>(),
        shifted.model.symbols.keys().collect::<Vec<_>>()
    );
}
#[test]
fn physically_invalid_point_addition_fails_before_execution() {
    assert!(
        fixture(
            crate::authored_transfer_tests::context(),
            "package direct {def Root {var t:Temperature;let out:Temperature=t+t;}}",
            &["out"]
        )
        .is_err()
    );
    let owner = fixture(
        crate::authored_transfer_tests::context(),
        "package direct {def Root {var t:Temperature;let out:Delta<Temperature> =t-t;}}",
        &["out"],
    )
    .unwrap();
    let body = owner.admit(&Arc::new(AtomicBool::new(false))).unwrap();
    assert_eq!(
        body.quantities(),
        &[pse_quantity::standard::ids::quantity(
            "temperature.difference"
        )]
    );
}
#[test]
fn short_circuit_guards_do_not_touch_unsafe_predicates_or_values() {
    let owner = scalar("if x != 0 and 1/x > 0 then log(x) else 0").unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let admitted = owner.admit(&cancel).unwrap();
    let artifact = owner
        .compile(&admitted, DerivativeOrder::Value, Optimization::default())
        .unwrap();
    let mut worker = artifact.worker();
    for (x, expected) in [(0.0, 0.0), (-1.0, 0.0), (2.0, 2.0_f64.ln())] {
        let actual = worker
            .evaluate(
                &[x],
                DerivativeOrder::Value,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel,
            )
            .unwrap();
        assert!((actual.values[0] - expected).abs() < 1e-14);
    }
    let live = scalar("if x > 0 then x else 0").unwrap();
    let admitted = live.admit(&cancel).unwrap();
    assert!(
        live.compile(&admitted, DerivativeOrder::First, Optimization::default())
            .is_err()
    );
    let fixed = scalar("if true then x else log(-1)").unwrap();
    let admitted = fixed.admit(&cancel).unwrap();
    let artifact = fixed
        .compile(&admitted, DerivativeOrder::Second, Optimization::default())
        .unwrap();
    assert_eq!(
        artifact
            .worker()
            .evaluate(
                &[2.0],
                DerivativeOrder::Second,
                &mut pse_kernels::ProviderBindings::new(),
                &cancel
            )
            .unwrap()
            .values,
        vec![2.0]
    );
}
#[test]
fn ordered_outputs_share_admitted_semantics_across_artifact_profiles() {
    let text =
        "package direct {def Root {var x:Scalar;let square:Scalar=x*x;let successor:Scalar=x+1;}}";
    let owner = fixture(
        crate::authored_transfer_tests::context(),
        text,
        &["square", "successor"],
    )
    .unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let admitted = owner.admit(&cancel).unwrap();
    let first = owner
        .compile(&admitted, DerivativeOrder::First, Optimization::default())
        .unwrap();
    let second = owner
        .compile(
            &admitted,
            DerivativeOrder::Second,
            Optimization {
                cores: 2,
                ..Optimization::default()
            },
        )
        .unwrap();
    assert!(!Arc::ptr_eq(&first, &second));
    let jet = second
        .worker()
        .evaluate(
            &[2.0],
            DerivativeOrder::Second,
            &mut pse_kernels::ProviderBindings::new(),
            &cancel,
        )
        .unwrap();
    assert_eq!(jet.values, vec![4.0, 3.0]);
    assert_eq!(jet.hessians, vec![2.0, 0.0]);
    let reversed = fixture(
        crate::authored_transfer_tests::context(),
        text,
        &["successor", "square"],
    )
    .unwrap();
    assert_ne!(
        admitted.spec().key(),
        reversed.admit(&cancel).unwrap().spec().key()
    );
}

#[test]
fn executable_provider_admission_preserves_physics_phase_and_recoverable_errors() {
    use pse_kernels::{
        Port, Provider, ProviderError, ProviderFactory, ProviderSpec, ProviderValues, Registration,
    };
    #[derive(Debug, Clone)]
    struct Square(ProviderSpec);
    impl Provider for Square {
        fn fill(
            &mut self,
            x: &[f64],
            request: &pse_kernels::PreparedProviderRequest,
            _runtime: pse_kernels::ProviderRuntime<'_>,
            mut output: pse_kernels::ProviderOutput<'_>,
            context: &pse_kernels::EvaluationContext<'_>,
        ) -> Result<(), ProviderError> {
            request.check_context(context)?;
            if x.len() != 1 || x[0] < 0.0 {
                return Err(ProviderError::Trial("selected physical branch".into()));
            }
            if request.order != DerivativeOrder::Value {
                return Err(ProviderError::Contract("value-only implementation".into()));
            }
            output.copy_from(&ProviderValues {
                values: vec![x[0] * x[0]],
                jacobian: vec![],
                hessians: vec![],
            })
        }
    }
    impl ProviderFactory for Square {
        fn spec(&self) -> &ProviderSpec {
            &self.0
        }
        fn create(&self) -> Result<pse_kernels::ProviderBinding, ProviderError> {
            Ok(pse_kernels::ProviderBinding::new(
                self.spec().clone(),
                Box::new(self.clone()),
            ))
        }
    }
    let mut context = crate::authored_transfer_tests::context();
    let id = SemanticId::from_bytes([3; 16]);
    let hash = ContentHash::from_bytes([3; 32]);
    let quantity = context.quantities.neutral_dimensionless().unwrap();
    let unit = context
        .quantities
        .quantity_type(quantity)
        .unwrap()
        .canonical_unit;
    let port = Port { id, quantity, unit };
    let registration = Registration::new(
        Arc::new(Square(ProviderSpec {
            shapes: pse_kernels::ProviderShapes::default(),
            derivative_source: pse_kernels::DerivativeSource::Analytic,
            id,
            revision: hash,
            data: hash,
            inputs: vec![port.clone()],
            outputs: vec![port],
            derivatives: DerivativeOrder::Value,
            smoothness: DerivativeOrder::Second,
        })),
        &context.quantities,
    )
    .unwrap();
    context.providers.insert(
        "square".into(),
        ProviderCall {
            descriptor: registration.descriptor(),
            output: 0,
        },
    );
    let text = format!(
        "package direct {{ fn square(x:Scalar)->Scalar external \"square\" revision \"{hash}\" data \"{hash}\" output 0 derivatives 0 source analytic smoothness 2; def Root {{ var x:Scalar; let out:Scalar=square(x)+1; }} }}"
    );
    let owner = fixture(context.clone(), &text, &["out"]).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let admitted = owner.admit(&cancel).unwrap();
    assert!(
        owner
            .compile(&admitted, DerivativeOrder::First, Optimization::default())
            .is_err()
    );
    let artifact = owner
        .compile(&admitted, DerivativeOrder::Value, Optimization::default())
        .unwrap();
    let mut extra = context;
    extra.providers.insert(
        "unused".into(),
        ProviderCall {
            descriptor: registration.descriptor(),
            output: 0,
        },
    );
    let extra = fixture(extra, &text, &["out"]).unwrap();
    assert_eq!(extra.admit(&cancel).unwrap().spec(), admitted.spec());
    assert_eq!(owner.providers.descriptors().count(), 1);
    assert_eq!(owner.providers.calls().count(), 1);
    assert_eq!(admitted.spec().providers.len(), 1);
    let mut workers = pse_kernels::ProviderBindings::from([(
        registration.spec().key(),
        registration.worker().unwrap(),
    )]);
    let mut worker = artifact.worker();
    assert!(
        matches!(worker.evaluate(&[-1.0], DerivativeOrder::Value, &mut workers, &cancel),
        Err(MathError::Provider { source_id, .. }) if admitted.occurrences.iter().any(|occurrence| occurrence.id == source_id))
    );
    assert_eq!(
        worker
            .evaluate(&[3.0], DerivativeOrder::Value, &mut workers, &cancel)
            .unwrap()
            .values,
        vec![10.0]
    );
    let other = Square(ProviderSpec {
        revision: ContentHash::from_bytes([4; 32]),
        ..registration.spec().clone()
    });
    let mut incompatible = pse_kernels::ProviderBindings::from([(
        registration.spec().key(),
        pse_kernels::ProviderBinding::new(other.0.clone(), Box::new(other)),
    )]);
    assert!(
        worker
            .evaluate(&[3.0], DerivativeOrder::Value, &mut incompatible, &cancel)
            .is_err()
    );
}

#[test]
fn checked_compound_guards_preserve_original_arithmetic_occurrences() {
    for (body, cases) in [
        (
            "if x != 0 and 1/x > 0 then x*x else x+x",
            [(0.0, 0.0), (-2.0, -4.0), (2.0, 4.0)],
        ),
        (
            "if x == 0 or 1/x > 0 then x+x else x*x",
            [(0.0, 0.0), (-2.0, 4.0), (2.0, 4.0)],
        ),
    ] {
        let text = format!(
            "package direct {{fn guarded(x:Scalar)->Scalar={body};def Root {{var x:Scalar;let out:Scalar=guarded(x);}} }}"
        );
        let owner = fixture(crate::authored_transfer_tests::context(), &text, &["out"]).unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let admitted = owner.admit(&cancel).unwrap();
        assert!(
            admitted
                .occurrences
                .iter()
                .all(|occurrence| owner.region.occurrences.contains_key(&occurrence.id)),
            "the actual producer supplies source correspondence for every lowered root and function occurrence"
        );
        let artifact = owner
            .compile(&admitted, DerivativeOrder::Value, Optimization::default())
            .unwrap();
        let mut worker = artifact.worker();
        for (x, expected) in cases {
            assert_eq!(
                worker
                    .evaluate(
                        &[x],
                        DerivativeOrder::Value,
                        &mut pse_kernels::ProviderBindings::new(),
                        &cancel
                    )
                    .unwrap()
                    .values,
                vec![expected]
            );
        }
    }
}

#[test]
fn portable_direct_round_trip_preserves_specification_guards_and_exact_value() {
    let owner = scalar("1/x").unwrap();
    let cancelled = Arc::new(AtomicBool::new(false));
    let original = owner.admit(&cancelled).unwrap();
    let payload = original.portable_payload().unwrap();
    let digest = super::portable_payload_hash(&payload);
    let restored = super::portable::reconstruct_fixture(
        &payload,
        digest,
        original.spec(),
        &owner.context.quantities,
        &cancelled,
    )
    .unwrap();
    assert_eq!(original.spec(), restored.spec());
    assert_eq!(original.quantities(), restored.quantities());
    assert_eq!(
        original.math().checked_members(),
        restored.math().checked_members()
    );
    assert_eq!(
        original.math().checked_sources(),
        restored.math().checked_sources()
    );
    assert_eq!(
        original.math().occurrence_tokens(),
        restored.math().occurrence_tokens()
    );
    let artifact = owner
        .compile(&restored, DerivativeOrder::Second, Optimization::default())
        .unwrap();
    let mut worker = artifact.worker();
    let result = worker
        .evaluate(
            &[2.0],
            DerivativeOrder::Second,
            &mut pse_kernels::ProviderBindings::new(),
            &cancelled,
        )
        .unwrap();
    assert_eq!(result.values, &[0.5]);
    assert!(
        worker
            .evaluate(
                &[0.0],
                DerivativeOrder::Value,
                &mut pse_kernels::ProviderBindings::new(),
                &cancelled
            )
            .is_err()
    );
    let mut changed = payload.clone();
    changed[0] = b'[';
    assert!(
        super::portable::reconstruct_fixture(
            &changed,
            digest,
            original.spec(),
            &owner.context.quantities,
            &cancelled
        )
        .is_err()
    );
}

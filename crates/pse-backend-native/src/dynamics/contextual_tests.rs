// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure whole-contract method selection controls, independent of linked native execution.
use super::*;
use crate::{
    execution::{BuildObservation, Snapshot},
    routing::AssessmentState,
    solve::Backend,
};
fn id(n: u8) -> SemanticId {
    SemanticId::from_bytes([n; 16])
}
fn contract() -> Contract {
    Contract {
        identity: ContentHash::from_bytes([1; 32]),
        states: vec![id(1)],
        differential: vec![true],
        parameters: vec![id(2)],
        outputs: vec![id(3)],
        events: vec![vec![]],
        signs: vec![],
        quadratures: vec![],
        balances: vec![],
        derivatives: pse_kernels::DerivativeOrder::Second,
    }
}
fn snapshot() -> Snapshot {
    Snapshot {
        adapters: [Backend::Diffsol, Backend::Idas]
            .into_iter()
            .map(|backend| {
                (
                    backend,
                    BuildObservation {
                        linked: true,
                        identity: None,
                    },
                )
            })
            .collect(),
        ipopt: None,
        scip_omp_cancellation: false,
    }
}
fn profile() -> Profile {
    Profile {
        parameter_scales: vec![1.],
        ..Profile::default()
    }
}
fn assess(p: &Profile, c: &Contract) -> DynamicDecision {
    p.assess_method(
        c,
        &[1.],
        &snapshot(),
        DynamicDemand::Base,
        c.derivatives,
        &[],
    )
    .unwrap()
}
#[test]
fn contextual_unit_dynamic_auto_uses_complete_event_and_trial_contract() {
    let mut c = contract();
    c.events[0].push(Event {
        id: id(4),
        terminal: false,
        next_mode: 0,
        tolerance: 1e-8,
        direction: EventDirection::Rising,
    });
    assert_eq!(assess(&profile(), &c).ready().unwrap(), Method::Idas);
    let p = Profile {
        trial_failures: TrialPolicy::Recoverable,
        sensitivity: DynamicSensitivity::Forward,
        ..profile()
    };
    let decision = assess(&p, &c);
    assert!(decision.selected.is_none());
    assert_eq!(decision.candidates[0].causes.len(), 2);
    assert!(
        decision
            .candidates
            .iter()
            .all(|candidate| candidate.state == AssessmentState::Refused)
    );
    assert!(decision.ready().is_err());
}
#[test]
fn contextual_unit_dynamic_conflicting_method_settings_are_preserved() {
    let p = Profile {
        diffsol: DiffsolSettings {
            method: DiffsolMethod::Tsit45,
            ..DiffsolSettings::default()
        },
        idas: IdasSettings {
            sensitivity: SensitivityCorrector::Staggered,
            ..IdasSettings::default()
        },
        ..profile()
    };
    let decision = assess(&p, &contract());
    assert!(decision.selected.is_none());
    assert!(
        decision
            .candidates
            .iter()
            .all(|candidate| !candidate.causes.is_empty())
    );
}
#[test]
fn contextual_unit_dynamic_exact_hessian_requires_selected_artifact() {
    let p = Profile {
        sensitivity: DynamicSensitivity::Adjoint,
        ..profile()
    };
    let c = contract();
    let decision = p
        .assess_method(
            &c,
            &[1.],
            &snapshot(),
            DynamicDemand::ExactHessian,
            pse_kernels::DerivativeOrder::First,
            &[0],
        )
        .unwrap();
    assert_eq!(decision.candidate().unwrap(), Method::Idas);
    assert_eq!(
        decision.candidates[1].state,
        AssessmentState::SupportedPendingArtifacts
    );
    assert!(decision.ready().is_err());
    let explicit = Profile {
        method: Method::Diffsol,
        ..p
    };
    assert!(
        explicit
            .assess_method(
                &c,
                &[1.],
                &snapshot(),
                DynamicDemand::ExactHessian,
                pse_kernels::DerivativeOrder::Second,
                &[0]
            )
            .unwrap()
            .selected
            .is_none()
    );
}
#[test]
fn contextual_unit_dynamic_resource_refusal_never_selects_another_method() {
    let p = Profile {
        max_cells: 1,
        ..profile()
    };
    assert!(matches!(
        p.assess_method(
            &contract(),
            &[1.],
            &snapshot(),
            DynamicDemand::Base,
            pse_kernels::DerivativeOrder::Second,
            &[]
        ),
        Err(ProblemError::Limit { .. })
    ));
}

#[test]
fn contextual_unit_idas_refuses_kinsol_only_block_factor_before_native_execution() {
    let c = contract();
    let mut p = profile();
    p.method = Method::Idas;
    for preconditioner in [
        crate::solve::Preconditioner::None,
        crate::solve::Preconditioner::Jacobi,
        crate::solve::Preconditioner::BlockFactor,
    ] {
        p.idas.linear = IdasLinear::Spgmr {
            dimension: PositiveCount::try_new(3).unwrap(),
            preconditioner,
        };
        let causes = p.method_refusals(&c);
        if preconditioner == crate::solve::Preconditioner::BlockFactor {
            assert!(
                causes
                    .iter()
                    .any(|cause| matches!(cause, ProblemError::Unsupported(_)))
            );
        } else {
            assert!(causes.is_empty(), "{causes:?}");
        }
    }
}

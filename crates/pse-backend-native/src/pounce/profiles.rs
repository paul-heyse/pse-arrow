// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure library second-opinion descriptions lowered to acting, immutable profiles.
use super::Settings;
use crate::{
    ProblemError,
    solve::{Controls, OptionValue, SolveReport},
};
use pounce_rs::pounce_algorithm::second_opinion::{
    SecondOpinionAvailability, SecondOpinionTrigger, second_opinion_rungs,
};
/// One noncumulative native profile. Every member is derived from the baseline.
#[derive(Clone, Debug)]
pub struct SecondOpinionProfile {
    /// The exact library rung label.
    pub label: &'static str,
    /// Acting options, preserving baseline set-ness.
    pub controls: Controls,
    /// Acting factor settings; execution rebuilds incompatible factor factories.
    pub settings: Settings,
    /// Whether this rung requires explicitly permitted replacement-start recovery.
    pub replaces_start: bool,
}
/// Lower the library's finite ladder without invoking its private retry driver.
/// The common driver admits and assesses every returned profile separately.
pub fn second_opinion_profiles(
    baseline: &Controls,
    settings: &Settings,
    report: &SolveReport,
    allow_replacement: bool,
) -> Result<Vec<SecondOpinionProfile>, ProblemError> {
    use pounce_rs::ApplicationReturnStatus as Status;
    let native_status = [
        Status::InfeasibleProblemDetected,
        Status::InvalidNumberDetected,
        Status::RestorationFailed,
        Status::MaximumIterationsExceeded,
    ]
    .into_iter()
    .find(|status| i64::from(status.as_int()) == report.termination.code);
    let Some(trigger) = native_status.and_then(SecondOpinionTrigger::for_status) else {
        return Ok(Vec::new());
    };
    // Typed terminal observations always dominate a native numerical status.
    if report.validation_failure().is_some()
        || report.evidence.callback.terminal_failure
        || report.evidence.abandoned.is_some()
    {
        return Ok(Vec::new());
    }
    let stats = report.pounce_statistics.as_deref();
    let escalations = stats.map_or(0, |stats| {
        u64::try_from(stats.quality_escalations).unwrap_or(0)
    });
    let already_adaptive = matches!(baseline.options.get("mu_strategy"),Some(OptionValue::Text(value)) if value=="adaptive");
    let availability = SecondOpinionAvailability {
        trigger,
        scaling_retry_enabled: true,
        mu_retry_enabled: true,
        perturbed_start_retry_enabled: allow_replacement,
        already_mc64: matches!(
            settings.linear.scaling,
            feral::scaling::ScalingStrategy::Mc64Symmetric
        ),
        already_adaptive,
        already_perturbed: matches!(baseline.options.get("start_point_perturbation"),Some(OptionValue::Real(value)) if *value>0.0),
        increase_quality_retry_enabled: true,
        already_no_increase_quality: !settings.linear.increase_quality,
        baseline_quality_escalations: escalations,
        baseline_scaling: Some("typed"),
    };
    let mut profiles = Vec::new();
    for rung in second_opinion_rungs(availability) {
        let mut controls = baseline.clone();
        let mut settings = settings.clone();
        let mut replaces_start = false;
        match rung.label {
            "feral_scaling=mc64" => {
                settings.linear.scaling = feral::scaling::ScalingStrategy::Mc64Symmetric
            }
            "mu_strategy=adaptive" => {
                controls
                    .options
                    .insert("mu_strategy".into(), OptionValue::Text("adaptive".into()));
            }
            "start_point_perturbation=1e-2" => {
                replaces_start = true;
                controls
                    .options
                    .insert("start_point_perturbation".into(), OptionValue::Real(1e-2));
            }
            "feral_increase_quality=no" => settings.linear.increase_quality = false,
            _ => {
                return Err(ProblemError::Contract(format!(
                    "unmapped library second-opinion rung {}",
                    rung.label
                )));
            }
        }
        profiles.push(SecondOpinionProfile {
            label: rung.label,
            controls,
            settings,
            replaces_start,
        });
    }
    Ok(profiles)
}

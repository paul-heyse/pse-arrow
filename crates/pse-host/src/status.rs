// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent native status observations; metadata never substitutes for live/drain evidence.
use crate::{
    HostError,
    catalog::description::Information,
    envelope::{self, Completeness, Envelope, ErrorRecord, RedactionPolicy},
    environment::Environment,
    generation,
    invocation::InvocationStore,
    paths::CheckoutState,
    repository,
};
use schemars::JsonSchema;
use serde::Serialize;
use std::{
    path::{Component, Path},
    sync::{Arc, atomic::AtomicBool},
};

/// Navigation copied only from the current index, not interpretation of a plan's status.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct PlanLink {
    /// Original rendered link label.
    pub label: String,
    /// Index-relative plan document path.
    pub document: String,
    /// Original fragment, when present.
    pub fragment: Option<String>,
}
/// Index order is navigation, never authorization or a completion claim.
#[derive(Debug, Serialize, JsonSchema)]
pub struct PlanNavigation {
    /// The only inspected plan source.
    pub source: String,
    /// First listed plan pointer; absence means the index listed no plan links.
    pub primary: Option<PlanLink>,
    /// All numeric plan links in source order, without following them.
    pub listed: Vec<PlanLink>,
}
/// Selected facts from a retained response, preserving its independent result dimensions.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ResponseFacts {
    /// Retained command outcome.
    pub outcome: envelope::CommandOutcome,
    /// Retained owner-supplied subject state, not a fresh live observation.
    pub subject_state: envelope::SubjectState,
    /// Original child status, when supplied.
    pub child_exit: Option<i32>,
    /// Retained reconciliation statement, not newly inferred from record existence.
    pub settlement: envelope::Settlement,
    /// Retained observation coverage and independent deadline.
    pub observation: envelope::Observation,
}
/// Authenticated metadata presence, explicitly distinct from running/finished state.
#[derive(Debug, Serialize, JsonSchema)]
pub struct InvocationPresence {
    /// Original CLI invocation identity.
    pub invocation_id: envelope::CliInvocationId,
    /// Original selected verb.
    pub command: String,
    /// Metadata creation time; this is not workload start time.
    pub created_unix_ms: u64,
    /// Original session association.
    pub session: Option<String>,
    /// Retained response when present; absence is not drain or liveness evidence.
    pub recorded_response: Information<Option<ResponseFacts>>,
    /// Actual current-work observation is outside the metadata reader's authority.
    pub live_state: envelope::SubjectState,
}
/// One composed status response. Each source remains independently available/unavailable.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Inspected checkout binding, not a path-derived scientific identity.
    pub checkout_id: Information<String>,
    /// Shared repository binding.
    pub repository_id: Information<String>,
    /// Native HEAD/index/tracked/untracked facts, retaining byte-path identity.
    pub repository: Information<repository::RepositoryObservation>,
    /// Navigation from the current README links only.
    pub plans: Information<PlanNavigation>,
    /// Authenticated source records; no active unit or process inference.
    pub invocations: Information<Vec<InvocationPresence>>,
    /// Passive prepared-host/source equivalence observation.
    pub host_tools: Information<generation::Freshness>,
    /// Main/linked worktrees when explicitly requested.
    pub worktrees: Information<Vec<repository::WorktreeObservation>>,
    /// Product/canonical interpretation requires its actual supplied owner association.
    pub product_observation: Information<()>,
    /// Neither directory traversal nor Git status is an atomic multi-source snapshot.
    pub atomic_snapshot: bool,
}
/// Extract actual CommonMark links, including reference links and table link labels.
pub fn plan_links(source: &str) -> PlanNavigation {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
    let mut current: Option<(String, String)> = None;
    let mut listed = Vec::new();
    for event in Parser::new_ext(source, Options::ENABLE_TABLES) {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                current = Some((dest_url.into_string(), String::new()))
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some((_, label)) = &mut current {
                    label.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some((_, label)) = &mut current {
                    label.push(' ');
                }
            }
            Event::End(TagEnd::Link) => {
                if let Some((target, label)) = current.take() {
                    let (document, fragment) = target
                        .split_once('#')
                        .map_or((target.as_str(), None), |(path, fragment)| {
                            (path, Some(fragment.to_owned()))
                        });
                    let path = Path::new(document);
                    if document.ends_with(".md")
                        && document.as_bytes().first().is_some_and(u8::is_ascii_digit)
                        && path
                            .components()
                            .all(|part| matches!(part, Component::Normal(_)))
                    {
                        listed.push(PlanLink {
                            label,
                            document: document.into(),
                            fragment,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    PlanNavigation {
        source: "docs/plans/README.md".into(),
        primary: listed.first().cloned(),
        listed,
    }
}
fn unavailable<T>(reason: impl Into<String>) -> Information<T> {
    Information::Unavailable {
        reason: reason.into(),
    }
}
fn capture<T>(
    scope: &str,
    result: Result<T, HostError>,
    failures: &mut Vec<ErrorRecord>,
    limitations: &mut Vec<String>,
    policy: &RedactionPolicy,
) -> Information<T> {
    match result {
        Ok(value) => Information::Available(value),
        Err(error) => {
            failures.push(ErrorRecord::from_typed(&error, policy));
            limitations.push(format!(
                "{scope} unavailable; inspect its original structured diagnostic"
            ));
            unavailable(format!("{scope} could not be established"))
        }
    }
}
/// Read status without environment activation, workload admission, rebuild or state creation.
pub fn inspect(
    root: &Path,
    inputs: &Environment,
    all_worktrees: bool,
    interrupt: Arc<AtomicBool>,
) -> Envelope<Report> {
    let policy = RedactionPolicy::from_environment();
    let mut failures = Vec::new();
    let mut limitations = Vec::new();
    let repository = capture(
        "Git status",
        repository::observe(root, interrupt).map_err(Into::into),
        &mut failures,
        &mut limitations,
        &policy,
    );
    let plans = capture(
        "current plan index",
        std::fs::read_to_string(root.join("docs/plans/README.md"))
            .map(|source| plan_links(&source))
            .map_err(Into::into),
        &mut failures,
        &mut limitations,
        &policy,
    );
    let host_tools = capture(
        "prepared host freshness",
        generation::freshness(root, inputs),
        &mut failures,
        &mut limitations,
        &policy,
    );
    let (checkout_id, repository_id, invocations) = match CheckoutState::resolve(root, inputs) {
        Ok(state) => {
            let checkout_id = Information::Available(state.checkout_id.as_str().to_owned());
            let repository_id = Information::Available(state.repository_id.as_str().to_owned());
            let store = InvocationStore::new(state);
            let invocations=match store.history::<Box<serde_json::value::RawValue>>() {
                Ok(records)=>Information::Available(records.into_iter().map(|record| {
                    let id=&record.selection.invocation_id;
                    let response=store.result::<Box<serde_json::value::RawValue>,Box<serde_json::value::RawValue>>(id).map(|value|value.map(|response|ResponseFacts{outcome:response.outcome,subject_state:response.subject_state,child_exit:response.child_exit,settlement:response.settlement,observation:response.observation}));
                    InvocationPresence{recorded_response:capture("retained invocation response",response,&mut failures,&mut limitations,&policy),invocation_id:record.selection.invocation_id,command:record.selection.command,created_unix_ms:record.created_unix_ms,session:record.selection.session,live_state:envelope::SubjectState::Unknown}
                }).collect()),
                Err(error)=>capture::<Vec<InvocationPresence>>("invocation metadata",Err(error),&mut failures,&mut limitations,&policy),
            };
            (checkout_id, repository_id, invocations)
        }
        Err(error) => {
            failures.push(ErrorRecord::from_typed(&error, &policy));
            limitations
                .push("checkout identity unavailable; invocation metadata was not read".into());
            (
                unavailable("checkout binding unavailable"),
                unavailable("repository binding unavailable"),
                unavailable("checkout invocation source unavailable"),
            )
        }
    };
    let worktrees = if all_worktrees {
        match repository::observe_worktrees(root) {
            Ok(inventory) => {
                for failure in inventory.failures {
                    failures.push(ErrorRecord::from_typed(&failure.error, &policy));
                    limitations.push(format!(
                        "worktree observation unavailable: {:?}",
                        failure.git_dir
                    ));
                }
                Information::Available(inventory.entries)
            }
            Err(error) => capture(
                "worktree inventory",
                Err(error.into()),
                &mut failures,
                &mut limitations,
                &policy,
            ),
        }
    } else {
        unavailable("not requested; select --all to enumerate main and linked worktrees")
    };
    let report = Report {
        checkout_id,
        repository_id,
        repository,
        plans,
        invocations,
        host_tools,
        worktrees,
        product_observation: unavailable(
            "no product-owner observation association was supplied; metadata cannot infer scientific or canonical state",
        ),
        atomic_snapshot: false,
    };
    let mut response = Envelope::response(
        "status",
        if failures.is_empty() {
            "native status observations available"
        } else {
            "native status observations partially unavailable"
        },
        report,
    );
    response.failures = failures;
    response.observation.scope = "requested_native_status_sources".into();
    response.observation.limitations = limitations;
    if !response.failures.is_empty() {
        response.observation.completeness = Completeness::Partial;
    }
    response
}
#[cfg(test)]
mod tests;

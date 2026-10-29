// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Direct, read-only diagnosis. Unavailable manager evidence is never empty success.
use crate::{HostError, HostManager, UnitObservation};
use pse_diagnostics::process_facts::{ProcessIdentity, ProcessRead, ProcessReader};
use serde::Serialize;

/// Facts about the observer, explicitly distinct from a requested unit's members.
#[derive(Debug, Serialize)]
pub struct ObserverFacts {
    /// Authenticated procfs identity when readable.
    pub identity: Option<ProcessIdentity>,
    /// Observer's unified hierarchy membership, not the requested unit's group.
    pub control_group: Option<String>,
    /// Missing or inconsistent kernel observations.
    pub unavailable: Vec<String>,
}

/// Partial diagnosis is useful even when workload admission cannot operate.
#[derive(Debug, Serialize)]
pub struct ActivityObservation {
    /// Explicit selection; absence selects the observer's actual manager unit.
    pub requested_unit: Option<String>,
    /// Authoritative manager observation, absent on failure.
    pub manager: Option<UnitObservation>,
    /// Original manager failure, not a guessed inactive state.
    pub manager_unavailable: Option<String>,
    /// Independently available observer facts; these grant no unit mutation rights.
    pub observer: ObserverFacts,
}

fn unified_group(contents: &str) -> Result<String, &'static str> {
    let mut groups = contents.lines().filter_map(|line| line.strip_prefix("0::"));
    let Some(group) = groups.next() else {
        return Err("observer cgroup-v2 membership unavailable");
    };
    if groups.next().is_some()
        || !group.starts_with('/')
        || std::path::Path::new(group).components().any(|part| {
            !matches!(part, std::path::Component::RootDir | std::path::Component::Normal(_))
        })
    {
        return Err("observer cgroup-v2 membership ambiguous or invalid");
    }
    Ok(group.to_owned())
}

impl ObserverFacts {
    fn read() -> Self {
        let mut unavailable = Vec::new();
        let identity = match ProcessReader::default().process(std::process::id()) {
            ProcessRead::Present(facts) => Some(facts.identity),
            ProcessRead::Absent => {
                unavailable.push("observer process disappeared".into());
                None
            }
            ProcessRead::Changed => {
                unavailable.push("observer process identity changed during observation".into());
                None
            }
            ProcessRead::Unavailable(detail) => {
                unavailable.push(detail);
                None
            }
        };
        let control_group = match std::fs::read_to_string("/proc/self/cgroup") {
            Ok(contents) => match unified_group(&contents) {
                Ok(group) => Some(group),
                Err(detail) => {
                    unavailable.push(detail.into());
                    None
                }
            },
            Err(error) => {
                unavailable.push(format!("observer cgroup: {error}"));
                None
            }
        };
        Self { identity, control_group, unavailable }
    }
}

impl ActivityObservation {
    fn assemble(requested_unit: Option<&str>, observer: ObserverFacts, result: Result<UnitObservation, HostError>) -> Self {
        let (manager, manager_unavailable) = match result {
            Ok(observation) => (Some(observation), None),
            Err(error) => (None, Some(error.to_string())),
        };
        Self { requested_unit: requested_unit.map(str::to_owned), manager, manager_unavailable, observer }
    }

    /// Read without environment activation, admission, source scans or repair effects.
    pub async fn read(requested_unit: Option<&str>) -> Self {
        let observer = ObserverFacts::read();
        let result = async {
            let manager = HostManager::connect_user(None).await?;
            match requested_unit {
                Some(unit) => manager.borrow(unit).observe().await,
                None => manager.current_unit().await,
            }
        }.await;
        Self::assemble(requested_unit, observer, result)
    }

    /// A partial observation is not a successfully observed manager unit.
    pub fn complete(&self) -> bool {
        self.manager.is_some() && self.observer.unavailable.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_manager_keeps_observer_facts_separate_from_selected_unit() {
        let report = ActivityObservation::assemble(Some("selected.service"), ObserverFacts {
            identity: None, control_group: Some("/observer.scope".into()), unavailable: vec![],
        }, Err(HostError::Ownership("bus unavailable".into())));
        assert!(!report.complete());
        assert!(report.manager.is_none());
        assert_eq!(report.requested_unit.as_deref(), Some("selected.service"));
        assert_eq!(report.observer.control_group.as_deref(), Some("/observer.scope"));
        assert!(report.manager_unavailable.as_deref().unwrap().contains("bus unavailable"));
    }

    #[test]
    fn unified_group_rejects_ambiguous_missing_and_relative_facts() {
        assert_eq!(unified_group("0::/user.slice/a.scope\n").unwrap(), "/user.slice/a.scope");
        for invalid in ["1:cpu:/a\n", "0::relative", "0::/a\n0::/b", "0::/a/../b"] {
            assert!(unified_group(invalid).is_err(), "{invalid:?}");
        }
    }
}

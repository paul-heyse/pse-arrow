// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native file discovery, relatedness and passive update intent retain distinct authorities.
use super::*;
use crate::semantic::{FactLocation, RunnableTargetFact, SnapshotUpdateFact, TestCandidateFact};

pub(super) fn inspect(
    db: &ra_ap_ide_db::RootDatabase,
    paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
    position: ra_ap_ide::FilePosition,
    targets: &[RunnableTargetFact],
    limit: usize,
) -> Result<(Vec<TestCandidateFact>, Vec<Section>)> {
    let analysis = ra_ap_ide::AnalysisHost::with_database(db.clone()).analysis();
    let discovered = native_result(analysis.runnables(position.file_id))?;
    let related = native_result(analysis.related_tests(position, None))?;
    let key = |runnable: &ra_ap_ide::Runnable| {
        (
            runnable.nav.file_id,
            runnable.nav.full_range,
            runnable.nav.focus_range,
            runnable.kind.clone(),
        )
    };
    let discovered_keys = discovered
        .iter()
        .map(&key)
        .collect::<std::collections::HashSet<_>>();
    let related_keys = related
        .iter()
        .map(&key)
        .collect::<std::collections::HashSet<_>>();
    let mut seen = std::collections::HashSet::new();
    let sema = Semantics::new(db);
    let mut tests = Vec::new();
    for runnable in related.into_iter().chain(discovered) {
        let identity = key(&runnable);
        if !seen.insert(identity.clone()) {
            continue;
        }
        let (kind, test_id) = match &runnable.kind {
            ra_ap_ide::RunnableKind::Test { test_id } => ("test", Some(test_id.to_string())),
            ra_ap_ide::RunnableKind::TestMod { path } => ("test_module", Some(path.clone())),
            ra_ap_ide::RunnableKind::Bench { test_id } => ("bench", Some(test_id.to_string())),
            ra_ap_ide::RunnableKind::DocTest { test_id } => ("doctest", Some(test_id.to_string())),
            ra_ap_ide::RunnableKind::Bin => ("binary", None),
        };
        let crates = sema
            .file_to_module_defs(runnable.nav.file_id)
            .map(|module| module.krate(db))
            .collect::<std::collections::HashSet<_>>();
        let roots = crates
            .iter()
            .filter_map(|krate| paths.get(&krate.root_file(db)))
            .collect::<BTreeSet<_>>();
        let cargo_targets = targets
            .iter()
            .filter(|target| roots.contains(&target.root))
            .cloned()
            .collect::<Vec<_>>();
        let cfg_eligible = if crates.len() == 1 {
            crates.iter().next().and_then(|krate| {
                runnable
                    .cfg
                    .as_ref()
                    .map_or(Some(true), |cfg| krate.cfg(db).check(cfg))
            })
        } else {
            None
        };
        let test_profile = if crates.len() == 1 {
            crates.iter().next().map(|krate| {
                krate
                    .cfg(db)
                    .get_cfg_keys()
                    .any(|key| key.as_str() == "test")
            })
        } else {
            None
        };
        let feature_eligible = match cargo_targets.as_slice() {
            [] => None,
            [target] => Some(target.required_features_satisfied),
            alternatives
                if alternatives
                    .iter()
                    .all(|target| !target.required_features_satisfied) =>
            {
                Some(false)
            }
            _ => None,
        };
        let requires_test_profile = matches!(
            runnable.kind,
            ra_ap_ide::RunnableKind::Test { .. }
                | ra_ap_ide::RunnableKind::TestMod { .. }
                | ra_ap_ide::RunnableKind::Bench { .. }
        );
        let eligible = if cfg_eligible == Some(false)
            || feature_eligible == Some(false)
            || (requires_test_profile && test_profile == Some(false))
        {
            Some(false)
        } else if cfg_eligible == Some(true)
            && feature_eligible == Some(true)
            && (!requires_test_profile || test_profile == Some(true))
        {
            Some(true)
        } else {
            None
        };
        let related = related_keys.contains(&identity);
        let discovered = discovered_keys.contains(&identity);
        let enclosing = runnable.nav.file_id == position.file_id
            && runnable.nav.full_range.contains(position.offset);
        let reason = if related {
            "native related_tests traversal from selected occurrence"
        } else if enclosing {
            "native enclosing runnable from file discovery"
        } else {
            "native file runnable discovery; no per-item relatedness established"
        };
        tests.push(TestCandidateFact {
            test_id,cfg:runnable.cfg.as_ref().map(|cfg|format!("{cfg:?}")),relatedness:Some(reason.into()),eligible,cfg_eligible,cargo_targets,
            snapshot_update:Some(SnapshotUpdateFact {expect_test:runnable.update_test.expect_test,insta:runnable.update_test.insta,snapbox:runnable.update_test.snapbox}),
            label:Some(runnable.nav.name.to_string()),
            location:location(paths,runnable.nav.file_id,runnable.nav.focus_range.unwrap_or(runnable.nav.full_range)).as_ref().map(fact_location),
            runnable_kind:Some(kind.into()),cargo_args:Vec::new(),executable_args:Vec::new(),
            original:json!({"kind":kind,"cfg":runnable.cfg.as_ref().map(|cfg|format!("{cfg:?}")),"native_generation":1,"file_discovery":discovered,"related_test":related,"enclosing":enclosing,"native_test_profile":test_profile,"eligibility":"selected native cfg/test profile and captured Cargo required features only; no rustc-unit admission, compilation, execution or coverage proof","snapshot_update":"passive native macro-use intent; no update authorization or environment emitted"}),
        });
    }
    // A required-feature-disabled binary can lack a native crate. Its original
    // Cargo target is still an ineligible coarse candidate, not an invented HIR runnable.
    let binary_targets = targets
        .iter()
        .filter(|target| {
            target.kind == "binary" && paths.get(&position.file_id) == Some(&target.root)
        })
        .cloned()
        .collect::<Vec<_>>();
    if let Some(path) = paths.get(&position.file_id)
        && !binary_targets.is_empty()
        && !tests
            .iter()
            .any(|test| test.runnable_kind.as_deref() == Some("binary"))
    {
        let eligible = binary_targets
            .iter()
            .all(|target| !target.required_features_satisfied)
            .then_some(false);
        let label = match binary_targets.as_slice() {
            [target] => Some(target.name.clone()),
            _ => None,
        };
        tests.push(TestCandidateFact {
            test_id:None,cfg:None,relatedness:Some("captured Cargo binary target at selected source; native runnable unavailable".into()),eligible,cfg_eligible:None,cargo_targets:binary_targets,snapshot_update:None,label,location:Some(FactLocation::Bytes {path:path.clone(),start:0,end:0}),runnable_kind:Some("binary".into()),cargo_args:Vec::new(),executable_args:Vec::new(),original:json!({"origin":"captured_cargo_target","native_generation":1,"native_runnable":"unavailable","eligibility":"only failure of every matching target's required features establishes ineligibility; plural target correspondence and successful features alone do not establish a native runnable or rustc-unit admission"}),
        });
    }
    let total = tests.len();
    tests.truncate(limit);
    let mut inactive = Vec::new();
    let mut modules = std::collections::HashSet::new();
    for module in sema.file_to_module_defs(position.file_id) {
        if !modules.insert(module) {
            continue;
        }
        let mut diagnostics = Vec::new();
        module.diagnostics(db, &mut diagnostics, false);
        for diagnostic in diagnostics {
            let ra_ap_hir::AnyDiagnostic::InactiveCode(diagnostic) = diagnostic else {
                continue;
            };
            let range = sema.diagnostics_display_range(diagnostic.node);
            if range.file_id != position.file_id {
                continue;
            }
            let mapped = location(paths, range.file_id, range.range);
            inactive.push(json!({"location":mapped,"cfg":format!("{:?}",diagnostic.cfg),"cfg_eligible":diagnostic.opts.check(&diagnostic.cfg),"runnable_kind":"unavailable: native inactive code is not an enumerated runnable"}));
        }
    }
    let inactive_total = inactive.len();
    inactive.truncate(limit);
    Ok((
        tests,
        vec![Section {
            category: "native_runnable_discovery".into(),
            evidence: json!({"file_discovery":discovered_keys.len(),"related_tests":related_keys.len(),"unique_candidates":total,"scope":"current file native runnables plus related tests; discovery and relevance are distinct","cfg_coverage":"inactive HIR items are excluded from runnable enumeration; native inactive regions remain separately observed","inactive_regions":inactive,"snapshot_updates":"passive intent; no update action, authorization or environment","execution":"not_run"}),
            omitted: total.saturating_sub(limit) + inactive_total.saturating_sub(limit),
            handles: Vec::new(),
        }],
    ))
}

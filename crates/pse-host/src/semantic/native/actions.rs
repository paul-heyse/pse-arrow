// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Native assists remain lazy and share the immutable proposal owner with LSP edits.
use super::*;

fn config() -> ra_ap_ide::AssistConfig {
    ra_ap_ide::AssistConfig {
        snippet_cap: None,
        allowed: None,
        insert_use: ra_ap_ide_db::imports::insert_use::InsertUseConfig {
            granularity: ra_ap_ide_db::imports::insert_use::ImportGranularity::Crate,
            enforce_granularity: true,
            prefix_kind: ra_ap_hir::PrefixKind::Plain,
            group: true,
            skip_glob_imports: true,
        },
        prefer_no_std: false,
        prefer_prelude: true,
        prefer_absolute: false,
        assist_emit_must_use: false,
        term_search_fuel: 400,
        code_action_grouping: true,
        expr_fill_default: ra_ap_ide_db::assists::ExprFillDefaultMode::Todo,
        prefer_self_ty: false,
        show_rename_conflicts: true,
    }
}
fn record(assist: &ra_ap_ide::Assist) -> Value {
    json!({"id":assist.id.0,"kind":assist.id.1.name(),"subtype":assist.id.2,"label":assist.label.to_string(),"start":usize::from(assist.target.start()),"end":usize::from(assist.target.end()),"term_search_fuel":400,"completeness":"unknown","imports":"native SourceChange only; synthesis imports require review"})
}
pub(super) fn assists(
    analysis: &ra_ap_ide::Analysis,
    position: ra_ap_ide::FilePosition,
    query: &Query,
) -> Result<(Section, Option<ra_ap_ide_db::source_change::SourceChange>)> {
    let config = config();
    // Disable diagnostic work: this request asks for transformations, with laziness
    // owned by AssistResolveStrategy rather than eager computation plus truncation.
    let mut diagnostics = ra_ap_ide::DiagnosticsConfig::test_sample();
    diagnostics.enabled = false;
    let range = ra_ap_ide::FileRange {
        file_id: position.file_id,
        range: native_ra_syntax::TextRange::empty(position.offset),
    };
    let assists = native_result(analysis.assists_with_fixes(
        &config,
        &diagnostics,
        ra_ap_ide::AssistResolveStrategy::None,
        range,
    ))?;
    let rows = assists.iter().map(record).collect::<Vec<_>>();
    if let Kind::Assist { index } = query.kind {
        if query.if_source.is_none()
            || query.if_generation.is_none()
            || query.if_candidate.is_none()
        {
            return Err(Error::InvalidRequest("native assist resolution requires source, generation and original candidate fingerprint".into()));
        }
        let assist = assists
            .get(index)
            .ok_or_else(|| Error::Unavailable("native assist candidate is absent".into()))?;
        super::super::engine::check_candidate(query, &rows[index])?;
        let resolve = ra_ap_ide::AssistResolveStrategy::Single(ra_ap_ide::SingleResolve {
            assist_id: assist.id.0.into(),
            assist_kind: assist.id.1,
            assist_subtype: assist.id.2,
        });
        let resolved =
            native_result(analysis.assists_with_fixes(&config, &diagnostics, resolve, range))?;
        let resolved = resolved
            .into_iter()
            .find(|candidate| candidate.id == assist.id)
            .ok_or(Error::Stale)?;
        let evidence = record(&resolved);
        let change = resolved.source_change.ok_or_else(|| {
            Error::Unavailable("native assist did not supply a SourceChange".into())
        })?;
        Ok((
            Section {
                category: "resolved_native_assist".into(),
                evidence,
                omitted: 0,
                handles: Vec::new(),
            },
            Some(change),
        ))
    } else {
        Ok((
            super::super::engine::section(
                "assist_candidates",
                serde_json::to_value(rows)?,
                query.limit,
            ),
            None,
        ))
    }
}

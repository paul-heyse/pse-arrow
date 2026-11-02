// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Occurrence selection lowers in the entity's prepared world, without narrowing resolution.
use super::*;
use ra_ap_ide_db::search::SearchScope;

pub(super) struct Occurrences {
    pub native: Option<SearchScope>,
    selected: Option<BTreeMap<PathBuf, Vec<Option<std::ops::Range<usize>>>>>,
    pub limitations: Vec<String>,
}
impl Occurrences {
    pub(super) fn prepare(
        db: &ra_ap_ide_db::RootDatabase,
        files: &BTreeMap<PathBuf, ra_ap_vfs::FileId>,
        digests: &BTreeMap<PathBuf, String>,
        requested: Option<&[super::super::OccurrenceFile]>,
    ) -> Result<Self> {
        let Some(requested) = requested else {
            return Ok(Self {
                native: None,
                selected: None,
                limitations: Vec::new(),
            });
        };
        let sema = Semantics::new(db);
        let mut selected = BTreeMap::<_, Vec<_>>::new();
        let mut ids = BTreeSet::new();
        let mut ranged = Vec::new();
        let mut limitations = Vec::new();
        for entry in requested {
            let path = entry.file.canonicalize()?;
            let id = *files.get(&path).ok_or_else(|| {
                context_failure(
                    super::super::FailureReason::OutsidePreparedWorld,
                    "reference_scope",
                    Some(&path),
                    "reference occurrence source is outside the prepared world",
                )
            })?;
            let source =
                crate::source_read::SourceRequest::Working { path: path.clone() }.acquire()?;
            if digests
                .get(&path)
                .is_none_or(|digest| digest != source.identity().source_sha256())
            {
                return Err(context_failure(
                    super::super::FailureReason::SourceChanged,
                    "reference_scope",
                    Some(&path),
                    "reference occurrence source differs from the prepared snapshot",
                ));
            }
            if let Some(range) = &entry.range
                && (range.start > range.end
                    || source.text().get(range.clone()).is_none()
                    || u32::try_from(range.end).is_err())
            {
                return Err(Error::InvalidRequest("reference occurrence range must be ordered UTF-8 byte boundaries within its original file".into()));
            }
            let interpretations = sema
                .file_to_module_defs(id)
                .map(|module| ra_ap_hir::EditionedFileId::new(db, id, module.krate(db).edition(db)))
                .collect::<BTreeSet<_>>();
            if interpretations.is_empty() {
                return Err(context_failure(
                    super::super::FailureReason::InactiveSource,
                    "reference_scope",
                    Some(&path),
                    "selected occurrence source has no active module in the prepared native context",
                ));
            }
            for editioned in interpretations {
                ids.insert(editioned);
                if let Some(range) = &entry.range {
                    ranged.push((editioned, range.clone()));
                }
            }
            selected.entry(path).or_default().push(entry.range.clone());
        }
        let mut native = SearchScope::files(&ids.into_iter().collect::<Vec<_>>());
        if !ranged.is_empty() {
            if requested.len() == 1 && ranged.len() == 1 {
                let (file_id, range) = &ranged[0];
                native = native.intersection(&SearchScope::file_range(ra_ap_hir::FileRange {
                    file_id: *file_id,
                    range: native_ra_syntax::TextRange::new(
                        (range.start as u32).into(),
                        (range.end as u32).into(),
                    ),
                }));
            } else {
                // 0.0.358 exposes no public union constructor for per-file ranges.
                // Keep every edition/module interpretation and filter exact original
                // occurrences after native execution instead of dropping targets.
                limitations.push("native occurrence files were restricted before reference search; plural byte ranges require exact original-occurrence filtering after native search because the pinned SearchScope has no public range-union constructor".into());
            }
        }
        Ok(Self {
            native: Some(native),
            selected: Some(selected),
            limitations,
        })
    }
    pub(super) fn contains(
        &self,
        paths: &BTreeMap<ra_ap_vfs::FileId, PathBuf>,
        id: ra_ap_vfs::FileId,
        range: native_ra_syntax::TextRange,
    ) -> bool {
        let Some(selected) = &self.selected else {
            return true;
        };
        paths
            .get(&id)
            .and_then(|path| selected.get(path))
            .is_some_and(|ranges| {
                ranges.iter().any(|selected| {
                    selected.as_ref().is_none_or(|selected| {
                        selected.start <= usize::from(range.start())
                            && usize::from(range.end()) <= selected.end
                    })
                })
            })
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Publication controls for the projection and writer boundaries.
use super::*;
use datafusion::arrow::ipc::{reader::FileReader, writer::FileWriter};
use std::{
    collections::BTreeSet,
    io::{self, Write},
};

struct RefusingWriter;
impl Write for RefusingWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("controlled IPC writer refusal"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn report_export_resolves_projections_before_writing_or_refusing_destinations() -> Result<()>
{
    let (directory, path) = fixture()?;
    let campaign = Campaign::load(&path, Some("first"), None)?;
    let run = campaign.runs().next().context("first run")?;
    let mut report = campaign
        .run_pure(run, directory.path(), &CancelSource::new())
        .await?;
    let target = directory.path().join("report.arrow");
    let existing = target.with_extension("fixtures.arrow");
    std::fs::write(&existing, b"original destination")?;
    let entries = || {
        std::fs::read_dir(directory.path())?
            .map(|entry| entry.map(|value| value.file_name()))
            .collect::<io::Result<BTreeSet<_>>>()
    };
    let original_entries = entries()?;
    let missing_parent = directory.path().join("missing/report.arrow");

    // The first temporary writer cannot be opened in a missing parent.
    let error = report.export(&missing_parent).unwrap_err();
    assert_eq!(
        error.downcast_ref::<io::Error>().unwrap().kind(),
        io::ErrorKind::NotFound,
    );
    // A later destination refusal leaves precisely the preceding finished IPC file.
    let error = report.export(&target).unwrap_err();
    assert_eq!(
        error
            .downcast_ref::<tempfile::PersistError>()
            .unwrap()
            .error
            .kind(),
        io::ErrorKind::AlreadyExists,
    );
    drop(error);
    let first = report.report.table()?;
    let mut reader = FileReader::try_new(std::fs::File::open(&target)?, None)?;
    assert_eq!(reader.schema(), first.batch().schema());
    assert_eq!(reader.next().transpose()?.as_ref(), Some(first.batch()));
    assert!(reader.next().is_none());
    assert_eq!(std::fs::read(&existing)?, b"original destination");
    let mut prefix_entries = original_entries.clone();
    prefix_entries.insert(target.file_name().unwrap().to_owned());
    assert_eq!(entries()?, prefix_entries);
    drop(reader);
    std::fs::remove_file(&target)?;

    // A foreign relation in the final projection makes its checked concatenation
    // refuse after the first five projections would have succeeded.
    let fixture = *report
        .report
        .fixture_statuses
        .keys()
        .next()
        .context("fixture")?;
    report.report.admissions.entry(fixture).or_default().insert(
        pse_relations::generated::runtime::structural_assessments::RELATION_ID,
        first,
    );
    let refusal = report.report.admission_tables().unwrap_err().to_string();
    let fresh = directory.path().join("fresh.arrow");
    for destination in [&fresh, &missing_parent, &target] {
        let error = report.export(destination).unwrap_err();
        assert!(
            error
                .downcast_ref::<pse_runtime::workflow::WorkflowError>()
                .is_some()
        );
        assert_eq!(error.to_string(), refusal);
        assert!(!destination.exists());
    }
    assert_eq!(std::fs::read(&existing)?, b"original destination");
    assert_eq!(entries()?, original_entries);

    // An interleaved single-table staging route sees this genuine IPC writer
    // failure before it has projected the final relation, even with private staging.
    let first = report.report.table()?;
    let writer = FileWriter::try_new(RefusingWriter, first.batch().schema().as_ref());
    assert!(
        writer
            .err()
            .unwrap()
            .to_string()
            .contains("controlled IPC writer refusal")
    );
    Ok(())
}

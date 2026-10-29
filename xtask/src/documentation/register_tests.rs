// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Register contracts challenged with explicit calendar dates and private Git history.
use super::*;

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 29).unwrap()
}
fn row(id: &str, last: &str, next: &str, status: &str) -> String {
    format!(
        "| {id} | deferred item | ADR-0001 | observable trigger | manual | owner | {last} | {next} | {status} |\n"
    )
}
fn inspect_fixture(text: &str, baseline: Option<&Baseline>) -> LintReport {
    inspect(
        text,
        today(),
        &BTreeSet::from(["ADR-0001".into()]),
        baseline,
    )
    .unwrap()
}
fn git(root: &Path, arguments: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "Git fixture {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn empty_register_retains_high_water_and_new_rows_must_raise_it() {
    assert!(
        inspect_fixture("Highest issued row id: R-07.\n", None)
            .errors
            .is_empty()
    );
    let added = format!(
        "Highest issued row id: R-07.\n{}",
        row("R-08", "2026-09-25", "2027-01-01", "open")
    );
    assert!(
        inspect_fixture(&added, None)
            .errors
            .iter()
            .any(|error| error.contains("raise the highest issued row id to R-08"))
    );
    assert!(
        inspect_fixture("# No deferrals\n", None)
            .errors
            .iter()
            .any(|error| error.contains("missing 'Highest issued row id"))
    );
}

#[test]
fn removed_rows_are_allowed_but_retired_ids_and_lowered_marks_are_not() {
    let baseline = Baseline {
        highest: 3,
        ids: BTreeSet::from(["R-02".into(), "R-03".into()]),
    };
    let retained = format!(
        "Highest issued row id: R-03.\n{}",
        row("R-03", "2026-09-25", "2027-01-01", "open")
    );
    assert!(
        inspect_fixture(&retained, Some(&baseline))
            .errors
            .is_empty()
    );
    let reused = retained.replace("| R-03 |", "| R-01 |");
    assert!(
        inspect_fixture(&reused, Some(&baseline))
            .errors
            .iter()
            .any(|error| error.contains("reuses a retired row id"))
    );
    let lowered = retained.replace("Highest issued row id: R-03", "Highest issued row id: R-02");
    assert!(
        inspect_fixture(&lowered, Some(&baseline))
            .errors
            .iter()
            .any(|error| error.contains("may not fall below R-03"))
    );
}

#[test]
fn origin_main_history_and_retained_adr_filenames_use_the_selected_checkout() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path();
    fs::create_dir_all(root.join("docs/adr"))?;
    fs::write(root.join("docs/adr/0001-example.md"), "retained decision")?;
    fs::write(
        root.join("docs/adr/0002-malformed.txt"),
        "not an ADR filename",
    )?;
    fs::write(
        root.join(REGISTER),
        format!(
            "Highest issued row id: R-03.\n{}",
            row("R-03", "2026-09-25", "2027-01-01", "open")
        ),
    )?;
    git(root, &["init", "-q", "-b", "main"])?;
    assert!(baseline(root)?.is_none());
    git(root, &["add", "."])?;
    git(
        root,
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    )?;
    git(root, &["update-ref", "refs/remotes/origin/main", "HEAD"])?;
    let base = baseline(root)?.context("baseline")?;
    assert_eq!(base.highest, 3);
    assert_eq!(base.ids, BTreeSet::from(["R-03".into()]));
    assert_eq!(known_adrs(root)?, BTreeSet::from(["ADR-0001".into()]));
    Ok(())
}

#[test]
fn malformed_duplicate_unknown_and_empty_required_cells_accumulate() {
    let good = row("R-01", "2026-09-25", "2027-01-01", "open");
    let malformed = "| R-02 | too | few |\n";
    let invalid = good
        .replace("| ADR-0001 |", "| ADR-0099, ADR-0001 |")
        .replace("| observable trigger |", "| |")
        .replace("| owner |", "| |")
        .replace("| open |", "| invalid |");
    let text = format!("Highest issued row id: R-03.\n{good}{invalid}{malformed}");
    let report = inspect_fixture(&text, None);
    assert_eq!(report.rows, 3);
    for expected in [
        "duplicate row id",
        "3 cells, expected 9",
        "references unknown ADR-0099",
        "owner is empty",
        "trigger is empty",
        "status",
    ] {
        assert!(
            report.errors.iter().any(|error| error.contains(expected)),
            "{expected}: {:?}",
            report.errors
        );
    }
}

#[test]
fn dates_keep_today_boundary_closed_exception_ordering_and_real_calendar_rules() {
    let text = format!(
        "Highest issued row id: R-04.\n{}{}{}{}",
        row("R-01", "2026-09-25", "2026-10-29", "open"),
        row("R-02", "2026-09-25", "2026-10-28", "closed"),
        row("R-03", "2026-10-30", "2026-10-29", "watch"),
        row("R-04", "2026-02-30", "2027-02-30", "open")
    );
    let errors = inspect_fixture(&text, None).errors;
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("R-03: next-check precedes last-checked"))
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.contains("must be valid YYYY-MM-DD"))
            .count(),
        2
    );
    assert!(date("2024-02-29").is_some());
    for invalid in [
        "2026-02-29",
        "2026-1-01",
        "0000-01-01",
        "2026-13-01",
        "2026-01-00",
    ] {
        assert!(date(invalid).is_none(), "{invalid}");
    }
    let overdue = format!(
        "Highest issued row id: R-01.\n{}",
        row("R-01", "2026-09-25", "2026-10-28", "open")
    );
    assert!(
        inspect_fixture(&overdue, None)
            .errors
            .iter()
            .any(|error| error.contains("in the past and the row is not closed"))
    );
}

#[test]
fn due_report_selects_today_and_past_runs_only_shell_cells_and_keeps_failures_nonfatal()
-> Result<()> {
    let current = row("R-01", "2026-09-25", "2026-10-29", "open")
        .replace("| manual |", "| $ `printf check; exit 7` |");
    let previous = row("R-02", "2026-09-25", "2026-10-28", "watch");
    let closed = row("R-03", "2026-09-25", "2026-10-28", "closed");
    let future = row("R-04", "2026-09-25", "2026-10-30", "open");
    let mut commands = Vec::new();
    let report = render_due(
        &format!("{current}{previous}{closed}{future}| R-05 | malformed |\n"),
        today(),
        |command| {
            commands.push(command.to_owned());
            "    check failed; reported, not fatal".into()
        },
    )?;
    assert_eq!(commands, ["printf check; exit 7"]);
    assert!(report.starts_with("# Register review 2026-10\n\n2 row(s) due on 2026-10-29:"));
    assert!(report.contains("## R-01 — deferred item"));
    assert!(report.contains("## R-02 — deferred item"));
    assert!(!report.contains("## R-03"));
    assert!(report.contains("- Check (manual): manual"));
    assert!(report.contains("reported, not fatal"));
    let mut empty_called = false;
    let empty = render_due("", today(), |_| {
        empty_called = true;
        String::new()
    })?;
    assert!(
        !empty_called,
        "an empty register must never invoke a command"
    );
    assert_eq!(
        empty,
        "# Register review 2026-10\n\nNo register rows are due on 2026-10-29.\n"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn due_shell_output_limits_lines_and_reports_original_exit_and_timeout() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let report = check_report(
        fixture.path(),
        "printf 'stdout\\n'; printf 'stderr\\n' >&2; exit 7",
        Duration::from_secs(2),
    );
    assert_eq!(
        report,
        "    stdout\n    stderr\n    (command exited 7; reported, not fatal)"
    );
    let report = check_report(
        fixture.path(),
        "i=0; while [ $i -lt 25 ]; do echo line-$i; i=$((i+1)); done",
        Duration::from_secs(2),
    );
    assert_eq!(report.lines().count(), 20);
    assert!(!report.contains("line-20"));
    assert_eq!(
        check_report(fixture.path(), "true", Duration::from_secs(2)),
        "    (no output)"
    );
    let report = check_report(
        fixture.path(),
        "while :; do :; done",
        Duration::from_millis(20),
    );
    assert!(report.contains("check could not be run: command timed out"));
    Ok(())
}

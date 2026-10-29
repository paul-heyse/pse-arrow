// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Deferred-decision row allocation, retained references and explicitly requested due checks.

use anyhow::{Context, Result};
use chrono::NaiveDate;
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const REGISTER: &str = "docs/adr/register.md";
const COLUMNS: [&str; 9] = [
    "id",
    "item",
    "adr",
    "trigger",
    "check",
    "owner",
    "last-checked",
    "next-check",
    "status",
];

/// Direct register operations; the outer driver selects the checkout.
#[derive(Clone, Debug, clap::Subcommand)]
pub enum RegisterCommand {
    /// Validate all retained rows, dates, ADR references and never-reused row IDs.
    Lint,
    /// Report due rows and run only their explicitly authored shell checks.
    Due,
}

/// Accumulated register diagnostics against the zero-error target.
#[derive(Debug, Default)]
pub struct LintReport {
    /// Number of row-shaped entries, including malformed rows.
    pub rows: usize,
    /// All observed contract violations.
    pub errors: Vec<String>,
}

#[derive(Debug)]
struct Row {
    line: usize,
    id: String,
    cells: BTreeMap<&'static str, String>,
    error: Option<String>,
}
#[derive(Debug)]
struct Baseline {
    highest: u32,
    ids: BTreeSet<String>,
}

fn rows(text: &str) -> Result<Vec<Row>> {
    let pattern = Regex::new(r"^\|\s*(R-[0-9]{2})\s*\|")?;
    let mut parsed = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let Some(capture) = pattern.captures(line) else {
            continue;
        };
        let cells: Vec<_> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let error = (cells.len() != COLUMNS.len())
            .then(|| format!("{} cells, expected {}", cells.len(), COLUMNS.len()));
        let values = if error.is_none() {
            COLUMNS
                .into_iter()
                .zip(cells.into_iter().map(str::to_owned))
                .collect()
        } else {
            BTreeMap::new()
        };
        parsed.push(Row {
            line: index + 1,
            id: capture[1].into(),
            cells: values,
            error,
        });
    }
    Ok(parsed)
}

fn highest(text: &str) -> Result<Option<u32>> {
    let pattern = Regex::new(r"(?m)^Highest issued row id: R-([0-9]{2})\.?$")?;
    pattern
        .captures(text)
        .map(|capture| Ok(capture[1].parse()?))
        .transpose()
}

fn baseline(root: &Path) -> Result<Option<Baseline>> {
    let output = Command::new("git")
        .args(["show", &format!("origin/main:{REGISTER}")])
        .current_dir(root)
        .output()
        .context("observing register history on origin/main")?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = std::str::from_utf8(&output.stdout).context("register history is not UTF-8")?;
    Ok(Some(Baseline {
        highest: highest(text)?.unwrap_or(0),
        ids: rows(text)?.into_iter().map(|row| row.id).collect(),
    }))
}

fn known_adrs(root: &Path) -> Result<BTreeSet<String>> {
    let directory = root.join("docs/adr");
    if !directory.exists() {
        return Ok(BTreeSet::new());
    }
    let pattern = Regex::new(r"^([0-9]{4})-.*\.md$")?;
    let mut known = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let name = entry?.file_name();
        if let Some(name) = name.to_str()
            && let Some(capture) = pattern.captures(name)
        {
            known.insert(format!("ADR-{}", &capture[1]));
        }
    }
    Ok(known)
}

fn date(text: &str) -> Option<NaiveDate> {
    let bytes = text.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return None;
    }
    let value = NaiveDate::parse_from_str(text, "%Y-%m-%d").ok()?;
    // Python date.fromisoformat admits years 0001..9999.
    (&text[..4] != "0000").then_some(value)
}

fn inspect(
    text: &str,
    today: NaiveDate,
    adrs: &BTreeSet<String>,
    base: Option<&Baseline>,
) -> Result<LintReport> {
    let parsed = rows(text)?;
    let mut report = LintReport {
        rows: parsed.len(),
        ..Default::default()
    };
    let mark = highest(text)?;
    if mark.is_none() {
        report
            .errors
            .push("register.md: missing 'Highest issued row id: R-NN' line".into());
    }
    if let (Some(base), Some(mark)) = (base, mark)
        && mark < base.highest
    {
        report.errors.push(format!(
            "register.md: the highest issued row id may not fall below R-{:02}",
            base.highest
        ));
    }
    let references = Regex::new(r"ADR-([0-9]{4})")?;
    let mut seen = BTreeSet::new();
    for row in parsed {
        let place = format!("register.md:{} {}", row.line, row.id);
        if let Some(error) = row.error {
            report.errors.push(format!("{place}: {error}"));
            continue;
        }
        if !seen.insert(row.id.clone()) {
            report.errors.push(format!("{place}: duplicate row id"));
        }
        let number: u32 = row.id[2..].parse()?;
        if mark.is_some_and(|highest| number > highest) {
            report.errors.push(format!(
                "{place}: raise the highest issued row id to {}",
                row.id
            ));
        }
        if let Some(base) = base
            && number <= base.highest
            && !base.ids.contains(&row.id)
        {
            report.errors.push(format!(
                "{place}: reuses a retired row id; take the next id instead"
            ));
        }
        let field = |name| row.cells.get(name).map_or("", String::as_str);
        let status = field("status");
        if !["open", "watch", "closed"].contains(&status) {
            report.errors.push(format!(
                "{place}: status {status:?} not in ['closed', 'open', 'watch']"
            ));
        }
        let last = date(field("last-checked"));
        let next = date(field("next-check"));
        for (key, value) in [("last-checked", last), ("next-check", next)] {
            if value.is_none() {
                report.errors.push(format!(
                    "{place}: {key} {:?} must be valid YYYY-MM-DD",
                    field(key)
                ));
            }
        }
        for key in ["owner", "trigger"] {
            if field(key).is_empty() {
                report.errors.push(format!("{place}: {key} is empty"));
            }
        }
        for reference in references.captures_iter(field("adr")) {
            let id = format!("ADR-{}", &reference[1]);
            if !adrs.contains(&id) {
                report
                    .errors
                    .push(format!("{place}: references unknown {id}"));
            }
        }
        if status != "closed" && next.is_some_and(|next| next < today) {
            report.errors.push(format!(
                "{place}: next-check {} is in the past and the row is not closed",
                field("next-check")
            ));
        }
        if let (Some(last), Some(next)) = (last, next)
            && next < last
        {
            report
                .errors
                .push(format!("{place}: next-check precedes last-checked"));
        }
    }
    Ok(report)
}

/// Validate authored register state against the available origin/main row history.
/// # Errors
/// Register, ADR-directory or history observation could not be read.
pub fn lint(root: &Path) -> Result<LintReport> {
    let text = fs::read_to_string(root.join(REGISTER))?;
    let base = baseline(root)?;
    inspect(
        &text,
        chrono::Local::now().date_naive(),
        &known_adrs(root)?,
        base.as_ref(),
    )
}

fn command_output(root: &Path, shell: &str, allowance: Duration) -> Result<(Option<i32>, String)> {
    // Files avoid pipe backpressure and inherited-pipe waits; the authored command retains
    // stdout-then-stderr report ordering. This helper owns only its original shell child.
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = Command::new("/bin/sh")
        .args(["-c", shell])
        .current_dir(root)
        .stdout(Stdio::from(stdout.try_clone()?))
        .stderr(Stdio::from(stderr.try_clone()?))
        .spawn()?;
    let end = Instant::now() + allowance;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= end {
            let killed = child.kill();
            let waited = child.wait();
            if let Err(error) = killed {
                return Err(error).context("due check timeout; signalling original shell failed");
            }
            waited.context("due check timeout; waiting for original shell failed")?;
            anyhow::bail!(
                "command timed out after {} seconds",
                allowance.as_secs_f64()
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    use std::io::{Seek, SeekFrom};
    stdout.seek(SeekFrom::Start(0))?;
    stderr.seek(SeekFrom::Start(0))?;
    let mut out = String::new();
    stdout.read_to_string(&mut out)?;
    stderr.read_to_string(&mut out)?;
    #[cfg(unix)]
    let code = {
        use std::os::unix::process::ExitStatusExt;
        status
            .code()
            .or_else(|| status.signal().map(|signal| -signal))
    };
    #[cfg(not(unix))]
    let code = status.code();
    Ok((code, out))
}

fn check_report(root: &Path, shell: &str, allowance: Duration) -> String {
    let (code, output) = match command_output(root, shell, allowance) {
        Ok(result) => result,
        Err(error) => return format!("    check could not be run: {error}"),
    };
    let output = output.trim();
    let output = if output.is_empty() {
        "(no output)"
    } else {
        output
    };
    let mut lines: Vec<_> = output
        .lines()
        .take(20)
        .map(|line| format!("    {line}"))
        .collect();
    if code != Some(0) {
        lines.push(format!(
            "    (command exited {}; reported, not fatal)",
            code.map_or_else(|| "unknown".into(), |code| code.to_string())
        ));
    }
    lines.join("\n")
}

fn render_due<F>(text: &str, today: NaiveDate, mut check: F) -> Result<String>
where
    F: FnMut(&str) -> String,
{
    let shape = Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$")?;
    let mut pending = Vec::new();
    for row in rows(text)? {
        if row.error.is_some() {
            continue;
        }
        let value = &row.cells["next-check"];
        if !shape.is_match(value) {
            continue;
        }
        let next = date(value).with_context(|| {
            format!(
                "register.md:{} {}: invalid next-check date {value:?}",
                row.line, row.id
            )
        })?;
        if next <= today && row.cells["status"] != "closed" {
            pending.push(row);
        }
    }
    let mut output = format!("# Register review {}\n\n", today.format("%Y-%m"));
    if pending.is_empty() {
        output.push_str(&format!("No register rows are due on {today}.\n"));
        return Ok(output);
    }
    output.push_str(&format!("{} row(s) due on {today}:\n\n", pending.len()));
    for row in pending {
        let adr = if row.cells["adr"].is_empty() {
            "—"
        } else {
            &row.cells["adr"]
        };
        output.push_str(&format!("## {} — {}\n- Decision record: {adr}\n- Trigger: {}\n- Owner: {}  ·  last checked {}  ·  due {}\n", row.id, row.cells["item"], row.cells["trigger"], row.cells["owner"], row.cells["last-checked"], row.cells["next-check"]));
        let command = &row.cells["check"];
        if let Some(shell) = command.strip_prefix("$ ") {
            let shell = shell.trim().trim_matches('`');
            output.push_str(&format!("- Check: `{shell}`\n{}\n", check(shell)));
        } else {
            output.push_str(&format!("- Check (manual): {command}\n"));
        }
        output.push('\n');
    }
    Ok(output)
}

/// Execute one direct register operation. Due check failures are reported, not fatal.
/// # Errors
/// Authored input, date interpretation or terminal output failed.
pub fn execute(root: &Path, command: RegisterCommand) -> Result<i32> {
    match command {
        RegisterCommand::Lint => {
            let report = lint(root)?;
            if report.errors.is_empty() {
                writeln!(
                    std::io::stdout().lock(),
                    "register lint: {} row(s) OK",
                    report.rows
                )?;
                return Ok(0);
            }
            let mut output = std::io::stderr().lock();
            for error in &report.errors {
                writeln!(output, "error: {error}")?;
            }
            writeln!(
                output,
                "register lint: {} problem(s) in {} row(s)",
                report.errors.len(),
                report.rows
            )?;
            Ok(1)
        }
        RegisterCommand::Due => {
            let text = fs::read_to_string(root.join(REGISTER))?;
            let rendered = render_due(&text, chrono::Local::now().date_naive(), |shell| {
                check_report(root, shell, Duration::from_secs(300))
            })?;
            std::io::stdout().lock().write_all(rendered.as_bytes())?;
            Ok(0)
        }
    }
}

#[cfg(test)]
#[path = "register_tests.rs"]
mod tests;

// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Parser contracts pinned with `try_parse_from`, plus the tree's own `debug_assert`.
use super::{
    BinKind, Cli, Command, Format, Reporting,
    agent_hook::HookAction,
    command,
    correctness::{self, Request},
    verb_word,
};
use crate::{environment::CacheMode, native};
use clap::{CommandFactory as _, FromArgMatches as _, error::ErrorKind};
use std::ffi::OsString;

fn words(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
fn parse(bin: BinKind, arguments: &[&str]) -> Result<Cli, clap::Error> {
    let matches = command(bin).try_get_matches_from(words(arguments))?;
    Cli::from_arg_matches(&matches)
}
fn boundary(arguments: &[&str]) -> Result<super::boundary::BoundaryArgs, clap::Error> {
    let mut argv = vec!["pse-host", "boundary"];
    argv.extend_from_slice(arguments);
    match parse(BinKind::PseHost, &argv)?.command {
        Command::Boundary(args) => Ok(args),
        other => panic!("boundary parsed as {other:?}"),
    }
}
fn kind(result: Result<impl std::fmt::Debug, clap::Error>) -> ErrorKind {
    result.expect_err("parse must fail").kind()
}

#[test]
fn every_tree_passes_clap_debug_assert() {
    for bin in [BinKind::Pse, BinKind::PseHost] {
        command(bin).debug_assert();
    }
    Cli::command().debug_assert();
    correctness::CorrectnessCli::command().debug_assert();
}

#[test]
fn boundary_first_word_starts_the_command_and_the_rest_is_data() {
    let args = boundary(&["--", "tool", "--native", "--store"]).unwrap();
    assert_eq!(args.command, words(&["tool", "--native", "--store"]));
    assert!(args.native.is_none() && !args.store);
    let args = boundary(&["--docs", "tool", "-h", "--", "--print"]).unwrap();
    assert!(args.docs);
    assert_eq!(args.command, words(&["tool", "-h", "--", "--print"]));
    // `-h` is help only before the command.
    assert_eq!(kind(boundary(&["--docs", "-h"])), ErrorKind::DisplayHelp);
    // An unknown option before the command is refused, never run as the command.
    assert_eq!(
        kind(boundary(&["--unknown", "tool"])),
        ErrorKind::UnknownArgument
    );
    assert_eq!(
        kind(boundary(&["--docs"])),
        ErrorKind::MissingRequiredArgument
    );
    assert_eq!(
        kind(boundary(&[])),
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    );
}

#[test]
fn boundary_native_is_last_wins_with_a_meaningful_empty_value() {
    let native = |arguments: &[&str]| boundary(arguments).unwrap().native;
    assert_eq!(
        native(&["--native", "--", "t"]),
        Some(native::Request::Default)
    );
    assert_eq!(native(&["--native", "t"]), Some(native::Request::Default));
    assert_eq!(
        native(&["--native=", "--", "t"]),
        Some(native::Request::Explicit(String::new()))
    );
    assert_eq!(
        native(&["--native", "--native=compiler,klu", "t"]),
        Some(native::Request::Explicit("compiler,klu".into()))
    );
    assert_eq!(
        native(&["--native=compiler,klu", "--native", "t"]),
        Some(native::Request::Default)
    );
    // `--native` never swallows the following word as its value.
    assert_eq!(
        boundary(&["--native", "compiler"]).unwrap().command,
        words(&["compiler"])
    );
    assert_eq!(
        kind(boundary(&["--native=unknown", "--", "t"])),
        ErrorKind::ValueValidation
    );
    let args = boundary(&["--cache", "off", "--cache", "on", "t"]).unwrap();
    assert_eq!(args.cache, CacheMode::On);
}

#[test]
fn boundary_refusals_and_exclusions_hold_before_effects() {
    for mode in ["--store", "--bind-allocation"] {
        let args = boundary(&[mode, "--", "/bin/true"]).unwrap();
        assert!(
            args.store || args.bind_allocation,
            "{mode} is refused by its handler"
        );
    }
    assert_eq!(
        kind(boundary(&["--resource-class", "unknown", "--", "t"])),
        ErrorKind::ValueValidation
    );
    assert_eq!(
        kind(boundary(&["--recipe-default-class", "unknown", "--", "t"])),
        ErrorKind::ValueValidation
    );
    assert_eq!(
        kind(boundary(&["--docs", "--native", "--", "t"])),
        ErrorKind::ArgumentConflict
    );
    assert_eq!(
        kind(boundary(&["--print", "--explain"])),
        ErrorKind::ArgumentConflict
    );
    assert_eq!(
        kind(boundary(&["--explain", "--no-scope"])),
        ErrorKind::ArgumentConflict
    );
    assert_eq!(
        kind(boundary(&["--print", "--native"])),
        ErrorKind::ArgumentConflict
    );
    assert_eq!(
        kind(boundary(&["--json", "--", "t"])),
        ErrorKind::ArgumentConflict
    );
    assert!(boundary(&["--json"]).is_err());
    let explain = boundary(&["--explain", "--json", "--native=compiler"]).unwrap();
    assert!(explain.explain && explain.json && explain.command.is_empty());
}

#[test]
fn run_passes_everything_after_the_separator_through() {
    let parsed = parse(
        BinKind::Pse,
        &["pse", "run", "--", "/bin/sh", "-c", "x", "--", "-h"],
    );
    let Command::Run(args) = parsed.unwrap().command else {
        panic!("expected run")
    };
    assert_eq!(args.command, words(&["/bin/sh", "-c", "x", "--", "-h"]));
    assert!(parse(BinKind::Pse, &["pse", "run", "/bin/true"]).is_err());
    assert_eq!(
        kind(parse(BinKind::Pse, &["pse", "run", "--"])),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn agent_hook_actions_and_hidden_verbs() {
    for (word, action) in [
        ("guard", HookAction::Guard),
        ("session-env", HookAction::SessionEnv),
    ] {
        let Command::AgentHook(args) = parse(BinKind::PseHost, &["pse-host", "agent-hook", word])
            .unwrap()
            .command
        else {
            panic!("expected agent-hook")
        };
        assert_eq!(args.action, action);
    }
    assert!(
        parse(
            BinKind::PseHost,
            &["pse-host", "agent-hook", "guard", "extra"]
        )
        .is_err()
    );
    assert!(parse(BinKind::PseHost, &["pse-host", "agent-hook", "other"]).is_err());
    let pse = command(BinKind::Pse);
    let host = command(BinKind::PseHost);
    for verb in super::HIDDEN_FROM_PSE {
        assert!(
            pse.find_subcommand(verb)
                .is_some_and(clap::Command::is_hide_set)
        );
        assert!(host.find_subcommand(verb).is_some_and(|c| !c.is_hide_set()));
        assert!(
            parse(BinKind::Pse, &["pse", verb, "-h"])
                .is_err_and(|e| e.kind() == ErrorKind::DisplayHelp)
        );
    }
}

#[test]
fn hook_and_boundary_failures_keep_their_reporting() {
    let tree = command(BinKind::PseHost);
    let class = |arguments: &[&str]| match verb_word(&tree, &words(arguments)).as_deref() {
        Some("boundary") => Reporting::Boundary,
        Some("agent-hook") => Reporting::Hook,
        _ => Reporting::Host,
    };
    assert_eq!(class(&["pse-host", "agent-hook", "bogus"]), Reporting::Hook);
    assert_eq!(
        class(&["pse-host", "-C", "/x", "agent-hook"]),
        Reporting::Hook
    );
    assert_eq!(
        class(&["pse-host", "--format=json", "boundary"]),
        Reporting::Boundary
    );
    assert_eq!(
        class(&["pse-host", "-vv", "boundary", "--x"]),
        Reporting::Boundary
    );
    assert_eq!(class(&["pse-host", "doctor"]), Reporting::Host);
    assert_eq!(class(&["pse-host", "--", "boundary"]), Reporting::Host);
}

#[test]
fn global_options_parse_in_any_verb_position() {
    let cli = parse(
        BinKind::Pse,
        &[
            "pse",
            "doctor",
            "-C",
            "/checkout",
            "--format",
            "json",
            "--json",
        ],
    )
    .unwrap();
    assert_eq!(
        cli.globals.checkout.as_deref(),
        Some(std::path::Path::new("/checkout"))
    );
    assert_eq!(cli.globals.format, Format::Json);
    assert!(matches!(cli.command, Command::Doctor(ref args) if args.json));
    assert_eq!(
        kind(parse(BinKind::Pse, &["pse", "-v", "-q", "doctor"])),
        ErrorKind::ArgumentConflict
    );
    assert_eq!(
        kind(parse(BinKind::Pse, &["pse", "--budget", "0", "doctor"])),
        ErrorKind::ValueValidation
    );
    // No prefix inference: an abbreviated verb is unknown.
    assert_eq!(
        kind(parse(BinKind::Pse, &["pse", "doc"])),
        ErrorKind::InvalidSubcommand
    );
    assert_eq!(
        kind(parse(BinKind::Pse, &["pse", "--version"])),
        ErrorKind::DisplayVersion
    );
}

#[test]
fn verb_grammars_pin_their_former_combinations() {
    let ok = |arguments: &[&str]| parse(BinKind::PseHost, arguments).map(|cli| cli.command);
    assert!(ok(&["pse-host", "env", "--print"]).is_ok());
    assert!(
        ok(&[
            "pse-host",
            "env",
            "--explain",
            "--json",
            "--docs",
            "--cache",
            "off"
        ])
        .is_ok()
    );
    assert!(ok(&["pse-host", "env"]).is_err());
    assert!(ok(&["pse-host", "env", "--print", "--explain"]).is_err());
    assert!(ok(&["pse-host", "env", "--print", "--json"]).is_err());
    assert!(ok(&["pse-host", "env", "--activation-data", "--cache", "auto"]).is_ok());
    assert!(
        ok(&[
            "pse-host",
            "study-module",
            "select",
            "--receipt",
            "/r",
            "--qualification",
            "--long-hold"
        ])
        .is_ok()
    );
    assert!(ok(&["pse-host", "study-module", "prepare", "--long-hold"]).is_err());
    assert!(ok(&["pse-host", "study-module", "select"]).is_err());
    assert!(
        ok(&[
            "pse-host",
            "study-module",
            "select",
            "--receipt",
            "/r",
            "--tool-root",
            "/t"
        ])
        .is_err()
    );
    assert!(ok(&["pse-host", "telemetry-service", "install", "--state", "/s"]).is_err());
    assert!(
        ok(&[
            "pse-host",
            "telemetry-service",
            "inspect",
            "--port",
            "health=1"
        ])
        .is_err()
    );
    assert!(
        ok(&[
            "pse-host",
            "telemetry-service",
            "setup",
            "--port",
            "health=1",
            "--port",
            "metrics=2",
            "--stream",
            "logs=l",
            "--organization",
            "a",
            "--organization",
            "b"
        ])
        .is_ok()
    );
    assert!(
        ok(&[
            "pse-host",
            "telemetry-service",
            "setup",
            "--port",
            "health=70000"
        ])
        .is_err()
    );
    assert!(
        ok(&[
            "pse-host",
            "telemetry-service",
            "inspect",
            "--state",
            "/a",
            "--state",
            "/b"
        ])
        .is_err()
    );
    assert!(ok(&["pse-host", "diagnostic-tools", "verify", "collector"]).is_ok());
    assert!(ok(&["pse-host", "diagnostic-tools", "install", "cargo-tools"]).is_err());
    assert!(ok(&["pse-host", "diagnostic-tools", "install", "rr", "extra"]).is_err());
    assert!(
        ok(&[
            "pse-host",
            "native",
            "prepare",
            "--capabilities",
            "compiler,klu"
        ])
        .is_ok()
    );
    assert!(
        ok(&[
            "pse-host",
            "native",
            "prepare",
            "--capabilities",
            "compiler,ipopt"
        ])
        .is_err()
    );
    assert!(ok(&["pse-host", "native", "prepare"]).is_err());
    assert!(ok(&["pse-host", "server", "prepare", "--from-release", "/r"]).is_ok());
    assert!(ok(&["pse-host", "server", "select", "--from-release", "/r"]).is_err());
    assert!(ok(&["pse-host", "activity", "a.service", "b.service"]).is_err());
    assert!(ok(&["pse-host", "doctor", "--yaml"]).is_err());
}

#[test]
fn correctness_argv_shapes_forward_the_original_bytes() {
    let request = |arguments: &[&str]| correctness::parse(&words(arguments));
    assert_eq!(
        request(&["test", "--no-run", "--", "tail"]).unwrap(),
        Request::Cargo(words(&["test", "--no-run", "--", "tail"]))
    );
    // clap consumes a leading `--`; Cargo still receives it.
    assert_eq!(
        request(&["run", "--", "--flag"]).unwrap(),
        Request::Cargo(words(&["run", "--", "--flag"]))
    );
    assert_eq!(
        request(&["check", "--help"]).unwrap(),
        Request::Cargo(words(&["check", "--help"]))
    );
    for mode in ["check", "build", "test", "bench", "run", "clippy"] {
        assert_eq!(request(&[mode]).unwrap(), Request::Cargo(words(&[mode])));
    }
    assert_eq!(
        request(&["nextest", "list", "--lib"]).unwrap(),
        Request::Nextest(words(&["nextest", "list", "--lib"]))
    );
    assert_eq!(
        request(&["--nextest-test-env"]).unwrap(),
        Request::TestEnvironment
    );
    assert!(request(&["--nextest-test-env", "check"]).is_err());
    assert!(request(&["nextest", "archive"]).is_err());
    assert!(request(&["miri", "test"]).is_err());
    assert_eq!(kind(request(&["--version"])), ErrorKind::DisplayVersion);
    assert_eq!(kind(request(&["-h"])), ErrorKind::DisplayHelp);
    assert_eq!(
        kind(request(&[])),
        ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    );
}

#[test]
fn version_lines_name_each_binary_and_the_protocol() {
    for (tree, name) in [
        (command(BinKind::Pse), "pse"),
        (command(BinKind::PseHost), "pse-host"),
        (
            correctness::CorrectnessCli::command(),
            "cargo-pse-correctness",
        ),
    ] {
        let rendered = tree.render_version();
        assert_eq!(
            rendered,
            format!("{name} {} protocol 1\n", env!("CARGO_PKG_VERSION"))
        );
    }
}

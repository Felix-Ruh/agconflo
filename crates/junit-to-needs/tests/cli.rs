//! The command line: what reaches stdout, and the exit status for each outcome.

use std::process::{Command, Output};

const REAL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/agconflo-core.xml");
const USAGE: &str = "usage: junit-to-needs";

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_junit-to-needs"))
        .args(args)
        .output()
        .expect("the binary runs")
}

#[test]
fn prints_the_needs_file() {
    let out = run(&["--crate", "agconflo-core", REAL]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let xml = std::fs::read_to_string(REAL).expect("the report is readable");
    let expected = junit_to_needs::import(&xml, &["agconflo-core"])
        .expect("imports")
        .to_json();
    assert_eq!(String::from_utf8(out.stdout).expect("UTF-8"), expected);
    assert!(out.stderr.is_empty());
}

#[test]
fn misuse_exits_2() {
    let cases: [&[&str]; 7] = [
        &[],
        &[REAL],
        &["--crate"],
        &["--crate", "agconflo-core"],
        &["--crate", "--crate", REAL],
        &["--crate", "agconflo-core", "--bogus", REAL],
        &["--crate", "agconflo-core", REAL, REAL],
    ];
    for args in cases {
        let out = run(args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(out.stdout.is_empty(), "{args:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(USAGE),
            "{args:?}"
        );
    }
}

#[test]
fn missing_report_exits_1() {
    let out = run(&["--crate", "agconflo-core", "no-such-report.xml"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot read no-such-report.xml"));
}

#[test]
fn import_error_exits_1() {
    let out = run(&["--crate", "agconflo-core", "--crate", "no-such-crate", REAL]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("`no-such-crate`"));
}

#[test]
fn help_exits_0() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains(USAGE));
    assert!(out.stderr.is_empty());
}

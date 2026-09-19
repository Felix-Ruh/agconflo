//! Reading a nextest JUnit report into runs.

use std::collections::BTreeSet;

use quick_junit::{DeserializeError, Report, TestCaseStatus};

use crate::id;
use crate::needs::{Outcome, Run, Runs};

/// Why a report was not imported.
///
/// Nothing is imported when any of these occurs: a file holding some of a
/// report's runs would read exactly like one holding all of them.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ImportError {
    #[error("no crate was named, so there is nothing to import")]
    NoCrates,

    #[error("the report could not be read as JUnit: {0}")]
    Unreadable(#[from] DeserializeError),

    #[error("the test `{name}` in suite `{suite}` has no classname, so its crate is unknown")]
    Unattributed { suite: String, name: String },

    #[error(
        "`{test}` makes no test case id: each part of its path must be ASCII letters, digits and underscores"
    )]
    Underivable { test: String },

    #[error("`{first}` and `{second}` would both have run {test_case}")]
    Duplicate {
        test_case: String,
        first: String,
        second: String,
    },

    #[error("`{test}` was skipped, and a skipped test has no outcome to record")]
    Skipped { test: String },

    #[error(
        "`{test}` was retried, and .config/nextest.toml configures no retries, so which run counts is undecided"
    )]
    Rerun { test: String },

    #[error("no test in the report belongs to the crate `{krate}`")]
    NoTests { krate: String },
}

/// The runs of the tests of `crates` in `xml`, a nextest JUnit report.
///
/// Every other crate's tests are left out. nextest writes one report for the
/// whole workspace, and a crate whose tests trace to no test case - this one,
/// for instance - would otherwise contribute runs of cases that do not exist.
///
/// Each named crate must have a test in the report. A misspelt name would
/// otherwise match nothing and import an empty file that looks like a clean one.
pub fn import(xml: &str, crates: &[&str]) -> Result<Runs, ImportError> {
    let crates: BTreeSet<&str> = crates.iter().copied().collect();
    if crates.is_empty() {
        return Err(ImportError::NoCrates);
    }
    let report = Report::deserialize_from_str(xml)?;

    let mut runs = Runs::default();
    let mut found = BTreeSet::new();
    for suite in &report.test_suites {
        for case in &suite.test_cases {
            let name = case.name.as_str();
            let Some(classname) = case.classname.as_ref().map(|c| c.as_str()) else {
                return Err(ImportError::Unattributed {
                    suite: suite.name.as_str().to_owned(),
                    name: name.to_owned(),
                });
            };
            let Some(&krate) = crates
                .iter()
                .find(|&&krate| id::belongs_to(classname, krate))
            else {
                continue;
            };
            found.insert(krate);

            let test = format!("{classname} {name}");
            let Some(test_case) = id::test_case_id(krate, classname, name) else {
                return Err(ImportError::Underivable { test });
            };
            let outcome = outcome(&case.status, &test)?;
            if let Some(earlier) = runs.insert(Run::new(test_case.clone(), test.clone(), outcome)) {
                // In name order, so the message does not depend on which of the
                // two finished first.
                let mut pair = [earlier.title().to_owned(), test];
                pair.sort();
                let [first, second] = pair;
                return Err(ImportError::Duplicate {
                    test_case,
                    first,
                    second,
                });
            }
        }
    }

    if let Some(krate) = crates.iter().find(|krate| !found.contains(*krate)) {
        return Err(ImportError::NoTests {
            krate: (*krate).to_owned(),
        });
    }
    Ok(runs)
}

/// A failure and an error both count as failed: the graph asks whether a test
/// passes, and nextest already reports a timeout as a failure
/// (EVD_NEXTEST_TIMEOUT).
fn outcome(status: &TestCaseStatus, test: &str) -> Result<Outcome, ImportError> {
    match status {
        TestCaseStatus::Success { flaky_runs } if flaky_runs.is_empty() => Ok(Outcome::Passed),
        TestCaseStatus::NonSuccess { reruns, .. } if reruns.runs.is_empty() => Ok(Outcome::Failed),
        TestCaseStatus::Skipped { .. } => Err(ImportError::Skipped {
            test: test.to_owned(),
        }),
        TestCaseStatus::Success { .. } | TestCaseStatus::NonSuccess { .. } => {
            Err(ImportError::Rerun {
                test: test.to_owned(),
            })
        }
    }
}

#[cfg(test)]
use std::collections::{BTreeMap, HashSet};
#[cfg(test)]
use std::time::Duration;

#[cfg(test)]
use chrono::{DateTime, FixedOffset};
#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
use quick_junit::{FlakyOrRerun, NonSuccessKind, TestCase, TestRerun, TestSuite};

/// This repository's own report, verbatim: `target/nextest/default/junit.xml`
/// as a run of the 19 tests of agconflo-core wrote it. Its ids are the ones
/// `docs/tests/context.rst` declares, which is what makes it worth keeping;
/// re-copy it from a run if the tests are ever renamed.
#[cfg(test)]
const REAL: &str = include_str!("../testdata/agconflo-core.xml");

/// The crates a generated report traces, and one whose tests it must leave out.
#[cfg(test)]
const TRACED: [&str; 2] = ["core-a", "core-b"];
#[cfg(test)]
const UNTRACED: &str = "tool";

/// A generated test: which binary it is in, its path there, and its outcome.
#[cfg(test)]
#[derive(Clone, Debug)]
struct Test {
    krate: &'static str,
    file: Option<String>,
    path: Vec<String>,
    passed: bool,
}

#[cfg(test)]
impl Test {
    fn classname(&self) -> String {
        match &self.file {
            Some(file) => format!("{}::{file}", self.krate),
            None => self.krate.to_owned(),
        }
    }

    fn name(&self) -> String {
        self.path.join("::")
    }

    /// Put together from the generated parts, rather than by splitting the
    /// names the importer reads.
    fn expected_id(&self) -> String {
        let parts = self.file.iter().chain(&self.path);
        let upper = parts.map(|part| part.to_ascii_uppercase());
        std::iter::once("TEST".to_owned())
            .chain(upper)
            .collect::<Vec<_>>()
            .join("_")
    }

    fn outcome(&self) -> Outcome {
        if self.passed {
            Outcome::Passed
        } else {
            Outcome::Failed
        }
    }
}

#[cfg(test)]
fn any_segment() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}"
}

#[cfg(test)]
fn any_test() -> impl Strategy<Value = Test> {
    (
        prop::sample::select(vec![TRACED[0], TRACED[1], UNTRACED]),
        prop::option::of(any_segment()),
        vec(any_segment(), 1..4),
        any::<bool>(),
    )
        .prop_map(|(krate, file, path, passed)| Test {
            krate,
            file,
            path,
            passed,
        })
}

/// Generated tests, led by one in each traced crate so that neither is ever
/// missing. At most one traced test per id: two paths joining to the same id
/// are a case of their own, below. The untraced crate's tests may repeat freely,
/// since none of them should be read.
#[cfg(test)]
fn any_tests() -> impl Strategy<Value = Vec<Test>> {
    vec(any_test(), 0..24).prop_map(|generated| {
        let lead = |krate: &'static str| Test {
            krate,
            file: None,
            path: vec![krate.replace('-', "_")],
            passed: true,
        };
        let mut ids = HashSet::new();
        [lead(TRACED[0]), lead(TRACED[1])]
            .into_iter()
            .chain(generated)
            .filter(|test| test.krate == UNTRACED || ids.insert(test.expected_id()))
            .collect()
    })
}

/// Everything in a report that is not an outcome.
#[cfg(test)]
#[derive(Clone, Debug)]
struct Noise {
    uuid: u128,
    seconds: i64,
    offset_hours: i32,
    millis: Vec<u32>,
    message: String,
    output: String,
    as_error: bool,
}

#[cfg(test)]
impl Noise {
    fn timestamp(&self, index: usize) -> DateTime<FixedOffset> {
        let offset = FixedOffset::east_opt(self.offset_hours * 3600).expect("within a day");
        let at = DateTime::from_timestamp(self.seconds + index as i64, 0).expect("in range");
        at.with_timezone(&offset)
    }

    fn duration(&self, index: usize) -> Duration {
        Duration::from_millis(self.millis[index % self.millis.len()].into())
    }
}

#[cfg(test)]
fn any_noise() -> impl Strategy<Value = Noise> {
    (
        any::<u128>(),
        0i64..4_000_000_000,
        -12i32..=14,
        vec(any::<u32>(), 1..8),
        "[ -~\n]{0,40}",
        "[ -~\n]{0,40}",
        any::<bool>(),
    )
        .prop_map(
            |(uuid, seconds, offset_hours, millis, message, output, as_error)| Noise {
                uuid,
                seconds,
                offset_hours,
                millis,
                message,
                output,
                as_error,
            },
        )
}

/// A report of `tests` as nextest writes one: a suite per test binary, suites
/// and tests in the order given. With `noise`, it also carries a run id,
/// timestamps, durations, failure text and output, and may write each failure
/// as an error instead.
#[cfg(test)]
fn report(tests: &[Test], noise: Option<&Noise>) -> String {
    let mut suites: Vec<TestSuite> = Vec::new();
    for (index, test) in tests.iter().enumerate() {
        let kind = match noise {
            Some(noise) if noise.as_error => NonSuccessKind::Error,
            _ => NonSuccessKind::Failure,
        };
        let mut status = if test.passed {
            TestCaseStatus::success()
        } else {
            TestCaseStatus::non_success(kind)
        };
        if let (Some(noise), false) = (noise, test.passed) {
            status
                .set_message(&noise.message)
                .set_type("test failure with exit code 101");
        }

        let classname = test.classname();
        let mut case = TestCase::new(test.name(), status);
        case.set_classname(&classname);
        if let Some(noise) = noise {
            case.set_timestamp(noise.timestamp(index))
                .set_time(noise.duration(index))
                .set_system_out(&noise.output);
        }

        match suites
            .iter_mut()
            .find(|suite| suite.name.as_str() == classname)
        {
            Some(suite) => {
                suite.add_test_case(case);
            }
            None => {
                let mut suite = TestSuite::new(&classname);
                suite.add_test_case(case);
                suites.push(suite);
            }
        }
    }

    let mut report = Report::new("nextest-run");
    if let Some(noise) = noise {
        report
            .set_uuid(uuid::Uuid::from_u128(noise.uuid))
            .set_timestamp(noise.timestamp(0))
            .set_time(noise.duration(0));
    }
    report.add_test_suites(suites);
    report.to_string().expect("a generated report serialises")
}

/// A one-suite report of hand-built test cases, for what generation never makes.
#[cfg(test)]
fn by_hand(
    cases: impl IntoIterator<Item = (&'static str, &'static str, TestCaseStatus)>,
) -> String {
    let mut suite = TestSuite::new("by-hand");
    for (classname, name, status) in cases {
        let mut case = TestCase::new(name, status);
        case.set_classname(classname);
        suite.add_test_case(case);
    }
    let mut report = Report::new("nextest-run");
    report.add_test_suite(suite);
    report.to_string().expect("a hand-built report serialises")
}

#[cfg(test)]
fn imported(xml: &str, crates: &[&str]) -> Result<Runs, TestCaseError> {
    import(xml, crates).map_err(|error| TestCaseError::fail(error.to_string()))
}

#[cfg(test)]
fn is_plain(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix).is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
    })
}

#[cfg(test)]
proptest! {
    /// Every test of a traced crate becomes exactly one run, with its outcome
    /// and its name, and nothing else becomes a run: the untraced crate's tests
    /// are in every report and must all be left out.
    #[test]
    fn every_traced_test_once(tests in any_tests()) {
        let runs = imported(&report(&tests, None), &TRACED)?;
        let got: BTreeMap<String, (String, Outcome)> = runs
            .iter()
            .map(|run| (run.test_case().to_owned(), (run.title().to_owned(), run.outcome())))
            .collect();
        let expected: BTreeMap<String, (String, Outcome)> = tests
            .iter()
            .filter(|test| test.krate != UNTRACED)
            .map(|test| {
                let title = format!("{} {}", test.classname(), test.name());
                (test.expected_id(), (title, test.outcome()))
            })
            .collect();
        prop_assert_eq!(runs.len(), expected.len());
        prop_assert_eq!(got, expected);
    }

    /// Two reports of the same outcomes give the same file byte for byte,
    /// however their run ids, timestamps, durations, failure text, output and
    /// order differ, and whether their failures were written as errors.
    #[test]
    fn only_outcomes_are_written(
        (tests, shuffled) in any_tests().prop_flat_map(|tests| (Just(tests.clone()), Just(tests).prop_shuffle())),
        noise in any_noise(),
    ) {
        let plain = imported(&report(&tests, None), &TRACED)?.to_json();
        let noisy = imported(&report(&shuffled, Some(&noise)), &TRACED)?.to_json();
        prop_assert_eq!(plain, noisy);
    }

    /// The file has the shape ubc imports, every need is a test_run joined to
    /// the test case it shares its id with, and every id has the graph's form.
    #[test]
    fn file_is_well_formed(tests in any_tests()) {
        let text = imported(&report(&tests, None), &TRACED)?.to_json();
        prop_assert!(!text.contains('\r'), "a carriage return in the file");
        prop_assert!(text.ends_with("}\n"), "the file does not end in a newline");

        let file: serde_json::Value = serde_json::from_str(&text).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(file.as_object().map(|file| file.len()), Some(2));
        prop_assert!(file["current_version"] == "");
        let needs = file["versions"][""]["needs"].as_object().ok_or_else(|| TestCaseError::fail("no needs"))?;
        prop_assert_eq!(needs.len(), tests.iter().filter(|test| test.krate != UNTRACED).count());
        for (key, need) in needs {
            prop_assert!(is_plain(key, "RUN_"), "{}", key);
            prop_assert!(need["id"] == key.as_str());
            prop_assert!(need["type"] == "test_run");
            prop_assert!(need["test_outcome"] == "passed" || need["test_outcome"] == "failed");
            prop_assert!(need["title"].is_string());
            let executes = need["executes"].as_array().ok_or_else(|| TestCaseError::fail("no executes"))?;
            prop_assert_eq!(executes.len(), 1);
            let test_case = executes[0].as_str().unwrap_or_default();
            prop_assert!(is_plain(test_case, "TEST_"), "{}", test_case);
            prop_assert_eq!(&test_case["TEST".len()..], &key["RUN".len()..]);
            prop_assert_eq!(need.as_object().map(|need| need.len()), Some(5));
        }
    }
}

#[test]
fn real_report() {
    let runs = import(REAL, &["agconflo-core"]).expect("this repository's report imports");
    let test_cases: Vec<&str> = runs.iter().map(Run::test_case).collect();
    assert_eq!(
        test_cases,
        [
            "TEST_CONTEXT_DEEP_NESTING_DROPS",
            "TEST_CONTEXT_DEEP_NESTING_RENDERS",
            "TEST_CONTEXT_EMPTY_TYPE_REFUSED",
            "TEST_CONTEXT_PARTS_ARE_ORIGINALS",
            "TEST_CONTEXT_PARTS_EDGE_CASES",
            "TEST_CONTEXT_READS_ARE_IDENTICAL",
            "TEST_CONTEXT_RENDER_EDGE_CASES",
            "TEST_CONTEXT_RENDER_IS_JOINED_PARTS",
            "TEST_CONTEXT_TEXT_EDGE_CASES",
            "TEST_CONTEXT_TEXT_ROUND_TRIPS",
            "TEST_CONTEXT_TYPE_IS_DECLARED",
            "TEST_ID_CANNOT_BE_FORGED",
            "TEST_ID_EXHAUSTION_IS_PERMANENT",
            "TEST_ID_NEVER_REPEATS",
            "TEST_ID_SOURCE_CANNOT_BE_COPIED",
            "TEST_LINEAGE_DEEP_NESTING_WALKS",
            "TEST_LINEAGE_DIAMONDS_ARE_LINEAR",
            "TEST_LINEAGE_EDGE_CASES",
            "TEST_LINEAGE_REACHES_EACH_ONCE",
        ]
    );
    assert!(runs.iter().all(|run| run.outcome() == Outcome::Passed));
    let first = runs.iter().next().expect("nineteen runs");
    assert_eq!(first.id(), "RUN_CONTEXT_DEEP_NESTING_DROPS");
    assert_eq!(first.title(), "agconflo-core context::deep_nesting_drops");
}

/// A real report holding failures, a test inside a tests module and an
/// integration test, compared with the whole file it must produce.
#[test]
fn mixed_report() {
    let runs = import(include_str!("../testdata/mixed.xml"), &["probe-core"]).expect("imports");
    let expected = r#"{
  "current_version": "",
  "versions": {
    "": {
      "needs": {
        "RUN_CONTEXT_FAILS": {
          "executes": [
            "TEST_CONTEXT_FAILS"
          ],
          "id": "RUN_CONTEXT_FAILS",
          "test_outcome": "failed",
          "title": "probe-core context::fails",
          "type": "test_run"
        },
        "RUN_CONTEXT_PASSES": {
          "executes": [
            "TEST_CONTEXT_PASSES"
          ],
          "id": "RUN_CONTEXT_PASSES",
          "test_outcome": "passed",
          "title": "probe-core context::passes",
          "type": "test_run"
        },
        "RUN_CONTEXT_SHOULD_PANIC_BUT_DOES_NOT": {
          "executes": [
            "TEST_CONTEXT_SHOULD_PANIC_BUT_DOES_NOT"
          ],
          "id": "RUN_CONTEXT_SHOULD_PANIC_BUT_DOES_NOT",
          "test_outcome": "failed",
          "title": "probe-core context::should_panic_but_does_not",
          "type": "test_run"
        },
        "RUN_CONTEXT_TESTS_NESTED": {
          "executes": [
            "TEST_CONTEXT_TESTS_NESTED"
          ],
          "id": "RUN_CONTEXT_TESTS_NESTED",
          "test_outcome": "passed",
          "title": "probe-core context::tests::nested",
          "type": "test_run"
        },
        "RUN_INTEG_IN_INTEGRATION": {
          "executes": [
            "TEST_INTEG_IN_INTEGRATION"
          ],
          "id": "RUN_INTEG_IN_INTEGRATION",
          "test_outcome": "passed",
          "title": "probe-core::integ in_integration",
          "type": "test_run"
        }
      }
    }
  }
}
"#;
    assert_eq!(runs.to_json(), expected);
}

#[test]
fn unreadable_reports() {
    let truncated = &REAL[..REAL.len() / 2];
    let missing_name = r#"<testsuites name="r"><testsuite name="s"><testcase classname="agconflo-core"/></testsuite></testsuites>"#;
    for xml in ["", "not a report", truncated, missing_name] {
        let result = import(xml, &["agconflo-core"]);
        assert!(
            matches!(result, Err(ImportError::Unreadable(_))),
            "{xml:.40}: {result:?}"
        );
    }
}

#[test]
fn underivable_tests_refused() {
    let cases = [
        ("core-a::bin/core-a", "in_binary"),
        ("core-a::foo-bar", "in_hyphenated_file"),
        ("core-a", "context::r#type"),
        ("core-a", "context::gr\u{f6}\u{df}e"),
    ];
    for (classname, name) in cases {
        let xml = by_hand([
            ("core-a", "fine", TestCaseStatus::success()),
            (classname, name, TestCaseStatus::success()),
        ]);
        let result = import(&xml, &["core-a"]);
        let expected = format!("{classname} {name}");
        assert!(
            matches!(&result, Err(ImportError::Underivable { test }) if *test == expected),
            "{expected}: {result:?}"
        );
    }
}

/// An untraced crate's tests are not read at all, so names that would make no
/// id there are no error.
#[test]
fn untraced_names_are_not_read() {
    let xml = by_hand([
        ("core-a", "fine", TestCaseStatus::success()),
        ("tool::bin/tool", "in_binary", TestCaseStatus::success()),
        ("tool", "r#type", TestCaseStatus::skipped()),
    ]);
    let runs = import(&xml, &["core-a"]).expect("only core-a is read");
    assert_eq!(
        runs.iter().map(Run::test_case).collect::<Vec<_>>(),
        ["TEST_FINE"]
    );
}

#[test]
fn colliding_ids() {
    let cases = [
        (("core-a", "a::b_c"), ("core-a", "a_b::c"), "TEST_A_B_C"),
        (("core-a", "a::B"), ("core-a", "a::b"), "TEST_A_B"),
        (
            ("core-a", "context::x"),
            ("core-b", "context::x"),
            "TEST_CONTEXT_X",
        ),
    ];
    for ((class_a, name_a), (class_b, name_b), id) in cases {
        let (a, b) = (format!("{class_a} {name_a}"), format!("{class_b} {name_b}"));
        for order in [
            [(class_a, name_a), (class_b, name_b)],
            [(class_b, name_b), (class_a, name_a)],
        ] {
            let xml = by_hand(order.map(|(class, name)| (class, name, TestCaseStatus::success())));
            let result = import(&xml, &["core-a", "core-b"]);
            assert!(
                matches!(&result, Err(ImportError::Duplicate { test_case, first, second })
                    if test_case == id && *first == a && *second == b),
                "{a} / {b}: {result:?}"
            );
        }
    }
}

#[test]
fn skipped_refused() {
    let xml = by_hand([("core-a", "context::later", TestCaseStatus::skipped())]);
    let result = import(&xml, &["core-a"]);
    assert!(
        matches!(&result, Err(ImportError::Skipped { test }) if test == "core-a context::later"),
        "{result:?}"
    );
}

#[test]
fn retried_refused() {
    let mut flaky = TestCaseStatus::success();
    flaky.add_rerun(TestRerun::new(NonSuccessKind::Failure));
    let mut retried = TestCaseStatus::non_success(NonSuccessKind::Failure);
    retried
        .add_rerun(TestRerun::new(NonSuccessKind::Failure))
        .set_rerun_kind(FlakyOrRerun::Rerun);
    for status in [flaky, retried] {
        let xml = by_hand([("core-a", "context::wobbles", status)]);
        let result = import(&xml, &["core-a"]);
        assert!(
            matches!(&result, Err(ImportError::Rerun { test }) if test == "core-a context::wobbles"),
            "{result:?}"
        );
    }
}

#[test]
fn crate_without_tests() {
    let result = import(REAL, &["agconflo-core", "agconflo-cor"]);
    assert!(
        matches!(&result, Err(ImportError::NoTests { krate }) if krate == "agconflo-cor"),
        "{result:?}"
    );

    let lookalike = by_hand([
        ("core-a-extra", "context::x", TestCaseStatus::success()),
        ("core-a-extra::integ", "y", TestCaseStatus::success()),
    ]);
    let result = import(&lookalike, &["core-a"]);
    assert!(
        matches!(&result, Err(ImportError::NoTests { krate }) if krate == "core-a"),
        "{result:?}"
    );
}

/// A test binary's id given where a crate name belongs matches nothing, rather
/// than matching that binary and dropping the file from the ids it makes.
#[test]
fn binary_id_is_not_a_crate() {
    let mixed = include_str!("../testdata/mixed.xml");
    assert_eq!(
        import(mixed, &["probe-core"])
            .expect("the crate imports")
            .iter()
            .filter(|run| run.test_case() == "TEST_INTEG_IN_INTEGRATION")
            .count(),
        1
    );
    let result = import(mixed, &["probe-core::integ"]);
    assert!(
        matches!(&result, Err(ImportError::NoTests { krate }) if krate == "probe-core::integ"),
        "{result:?}"
    );
}

#[test]
fn lookalike_crate_left_out() {
    let xml = by_hand([
        ("core-a", "x", TestCaseStatus::success()),
        ("core-a-extra", "y", TestCaseStatus::success()),
    ]);
    let runs = import(&xml, &["core-a"]).expect("imports");
    assert_eq!(
        runs.iter().map(Run::test_case).collect::<Vec<_>>(),
        ["TEST_X"]
    );
}

#[test]
fn no_crates() {
    let result = import(REAL, &[]);
    assert!(matches!(result, Err(ImportError::NoCrates)), "{result:?}");
}

#[test]
fn unattributed_test() {
    let mut suite = TestSuite::new("somewhere");
    suite.add_test_case(TestCase::new("context::orphan", TestCaseStatus::success()));
    let mut report = Report::new("nextest-run");
    report.add_test_suite(suite);
    let xml = report.to_string().expect("serialises");
    let result = import(&xml, &["core-a"]);
    assert!(
        matches!(&result, Err(ImportError::Unattributed { suite, name })
            if suite == "somewhere" && name == "context::orphan"),
        "{result:?}"
    );
}

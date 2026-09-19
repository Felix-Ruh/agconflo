//! The `test_run` needs, and the file ubc imports them from.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

/// Whether a test passed.
///
/// There is no third value, and that was measured rather than chosen: nextest
/// writes no test case at all for an ignored test (EVD_NEXTEST_IGNORED_ABSENT),
/// so nothing a report says was run can have been skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed,
}

impl Outcome {
    /// The value of the need's `test_outcome` field.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Passed => "passed",
            Outcome::Failed => "failed",
        }
    }
}

/// The latest run of one test: which test case it ran, and how that went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    test_case: String,
    title: String,
    outcome: Outcome,
}

impl Run {
    pub(crate) fn new(test_case: String, title: String, outcome: Outcome) -> Self {
        debug_assert!(test_case.starts_with("TEST_"), "{test_case}");
        Run {
            test_case,
            title,
            outcome,
        }
    }

    /// `RUN_` and the rest of the test case's id, so a case's run is found from
    /// the case's id alone.
    pub fn id(&self) -> String {
        format!("RUN{}", &self.test_case["TEST".len()..])
    }

    /// The id of the test case this run executed.
    pub fn test_case(&self) -> &str {
        &self.test_case
    }

    /// The test as nextest names it: its classname, a space, and its name.
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }
}

/// Every run read from one report, ordered by test case id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Runs {
    runs: BTreeMap<String, Run>,
}

impl Runs {
    /// Adds a run, handing back the one it displaced if its test case already
    /// had one.
    pub(crate) fn insert(&mut self, run: Run) -> Option<Run> {
        self.runs.insert(run.test_case.clone(), run)
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    /// The runs in test case id order.
    pub fn iter(&self) -> impl Iterator<Item = &Run> {
        self.runs.values()
    }

    /// The needs file ubc imports as external needs.
    ///
    /// Its version is left empty: ubc imports the needs whatever it says but
    /// refuses a file with none (EVD_EXTERNAL_VERSION_FREE), and a version
    /// following the project's would change the file on every release.
    ///
    /// The same runs always give the same bytes. Keys are in sorted order at
    /// every level, inserted in that order too, so the output stays the same if
    /// another crate in the build ever switches on serde_json's `preserve_order`.
    /// Lines end in `\n` alone, and the file ends with one.
    pub fn to_json(&self) -> String {
        let mut needs = Map::new();
        for run in self.runs.values() {
            let id = run.id();
            let need = json!({
                "executes": [run.test_case],
                "id": id,
                "test_outcome": run.outcome.as_str(),
                "title": run.title,
                "type": "test_run",
            });
            needs.insert(id, need);
        }
        let file = json!({
            "current_version": "",
            "versions": { "": { "needs": Value::Object(needs) } },
        });
        let mut text = serde_json::to_string_pretty(&file).expect("a JSON value always serialises");
        text.push('\n');
        text
    }
}

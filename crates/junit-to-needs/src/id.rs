//! Which crate a test belongs to, and which test case it ran, from the names
//! nextest gives it.

/// Whether a test named with `classname` belongs to `krate`: the crate matched
/// whole, alone or before `::`. A `krate` holding `::` is a test binary's id,
/// not a crate, and matches nothing.
// @A crate matched whole,TRACE_ID_CRATE_MATCHED_WHOLE,trace,[],[DEC_IMPORT_WHOLE_OR_REFUSED]
pub(crate) fn belongs_to(classname: &str, krate: &str) -> bool {
    !krate.contains(':')
        && classname
            .strip_prefix(krate)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
}

/// The id of the test case a test ran, or `None` when its names make none.
///
/// `TEST_` and each segment of the test's path uppercased, joined by `_`, an
/// integration test's file first. A segment that is empty or holds anything
/// but ASCII letters, digits and underscores makes no id.
// @A test case id from the test path,TRACE_ID_FROM_TEST_PATH,trace,[],[DEC_TESTS_UNDER_NEXTEST, DEC_IMPORT_WHOLE_OR_REFUSED]
pub(crate) fn test_case_id(krate: &str, classname: &str, name: &str) -> Option<String> {
    let file = match classname.strip_prefix(krate)? {
        "" => None,
        rest => Some(rest.strip_prefix("::")?),
    };
    let mut id = String::from("TEST");
    for segment in file.into_iter().chain(name.split("::")) {
        let plain = segment
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_');
        if segment.is_empty() || !plain {
            return None;
        }
        id.push('_');
        id.push_str(&segment.to_ascii_uppercase());
    }
    Some(id)
}

#[test]
fn unit_and_integration_tests() {
    let cases = [
        (
            "agconflo-core",
            "context::text_round_trips",
            "TEST_CONTEXT_TEXT_ROUND_TRIPS",
        ),
        (
            "agconflo-core",
            "context::tests::nested",
            "TEST_CONTEXT_TESTS_NESTED",
        ),
        ("agconflo-core", "top_level", "TEST_TOP_LEVEL"),
        (
            "agconflo-core::lineage",
            "reaches_each_once",
            "TEST_LINEAGE_REACHES_EACH_ONCE",
        ),
        (
            "agconflo-core::lineage",
            "walk::deep",
            "TEST_LINEAGE_WALK_DEEP",
        ),
    ];
    for (classname, name, expected) in cases {
        assert!(belongs_to(classname, "agconflo-core"), "{classname}");
        assert_eq!(
            test_case_id("agconflo-core", classname, name).as_deref(),
            Some(expected),
            "{classname} {name}"
        );
    }
}

#[test]
fn names_that_make_no_id() {
    let cases = [
        ("agconflo-core::bin/agconflo-core", "in_binary"),
        ("agconflo-core::foo-bar", "in_hyphenated_file"),
        ("agconflo-core", "context::r#type"),
        ("agconflo-core", "context::gr\u{f6}\u{df}e"),
        ("agconflo-core", "context::"),
        ("agconflo-core", ""),
        ("agconflo-core::", "empty_file"),
        ("agconflo-core", "context::has space"),
    ];
    for (classname, name) in cases {
        assert_eq!(
            test_case_id("agconflo-core", classname, name),
            None,
            "{classname} {name}"
        );
    }
}

#[test]
fn crates_match_whole() {
    assert!(belongs_to("agconflo-core", "agconflo-core"));
    assert!(belongs_to("agconflo-core::lineage", "agconflo-core"));
    assert!(!belongs_to("agconflo-core-extra", "agconflo-core"));
    assert!(!belongs_to("agconflo-core-extra::lineage", "agconflo-core"));
    assert!(!belongs_to("agconflo", "agconflo-core"));
    assert!(!belongs_to("agconflo-cor", "agconflo-core"));
    assert!(!belongs_to(
        "agconflo-core::lineage",
        "agconflo-core::lineage"
    ));
}

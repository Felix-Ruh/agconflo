//! Which crate a test belongs to, and which test case it ran, from the names
//! nextest gives it.

/// Whether a test named with `classname` belongs to `krate`.
///
/// nextest's classname is the crate for a unit test and `<crate>::<file>` for an
/// integration test, so the crate must match whole: `agconflo-core-extra` is
/// not `agconflo-core`.
///
/// A crate name holding `::` is not one: it is a test binary's id, such as
/// `agconflo-core::lineage`. Taken for a crate it would match that binary and
/// leave the file out of every id made from it - `TEST_REACHES_EACH_ONCE` for a
/// test whose case is `TEST_LINEAGE_REACHES_EACH_ONCE`, which is wrong and
/// looks right. Matching nothing instead stops the import with a crate that has
/// no tests.
pub(crate) fn belongs_to(classname: &str, krate: &str) -> bool {
    !krate.contains(':')
        && classname
            .strip_prefix(krate)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"))
}

/// The id of the test case a test ran, or `None` when its names make none.
///
/// nextest names a unit test `<crate>` / `<module path>::<function>`, and an
/// integration test `<crate>::<file>` / `<function>` (EVD_NEXTEST_TEST_PATHS).
/// The file stands where a module would, so both come out the same way: `TEST_`
/// and each segment uppercased, joined by `_`.
///
/// A segment that is not ASCII letters, digits and underscores makes no id: a
/// binary target's `bin/<name>` (EVD_NEXTEST_IGNORED_ABSENT), a hyphenated
/// test file, a raw identifier, a non-ASCII name. Refusing here names the test,
/// where an id built from them would fail the graph's id pattern naming only
/// the id.
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

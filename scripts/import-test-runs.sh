#!/bin/sh
#
# Bring the test results into the requirements project.
#
#     sh scripts/import-test-runs.sh            rewrite docs/test-runs.json
#     sh scripts/import-test-runs.sh --check    compare only, and never write
#
# The file holds one `test_run` need per test: the test case it ran and whether
# it passed, and nothing that changes between two runs of the same tests. It is
# COMMITTED rather than produced on demand, because an absent external needs
# file is reported, and every gate here checks at a level where that report
# fails (EVD_EXTERNAL_MISSING_WARNS) - so a fresh clone would otherwise be
# unable to check its documentation until it had run the tests.
#
# --check is what the gates run, and it VERIFIES RATHER THAN REWRITES, like
# `cargo fmt --check` and the golden files: a gate that silently fixed its own
# subject would report success over a file nobody had read. Rewriting is a
# person's job, and the diff is theirs to review.
#
# It reads the report nextest writes at target/nextest/default/junit.xml, whose
# path is set in .config/nextest.toml, so the tests must have run first. A run
# under `cargo test` writes no report at all (EVD_STABLE_NO_JUNIT), which is why
# the hook skips this check when it had to fall back to that.
#
# Exit status: 0 when the file matches, or was written; 1 when it differs, is
# missing, or cannot be produced; 2 for an unknown argument.

set -u

LC_ALL=C
export LC_ALL

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)

case "${1:-}" in
    '') check=0 ;;
    --check) check=1 ;;
    *)
        echo "import-test-runs: unknown argument '$1' (only --check is accepted)" >&2
        exit 2
        ;;
esac

# The crates whose tests have test cases in docs/. junit-to-needs is
# deliberately absent: it is tooling, its tests answer to no requirement, and a
# run naming a test case that does not exist is a dead link that would fail the
# documentation build. A case id carries no crate name, so the traced crates'
# module names must not repeat one another.
CRATES="agconflo-core agconflo-lua"

report="$root/target/nextest/default/junit.xml"
committed="$root/docs/test-runs.json"
produced="$root/target/test-runs.json"

if ! command -v cargo >/dev/null 2>&1; then
    echo "import-test-runs: cargo not found - skipping"
    echo "import-test-runs: install the Rust toolchain from https://rustup.rs"
    exit 0
fi

if [ ! -f "$report" ]; then
    echo "import-test-runs: no test report at ${report#"$root"/}" >&2
    echo "import-test-runs: run the tests first: tools/cargo-nextest nextest run --workspace --all-targets" >&2
    exit 1
fi

# The importer prints the file and writes nothing itself, so a failure here
# cannot leave a half-written file behind for the next run to compare against.
crate_args=""
for crate in $CRATES; do
    crate_args="$crate_args --crate $crate"
done

# $crate_args is split on purpose: each word is one argument, and crate names
# hold no spaces.
# shellcheck disable=SC2086
if ! cargo run --quiet --package junit-to-needs -- $crate_args "$report" > "$produced"; then
    echo "import-test-runs: the importer refused the report - see its message above" >&2
    rm -f "$produced"
    exit 1
fi

if [ "$check" -eq 0 ]; then
    mv "$produced" "$committed"
    echo "import-test-runs: wrote ${committed#"$root"/} - review the diff before committing"
    exit 0
fi

if [ ! -f "$committed" ]; then
    echo "import-test-runs: ${committed#"$root"/} is missing" >&2
    echo "import-test-runs: write it with 'sh scripts/import-test-runs.sh' and commit it" >&2
    rm -f "$produced"
    exit 1
fi

if cmp -s "$committed" "$produced"; then
    rm -f "$produced"
    echo "import-test-runs: ok - the committed runs match this run"
    exit 0
fi

echo "import-test-runs: FAIL - the committed runs do not match this run" >&2
diff -u "$committed" "$produced" 2>/dev/null | head -40 >&2
echo "import-test-runs: rewrite it with 'sh scripts/import-test-runs.sh', review the diff, and stage it" >&2
rm -f "$produced"
exit 1

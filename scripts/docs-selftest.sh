#!/bin/sh
#
# Prove that the metamodel's rules still fire.
#
# Every constraint in docs/ubproject.toml and every rule in docs/schemas.json
# gets one fixture under docs-selftest/fixtures/ that breaks it on purpose, plus
# a golden file under docs-selftest/expected/ holding the exact diagnostics that
# break must produce.
#
#     sh scripts/docs-selftest.sh            check every fixture
#     sh scripts/docs-selftest.sh --bless    rewrite every golden file
#
# WHY THIS EXISTS. A wrongly shaped schema rule is not rejected by ubc, it is
# silently ignored - measured four ways: a composite keyword placed directly
# under validate.local, a misspelled keyword, a keyword of the wrong kind for the
# field's type, and any rule about a need's body. All four leave the project
# green. So a rule that has never been seen to fail cannot be assumed to work,
# and "the schema checks passed" means nothing without this.
#
# --bless is for after a deliberate rule change or a ubc version bump. It
# rewrites ALL golden files, never one: if an unrelated fixture has drifted, that
# belongs in the diff you are about to read rather than quietly kept as a stale
# expectation. Read the diff - it is the review. Blessing without reading turns a
# broken rule into an expectation, which is the one way this harness can fail
# while staying green.

set -u

# Collation-independent, so glob order and grep behaviour do not vary with the
# machine's locale.
LC_ALL=C
export LC_ALL

# Derived from this script's own location rather than from git, so it also works
# in an archive export - same reason as scripts/get-ubc.sh.
root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
selftest="$root/docs-selftest"
schemas="$root/docs/schemas.json"

# Not on PATH by design; see .githooks/pre-commit. One POSIX script covers both
# platforms and the Windows binary is ubc.exe.
UBC="$root/tools/ubc"
[ -x "$UBC" ] || UBC="$UBC.exe"

bless=0
case "${1:-}" in
    '') ;;
    --bless) bless=1 ;;
    *)
        echo "docs-selftest: unknown argument '$1' (only --bless is accepted)" >&2
        exit 2
        ;;
esac

abort() {
    echo "docs-selftest: $1" >&2
    exit 1
}

# Both skips mirror the pre-commit hook: a clone that has not run the installer
# can still commit, and CI is the authority.
if [ ! -f "$selftest/ubproject.toml" ]; then
    echo "docs-selftest: no docs-selftest/ubproject.toml - skipping"
    exit 0
fi

if [ ! -x "$UBC" ]; then
    echo "docs-selftest: ubc is not installed - skipping"
    echo "docs-selftest: install it with 'sh scripts/get-ubc.sh'"
    exit 0
fi

failures=0
total=0

for fixture in "$selftest"/fixtures/*.rst; do
    # An unmatched glob expands to the pattern itself. Without this the harness
    # would report success over zero fixtures, or blame one missing file, rather
    # than saying it is proving nothing.
    [ -e "$fixture" ] || abort "no fixtures found in docs-selftest/fixtures/"

    name=$(basename "$fixture" .rst)
    golden="$selftest/expected/$name.txt"
    total=$((total + 1))

    # ONE FIXTURE, ONE FILE, and that invariant earns its place twice.
    # `source.include` narrows the project to this single file, so a fixture's
    # diagnostics cannot depend on any other fixture - which is what lets each
    # golden file hold exactly one rule's output. It also keeps every run inside
    # `ubc check`'s five-file unlicensed free tier, so this self-test still works
    # offline when a whole-project check cannot, and needs none of the licence
    # handling the hook carries. A fixture needing two needs puts both in the
    # same file, as duplicate_id does.
    #
    # No path argument: `ubc check <path>` filters diagnostics instead of scoping
    # the project, so it would silently drop findings rather than narrow them.
    # 2>&1 because diagnostics go to stdout and licence refusals to stderr. This
    # is a subshell and not a pipe, so `$?` really is ubc's status.
    #
    # `-q` keeps the output free of "Building index for <absolute path>", which
    # is what makes a golden file portable between this machine and a runner.
    # `--deny warning` is ubc's default, passed explicitly so a future change to
    # that default cannot quietly loosen the gate.
    actual=$(
        cd "$selftest" && "$UBC" check -q --deny warning \
            -c "source.include=[\"fixtures/$name.rst\"]" 2>&1
    )
    status=$?

    # An environment problem must not be diffed as though it were a diagnostic.
    # ubc prints cache-write failures on STDOUT, mixed in with the findings, so
    # one would otherwise land in every golden file at once and bury whatever
    # really changed. It also says the cache "may be inconsistent", which means
    # the comparison is worthless in that state, so this aborts rather than
    # reporting failures it cannot stand behind.
    #
    # Seen for real: a clone under a deep path, where the cache's own
    # needs_index.sqlite reached 254 characters and SQLite could not create its
    # journal beside it within Windows' 260-character limit. docs-selftest/ hits
    # this eight characters before docs/ does.
    case "$actual" in
        *'Failed to write the project cache'*)
            echo "$actual" >&2
            abort "ubc could not write its cache, so the index may be inconsistent and these
             results cannot be trusted. On Windows this is usually the 260-character path
             limit - try a clone whose path is shorter."
            ;;
    esac

    # The exit code is recorded as well as the output, because the two can move
    # independently. Every diagnostic these fixtures provoke is a WARNING, so
    # `--deny warning` is the only thing making them a gate: if that ever
    # loosened, all of them would still print exactly the same text and quietly
    # exit 0. This line is what catches that.
    actual="# exit: $status
$actual"

    if [ "$bless" -eq 1 ]; then
        mkdir -p "$selftest/expected"
        printf '%s\n' "$actual" > "$golden"
        echo "docs-selftest: blessed $name"
        continue
    fi

    if [ ! -f "$golden" ]; then
        echo "docs-selftest: FAIL $name - no golden file at expected/$name.txt"
        echo "docs-selftest:      run 'sh scripts/docs-selftest.sh --bless' and read the diff"
        failures=$((failures + 1))
        continue
    fi

    # Compared on emptiness rather than on diff's exit status, so the result does
    # not depend on which end of a pipe `$?` reports. --label keeps absolute
    # paths and timestamps out of the failure output.
    diff_output=$(
        printf '%s\n' "$actual" |
            diff -u --label "expected/$name.txt" --label 'actual' "$golden" -
    )

    if [ -n "$diff_output" ]; then
        echo "docs-selftest: FAIL $name"
        printf '%s\n' "$diff_output" | sed 's/^/docs-selftest:   /'
        failures=$((failures + 1))
    fi
done

if [ "$bless" -eq 1 ]; then
    echo "docs-selftest: blessed $total fixture(s)"
    exit 0
fi

# A golden file with no fixture is a dead expectation: the rule it covered may
# have been deleted or its fixture renamed, and either way it has silently
# stopped proving anything while still looking like coverage.
for golden in "$selftest"/expected/*.txt; do
    [ -e "$golden" ] || break
    name=$(basename "$golden" .txt)
    if [ ! -f "$selftest/fixtures/$name.rst" ]; then
        echo "docs-selftest: FAIL - expected/$name.txt has no fixtures/$name.rst"
        failures=$((failures + 1))
    fi
done

# EVERY RULE MUST HAVE A FIXTURE, and this enforces it rather than trusting
# discipline: adding a rule to schemas.json without a fixture that fails it is
# how this harness would decay into decoration.
#
# A rule id reaches the output as `rule-id[index]/json/pointer`, so the trailing
# bracket is matched too - without it a rule whose id is a prefix of another
# rule's id would read as covered. Dormant until docs/schemas.json exists.
if [ -f "$schemas" ]; then
    ids=$(
        grep -oE '"id"[[:space:]]*:[[:space:]]*"[^"]+"' "$schemas" |
            sed 's/.*"\([^"]*\)"$/\1/'
    )
    for id in $ids; do
        # Braces are required, not stylistic: "$id[" reads as an array expansion.
        if ! grep -qF "${id}[" "$selftest"/expected/*.txt 2>/dev/null; then
            echo "docs-selftest: FAIL - rule '$id' has no fixture that fails it"
            failures=$((failures + 1))
        fi
    done
fi

if [ "$failures" -ne 0 ]; then
    echo "docs-selftest: $failures problem(s) across $total fixture(s)" >&2
    exit 1
fi

echo "docs-selftest: $total fixture(s) ok"

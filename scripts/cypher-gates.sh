#!/bin/sh
#
# Run the Cypher gates, or the Cypher review reports.
#
#     sh scripts/cypher-gates.sh             the gates: fail on any row in docs/
#     sh scripts/cypher-gates.sh --report    the reports: print their rows in docs/
#
# A GATE is one file, scripts/gates/<name>.cypher, holding one read-only query
# that returns a column named `offender`, and it PASSES when the query returns
# nothing. Schema rules in docs/schemas.json see one need at a time plus its
# direct links; a gate is for what they cannot express, such as comparing a
# requirement's statement with the title of the component it is allocated to.
#
# A REPORT, scripts/reports/<name>.cypher, has the same shape and a different
# consequence: its rows in docs/ are printed for a person to read, and never
# fail the run. It is for a question whose answer is sometimes legitimately
# "yes" - a parent with one child, say - where a gate would either block honest
# material or have to be weakened until it caught nothing.
#
# EVERY QUERY IS PROVED TO FIRE BEFORE IT IS BELIEVED, gate or report, for the
# same reason every schema rule has a fixture: a query that can never match
# looks exactly like one that found nothing. So each has a fixture,
# docs-selftest/fixtures/gate_<name>.rst or report_<name>.rst, and runs against
# it first. The fixture names its expectations by convention: every need whose
# id contains _BAD_ must be reported, and no other need may be. Both halves are
# checked - the controls are what catch a query that reports too much.
#
# Exit status: 0 when every query fired exactly as its fixture says and, for
# gates, found nothing in docs/; 1 otherwise; 2 for an unknown argument. When
# ubc itself fails, its output is printed verbatim and the run stops, so that
# .githooks/pre-commit can recognise an unavailable licence grant by its
# wording.

set -u

LC_ALL=C
export LC_ALL

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
selftest="$root/docs-selftest"

case "${1:-}" in
    '')
        kind=gate
        queries="$root/scripts/gates"
        ;;
    --report)
        kind=report
        queries="$root/scripts/reports"
        ;;
    *)
        echo "cypher-gates: unknown argument '$1' (only --report is accepted)" >&2
        exit 2
        ;;
esac

UBC="$root/tools/ubc"
[ -x "$UBC" ] || UBC="$UBC.exe"

# Same skips as scripts/docs-selftest.sh, for the same reasons: a clone that
# has not run the installer can still commit, and CI is the authority. The
# `--version` probe is needed because `[ -x ]` answers yes for a binary that
# group policy refuses to run.
if [ ! -x "$UBC" ]; then
    echo "cypher-gates: ubc is not installed - skipping"
    echo "cypher-gates: install it with 'sh scripts/get-ubc.sh'"
    exit 0
fi
if ! "$UBC" --version >/dev/null 2>&1; then
    echo "cypher-gates: ubc is installed but will not run - skipping"
    echo "cypher-gates: nothing is wrong with the binary; this is normally a policy"
    echo "cypher-gates: allowing execution only from certain paths. CI runs this."
    exit 0
fi

# Runs one query from INSIDE a project directory, and that is load-bearing.
#
# `-c 'source.include=[...]'` resolves its paths against the CURRENT DIRECTORY,
# not against `--project`. Measured: from the repository root,
# `--project docs-selftest -c 'source.include=["fixtures/x.rst"]'` matched zero
# needs and printed `[]` with exit 0 - a gate that cannot fail, reading as a gate
# that passed. Running from the project directory makes the relative include
# mean what it says.
#
# `--strict` turns a query naming a label, link or field the project lacks into
# exit 1 instead of a warning and an empty result. On any non-zero exit ubc's
# output goes to STDERR and the run stops: an environment failure has to be seen
# as one, never counted as zero rows. Stderr rather than stdout because every
# caller runs this inside `$( )`, which would capture the text - including the
# licence wording the pre-commit hook recognises - and discard it.
run_query() {
    project_dir=$1
    query=$2
    shift 2
    output=$( (cd "$project_dir" && "$UBC" query cypher --strict -f json "$@" "$query") 2>&1)
    status=$?
    case "$output" in
        *'Failed to write the project cache'*) status=1 ;;
    esac
    if [ "$status" -ne 0 ]; then
        echo "$output" >&2
        echo "cypher-gates: ubc could not run a $kind query (exit $status) - stopping" >&2
        exit 1
    fi
    printf '%s\n' "$output"
}

# The offender ids in a JSON result, one per line, sorted.
#
# A result that is not `[]` but names no offender means the query returns rows
# without the `offender` column, which would otherwise count as zero rows - the
# vacuous case again - so it is refused rather than passed.
offenders() {
    ids=$(printf '%s\n' "$1" | grep -o '"offender": "[^"]*"' | sed 's/^"offender": "//; s/"$//' | sort)
    if [ -z "$ids" ] && [ "$(printf '%s' "$1" | tr -d '[:space:]')" != '[]' ]; then
        echo "cypher-gates: FAIL $2 - rows came back without an 'offender' column" >&2
        return 1
    fi
    printf '%s' "$ids"
}

count_lines() {
    if [ -z "$1" ]; then echo 0; else printf '%s\n' "$1" | wc -l | tr -d ' '; fi
}

failures=0
total=0

for file in "$queries"/*.cypher; do
    [ -e "$file" ] || { echo "cypher-gates: no ${kind}s found in ${queries#"$root"/}/" >&2; exit 1; }

    name=$(basename "$file" .cypher)
    fixture="docs-selftest/fixtures/${kind}_$name.rst"
    query=$(cat "$file")
    total=$((total + 1))

    if [ ! -f "$root/$fixture" ]; then
        echo "cypher-gates: FAIL $name - no fixture at $fixture"
        failures=$((failures + 1))
        continue
    fi

    # 1. The query must fire on its fixture, exactly.
    wanted=$(grep -o ':id: [A-Z0-9_]*_BAD_[A-Z0-9_]*' "$root/$fixture" | sed 's/^:id: //' | sort)
    if [ -z "$wanted" ]; then
        echo "cypher-gates: FAIL $name - $fixture plants no _BAD_ need, so proves nothing"
        failures=$((failures + 1))
        continue
    fi

    result=$(run_query "$selftest" "$query" -c "source.include=[\"fixtures/${kind}_$name.rst\"]") || exit 1
    got=$(offenders "$result" "$name") || { failures=$((failures + 1)); continue; }

    if [ "$got" != "$wanted" ]; then
        echo "cypher-gates: FAIL $name - on its fixture the $kind reported the wrong needs"
        missing=$(printf '%s\n' "$wanted" | grep -vxF -e "$got" || true)
        extra=$(printf '%s\n' "$got" | grep -vxF -e "$wanted" || true)
        [ -n "$missing" ] && printf '%s\n' "$missing" | sed 's/^/cypher-gates:   not reported: /'
        [ -n "$extra" ] && printf '%s\n' "$extra" | sed 's/^/cypher-gates:   wrongly reported: /'
        failures=$((failures + 1))
        continue
    fi
    proved=$(count_lines "$wanted")

    # 2. Only a query that has just been seen to fire is run on the project.
    result=$(run_query "$root/docs" "$query") || exit 1
    got=$(offenders "$result" "$name") || { failures=$((failures + 1)); continue; }
    rows=$(count_lines "$got")

    if [ "$kind" = report ]; then
        echo "cypher-gates: report $name (proved on $proved planted row(s)) - $rows row(s) in docs/ to review"
        [ "$rows" -gt 0 ] && printf '%s\n' "$result" | sed 's/^/cypher-gates:   /'
        continue
    fi

    if [ "$rows" -gt 0 ]; then
        echo "cypher-gates: FAIL $name - the requirements project has offenders:"
        printf '%s\n' "$result" | sed 's/^/cypher-gates:   /'
        failures=$((failures + 1))
        continue
    fi

    echo "cypher-gates: ok $name ($proved planted offender(s) caught, none in docs/)"
done

if [ "$failures" -ne 0 ]; then
    echo "cypher-gates: $failures of $total $kind(s) failed" >&2
    exit 1
fi

echo "cypher-gates: $total $kind(s) ok"

#!/bin/sh
#
# Refuse a change to an existing requirement that no change record names.
#
#     sh scripts/change-records.sh --staged    the index against HEAD (the hook)
#     sh scripts/change-records.sh <base>      HEAD against <base> (CI)
#     sh scripts/change-records.sh --selftest  prove the check on planted changes
#
# A requirement answers to its parents, and changing one is a change request
# with a fixed procedure (AGENTS.md, "Changing a requirement"). The procedure is
# judgement; this is the part of it a machine can hold: whatever changed what an
# existing requirement obliges must arrive with a record of why.
#
# GOVERNED is what obliges: a stakeholder, feature or component requirement's
# statement, its EARS pattern, its verification method, its stakeholder, and
# the links that place it - derived_from, and an architecture's realises and
# uses, and a component requirement's allocated_to. Removing a requirement
# changes all of them. Editing a body's prose changes none, and neither does
# reordering a link list or re-wrapping it. Adding a requirement is not a change.
#
# A RECORD is a line added to docs/decisions/changes.rst, over the same range,
# naming the requirement's id as a word. By convention that line is in the body
# of the `dec` recording the change; the check holds only to the name.
#
# Exit status: 0 when every changed requirement is named by a record; 1 when one
# is not, naming each; 2 for a usage error or a git failure.

set -u

LC_ALL=C
export LC_ALL

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
records=docs/decisions/changes.rst

# The governed fields of every requirement in the docs/ of one tree, one line
# each: "ID<TAB>field<TAB>value", link lists split, trimmed and sorted so that
# their order and wrapping are not a change. A requirement with none of the
# fields still gets an "ID<TAB>exists" line, so that removing it is seen.
#
# $1 is the tree: INDEX for the staged one, anything else a revision.
governed() {
    if [ "$1" = INDEX ]; then
        files=$(git ls-files -- 'docs/*.rst')
    else
        files=$(git ls-tree -r --name-only "$1" -- docs | grep '\.rst$')
    fi
    for file in $files; do
        if [ "$1" = INDEX ]; then
            git show ":$file"
        else
            git show "$1:$file"
        fi
    done | awk '
        function flush() {
            if (id != "") print id "\texists"
        }
        function emit(field, value,    n, i, items, j, k, t) {
            if (field ~ /^(derived_from|realises|uses|allocated_to)$/) {
                n = split(value, items, ",")
                for (i = 1; i <= n; i++) { gsub(/^[ \t]+|[ \t]+$/, "", items[i]) }
                for (i = 2; i <= n; i++) for (j = i; j > 1 && items[j-1] > items[j]; j--) {
                    t = items[j]; items[j] = items[j-1]; items[j-1] = t
                }
                value = ""
                for (i = 1; i <= n; i++) if (items[i] != "") value = value (value == "" ? "" : ",") items[i]
            }
            pending[++count] = field "\t" value
        }
        function close_need(    i) {
            flush()
            for (i = 1; i <= count; i++) if (id != "") print id "\t" pending[i]
            id = ""; count = 0; inside = 0; field = ""
        }
        /^\.\. (stkh_req|feat_req|feat_arch|comp_req)::/ { close_need(); inside = 1; next }
        inside && /^   :[a-z_]+:/ {
            line = $0
            sub(/^   :/, "", line)
            name = line; sub(/:.*/, "", name)
            value = line; sub(/^[a-z_]+:[ \t]*/, "", value)
            if (name == "id") { id = value; field = ""; next }
            if (name ~ /^(statement|ears_pattern|verification_method|stakeholder|derived_from|realises|uses|allocated_to)$/) {
                field = name; fields[field] = value; order[++seen] = field
            } else { field = "" }
            next
        }
        inside && field != "" && /^      +[^ ]/ {
            line = $0; sub(/^ +/, "", line)
            fields[field] = fields[field] " " line
            next
        }
        inside && /^ *$/ {
            for (i = 1; i <= seen; i++) emit(order[i], fields[order[i]])
            seen = 0; delete fields; close_need(); next
        }
        inside && !/^   :/ { field = "" }
        END { for (i = 1; i <= seen; i++) emit(order[i], fields[order[i]]); close_need() }
    ' | sort -u
}

# The requirement ids whose governed lines differ between two such listings.
changed_ids() {
    base_lines=$1
    tree_lines=$2
    base_ids=$(cut -f1 "$base_lines" | sort -u)
    {
        comm -23 "$base_lines" "$tree_lines"
        comm -13 "$base_lines" "$tree_lines"
    } | cut -f1 | sort -u | while read -r id; do
        printf '%s\n' "$base_ids" | grep -qx -- "$id" && printf '%s\n' "$id"
    done
}

# Check one range. $1 the base revision, $2 the tree (INDEX or a revision).
check() {
    base=$1
    tree=$2
    work=$(mktemp -d) || exit 2
    governed "$base" > "$work/base" || exit 2
    governed "$tree" > "$work/tree" || exit 2
    changed_ids "$work/base" "$work/tree" > "$work/changed"

    if [ "$tree" = INDEX ]; then
        git diff --cached "$base" -- "$records" > "$work/diff" || exit 2
    else
        git diff "$base" "$tree" -- "$records" > "$work/diff" || exit 2
    fi
    grep '^+' "$work/diff" | grep -v '^+++' > "$work/added" || true

    missing=0
    while read -r id; do
        if ! grep -qw -- "$id" "$work/added"; then
            echo "change-records: $id changed with no record naming it in $records" >&2
            missing=1
        fi
    done < "$work/changed"

    count=$(wc -l < "$work/changed" | tr -d ' ')
    rm -rf "$work"
    if [ "$missing" -eq 0 ]; then
        echo "change-records: ok - $count changed requirement(s), each named by a record"
    fi
    return "$missing"
}

# Build a scratch repository, plant each kind of change, and require the check
# to refuse exactly the ones it should. Run against both modes.
selftest() {
    scratch=$(mktemp -d) || exit 2
    trap 'rm -rf "$scratch"' EXIT
    cd "$scratch" || exit 2
    git init -q . && git config user.email t@t && git config user.name t && git config commit.gpgsign false && git config core.autocrlf false
    mkdir -p docs/decisions
    cat > docs/reqs.rst <<'RST'
Reqs
====

.. feat_req:: A thing
   :id: FEAT_A
   :derived_from: STKH_X, STKH_Y
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Agconflo shall do a thing.

   Body prose.

.. comp_req:: A part
   :id: CREQ_B
   :derived_from: FEAT_A
   :allocated_to: COMP_C
   :ears_pattern: ubiquitous
   :verification_method: test
   :statement: Part shall do a part.

.. dec:: A decision
   :id: DEC_D
   :dec_status: accepted
   :decided_on: 2026-01-01
   :statement: Agconflo shall decide.
RST
    printf 'Changes\n=======\n' > "$records"
    git add -A && git commit -q -m base
    base=$(git rev-parse HEAD)

    failures=0
    # $1 name, $2 expected exit, $3 sed program on reqs.rst, $4 record line or empty
    plant() {
        git reset -q --hard "$base"
        sed -i "$3" docs/reqs.rst
        [ -n "$4" ] && printf '%s\n' "$4" >> "$records"
        git add -A
        out=$(check "$base" INDEX 2>&1); got=$?
        git commit -q -m planted --allow-empty
        out2=$(check "$base" HEAD 2>&1); got2=$?
        if [ "$got" -eq "$2" ] && [ "$got2" -eq "$2" ]; then
            echo "selftest ok   : $1"
        else
            echo "selftest FAILS: $1 - expected $2, staged gave $got, revision gave $got2: $out $out2"
            failures=1
        fi
    }
    plant "body prose edited"                 0 's/Body prose\./Other prose./' ''
    plant "statement changed, no record"      1 's/do a thing/do another thing/' ''
    plant "statement changed, record"         0 's/do a thing/do another thing/' 'Amends ``FEAT_A``.'
    plant "record names another id"           1 's/do a thing/do another thing/' 'Amends ``FEAT_AB``.'
    plant "derived_from changed"              1 's/STKH_X, STKH_Y/STKH_X/' ''
    plant "links reordered"                   0 's/STKH_X, STKH_Y/STKH_Y,  STKH_X/' ''
    plant "allocated_to changed"              1 's/COMP_C/COMP_E/' ''
    plant "pattern changed"                   1 '0,/ubiquitous/s//event/' ''
    plant "statement wrapped, same words"     0 's/do a thing\./do a\n      thing./' ''
    plant "requirement removed"               1 '/^\.\. comp_req::/,/Part shall do a part\./d' ''
    plant "requirement added"                 0 '$a\
\
.. feat_req:: New\
   :id: FEAT_NEW\
   :statement: Agconflo shall be new.' ''
    plant "decision changed"                  0 's/shall decide/shall decide again/' ''
    return "$failures"
}

case ${1:-} in
    --staged)
        cd "$root" || exit 2
        git rev-parse -q --verify HEAD > /dev/null || { echo "change-records: no HEAD yet - skipped"; exit 0; }
        check HEAD INDEX
        ;;
    --selftest)
        selftest
        ;;
    ''|-*)
        echo "usage: sh scripts/change-records.sh --staged | <base-revision> | --selftest" >&2
        exit 2
        ;;
    *)
        cd "$root" || exit 2
        git rev-parse -q --verify "$1^{commit}" > /dev/null || { echo "change-records: no revision $1" >&2; exit 2; }
        check "$1" HEAD
        ;;
esac

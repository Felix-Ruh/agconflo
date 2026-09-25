#!/bin/sh
#
# Hold the Rust source to the parts of DEC_COMMENTS_SAY_WHAT a machine can check
# (DEC_COMMENT_CHECKS): no need id in a comment that is not a code marker, and no
# code marker line that is not of the marker's shape.
#
#     sh scripts/comment-rules.sh --staged            the index, every held crate (the hook)
#     sh scripts/comment-rules.sh                     the working tree, every held crate (CI)
#     sh scripts/comment-rules.sh --crate <name>      the working tree, one crate, held or not
#     sh scripts/comment-rules.sh --report [<name>..] long comments, for review; gates nothing
#     sh scripts/comment-rules.sh --selftest          prove the checks on planted comments
#
# A crate is held once its comments follow the decisions; HELD lists them.
#
# Exit status: 0 when nothing is refused; 1 when something is, each located; 2 for
# a usage error.

set -u

LC_ALL=C
export LC_ALL

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)

HELD='junit-to-needs agconflo-lua agconflo-core agconflo-runner'

# The report's thresholds: a comment over this many lines, a docstring over that.
LONG_COMMENT=3
LONG_DOCSTRING=12

# Check Rust text on stdin, naming it $1 in every finding.
check_text() {
    awk -v file="$1" '
        BEGIN {
            ids = "(STKH|FEAT|ARCH|COMP|CREQ|DEC|EVD|TEST|IMPL|TRACE|NOTE|RUN)_[A-Z0-9]"
            marker = "^[ \t]*///? ?@[^,]+,(IMPL|TRACE)_[A-Z0-9_]+,(impl|trace),\\[[A-Z0-9_, ]*\\](,\\[[A-Z0-9_, ]*\\])?[ \t]*$"
        }
        {
            if (match($0, /^[ \t]*\/\/[\/!]?/)) {
                text = substr($0, RLENGTH + 1)
            } else if (match($0, /[ \t]\/\/[\/!]?[ \t]/)) {
                text = substr($0, RSTART + RLENGTH)
            } else {
                next
            }
            body = text
            sub(/^[ \t]*/, "", body)
            if (substr(body, 1, 1) == "@") {
                if ($0 !~ marker) {
                    print file ":" NR ": a code marker not of the shape @<title>,<id>,<impl|trace>,[<ids>],[<ids>]"
                    refused = 1
                }
                next
            }
            rest = body
            while (match(rest, "(^|[^A-Za-z0-9_])" ids "[A-Z0-9_]*")) {
                found = substr(rest, RSTART, RLENGTH)
                sub(/^[^A-Z]/, "", found)
                print file ":" NR ": need id " found " in a comment; name it in the code marker instead"
                refused = 1
                rest = substr(rest, RSTART + RLENGTH)
            }
        }
        END { exit refused }
    '
}

# Every Rust file of crate $2 under the tree at $1, one path per line.
crate_files() {
    ( cd "$1" && git ls-files -- "crates/$2/*.rs" )
}

# Check crates $3.. of the repository at $1, as its index when $2 is INDEX and
# as its working tree otherwise.
check_crates() {
    repo=$1
    from=$2
    shift 2
    refused=0
    for crate in "$@"; do
        for file in $(crate_files "$repo" "$crate"); do
            if [ "$from" = INDEX ]; then
                ( cd "$repo" && git show ":$file" ) | check_text "$file" || refused=1
            else
                check_text "$file" < "$repo/$file" || refused=1
            fi
        done
    done
    return "$refused"
}

# Every comment block over its threshold in crates $@, longest first.
report() {
    for crate in "$@"; do
        for file in $(crate_files "$root" "$crate"); do
            awk -v file="$file" -v long_comment="$LONG_COMMENT" -v long_doc="$LONG_DOCSTRING" '
                function close_block() {
                    if (kind == "//" && size > long_comment) print size "\t" file ":" start ": " size "-line comment"
                    if (kind != "//" && kind != "" && size > long_doc) print size "\t" file ":" start ": " size "-line docstring"
                    kind = ""; size = 0
                }
                {
                    this = ""
                    if ($0 ~ /^[ \t]*\/\/!/) this = "//!"
                    else if ($0 ~ /^[ \t]*\/\/\//) this = "///"
                    else if ($0 ~ /^[ \t]*\/\/ ?@/) this = ""
                    else if ($0 ~ /^[ \t]*\/\//) this = "//"
                    if (this != kind) { close_block(); if (this != "") { kind = this; start = NR } }
                    if (this != "") size++
                }
                END { close_block() }
            ' "$root/$file"
        done
    done | sort -t "$(printf '\t')" -k1,1nr | cut -f2-
}

selftest() {
    scratch=$(mktemp -d) || exit 2
    trap 'rm -rf "$scratch"' EXIT
    ( cd "$scratch" && git init -q . && git config core.autocrlf false ) || exit 2
    mkdir -p "$scratch/crates/demo/src"
    failures=0
    # $1 name, $2 expected exit, $3 the file's text
    plant() {
        printf '%s\n' "$3" > "$scratch/crates/demo/src/lib.rs"
        ( cd "$scratch" && git add -A )
        out=$(check_crates "$scratch" INDEX demo 2>&1); got=$?
        out2=$(check_crates "$scratch" TREE demo 2>&1); got2=$?
        if [ "$got" -eq "$2" ] && [ "$got2" -eq "$2" ]; then
            echo "selftest ok   : $1"
        else
            echo "selftest FAILS: $1 - expected $2, staged gave $got, tree gave $got2: $out $out2"
            failures=1
        fi
    }
    plant "one-line comment"                   0 '// Reads the record.'
    plant "several lines, no id"               0 '/// Reads the record.
///
/// # Errors
///
/// A record that is not TOML.'
    plant "id in a docstring"                  1 '/// Reads the record (DEC_RECORD_IN_TOML).'
    plant "id in a comment"                    1 '    // Meets CREQ_READER_CALLS here.'
    plant "id in a module docstring"           1 '//! Answers to STKH_PROVENANCE.'
    plant "id in a comment after code"         1 'let x = 1; // see EVD_LUA_LIMITS_STOP'
    plant "second id on a line"                1 '// Reads the record, which is fine, and FEAT_TOPOLOGY_READS.'
    plant "marker implementing"                0 '// @Reads the record,IMPL_RECORD_READ,impl,[CREQ_RECORD_HOLDS_THE_RUN]'
    plant "marker implementing and following"  0 '// @Reads the record,IMPL_RECORD_READ,impl,[CREQ_RECORD_HOLDS_THE_RUN],[DEC_RECORD_IN_TOML]'
    plant "trace marker"                       0 '    // @Pinned by a decision,TRACE_RECORD_VERSION,trace,[],[DEC_RECORD_HOLDS_EXCHANGES, NOTE_RECORD_LAYOUT]'
    plant "marker in a docstring"              0 '/// @Reads the record,IMPL_RECORD_READ,impl,[CREQ_RECORD_HOLDS_THE_RUN]'
    plant "marker title with a comma"          1 '// @Reads the record, whole,IMPL_RECORD_READ,impl,[CREQ_RECORD_HOLDS_THE_RUN]'
    plant "marker missing its type"            1 '// @Reads the record,IMPL_RECORD_READ,[CREQ_RECORD_HOLDS_THE_RUN]'
    plant "marker of an unknown type"          1 '// @Reads the record,IMPL_RECORD_READ,code,[CREQ_RECORD_HOLDS_THE_RUN]'
    plant "marker with three lists"            1 '// @Reads the record,IMPL_RECORD_READ,impl,[CREQ_A],[DEC_B],[DEC_C]'
    plant "marker in a module docstring"       1 '//! @Reads the record,IMPL_RECORD_READ,impl,[CREQ_RECORD_HOLDS_THE_RUN]'
    plant "id in a string, not a comment"      0 'let id = "DEC_RECORD_IN_TOML";'
    plant "url in a string"                    0 'let base = format!("http://{address}/v1/");'
    plant "prefix inside a longer word"        0 '// Runs SUBTEST_CASE and MYDEC_THING.'
    plant "lowercase id-like word"             0 '// Calls dec_record_in_toml.'
    return "$failures"
}

case ${1:-} in
    --staged)
        cd "$root" || exit 2
        if [ -z "$HELD" ]; then echo "comment-rules: no crate held yet"; exit 0; fi
        # shellcheck disable=SC2086
        check_crates "$root" INDEX $HELD
        ;;
    '')
        if [ -z "$HELD" ]; then echo "comment-rules: no crate held yet"; exit 0; fi
        # shellcheck disable=SC2086
        check_crates "$root" TREE $HELD
        ;;
    --crate)
        [ $# -eq 2 ] || { echo "usage: sh scripts/comment-rules.sh --crate <name>" >&2; exit 2; }
        [ -d "$root/crates/$2" ] || { echo "comment-rules: no crate $2" >&2; exit 2; }
        check_crates "$root" TREE "$2"
        ;;
    --report)
        shift
        if [ $# -eq 0 ]; then
            # shellcheck disable=SC2046
            set -- $(cd "$root/crates" && ls)
        fi
        report "$@"
        ;;
    --selftest)
        selftest
        ;;
    *)
        echo "usage: sh scripts/comment-rules.sh [--staged | --crate <name> | --report [<name>...] | --selftest]" >&2
        exit 2
        ;;
esac

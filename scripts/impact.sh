#!/bin/sh
#
# Print everything a change to one need can reach: the impact analysis a change
# request starts from (AGENTS.md, "Changing a requirement").
#
#     sh scripts/impact.sh <NEED_ID>
#
# Four directions, each a question the analysis has to answer for every row:
#
#   UP        the chain the need answers to - its parents, up to the stakeholder
#             requirement, and for an architecture the features it realises. A
#             change is justified against these and nothing else.
#   DOWN      everything that depends on it through a link: requirements derived
#             from it, the architecture realising it, code markers, test cases
#             and their runs, decisions resting on it. Each either still holds
#             after the change or changes with it, in the same pull request.
#   SIDEWAYS  what it shares a place with without depending on it: requirements
#             allocated to the same component as it or anything below it, those
#             allocated to the components an architecture uses, and feature
#             requirements realised by the same architecture. A change
#             is most likely to break these by accident, because no link says so.
#   TEXT      every line naming it in prose - decision and evidence bodies, doc
#             comments, the README - which no link carries. A need's body is in
#             the graph as its content, but the comments and the README are not,
#             so this direction searches the files.
#
# A need id nothing matches is refused rather than reported as having no impact:
# an empty analysis and a mistyped id look identical otherwise.
#
# Exit status: 0 when the need exists and every query ran; 1 when ubc failed;
# 2 for a missing argument or an unknown need.

set -u

LC_ALL=C
export LC_ALL

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)

if [ "$#" -ne 1 ]; then
    echo "usage: sh scripts/impact.sh <NEED_ID>" >&2
    exit 2
fi
id=$1

case $id in
    *[!A-Z0-9_]*|'')
        echo "impact: '$id' is not a need id - ids are capitals, digits and underscores" >&2
        exit 2
        ;;
esac

ubc="$root/tools/ubc"
if [ ! -e "$ubc" ] && [ -e "$ubc.exe" ]; then
    ubc="$ubc.exe"
fi

# Run one query from inside docs/, as every scoped query here is (AGENTS.md,
# "Gates that go green without checking anything"), with --strict so that a
# misspelt label fails rather than returning nothing.
query() {
    ( cd "$root/docs" && "$ubc" query cypher --strict --project . "$1" ) || {
        echo "impact: ubc failed on the query above" >&2
        exit 1
    }
}

found=$(query "MATCH (n {id: '$id'}) RETURN n.id AS id")
case $found in
    *"$id"*) ;;
    *)
        echo "impact: no need has the id $id" >&2
        exit 2
        ;;
esac

# An architecture has no derived_from: it answers to the feature requirements it
# realises, and through them to theirs. Following derived_from alone reported an
# architecture as answering to nothing.
echo "== UP: what $id answers to"
query "MATCH (n {id: '$id'})-[:derived_from|realises*1..]->(p) RETURN DISTINCT p.id AS id, p.type AS type ORDER BY type, id"

echo
echo "== DOWN: what depends on $id through a link"
query "MATCH (n {id: '$id'})<-[*1..8]-(m) RETURN DISTINCT m.id AS id, m.type AS type ORDER BY type, id"

echo
echo "== SIDEWAYS: requirements sharing a component with $id or anything below it"
query "MATCH (n {id: '$id'})<-[*0..8]-(c:comp_req)-[:allocated_to]->(k:comp)<-[:allocated_to]-(o:comp_req) WHERE NOT (o)-[*0..8]->(n) RETURN DISTINCT k.id AS component, o.id AS requirement ORDER BY component, requirement"

echo
echo "== SIDEWAYS: requirements allocated to a component $id uses (an architecture's)"
query "MATCH (n {id: '$id'})-[:uses]->(k:comp)<-[:allocated_to]-(o:comp_req) RETURN DISTINCT k.id AS component, o.id AS requirement ORDER BY component, requirement"

echo
echo "== SIDEWAYS: feature requirements sharing an architecture with $id"
query "MATCH (n {id: '$id'})<-[:realises]-(a:feat_arch)-[:realises]->(o) WHERE o.id <> n.id RETURN DISTINCT a.id AS architecture, o.id AS requirement ORDER BY requirement"

echo
echo "== TEXT: lines naming $id outside its own definition"
# Link fields and the generated test-run file are left out: what they carry is
# already in DOWN, by link.
( cd "$root" && git grep -n -w -e "$id" -- docs crates scripts README.md AGENTS.md ':!docs/test-runs.json' ) \
    | grep -v -e ":id: $id\$" -e ':\(derived_from\|realises\|uses\|allocated_to\|verifies\|supported_by\|supersedes\): ' \
    || echo "(none)"

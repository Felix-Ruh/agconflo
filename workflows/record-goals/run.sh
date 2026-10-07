#!/bin/sh
# Run the record-goals workflow in a folder prepare.sh made.
#
#   sh workflows/record-goals/run.sh <run folder> <brief>
#
# with OPEN_ROUTER_API_KEY in this command's environment and no other's.
# Keeps the record in <run folder>/run.toml. Stops where agconflo stops: exit 3
# is the approval step awaiting a person, with the change staged in
# <run folder>/work; `git -C <run folder>/work diff --cached` shows it.
# Run from the repository root after `cargo build`.
set -eu

if [ $# -ne 2 ]; then
    echo "usage: sh workflows/record-goals/run.sh <run folder> <brief>" >&2
    exit 2
fi
run=$1
brief=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
agconflo=${AGCONFLO:-$(pwd)/target/debug/agconflo}

cd "$run/workflow"
exec "$agconflo" run manifest.toml --models models.toml --grants ../grants.toml --record ../run.toml \
    --arg-file brief brief "$brief"

#!/bin/sh
#
# Provision a Claude Code on the web session so every gate runs: the steps of
# the README's "Setup", the Docker daemon, the images the tool tests pin, and a
# first build. A local clone never runs it; it stops at once off the web.
#
# Idempotent and non-interactive. Each step reports and moves on when it
# fails, so one missing piece leaves the others in place and the session still
# starts; what failed is printed for the session to read.

set -u

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
    exit 0
fi

cd "${CLAUDE_PROJECT_DIR:-$(dirname "$0")/../..}" || exit 1

failed=''
step() {
    name=$1
    shift
    if ! "$@"; then
        echo "session-start: $name failed" >&2
        failed="$failed $name"
        return 1
    fi
}

# The README's three setup steps.
step ubc sh scripts/get-ubc.sh
step nextest sh scripts/get-nextest.sh
step hooks git config core.hooksPath .githooks

# The daemon is not running when a session starts. Detached, so it outlives
# this script, and waited for, up to 30 s.
start_docker() {
    docker info >/dev/null 2>&1 && return 0
    command -v dockerd >/dev/null 2>&1 || { echo 'session-start: dockerd not found' >&2; return 1; }
    nohup setsid dockerd >/tmp/dockerd.log 2>&1 &
    i=0
    while [ "$i" -lt 30 ]; do
        docker info >/dev/null 2>&1 && return 0
        sleep 1
        i=$((i + 1))
    done
    echo 'session-start: dockerd did not come up; see /tmp/dockerd.log' >&2
    return 1
}

# The images the tool tests make their containers from, read from where the
# tests pin them. An image already present is not pulled again, which keeps
# Docker Hub's anonymous rate limit out of reach.
pull_images() {
    images=$(grep -o '[a-z][a-z0-9._/-]*@sha256:[0-9a-f]\{64\}' crates/agconflo-runner/src/testing.rs | sort -u)
    [ -n "$images" ] || { echo 'session-start: no pinned images found in testing.rs' >&2; return 1; }
    for image in $images; do
        docker image inspect "$image" >/dev/null 2>&1 && continue
        docker pull -q "$image" >/dev/null || return 1
    done
}

if step docker start_docker; then
    step images pull_images
fi

# Build what the gates build, so the first test run does not start cold.
step build cargo build -q --workspace --all-targets

if [ -n "$failed" ]; then
    echo "session-start: finished with failures:$failed" >&2
else
    echo 'session-start: ubc, nextest, the hook, Docker, its images and the build are ready'
fi
exit 0

#!/bin/sh
#
# Download the pinned `ubc` (ubCode) binary into tools/.
#
# Agconflo's documentation toolchain is `ubc` and nothing else - no Python, no
# Sphinx, no Java. This script is the whole installation step. Each clone runs
# it once:
#
#     sh scripts/get-ubc.sh
#
# Re-running is free: an already-correct binary is detected and left alone.
# Pass --force to download again anyway.
#
# THIS SCRIPT COVERS WINDOWS TOO, via Git Bash - it is not a Unix-only
# alternative to a PowerShell script. That is deliberate. A POSIX `sh` is
# already a hard requirement on Windows here, because .githooks/pre-commit is a
# POSIX script that git runs through it, so depending on it costs nothing new
# and one tested code path beats two.
#
# The binary is 58-66 MB depending on platform and is NEVER committed; tools/ is
# gitignored.

set -eu

# -----------------------------------------------------------------------------
# The pin.
#
# 0.31.2b1 is a PRE-RELEASE, and that is forced rather than chosen: `ubc query
# cypher` is a design commitment for this project and openCypher only arrived in
# 0.31.1b1. Newest stable (0.30.3) offers `query filter` alone. Revisit when
# 0.31.x goes stable.
#
# THE SAME VERSION HAS THREE SPELLINGS. This is the trap in any "is it already
# installed?" check, so both forms we need are recorded literally rather than
# derived from one another:
#
#     git tag                  v0.31.2b1
#     URL path and filename    0.31.2b1              <- VERSION
#     `ubc --version` output   ubc 0.31.2-beta.1     <- VERSION_STRING
# -----------------------------------------------------------------------------

VERSION='0.31.2b1'
VERSION_STRING='ubc 0.31.2-beta.1'
BASE_URL='https://download.useblocks.com/ubc'

# -----------------------------------------------------------------------------
# Checksums.
#
# useblocks publishes no checksum file - .sha256, .sha256sum, SHA256SUMS and
# checksums.txt all return 403 next to the binary (probed 2026-08-16). So these
# hashes were computed from a first download rather than obtained from the
# vendor.
#
# BE CLEAR ABOUT WHAT THAT DOES AND DOES NOT BUY. It is trust-on-first-use: it
# PINS the artefact - a silently replaced or truncated download is caught, and
# every later clone provably gets the same bytes this project was developed
# against. It does NOT authenticate the artefact against useblocks. The real
# risk it addresses is mundane: a pre-release artefact being rebuilt or removed
# under a URL we depend on.
#
# Only the platforms this project actually uses are pinned: windows-x64 (the
# development machine) and linux-x64 (CI, when it arrives). Adding another means
# downloading it once and recording its hash here - deliberately a manual step,
# because an unverified platform would defeat the point of the check. Note that
# darwin-x64 is not published at all (403): there is no Intel-Mac build.
# -----------------------------------------------------------------------------

SHA256_WINDOWS_X64='3dfefa2f33b29182db7cf1ccd3c1114a7f8e05bf902d9578a6d2988c9ba27550'  # 69,082,112 bytes
SHA256_LINUX_X64='d7121814e8747bedbacc8f0aa89eb908482cc02d98568563c7e658c5df193e61'    # 60,413,416 bytes

FORCE=0
if [ "${1:-}" = '--force' ]; then
    FORCE=1
elif [ -n "${1:-}" ]; then
    echo "get-ubc: unknown argument '$1' (only --force is accepted)" >&2
    exit 2
fi

# Repository root, derived from this script's own location rather than from git,
# so the script works in an archive export or a copy with no .git directory.
SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
TOOLS_DIR="$REPO_ROOT/tools"

# --- platform ----------------------------------------------------------------
#
# Git Bash reports `MINGW64_NT-10.0-19045`; MSYS2 and Cygwin use their own
# prefixes. All three are Windows and want the .exe.

os=$(uname -s)
arch=$(uname -m)

case "$os" in
    MINGW*|MSYS*|CYGWIN*) os='Windows' ;;
esac

case "$os $arch" in
    'Windows x86_64')
        PLATFORM='windows-x64'
        EXPECTED_SHA="$SHA256_WINDOWS_X64"
        SUFFIX='.exe'
        SIZE_HINT='66 MB'
        ;;
    'Linux x86_64')
        PLATFORM='linux-x64'
        EXPECTED_SHA="$SHA256_LINUX_X64"
        SUFFIX=''
        SIZE_HINT='58 MB'
        ;;
    *)
        cat >&2 <<EOF
get-ubc: no pinned checksum for this platform ($os $arch).

Pinned here: Windows x64 (including Git Bash) and Linux x86_64.

To add one, download the binary once and record its SHA-256 in this script:

    https://download.useblocks.com/ubc/$VERSION/ubc-<platform>-$VERSION

where <platform> is one of linux-x64, linux-arm64, darwin-arm64. There is no
darwin-x64 build - that URL returns 403.
EOF
        exit 1
        ;;
esac

TARGET="$TOOLS_DIR/ubc$SUFFIX"

# --- tools -------------------------------------------------------------------

if command -v sha256sum >/dev/null 2>&1; then
    sha256_of() { sha256sum "$1" | cut -d' ' -f1; }
elif command -v shasum >/dev/null 2>&1; then
    # macOS ships shasum rather than sha256sum.
    sha256_of() { shasum -a 256 "$1" | cut -d' ' -f1; }
else
    echo 'get-ubc: neither sha256sum nor shasum found; cannot verify the download' >&2
    exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
    echo 'get-ubc: curl not found' >&2
    exit 1
fi

# --- already installed? ------------------------------------------------------

installed=''
if [ -x "$TARGET" ]; then
    # A binary that exists but cannot run (truncated, wrong architecture) must
    # read as "not installed", not abort the script under `set -e`.
    installed=$("$TARGET" --version 2>/dev/null | head -n 1 || true)
fi

if [ "$FORCE" -eq 0 ] && [ "$installed" = "$VERSION_STRING" ]; then
    echo "get-ubc: $VERSION_STRING already installed at $TARGET"
    exit 0
fi

if [ -n "$installed" ]; then
    echo "get-ubc: replacing $installed with $VERSION_STRING"
fi

# --- download ----------------------------------------------------------------

file_name="ubc-$PLATFORM-$VERSION$SUFFIX"
url="$BASE_URL/$VERSION/$file_name"

mkdir -p "$TOOLS_DIR"

# Download beside the target and move into place only after the hash checks out,
# so an interrupted or corrupt download can never leave a half-written binary
# where the pre-commit hook and CI expect a working one.
temp="$TARGET.download"
rm -f "$temp"
trap 'rm -f "$temp"' EXIT

echo "get-ubc: downloading $file_name (~$SIZE_HINT)"

# --fail turns an HTTP error into a non-zero exit instead of a body written to
# disk, which would otherwise be "downloaded" and then fail the hash check with
# a misleading message.
curl --fail --silent --show-error --location --retry 3 --output "$temp" "$url"

# --- verify ------------------------------------------------------------------

actual=$(sha256_of "$temp")

if [ "$actual" != "$EXPECTED_SHA" ]; then
    cat >&2 <<EOF

get-ubc: SHA-256 mismatch for $file_name - refusing to install.

    expected  $EXPECTED_SHA
    actual    $actual

The pinned artefact is not the one this project was built against. Do not work
around this by editing the hash: find out why it changed first. The likely
causes are a rebuilt or replaced pre-release artefact upstream, or a corrupt or
intercepted download.
EOF
    exit 1
fi

chmod +x "$temp"
mv -f "$temp" "$TARGET"
trap - EXIT

# --- confirm -----------------------------------------------------------------

installed=$("$TARGET" --version 2>/dev/null | head -n 1 || true)

if [ "$installed" != "$VERSION_STRING" ]; then
    echo "get-ubc: installed binary reports '$installed', expected '$VERSION_STRING'" >&2
    exit 1
fi

echo "get-ubc: installed $VERSION_STRING at $TARGET"

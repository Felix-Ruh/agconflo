#!/bin/sh
#
# Download the pinned `cargo-nextest` into tools/.
#
# Agconflo's tests run under nextest rather than `cargo test`, because nextest
# writes the JUnit report that test results are imported from and stable
# `cargo test` cannot (DEC_TESTS_UNDER_NEXTEST). Each clone runs this once:
#
#     sh scripts/get-nextest.sh
#
# Re-running is free: an already-correct binary is detected and left alone.
# Pass --force to download again anyway.
#
# Shaped like scripts/get-ubc.sh, and one POSIX script for Windows (Git Bash)
# and Linux alike, for the reasons given there. The differences are that
# nextest ships an archive rather than a bare binary, and that its vendor
# publishes checksums.
#
# The binary is invoked by path and never committed; tools/ is gitignored.

set -eu

# -----------------------------------------------------------------------------
# The pin.
#
# The newest release, pre-releases included, checked 2026-09-19:
#
#     gh api repos/nextest-rs/nextest/releases --jq '.[].tag_name'
#
# The release tag is cargo-nextest-<VERSION>, and `--version` prints the
# version followed by a build hash and date, so only its first two words are
# compared.
# -----------------------------------------------------------------------------

VERSION='0.9.145'
VERSION_STRING='cargo-nextest 0.9.145'
BASE_URL="https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-$VERSION"

# -----------------------------------------------------------------------------
# Checksums, of the archives.
#
# Unlike ubc's, these are the vendor's own: each was read from the .sha256 file
# published beside its archive, and matched both GitHub's digest for that asset
# and a download of it (2026-09-19). So a mismatch here means the bytes differ
# from what nextest published, not merely from a first download.
#
# Only the platforms this project uses are pinned: Windows x64 (the development
# machine) and Linux x86_64 (CI). The Windows build is `x86_64-pc-windows-msvc`;
# the short name `windows-x86` on nextest's download host is a 32-bit build.
# -----------------------------------------------------------------------------

SHA256_WINDOWS_X64='5bc4b6789103e9834f596eff2b290cfb5b2b69e5413c78bbaa5acc835ac5baa0'  # 7,690,101 bytes
SHA256_LINUX_X64='32aa82416099eb12fffae9cf1a279ad201fecbd3f74826c613e32e9006b29867'    # 12,050,698 bytes

FORCE=0
if [ "${1:-}" = '--force' ]; then
    FORCE=1
elif [ -n "${1:-}" ]; then
    echo "get-nextest: unknown argument '$1' (only --force is accepted)" >&2
    exit 2
fi

SCRIPT_DIR=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH='' cd -- "$SCRIPT_DIR/.." && pwd)
TOOLS_DIR="$REPO_ROOT/tools"

# --- platform ----------------------------------------------------------------

os=$(uname -s)
arch=$(uname -m)

case "$os" in
    MINGW*|MSYS*|CYGWIN*) os='Windows' ;;
esac

case "$os $arch" in
    'Windows x86_64')
        ARCHIVE="cargo-nextest-$VERSION-x86_64-pc-windows-msvc.zip"
        EXPECTED_SHA="$SHA256_WINDOWS_X64"
        BINARY='cargo-nextest.exe'
        ;;
    'Linux x86_64')
        ARCHIVE="cargo-nextest-$VERSION-x86_64-unknown-linux-gnu.tar.gz"
        EXPECTED_SHA="$SHA256_LINUX_X64"
        BINARY='cargo-nextest'
        ;;
    *)
        cat >&2 <<EOF
get-nextest: no pinned checksum for this platform ($os $arch).

Pinned here: Windows x64 (including Git Bash) and Linux x86_64. To add one,
take its archive's SHA-256 from the .sha256 file published beside it:

    $BASE_URL/
EOF
        exit 1
        ;;
esac

TARGET="$TOOLS_DIR/$BINARY"

# --- tools -------------------------------------------------------------------

if command -v sha256sum >/dev/null 2>&1; then
    sha256_of() { sha256sum "$1" | cut -d' ' -f1; }
elif command -v shasum >/dev/null 2>&1; then
    sha256_of() { shasum -a 256 "$1" | cut -d' ' -f1; }
else
    echo 'get-nextest: neither sha256sum nor shasum found; cannot verify the download' >&2
    exit 1
fi

for tool in curl tar unzip; do
    case "$tool:$ARCHIVE" in
        tar:*.zip|unzip:*.tar.gz) continue ;;
    esac
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "get-nextest: $tool not found" >&2
        exit 1
    fi
done

# --- already installed? ------------------------------------------------------

installed=''
if [ -x "$TARGET" ]; then
    installed=$("$TARGET" nextest --version 2>/dev/null | head -n 1 | cut -d' ' -f1-2 || true)
fi

if [ "$FORCE" -eq 0 ] && [ "$installed" = "$VERSION_STRING" ]; then
    echo "get-nextest: $VERSION_STRING already installed at $TARGET"
    exit 0
fi

if [ -n "$installed" ]; then
    echo "get-nextest: replacing $installed with $VERSION_STRING"
fi

# --- download and verify -----------------------------------------------------
#
# The archive is fetched and unpacked in a directory beside the target, and the
# binary is moved into place only after the archive's hash checks out, so an
# interrupted or corrupt download never leaves a half-written binary where the
# hook and CI expect a working one.

mkdir -p "$TOOLS_DIR"
work="$TOOLS_DIR/.nextest-download"
rm -rf "$work"
mkdir "$work"
trap 'rm -rf "$work"' EXIT

echo "get-nextest: downloading $ARCHIVE"
curl --fail --silent --show-error --location --retry 3 --output "$work/$ARCHIVE" "$BASE_URL/$ARCHIVE"

actual=$(sha256_of "$work/$ARCHIVE")
if [ "$actual" != "$EXPECTED_SHA" ]; then
    cat >&2 <<EOF

get-nextest: SHA-256 mismatch for $ARCHIVE - refusing to install.

    expected  $EXPECTED_SHA
    actual    $actual

These are the vendor's published checksums, so the download is not what nextest
released. Do not work around this by editing the hash: find out why first.
EOF
    exit 1
fi

case "$ARCHIVE" in
    *.zip) unzip -q "$work/$ARCHIVE" "$BINARY" -d "$work" ;;
    *.tar.gz) tar -xzf "$work/$ARCHIVE" -C "$work" "$BINARY" ;;
esac

chmod +x "$work/$BINARY"
mv -f "$work/$BINARY" "$TARGET"

# --- confirm -----------------------------------------------------------------
#
# The same two failures get-ubc.sh separates, for the same reason: a binary that
# runs and reports the wrong version, and one that is refused - exit 126 on
# Windows when group policy allows execution only from certain paths. The bytes
# were verified above, so the second is never a bad download.

probe_err="$work/probe.err"
probe_status=0
probe_out=$("$TARGET" nextest --version 2>"$probe_err") || probe_status=$?
installed=$(printf '%s\n' "$probe_out" | head -n 1 | cut -d' ' -f1-2)

if [ "$probe_status" -ne 0 ] && [ -z "$installed" ]; then
    echo "get-nextest: the binary was installed but will not run (exit $probe_status)." >&2
    if [ -s "$probe_err" ]; then
        sed 's/^/get-nextest:   /' "$probe_err" >&2
    fi
    echo "get-nextest: its archive matched the vendor's checksum, so this is not a bad" >&2
    echo "get-nextest: download. Exit 126 normally means a group policy allowing execution" >&2
    echo "get-nextest: only from certain paths. The binary is left in place." >&2
    exit 1
fi

if [ "$installed" != "$VERSION_STRING" ]; then
    echo "get-nextest: installed binary reports '$installed', expected '$VERSION_STRING'" >&2
    exit 1
fi

echo "get-nextest: installed $VERSION_STRING at $TARGET"

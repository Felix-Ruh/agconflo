#!/bin/sh
# Prepare a run folder for the record-goals workflow.
#
#   sh workflows/record-goals/prepare.sh <run folder> <commit> [certificates]
#
# Builds the two images the workflow's tool steps run in, clones the repository
# from GitHub at <commit> into <run folder>/work, and copies the workflow there
# with its placeholders filled: the ubc image in manifest.toml, and the images,
# the work folder and, when given, the certificates a step trusts in grants.toml.
# Run from the repository root, with Docker running, on any machine: the ubc
# image is given ubc's Linux build, pinned as scripts/get-ubc.sh pins it.
set -eu

if [ $# -lt 2 ]; then
    echo "usage: sh workflows/record-goals/prepare.sh <run folder> <commit> [certificates]" >&2
    exit 2
fi
run=$1
commit=$2
certs=${3:-}
here=workflows/record-goals

if [ -e "$run" ]; then
    echo "prepare: $run exists; give a new run folder" >&2
    exit 2
fi
mkdir -p "$run"
# The current folder as the native agconflo reads it: Git Bash's own form of a
# Windows path, /c/..., is not one Windows opens.
native() { pwd -W 2>/dev/null || pwd; }
run=$(cd "$run" && native)

# The images. ubc's Linux build, at the version and checksum get-ubc.sh pins,
# whatever this machine runs. A proxy that intercepts TLS needs its
# certificates at build time.
pin() { sed -n "s/^$1='\([^']*\)'.*/\1/p" scripts/get-ubc.sh; }
version=$(pin VERSION)
pinned=$(pin SHA256_LINUX_X64)
context=$(mktemp -d)
curl --fail --silent --show-error --location --retry 3 --output "$context/ubc" \
    "$(pin BASE_URL)/$version/ubc-linux-x64-$version"
if [ "$(sha256sum "$context/ubc" | cut -d' ' -f1)" != "$pinned" ]; then
    echo "prepare: ubc $version for Linux does not match its pinned checksum" >&2
    rm -rf "$context"
    exit 1
fi
chmod +x "$context/ubc"
docker build -q --network host -t agconflo-record-goals-ubc -f "$here/images/ubc/Dockerfile" "$context" >/dev/null
rm -rf "$context"
if [ -n "$certs" ]; then
    DOCKER_BUILDKIT=1 docker build -q --network host --secret "id=ca,src=$certs" \
        -t agconflo-record-goals-git "$here/images/git" >/dev/null
else
    DOCKER_BUILDKIT=1 docker build -q --network host -t agconflo-record-goals-git "$here/images/git" >/dev/null
fi
ubc_image=$(docker image inspect -f '{{.Id}}' agconflo-record-goals-ubc)
git_image=$(docker image inspect -f '{{.Id}}' agconflo-record-goals-git)

# The clone. ubc grants its licence by the repository's remote, so it is cloned
# from GitHub rather than from this checkout.
git clone -q https://github.com/Felix-Ruh/agconflo "$run/work"
git -C "$run/work" checkout -q "$commit"

# The workflow, with what this machine decides filled in.
mkdir "$run/workflow"
cp -r "$here"/*.lua "$here"/lib "$here"/flow.toml "$here"/types.toml "$here"/models.toml "$run/workflow/"
sed "s|@UBC_IMAGE@|$ubc_image|" "$here/manifest.toml" > "$run/workflow/manifest.toml"
trust=""
if [ -n "$certs" ]; then
    trust="trust = \"$(cd "$(dirname "$certs")" && native)/$(basename "$certs")\""
fi
sed -e "s|@GIT_IMAGE@|$git_image|" -e "s|@UBC_IMAGE@|$ubc_image|" -e "s|@WORK@|$run/work|" -e "s|@TRUST@|$trust|" \
    "$here/grants.template.toml" > "$run/grants.toml"

echo "prepared $run at $(git -C "$run/work" log --oneline -1)"

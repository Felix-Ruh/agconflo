#!/bin/sh
# Prepare a run folder for the record-goals workflow.
#
#   sh workflows/record-goals/prepare.sh <run folder> <commit> [certificates]
#
# Builds the two images the workflow's tool steps run in, clones the repository
# from GitHub at <commit> into <run folder>/work, and copies the workflow there
# with its placeholders filled: the ubc image in manifest.toml, and the images,
# the work folder and, when given, the certificates a step trusts in grants.toml.
# Run from the repository root, with tools/ubc installed and Docker running.
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
run=$(cd "$run" && pwd)

# The images. A proxy that intercepts TLS needs its certificates at build time.
context=$(mktemp -d)
cp tools/ubc "$context/ubc"
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
    trust="trust = \"$(cd "$(dirname "$certs")" && pwd)/$(basename "$certs")\""
fi
sed -e "s|@GIT_IMAGE@|$git_image|" -e "s|@UBC_IMAGE@|$ubc_image|" -e "s|@WORK@|$run/work|" -e "s|@TRUST@|$trust|" \
    "$here/grants.template.toml" > "$run/grants.toml"

echo "prepared $run at $(git -C "$run/work" log --oneline -1)"

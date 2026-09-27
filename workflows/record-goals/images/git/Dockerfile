# The image the workflow's command steps run in: alpine with git.
# Behind a proxy that intercepts TLS, pass its certificates as the build secret
# `ca`; without it the build uses the image's own.
FROM alpine@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6
RUN --mount=type=secret,id=ca,required=false \
    if [ -f /run/secrets/ca ]; then export SSL_CERT_FILE=/run/secrets/ca; fi; \
    apk add --no-cache git

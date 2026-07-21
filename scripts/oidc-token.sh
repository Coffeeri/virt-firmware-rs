#!/bin/sh

# config
URL="https://crates.io/api/v1/trusted_publishing/tokens"
UA="gitlab-trusted-publishing (kraxel@redhat.com)"

set -e
response="$(mktemp ${TMPDIR-/tmp}/response-XXXXXXXX.json)"
curl -s -X POST "$URL" \
     -H "Content-Type: application/json" \
     -H "User-Agent: $UA" \
     -d "{\"jwt\": \"$CRATES_IO_ID_TOKEN\"}" \
     > "$response"

if "$(jq .errors)" != "null"; then
    jq .errors "$response" 1>&2
    exit 1
fi

jq . "$response" | tr "a-fA-F" "x"

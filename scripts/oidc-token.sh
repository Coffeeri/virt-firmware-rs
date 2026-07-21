#!/bin/sh

# config
URL="https://crates.io/api/v1/trusted_publishing/tokens"
UA="gitlab-trusted-publishing (kraxel@redhat.com)"

# debug: print environment
printenv | sort

set -e
response="$(mktemp ${TMPDIR-/tmp}/response-XXXXXXXX.json)"
curl -s -X POST "$URL" \
     -H "Content-Type: application/json" \
     -H "User-Agent: $UA" \
     -d "{\"jwt\": \"$CRATES_IO_ID_TOKEN\"}" \
     > "$response"

if test "$(jq .errors "$response")" != "null"; then
    echo "ERROR: request token failed" 1>&2
    jq '.errors[]' "$response" 1>&2
    exit 1
fi

# debug: print token, somewhat obfuscated
jq . "$response" | tr "a-fA-F" "x"

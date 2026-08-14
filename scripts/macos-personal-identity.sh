#!/bin/sh

# The free Personal Team installed on this Mac. Override this variable when
# using the helper on another Mac with a different Personal Team.
PERSONAL_APPLE_TEAM_ID="${PERSONAL_APPLE_TEAM_ID:-P3F5KAG4P7}"

find_personal_apple_signing_identity() {
  security find-identity -v -p codesigning |
    awk '
      /"Apple Development: / {
        certificate = $0
        identity = $0
        sub(/^[[:space:]]*[0-9]+\)[[:space:]]*/, "", certificate)
        sub(/[[:space:]]+".*$/, "", certificate)
        sub(/^[^"]*"/, "", identity)
        sub(/"[^"]*$/, "", identity)
        print certificate "\t" identity
      }
    ' |
    while IFS="$(printf '\t')" read -r certificate_hash certificate_name; do
      [ -n "$certificate_hash" ] || continue

      certificate_team="$(
        security find-certificate -a -p -c "$certificate_name" 2>/dev/null |
          openssl x509 -noout -subject 2>/dev/null |
          sed -n 's#.*\/OU=\([^/]*\).*#\1#p' |
          head -n 1
      )"
      if [ "$certificate_team" = "$PERSONAL_APPLE_TEAM_ID" ]; then
        printf '%s\n' "$certificate_hash"
        break
      fi
    done
}

#!/bin/sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "$SCRIPT_DIR/macos-personal-identity.sh"

APPLE_SIGNING_IDENTITY="$(find_personal_apple_signing_identity || true)"

if [ -z "$APPLE_SIGNING_IDENTITY" ]; then
  echo "No personal Apple Development identity found for Team $PERSONAL_APPLE_TEAM_ID." >&2
  echo "Run 'security find-identity -v -p codesigning' to inspect local certificates." >&2
  exit 2
fi

echo "Using personal Apple Development identity SHA-1: $APPLE_SIGNING_IDENTITY (Team $PERSONAL_APPLE_TEAM_ID)" >&2
export APPLE_SIGNING_IDENTITY

exec sh "$SCRIPT_DIR/build-macos-signed.sh"

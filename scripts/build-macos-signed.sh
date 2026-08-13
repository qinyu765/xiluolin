#!/bin/sh

set -eu

if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
  echo "APPLE_SIGNING_IDENTITY must name a local Apple Development identity." >&2
  exit 2
fi

if [ "$APPLE_SIGNING_IDENTITY" = "-" ]; then
  echo "The signed build requires an Apple Development identity, not ad-hoc '-'." >&2
  exit 2
fi

export MACOSX_DEPLOYMENT_TARGET=13.0
export CMAKE_OSX_DEPLOYMENT_TARGET=13.0

exec pnpm tauri build --target aarch64-apple-darwin --bundles app,dmg

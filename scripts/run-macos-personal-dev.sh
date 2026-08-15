#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
. "$SCRIPT_DIR/macos-personal-identity.sh"

# Tauri passes Cargo's `run ... -- [app args]` to a custom runner. Build first
# so the executable can be signed before it ever touches the Keychain.
if [[ "${1:-}" != "run" ]]; then
  exec cargo "$@"
fi
shift

cargo_args=()
app_args=()
parsing_cargo_args=true
for argument in "$@"; do
  if $parsing_cargo_args && [[ "$argument" == "--" ]]; then
    parsing_cargo_args=false
    continue
  fi

  if $parsing_cargo_args; then
    cargo_args+=("$argument")
  else
    app_args+=("$argument")
  fi
done

target_triple=""
profile="debug"
for ((index = 0; index < ${#cargo_args[@]}; index++)); do
  argument="${cargo_args[index]}"
  case "$argument" in
    --release)
      profile="release"
      ;;
    --target)
      if ((index + 1 < ${#cargo_args[@]})); then
        target_triple="${cargo_args[index + 1]}"
      fi
      ;;
    --target=*)
      target_triple="${argument#--target=}"
      ;;
  esac
done

build_directory="$REPOSITORY_ROOT/src-tauri"
(cd "$build_directory" && cargo build "${cargo_args[@]}")

target_directory="${CARGO_TARGET_DIR:-$build_directory/target}"
if [[ "$target_directory" != /* ]]; then
  target_directory="$build_directory/$target_directory"
fi

if [[ -n "$target_triple" ]]; then
  app_binary="$target_directory/$target_triple/$profile/xiluolin"
else
  app_binary="$target_directory/$profile/xiluolin"
fi

if [[ ! -x "$app_binary" ]]; then
  echo "Tauri built the application, but the expected binary was not found: $app_binary" >&2
  exit 2
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  if ((${#app_args[@]} > 0)); then
    exec "$app_binary" "${app_args[@]}"
  else
    exec "$app_binary"
  fi
fi

apple_signing_identity="$(find_personal_apple_signing_identity || true)"
if [[ -z "$apple_signing_identity" ]]; then
  echo "No personal Apple Development identity found for Team $PERSONAL_APPLE_TEAM_ID." >&2
  echo "Refusing to start an ad-hoc binary because it would make Keychain permissions unstable." >&2
  exit 2
fi

if ! /usr/bin/codesign \
  --force \
  --sign "$apple_signing_identity" \
  --identifier com.xiluolin.desktop \
  --timestamp=none \
  "$app_binary"; then
  echo "Could not sign the macOS dev binary with personal Team $PERSONAL_APPLE_TEAM_ID." >&2
  exit 2
fi

echo "Using personal Apple Development Team $PERSONAL_APPLE_TEAM_ID for tauri dev." >&2
if ((${#app_args[@]} > 0)); then
  exec "$app_binary" "${app_args[@]}"
else
  exec "$app_binary"
fi

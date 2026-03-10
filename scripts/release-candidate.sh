#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -lt 2 ]]; then
  echo "usage: $0 <signed|notarized> <questionnaire|binder|sop> [additional apps...]" >&2
  exit 1
fi

MODE="$1"
shift

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# shellcheck disable=SC1091
source "$REPO_ROOT/.codex/actions/_artifact_env.sh"

cleanup_paths=()

decode_base64_to_file() {
  local encoded="$1"
  local output_path="$2"

  if (base64 --help 2>&1 || true) | grep -q -- '--decode'; then
    printf '%s' "$encoded" | base64 --decode > "$output_path"
  else
    printf '%s' "$encoded" | base64 -D > "$output_path"
  fi
}

cleanup() {
  local path
  for path in "${cleanup_paths[@]}"; do
    rm -f "$path"
  done
}

trap cleanup EXIT

if [[ -n "${APPLE_API_KEY_P8_BASE64:-}" && -z "${APPLE_API_KEY_PATH:-}" ]]; then
  if [[ -z "${APPLE_API_KEY:-}" ]]; then
    echo "APPLE_API_KEY must be set when using APPLE_API_KEY_P8_BASE64" >&2
    exit 1
  fi

  key_dir="$CODEX_LOG_RUN_DIR/private_keys"
  mkdir -p "$key_dir"
  key_path="$key_dir/AuthKey_${APPLE_API_KEY}.p8"
  decode_base64_to_file "$APPLE_API_KEY_P8_BASE64" "$key_path"
  chmod 600 "$key_path"
  export APPLE_API_KEY_PATH="$key_path"
  cleanup_paths+=("$key_path")
fi

"$REPO_ROOT/scripts/check-release-env.sh" "$MODE"

echo "[release-candidate] mode=$MODE targets=$*"
CODEX_BUNDLE_MODE="$MODE" "$REPO_ROOT/scripts/package-app.sh" "$@"

echo "[release-candidate] completed"

#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: $0 <questionnaire|binder|sop> [additional apps...]" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# shellcheck disable=SC1091
source "$REPO_ROOT/.codex/actions/_artifact_env.sh"

cd "$REPO_ROOT"

BUNDLE_MODE="${CODEX_BUNDLE_MODE:-rehearsal}"
ARTIFACT_ROOT="$CODEX_LOG_RUN_DIR/bundle-artifacts/$BUNDLE_MODE"
mkdir -p "$ARTIFACT_ROOT"

app_label() {
  case "$1" in
    questionnaire)
      printf '%s' 'Compliance Suite - Questionnaire'
      ;;
    binder)
      printf '%s' 'Compliance Suite - Binder'
      ;;
    sop)
      printf '%s' 'Compliance Suite - SOP'
      ;;
    *)
      return 1
      ;;
  esac
}

first_dmg_for_label() {
  local label="$1"
  find "$CARGO_TARGET_DIR/release/bundle/dmg" -maxdepth 1 -type f -name "$label*.dmg" -print | sort | head -n 1
}

for app in "$@"; do
  case "$app" in
    questionnaire)
      app_dir="apps/questionnaire"
      ;;
    binder)
      app_dir="apps/binder"
      ;;
    sop)
      app_dir="apps/sop"
      ;;
    *)
      echo "unknown app: $app" >&2
      exit 1
      ;;
  esac

  echo ""
  echo "=== Packaging $app ==="
  pnpm --dir "$app_dir" run tauri build

  label="$(app_label "$app")"
  built_app="$CARGO_TARGET_DIR/release/bundle/macos/$label.app"
  built_dmg="$(first_dmg_for_label "$label")"
  archive_dir="$ARTIFACT_ROOT/$app"

  if [[ ! -d "$built_app" ]]; then
    echo "missing expected app bundle after packaging: $built_app" >&2
    exit 1
  fi
  if [[ -z "$built_dmg" ]]; then
    echo "missing expected dmg after packaging: $label" >&2
    exit 1
  fi

  rm -rf "$archive_dir"
  mkdir -p "$archive_dir"
  cp -R "$built_app" "$archive_dir/"
  cp "$built_dmg" "$archive_dir/"
done

"$REPO_ROOT/scripts/collect-bundle-evidence.sh" "$BUNDLE_MODE" "$@"

echo ""
echo "=== Package rehearsal complete ==="

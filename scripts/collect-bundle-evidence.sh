#!/usr/bin/env bash
set -euo pipefail

if [[ "$#" -lt 2 ]]; then
  echo "usage: $0 <rehearsal|signed|notarized> <questionnaire|binder|sop> [additional apps...]" >&2
  exit 1
fi

MODE="$1"
shift

case "$MODE" in
  rehearsal|signed|notarized)
    ;;
  *)
    echo "unsupported bundle evidence mode: $MODE" >&2
    exit 1
    ;;
esac

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# shellcheck disable=SC1091
source "$REPO_ROOT/.codex/actions/_artifact_env.sh"
shopt -s nullglob

bundle_root="${CARGO_TARGET_DIR:-$CODEX_BUILD_RUST_DIR}/release/bundle"
artifact_root="$CODEX_LOG_RUN_DIR/bundle-artifacts/$MODE"
evidence_dir="$CODEX_LOG_RUN_DIR/bundle-evidence/$MODE"
mkdir -p "$evidence_dir"

timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
git_commit="$(git -C "$REPO_ROOT" rev-parse HEAD)"
git_branch="$(git -C "$REPO_ROOT" branch --show-current)"

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

app_identifier() {
  case "$1" in
    questionnaire)
      printf '%s' 'com.compliancesuite.questionnaire'
      ;;
    binder)
      printf '%s' 'com.compliancesuite.binder'
      ;;
    sop)
      printf '%s' 'com.compliancesuite.sop'
      ;;
    *)
      return 1
      ;;
  esac
}

json_escape() {
  node -e 'process.stdout.write(JSON.stringify(process.argv[1]))' "$1"
}

credential_mode="local-keychain"
if [[ -n "${APPLE_CERTIFICATE:-}" ]]; then
  credential_mode="certificate-env"
fi
if [[ -n "${APPLE_API_KEY_P8_BASE64:-}" ]]; then
  credential_mode="${credential_mode}+api-key-base64"
elif [[ -n "${APPLE_API_KEY_PATH:-}" ]]; then
  credential_mode="${credential_mode}+api-key-path"
elif [[ -n "${APPLE_ID:-}" ]]; then
  credential_mode="${credential_mode}+apple-id"
fi
if [[ "$MODE" == "rehearsal" ]]; then
  credential_mode="not-applicable"
fi

bundle_identifier_for() {
  /usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$1"
}

bundle_version_for() {
  /usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$1"
}

bundle_executable_for() {
  /usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$1"
}

first_dmg_for_label() {
  local label="$1"
  find "$bundle_root/dmg" -maxdepth 1 -type f -name "$label*.dmg" -print | sort | head -n 1
}

artifact_app_path_for() {
  local app="$1"
  local label="$2"
  local archived_app="$artifact_root/$app/$label.app"
  if [[ -d "$archived_app" ]]; then
    printf '%s' "$archived_app"
  else
    printf '%s' "$bundle_root/macos/$label.app"
  fi
}

artifact_dmg_path_for() {
  local app="$1"
  local label="$2"
  local archived_dmg
  archived_dmg="$(find "$artifact_root/$app" -maxdepth 1 -type f -name "$label*.dmg" -print 2>/dev/null | sort | head -n 1)"
  if [[ -n "$archived_dmg" ]]; then
    printf '%s' "$archived_dmg"
  else
    first_dmg_for_label "$label"
  fi
}

json_path="$evidence_dir/bundle-evidence.json"
md_path="$evidence_dir/bundle-evidence.md"

{
  echo "{"
  echo "  \"generated_at\": $(json_escape "$timestamp"),"
  echo "  \"bundle_mode\": $(json_escape "$MODE"),"
  echo "  \"credential_mode\": $(json_escape "$credential_mode"),"
  echo "  \"git_commit\": $(json_escape "$git_commit"),"
  echo "  \"git_branch\": $(json_escape "$git_branch"),"
  echo "  \"bundle_root\": $(json_escape "$bundle_root"),"
  echo "  \"artifact_root\": $(json_escape "$artifact_root"),"
  echo "  \"apps\": ["

  first_app=true
  for app in "$@"; do
    label="$(app_label "$app")"
    expected_identifier="$(app_identifier "$app")"
    app_path="$(artifact_app_path_for "$app" "$label")"
    dmg_path="$(artifact_dmg_path_for "$app" "$label")"

    if [[ ! -d "$app_path" ]]; then
      echo "missing expected app bundle: $app_path" >&2
      exit 1
    fi
    if [[ -z "$dmg_path" ]]; then
      echo "missing expected dmg for $app" >&2
      exit 1
    fi

    info_plist="$app_path/Contents/Info.plist"
    if [[ ! -f "$info_plist" ]]; then
      echo "missing expected Info.plist: $info_plist" >&2
      exit 1
    fi

    bundle_identifier="$(bundle_identifier_for "$info_plist")"
    if [[ "$bundle_identifier" != "$expected_identifier" ]]; then
      echo "unexpected bundle identifier for $app: $bundle_identifier" >&2
      exit 1
    fi

    bundle_version="$(bundle_version_for "$info_plist")"
    bundle_executable="$(bundle_executable_for "$info_plist")"
    executable_path="$app_path/Contents/MacOS/$bundle_executable"
    if [[ ! -f "$executable_path" ]]; then
      echo "missing expected bundle executable: $executable_path" >&2
      exit 1
    fi

    app_size_bytes="$(du -sk "$app_path" | awk '{print $1 * 1024}')"
    dmg_size_bytes="$(stat -f '%z' "$dmg_path")"

    if [[ "$first_app" == true ]]; then
      first_app=false
    else
      echo "    ,"
    fi

    {
      echo "    {"
      echo "      \"app\": $(json_escape "$app"),"
      echo "      \"product_name\": $(json_escape "$label"),"
      echo "      \"bundle_identifier\": $(json_escape "$bundle_identifier"),"
      echo "      \"bundle_version\": $(json_escape "$bundle_version"),"
      echo "      \"bundle_executable\": $(json_escape "$bundle_executable"),"
      echo "      \"app_bundle_path\": $(json_escape "$app_path"),"
      echo "      \"app_bundle_size_bytes\": $app_size_bytes,"
      echo "      \"dmg_path\": $(json_escape "$dmg_path"),"
      echo "      \"dmg_size_bytes\": $dmg_size_bytes"
      echo "    }"
    }
  done

  echo "  ]"
  echo "}"
} > "$json_path"

{
  echo "# Bundle Evidence"
  echo
  echo "- Generated at: $timestamp"
  echo "- Bundle mode: \`$MODE\`"
  echo "- Credential mode: \`$credential_mode\`"
  echo "- Commit: \`$git_commit\`"
  echo "- Branch: \`$git_branch\`"
  echo "- Bundle root: \`$bundle_root\`"
  echo "- Artifact root: \`$artifact_root\`"
  echo
  echo "## Artifacts"
  echo
  for app in "$@"; do
    label="$(app_label "$app")"
    app_path="$(artifact_app_path_for "$app" "$label")"
    info_plist="$app_path/Contents/Info.plist"
    bundle_identifier="$(bundle_identifier_for "$info_plist")"
    bundle_version="$(bundle_version_for "$info_plist")"
    bundle_executable="$(bundle_executable_for "$info_plist")"
    dmg_path="$(artifact_dmg_path_for "$app" "$label")"
    app_size_bytes="$(du -sk "$app_path" | awk '{print $1 * 1024}')"
    dmg_size_bytes="$(stat -f '%z' "$dmg_path")"

    echo "### $label"
    echo
    echo "- Bundle identifier: \`$bundle_identifier\`"
    echo "- Bundle version: \`$bundle_version\`"
    echo "- Bundle executable: \`$bundle_executable\`"
    echo "- App bundle: \`$app_path\`"
    echo "- App bundle size (bytes): \`$app_size_bytes\`"
    echo "- DMG: \`$dmg_path\`"
    echo "- DMG size (bytes): \`$dmg_size_bytes\`"
    echo
  done

  echo "## Operator Follow-Up"
  echo
  if [[ "$MODE" == "rehearsal" ]]; then
    echo "- Use this evidence to confirm packaged bundle shape before promoting a lane to release-candidate testing."
  else
    echo "- Confirm whether the run was local-only or CI-backed."
    echo "- For notarized runs, record the successful notarization signal from the Tauri build logs."
    echo "- If this candidate is withdrawn, cross-link the rollback record in \`docs/release/rollback-playbook.md\`."
  fi
} > "$md_path"

echo "[bundle-evidence] markdown=$md_path"
echo "[bundle-evidence] json=$json_path"

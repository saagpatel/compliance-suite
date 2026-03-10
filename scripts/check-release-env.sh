#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-}"

if [[ -z "$MODE" ]]; then
  echo "usage: $0 <signed|notarized>" >&2
  exit 1
fi

case "$MODE" in
  signed|notarized)
    ;;
  *)
    echo "unsupported release mode: $MODE" >&2
    exit 1
    ;;
esac

missing=()

require_env() {
  local name="$1"
  if [[ -z "${!name:-}" ]]; then
    missing+=("$name")
  fi
}

has_local_signing_identity() {
  if ! command -v security >/dev/null 2>&1; then
    return 1
  fi

  security find-identity -v -p codesigning 2>/dev/null \
    | grep -Fq "${APPLE_SIGNING_IDENTITY:-}"
}

require_env APPLE_SIGNING_IDENTITY

if [[ ${#missing[@]} -gt 0 ]]; then
  printf 'missing required signing variables: %s\n' "${missing[*]}" >&2
  exit 1
fi

if [[ -n "${APPLE_CERTIFICATE:-}" && -n "${APPLE_CERTIFICATE_PASSWORD:-}" ]]; then
  signing_ready=true
elif has_local_signing_identity; then
  signing_ready=true
else
  signing_ready=false
fi

if [[ "$signing_ready" != true ]]; then
  cat >&2 <<'EOF'
missing signing material:
- Provide APPLE_CERTIFICATE and APPLE_CERTIFICATE_PASSWORD for CI or clean-room signing
- or install the signing identity in the local keychain so `security find-identity` can resolve it
EOF
  exit 1
fi

if [[ "$MODE" == "notarized" ]]; then
  if [[ -n "${APPLE_ID:-}" || -n "${APPLE_PASSWORD:-}" || -n "${APPLE_TEAM_ID:-}" ]]; then
    require_env APPLE_ID
    require_env APPLE_PASSWORD
    require_env APPLE_TEAM_ID
  elif [[ -n "${APPLE_API_KEY:-}" || -n "${APPLE_API_ISSUER:-}" || -n "${APPLE_API_KEY_PATH:-}" || -n "${APPLE_API_KEY_P8_BASE64:-}" ]]; then
    require_env APPLE_API_KEY
    require_env APPLE_API_ISSUER
    if [[ -z "${APPLE_API_KEY_PATH:-}" && -z "${APPLE_API_KEY_P8_BASE64:-}" ]]; then
      missing+=("APPLE_API_KEY_PATH or APPLE_API_KEY_P8_BASE64")
    fi
  else
    missing+=("notarization credentials")
  fi
fi

if [[ ${#missing[@]} -gt 0 ]]; then
  printf 'missing required %s variables: %s\n' "$MODE" "${missing[*]}" >&2
  exit 1
fi

echo "[release-env] $MODE requirements satisfied"

#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMMANDS_FILE="${1:-$REPO_ROOT/.codex/verify.commands}"

if [[ ! -f "$COMMANDS_FILE" ]]; then
  echo "Missing $COMMANDS_FILE"
  exit 2
fi

# shellcheck disable=SC1091
source "$REPO_ROOT/.codex/actions/_artifact_env.sh"

cd "$REPO_ROOT"

step=0

while IFS= read -r cmd || [[ -n "$cmd" ]]; do
  [[ -z "${cmd//[[:space:]]/}" ]] && continue
  [[ "$cmd" =~ ^[[:space:]]*# ]] && continue

  step=$((step + 1))
  echo "[verify:$step] $cmd"
  bash -lc "$cmd"
done < "$COMMANDS_FILE"

echo "[verify] all commands passed"

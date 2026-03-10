#!/usr/bin/env bash
set -euo pipefail

echo "=== Running Rust linting ==="
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo ""
echo "=== Running TypeScript linting ==="
pnpm --dir apps/questionnaire run lint
pnpm --dir apps/binder run lint
pnpm --dir apps/sop run lint

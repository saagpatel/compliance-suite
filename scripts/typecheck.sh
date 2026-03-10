#!/usr/bin/env bash
set -euo pipefail

# Keep Rust compilation healthy and typecheck the active UI lanes.
cargo check --workspace
pnpm --dir apps/questionnaire exec tsc --noEmit
pnpm --dir apps/binder exec tsc --noEmit
pnpm --dir apps/sop exec tsc --noEmit

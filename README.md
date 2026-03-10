# Compliance Ops Suite

Local-first compliance suite monorepo.

## Current Status

- Active production lane: `questionnaire`
- Future lanes: `binder`, `sop`
- Current production target: macOS-first desktop build
- Canonical verification command: `pnpm verify`
- Manual package rehearsal commands:
  - `pnpm package:questionnaire`
  - `pnpm package:binder`
  - `pnpm package:sop`
  - these commands now emit bundle-evidence files in the active Codex run log directory
- Manual signed release-candidate command:
  - `pnpm release:signed:questionnaire`
- Manual notarized release-candidate command:
  - `pnpm release:notarized:questionnaire`
- Release preflight commands:
  - `pnpm release:check:signed`
  - `pnpm release:check:notarized`
- Release templates and checklists live in `docs/release/`

The repo now treats `.codex/verify.commands` as the source of truth for local
quality gates, with `.codex/scripts/run_verify_commands.sh` as the deterministic
runner used by both humans and CI.

## Quickstart
1. Install dependencies:
   `pnpm install`
2. Start the desktop app (normal dev):
   `pnpm dev`
3. Start the desktop app in low-disk lean mode:
   `pnpm dev:lean`
4. Run verification:
   `pnpm verify`
5. Rehearse packaged builds when needed:
   `pnpm package:questionnaire`
   `pnpm package:binder`
   `pnpm package:sop`
6. Build signed or notarized release candidates only when Apple credentials are ready:
   `pnpm release:signed:questionnaire`
   `pnpm release:notarized:questionnaire`
7. Check release credential readiness before a real release-candidate run:
   `pnpm release:check:signed`
   `pnpm release:check:notarized`

## Execution Foundation

- Project Codex defaults: `.codex/config.toml`
- Worktree bootstrap: `.codex/actions/setup-worktree.sh`
- Local-environment wiring: `.codex/local-environment/README.md`
- Verification contract: `.codex/verify.commands`
- Execution control plane: `docs/execution/control-plane.md`
- Batch handoff template: `docs/execution/batch-handoff-template.md`

For a fresh worktree or desktop-side Codex run, use the shared setup script
before running dev or verification commands.

## Dev Modes
- Normal dev (`pnpm dev`):
  - Fastest repeated startups.
  - Uses persistent local build outputs (`target/`, `apps/questionnaire/node_modules/.vite`).
- Lean dev (`pnpm dev:lean`):
  - Uses temporary cache locations for Rust and Vite build outputs.
  - Automatically removes heavy build artifacts when the app exits.
  - Keeps dependency installs (`node_modules`, global Cargo/pnpm caches) so restarts stay reasonable.

## Cleanup Commands
- Heavy artifacts only (safe daily cleanup):
  - `pnpm clean:heavy`
  - Removes build outputs such as `target/`, `apps/questionnaire/dist`, and Vite/Tauri generated artifacts.
- Full local reproducible cleanup:
  - `pnpm clean:local`
  - Includes heavy artifacts plus local dependency installs (`node_modules`) and local pnpm store (`.pnpm-store`), all of which can be recreated.

## Structure
- `core/`: Rust domain + storage + audit chain + deterministic export packs
- `apps/*`: desktop apps (Tauri + React) that call into `core/`
- `packages/*`: shared TypeScript packages (DTOs, UI)
- `docs/`: source-of-truth docs for execution, architecture, and release state
- `.codex/`: repo-scoped execution bootstrap, rules, and verification helpers

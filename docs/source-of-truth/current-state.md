# Current State

Last updated: 2026-03-10

## Product Shape

This monorepo is a local-first desktop compliance suite built around a shared
Rust core and Tauri desktop shells.

Current app reality:

- `questionnaire` is the most complete product lane and the current release candidate.
- `binder` now has a usable single-screen workflow for vault access, evidence
  import, control tracking, evidence linkage, reporting-period summaries,
  completion guidance, a distinct reviewing stage, and gated export-pack
  generation, plus successful manual macOS package rehearsal, but it is still
  earlier than `questionnaire`.
- `sop` now has a usable first workflow for vault access, SOP drafting,
  versioning, approvals, publication, acknowledgments, and gated export-pack
  generation, plus successful manual macOS package rehearsal, but it is still
  earlier than `questionnaire`.

## What Works Today

- Root install, lint, typecheck, local tests, and desktop build commands exist.
- The Rust core already includes vault storage, audit-chain logic, questionnaire
  import, answer-bank operations, licensing, export-pack generation, and a
  Binder control/evidence domain slice now exposed through a real UI.
- The Rust core now also includes an initial SOP document lifecycle slice with
  versioning, approvals, acknowledgments, and publish-state persistence.
- The SOP app now includes a working frontend for creating procedures, saving
  revisions, publishing the latest version, checking local license readiness,
  generating export packs, and tracking approvals plus acknowledgments from a
  local vault.
- Desktop audit events now use a real local-session actor identity derived from
  the running machine or `CODEX_ACTOR`, instead of a hard-coded placeholder actor.
- The release system now distinguishes between package rehearsal and guarded
  signed or notarized release-candidate packaging, with rollback and CI policy
  docs in-repo, plus bundle-evidence generation for rehearsal and
  credentialed runs.
- The questionnaire app has a visible five-step UI covering import, map, answer
  bank, review, and export.
- The questionnaire app now includes evidence browsing and evidence import from
  the UI, plus row-by-row review progress controls.

## What Is Still Incomplete

- Binder still lacks deeper review mechanics, stronger release hardening, and
  always-on packaged-app coverage needed to match the questionnaire lane.
- SOP still lacks broader hardening and always-on packaged-app coverage needed
  to match the questionnaire lane.
- The suite still needs real Apple credentials plus validated signing/notarization
  execution in CI.
- Cross-platform support beyond macOS remains future work.

## Supported Platform Target

- Current production target: macOS-first desktop distribution.
- Linux and Windows support should be treated as future hardening work until the
  shell-dependent runtime paths are removed and desktop packaging is validated.

## Definition of Done for the Current Milestone

The current milestone is complete when:

1. The repo bootstraps cleanly in a fresh worktree.
2. The canonical verification path is truthful and repeatable.
3. The questionnaire app works from cold start without hidden prerequisites.
4. The shared Rust core no longer depends on `sqlite3` or `shasum` binaries.
5. Binder and SOP each have at least one complete end-to-end product workflow.

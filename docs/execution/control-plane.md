# Execution Control Plane

Last updated: 2026-03-10

## Objective

Define how this repo moves from planning to implementation to release without
hidden setup, placeholder gates, or drifting definitions of done.

## Current Finish-Line Rule

The active production lane is `questionnaire`. `binder` and `sop` remain in the
repo, but they are not yet held to production-ready gates.

## Required Execution Lanes

- Planner lane: freezes scope, acceptance criteria, and the next batch.
- Worker lane: implements code and keeps changes atomic by concern.
- QA lane: runs verification, restores trust in tests, and blocks false green.
- Release lane: owns packaged-build rehearsal, release evidence, and rollback posture.
- Docs/comms lane: keeps repo docs aligned with shipped behavior.

## Batch Progression Rule

A batch is complete only when:

1. The planned artifact exists in the repo.
2. The canonical verification path is run, or any skipped checks are called out.
3. Remaining risks are explicit.
4. The next batch is defined concretely enough to execute without rediscovery.

## Branch and Worktree Contract

- Work on a non-default branch that matches `codex/<type>/<slug>`.
- Treat worktrees as isolated execution environments.
- Do not check out the same branch in more than one worktree at a time.
- Use the shared local-environment setup script for new worktrees.

## Canonical Verification Contract

- Source of truth: `.codex/verify.commands`
- Deterministic runner: `.codex/scripts/run_verify_commands.sh`
- Package shortcut: `pnpm verify`

## Safety Posture

- Prefer stable Codex features over experimental orchestration on the critical path.
- Prefer non-destructive repo scripts over ad hoc cleanup commands.
- No phase or batch advances on misleading green signals.

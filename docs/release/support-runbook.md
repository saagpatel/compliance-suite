# Support Runbook

Last updated: 2026-03-10

## First Response Checklist

1. Confirm which app is affected: `questionnaire`, `binder`, or `sop`.
2. Confirm whether the issue happens in dev, verification, or packaged build.
3. Capture the exact vault path used for reproduction.
4. Capture whether a license is installed and whether export access is expected.

## Minimum Evidence to Gather

- App name and app version under test
- macOS version
- Whether the issue happens in a packaged build or only in local dev
- The exact user action being attempted
- The visible error message, if any
- The export pack path, if the issue is export-related
- The bundle-evidence artifact, if the issue is package- or release-related

## App-Specific Evidence

- `questionnaire`
  - import file path and format
  - import ID if known
  - whether the issue occurs during map, review, or export
- `binder`
  - reporting period involved
  - whether evidence was already imported
  - current control status when the issue occurs
- `sop`
  - document slug or title
  - current lifecycle state: draft, in review, approved, or published
  - whether the issue is in approval, publication, acknowledgment, or export

## Useful Validation Commands

- `pnpm verify`
- `pnpm package:questionnaire`
- `pnpm package:binder`
- `pnpm package:sop`
- `pnpm release:signed:questionnaire`
- `pnpm release:notarized:questionnaire`
- `pnpm release:check:signed`
- `pnpm release:check:notarized`

## Recovery Guidance

- If the problem is local-state or vault-specific, try reproducing in a fresh
  vault before assuming a suite-wide regression.
- If export is blocked, confirm license status before debugging export logic.
- If a packaged build fails while verification is green, treat it as a release
  issue rather than an application logic issue first.
- Use `docs/release/rollback-playbook.md` if a release candidate needs to be
  withdrawn.
- Use `docs/release/first-credentialed-run-checklist.md` before the first live
  Apple-credentialed run for a lane.
- Use `docs/release/manual-smoke-checklist.md` when checking packaged app behavior.

## Escalation Triggers

- audit or vault corruption indicators
- repeatable packaged-build failures on the current mainline
- licensing validation failures that contradict expected local policy
- migration failures on an existing vault

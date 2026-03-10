# Manual Smoke Checklist

Last updated: 2026-03-10

## Purpose

Use this checklist after a package rehearsal or release-candidate build to make
sure the produced desktop app is basically usable before the run is treated as
good.

## Shared Checks For Any App

1. Open the packaged `.app` successfully from Finder.
2. Confirm the window title matches the intended app lane.
3. Confirm the app opens without an immediate crash dialog.
4. Confirm a new vault can be created or an existing vault can be opened.
5. Confirm the app can be closed and reopened without losing the basic vault path.

## Questionnaire Smoke

1. Open the packaged Questionnaire app.
2. Create or open a vault.
3. Confirm Import, Map, Review, and Export screens render.
4. Confirm import guidance is visible even before a file is selected.
5. Confirm export state explains why export is blocked if no valid license is installed.

## Binder Smoke

1. Open the packaged Binder app.
2. Create or open a vault.
3. Confirm evidence import controls render.
4. Confirm the control creation form renders.
5. Confirm the export area renders and explains license state clearly.

## SOP Smoke

1. Open the packaged SOP app.
2. Create or open a vault.
3. Confirm document drafting and version history surfaces render.
4. Confirm approval and acknowledgment areas render.
5. Confirm the export area renders and explains license state clearly.

## Pass Rule

Treat the package as smoke-passed only if:

- the app launches cleanly
- the main workflow surface renders
- there is no immediate crash or broken navigation
- the lane-specific export state is understandable

## If Smoke Fails

- Keep the generated bundle-evidence artifact with the failure notes
- Use `docs/release/support-runbook.md` for investigation
- Use `docs/release/rollback-playbook.md` if a release candidate must be withdrawn

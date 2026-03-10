# Rollback Playbook

Last updated: 2026-03-10

## When To Use This

Use this playbook when a packaged candidate should no longer be treated as safe
to test or distribute.

Common triggers:

- repeatable packaged-build failure after a previously green candidate
- broken import, review, export, approval, or acknowledgment flow in a packaged app
- signing or notarization output that does not match the expected target
- migration, vault, or audit behavior that raises trust concerns

## Immediate Actions

1. Stop sharing the affected candidate build.
2. Record the app, version, commit, and package path involved.
3. Mark the candidate as withdrawn in the release notes or operator log.
4. Decide whether the issue is:
   - release-only
   - app logic
   - vault or migration safety

## If The Problem Is Release-Only

Use this path when verification is green but the packaged build is bad.

1. Keep the current mainline commit untouched until the failure is understood.
2. Re-run the matching `pnpm package:<app>` command to confirm the issue is reproducible.
3. Re-run the matching signed or notarized release-candidate command only after
   confirming credentials and environment assumptions.
4. If the bad candidate was uploaded anywhere, replace or remove it before
   sharing a new one.

## If The Problem Is Application Logic

Use this path when both verification and packaged behavior point to the same
product defect.

1. Open a fix branch from the current mainline.
2. Ship the smallest corrective patch that removes the regression.
3. Re-run `pnpm verify`.
4. Re-run package rehearsal for the affected app.
5. Re-run signed or notarized release-candidate packaging only after the fix is green.

## If The Problem Touches Vault, Audit, Or Migration Trust

Use this path when data safety is in doubt.

1. Treat the candidate as blocked immediately.
2. Preserve a failing vault sample if one exists.
3. Stop distribution until the failure is root-caused and fixed.
4. Require a fresh `pnpm verify` plus targeted reproduction before any new candidate is shared.

## Exit Criteria

Rollback work is complete when:

- the withdrawn candidate is no longer the active test or distribution target
- the replacement candidate is identified clearly
- verification and the relevant package or release-candidate path are green again
- the operator log explains what was withdrawn and why

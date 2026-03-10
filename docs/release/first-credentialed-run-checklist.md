# First Credentialed Run Checklist

Last updated: 2026-03-10

## Purpose

Use this checklist for the first real Apple-credentialed release-candidate run,
especially for `questionnaire`.

## Before You Start

1. Confirm the target commit has a green `pnpm verify`.
2. Confirm package rehearsal is already green for the same app target.
3. Confirm which release mode you are attempting:
   - `signed`
   - `notarized`
4. Confirm the app target:
   - `questionnaire` first
   - `binder` only after questionnaire release-candidate runs are stable
   - `sop` only after questionnaire release-candidate runs are stable

## Secret Readiness

1. Confirm `APPLE_SIGNING_IDENTITY` is available.
2. Confirm one signing path is ready:
   - installed local keychain identity
   - or `APPLE_CERTIFICATE` plus `APPLE_CERTIFICATE_PASSWORD`
3. For notarized runs, confirm one notarization path is ready:
   - `APPLE_API_KEY`, `APPLE_API_ISSUER`, and `APPLE_API_KEY_PATH`
   - or `APPLE_API_KEY`, `APPLE_API_ISSUER`, and `APPLE_API_KEY_P8_BASE64`
   - or `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID`

## Recommended Dry Run Order

1. Local preflight only:
   - `pnpm release:check:signed`
   - or `pnpm release:check:notarized`
2. Manual GitHub workflow:
   - `.github/workflows/release-candidate.yml`
3. Target the `questionnaire` lane first.
4. Record the generated release evidence artifact.
5. Prepare an operator note using `docs/release/operator-log-template.md`.

## Evidence To Keep

- workflow run link
- release mode used
- app target used
- generated `bundle-evidence.md`
- generated `bundle-evidence.json`
- `.app` and `.dmg` artifact names
- note confirming whether notarization completed successfully
- operator log

## Promotion Signal

Treat the first credentialed run as successful only if:

- preflight passes cleanly
- packaging completes cleanly
- release evidence artifacts are present
- the produced bundle opens as expected for a smoke check
- there is no operator-only repair needed after the run starts

## If The Run Fails

- Use `docs/release/rollback-playbook.md`
- Keep the failure notes with the same run record
- Do not promote release-candidate packaging into required CI based on a partial run

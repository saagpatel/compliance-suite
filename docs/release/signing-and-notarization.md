# Signing and Notarization

Last updated: 2026-03-10

## Purpose

Separate everyday package rehearsal from real release-candidate packaging.

- Package rehearsal proves that the app can bundle.
- Signed release-candidate packaging proves that the bundle can be distributed
  with the expected Apple signing material.
- Notarized release-candidate packaging adds the Apple notarization step needed
  for macOS distribution confidence.

## Current Status

- Repo support exists for signed and notarized release-candidate runs.
- The release path is still manual until Apple credentials are installed in the
  target environment and validated.
- The packaging workflows do not store credentials in the repo.

## Local Entry Points

- `pnpm release:signed:questionnaire`
- `pnpm release:signed:binder`
- `pnpm release:signed:sop`
- `pnpm release:notarized:questionnaire`
- `pnpm release:notarized:binder`
- `pnpm release:notarized:sop`

Use `scripts/release-candidate.sh` directly if you need multiple app targets in
one run.

## CI Entry Point

- Manual workflow: `.github/workflows/release-candidate.yml`

This workflow is intentionally manual because signed and notarized packaging is
still an operator action, not a default branch gate.

## Preflight Entry Points

- `pnpm release:check:signed`
- `pnpm release:check:notarized`

Use these before a real credentialed run if you want to validate that the
expected environment variables are present without starting a package build.

## Required Signing Material

All signed or notarized release-candidate runs require:

- `APPLE_SIGNING_IDENTITY`
- either:
  - `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD`
  - or a matching signing identity already installed in the local macOS keychain

## Required Notarization Material

Notarized release-candidate runs require the signing material above plus one of
the following notarization credential sets:

### App Store Connect API key path

- `APPLE_API_KEY`
- `APPLE_API_ISSUER`
- `APPLE_API_KEY_PATH`

### App Store Connect API key passed as base64

- `APPLE_API_KEY`
- `APPLE_API_ISSUER`
- `APPLE_API_KEY_P8_BASE64`

The release script will decode the base64 key into a temporary file and clean it
up after the run.

### Apple ID flow

- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

## Operator Flow

1. Confirm `pnpm verify` is green on the exact commit under release review.
2. Run `pnpm package:<app>` first if you only need bundle confidence.
3. Run `pnpm release:signed:<app>` when you need a signed candidate.
4. Run `pnpm release:notarized:<app>` when you need the macOS distribution
   candidate.
5. Keep the generated release evidence report from
   `bundle-evidence/<mode>/bundle-evidence.md` and
   `bundle-evidence/<mode>/bundle-evidence.json`.
6. Record the produced `.app` and `.dmg` paths plus the credential mode used in
   the release notes or run log.
7. Run the relevant checks from `docs/release/manual-smoke-checklist.md`.

## Success Evidence

- Signed candidate:
  - packaged `.app`
  - packaged `.dmg`
  - no credential-validation failure from `scripts/check-release-env.sh`
  - generated release evidence files
- Notarized candidate:
  - signed candidate evidence above
  - successful notarization-capable credential validation
  - release operator confirmation that notarization completed in the Tauri build output

## Current Boundaries

- Real notarization success still depends on valid Apple credentials in the
  active environment.
- This repo prepares the path and validates the required variables, but it
  cannot prove Apple account setup without operator-provided secrets.
- Use `docs/release/operator-log-template.md` and
  `docs/release/release-notes-template.md` when recording the run.

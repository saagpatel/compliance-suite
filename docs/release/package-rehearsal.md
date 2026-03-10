# Package Rehearsal

Last updated: 2026-03-10

## Purpose

Run intentional packaged-build rehearsals for each desktop lane without making
the default verification path dramatically heavier.

## Local Entry Points

- `pnpm package:questionnaire`
- `pnpm package:binder`
- `pnpm package:sop`
- `pnpm package:all`

These commands call `scripts/package-app.sh` and run `tauri build` for the
selected app or apps. They also emit `bundle-evidence.md` and
`bundle-evidence.json` in the active Codex run log directory.

## Codex Action Shortcuts

- `.codex/actions/package-questionnaire.sh`
- `.codex/actions/package-binder.sh`
- `.codex/actions/package-sop.sh`
- `.codex/actions/package-suite.sh`

## CI Rehearsal

- Manual workflow: `.github/workflows/package-rehearsal.yml`
- Use it when you want a macOS packaging confidence check without turning every
  pull request into a full three-app package build.
- The workflow now uploads packaged artifacts plus rehearsal bundle-evidence artifacts.
- Signed and notarized candidates use `.github/workflows/release-candidate.yml`
  instead of this lighter rehearsal path.
- Environment-only release preflight is available through
  `pnpm release:check:signed` and `pnpm release:check:notarized`.

## Expected Outcome

- `questionnaire` should produce a macOS app bundle and DMG.
- `binder` should produce a packaged desktop bundle for release rehearsal.
- `sop` should produce a packaged desktop bundle for release rehearsal.
- rehearsal runs should also produce:
  - `bundle-evidence.md`
  - `bundle-evidence.json`

## Current Boundaries

- Package rehearsal is not the same as production release.
- Signing, notarization, stapling, and final distribution validation are still
  separate release tasks.
- Binder and SOP rehearsals are confidence checks, not yet a claim of full
  production release readiness.
- Package rehearsal stays manual by policy until the criteria in
  `docs/release/ci-policy.md` are met.

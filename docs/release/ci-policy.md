# Release CI Policy

Last updated: 2026-03-10

## Default Policy

- `pnpm verify` remains the canonical day-to-day quality gate.
- `.github/workflows/quality-gates.yml` is the truth-aligned CI path for normal development.
- `.github/workflows/package-rehearsal.yml` remains manual.
- `.github/workflows/release-candidate.yml` remains manual.

## Why Package Jobs Stay Manual For Now

Packaging is much heavier than the normal quality path, and signed or notarized
packaging also depends on Apple credentials that should not be treated like
ordinary branch-level CI inputs.

For this repo today, manual release packaging is the safer default because:

- `questionnaire` is still the main release candidate lane
- Binder and SOP have working package rehearsal but are not general-release lanes
- signing and notarization depend on environment secrets that are not yet proven
  in every CI context

## Promotion Rule For Enforced Packaging

Do not promote package or release-candidate workflows into required CI checks
until all of the following are true:

1. Apple signing material is configured and validated in CI.
2. Notarization credentials are configured and validated in CI.
3. The manual release-candidate workflow has passed three consecutive runs for
   the target lane without operator-only fixes between runs.
4. Runtime is acceptable for the branch protection level being considered.
5. Release evidence artifacts have been retained for those successful runs.
6. The release target for that lane has been explicitly promoted beyond package rehearsal.

## Recommended Promotion Order

1. Keep `quality-gates.yml` as the only always-on quality gate.
2. Use `package-rehearsal.yml` manually for bundle confidence on demand.
3. Use `release-candidate.yml` manually for signed or notarized candidates.
4. After the promotion rule is satisfied, consider making package rehearsal a
   required check only on `main` or `release/*`.
5. Only consider enforced signed or notarized packaging after credentials and
   runtime are stable enough for predictable CI use.

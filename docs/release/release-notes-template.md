# Release Notes Template

Last updated: 2026-03-10

Use this template when preparing a release candidate or a manually shared
packaged build.

```md
# Release Notes

## Release Summary
- App:
- Version:
- Commit:
- Release mode: rehearsal | signed | notarized

## What Changed
- <user-visible improvement>
- <user-visible improvement>

## Validation
- `pnpm verify` -> pass | fail
- Package command -> pass | fail
- Manual smoke checklist -> pass | fail

## Artifacts
- App bundle:
- DMG:
- Bundle evidence:

## Known Issues
- <issue or none>

## Rollback Plan
- See `docs/release/rollback-playbook.md`
```

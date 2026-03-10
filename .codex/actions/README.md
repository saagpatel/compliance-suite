# Codex Actions

These scripts are the repo-scoped execution helpers for local work, worktrees,
and later automation runs.

- `setup-worktree.sh` - shared dependency/bootstrap step for fresh worktrees
- `run-verify.sh` - canonical repo verification entrypoint
- `dev-questionnaire.sh` - questionnaire desktop dev loop
- `package-questionnaire.sh` - local packaged-build rehearsal
- `package-binder.sh` - Binder packaged-build rehearsal
- `package-sop.sh` - SOP packaged-build rehearsal
- `package-suite.sh` - run all packaged-build rehearsals in sequence
- `release-check-signed.sh` - validate signed release credentials without packaging
- `release-check-notarized.sh` - validate notarization credentials without packaging
- `release-candidate.sh` - guarded signed/notarized release-candidate packaging

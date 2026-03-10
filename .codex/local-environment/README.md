# Local Environment Scripts

Use these scripts when configuring Codex app local environments for this repo.

Setup script:
- `.codex/local-environment/setup.sh`

Suggested actions:
- `Run Questionnaire` -> `.codex/local-environment/action_run.sh`
- `Verify` -> `.codex/local-environment/action_verify.sh`
- `Package Questionnaire` -> `.codex/local-environment/action_package.sh`
- `Package Binder` -> `.codex/local-environment/action_package_binder.sh`
- `Package SOP` -> `.codex/local-environment/action_package_sop.sh`
- `Release Check (Signed)` -> `.codex/local-environment/action_release_check_signed.sh`
- `Release Check (Notarized)` -> `.codex/local-environment/action_release_check_notarized.sh`

Execution expectations:
- Run setup on worktree creation.
- Keep worktree threads and future automations on the same setup script.
- Treat package generation as an intentional local rehearsal, not a background default.
- Keep Binder and SOP package rehearsal manual until signing/notarization posture is finalized.
- Treat signed or notarized release-candidate builds as operator-only runs that
  require explicit Apple credential setup.

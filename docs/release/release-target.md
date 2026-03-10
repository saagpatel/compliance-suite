# Release Target

Last updated: 2026-03-10

## Near-Term Release Target

- Product target: `questionnaire`
- Platform target: macOS-first desktop build
- Release confidence target: canonical local verification, packaged-build rehearsal,
  and a manual signed/notarized release-candidate path
- Package rehearsal entry points: `pnpm package:questionnaire`, `pnpm package:binder`, `pnpm package:sop`
- Release preflight entry points: `pnpm release:check:signed`, `pnpm release:check:notarized`
- Release-candidate workflow: `.github/workflows/release-candidate.yml`

## Not Yet Release-Ready

- `binder` as a generally released product lane
- `sop` as a generally released product lane
- performance-enforced CI gates
- validated Apple credentials for real signed/notarized CI execution

These remain intentional follow-on work and should not be represented as fully
shipped today.

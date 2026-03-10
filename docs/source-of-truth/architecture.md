# Architecture Overview

Last updated: 2026-03-10

## Layers

- `core/`
  - Rust domain logic for vaults, audit, questionnaire processing, answer bank,
    licensing, Binder control tracking, SOP document lifecycle, export, and
    supporting utilities.
- `apps/questionnaire`
  - Tauri desktop shell plus React UI for the active product lane.
- `apps/binder`
  - Emerging desktop lane for control-to-evidence binder workflows. The current
    implementation now includes a working single-screen React flow on top of the
    Binder Tauri shell.
- `apps/sop`
  - Emerging desktop lane for SOP authoring, approval, publication, and
    acknowledgment workflows. The current implementation includes a working
    single-screen React flow on top of the SOP Tauri shell.
- `packages/types`
  - Shared TypeScript DTOs and error contracts used by the UI layer.

## Active Service Boundaries

- Tauri command handlers call into `core`.
- `core` owns vault storage, audit validation, import parsing, and export assembly.
- React UI owns navigation, user interaction, and progress messaging.
- Each desktop lane now resolves a local actor identity for audit events from
  the local environment instead of writing a fixed placeholder actor.
- Release operations now have two intentional lanes: package rehearsal for bundle
  confidence and guarded release-candidate packaging for signed/notarized macOS
  distribution work, with structured bundle-evidence emitted after rehearsal
  and credentialed runs.
- Binder now has a working vertical slice for vault access, evidence import,
  control creation, evidence linkage, reporting-period status tracking, export
  gating, completion-focused control filtering, a distinct reviewing stage, and
  successful manual package rehearsal.
- SOP now has a working vertical slice for document creation, versioning,
  approval actions, publication, acknowledgment tracking, vault-backed
  persistence, license-aware export generation, successful manual package
  rehearsal, and a first frontend workflow for the full governed lifecycle.

## Known Architecture Risks

- Binder still needs richer review mechanics and more release-grade coverage to
  match the questionnaire lane.
- SOP still needs broader release-grade hardening and support tooling, which
  keeps the suite architecture uneven.
- Shared TypeScript DTOs need ongoing parity checks against live Tauri responses.

## Immediate Architecture Priorities

1. Extend Binder beyond the new usable frontend slice into richer control
   review and release-grade coverage.
2. Extend SOP beyond the new governed workflow into release-grade coverage and
   support tooling.
3. Keep expanding test coverage so the active architecture stays protected.

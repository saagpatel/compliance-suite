# Compliance Ops Suite

[![Rust](https://img.shields.io/badge/Rust-dea584?style=flat-square&logo=rust)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> Compliance workflows that stay on your machine — questionnaires, binders, and SOPs without the SaaS overhead.

A local-first compliance suite monorepo for macOS. The active production lane is `questionnaire`, with `binder` and `sop` lanes in development. A Rust domain core handles storage, audit chain, and deterministic export packs; Tauri + React desktop apps sit on top for each workflow.

## Features

- **Questionnaire app** — structured compliance questionnaire workflow with guided response capture
- **Audit chain** — tamper-evident local audit log for all compliance actions
- **Deterministic export packs** — reproducible bundle artifacts for review and sign-off
- **Signed release pipeline** — macOS code signing and notarization integrated into the build
- **Verification contract** — `.codex/verify.commands` as source of truth for quality gates

## Quick Start

### Prerequisites
- Node.js 18+, pnpm
- Rust stable toolchain
- macOS (primary target)

### Installation
```bash
pnpm install
```

### Usage
```bash
# Start the questionnaire app (dev)
pnpm dev

# Run all verification gates
pnpm verify

# Rehearse a packaged build
pnpm package:questionnaire
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Language | Rust + TypeScript |
| Desktop runtime | Tauri 2 |
| Frontend | React + Vite + Tailwind CSS |
| Core domain | Rust (audit chain, storage, export) |
| Storage | SQLite |
| Build | pnpm workspaces + Cargo workspace |

## License

MIT

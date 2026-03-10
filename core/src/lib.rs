//! Compliance Suite core platform.
//!
//! Phase 1 provides: SQLite-backed storage, evidence filesystem, tamper-evident
//! audit log hash chain, and deterministic export packs.

pub mod prelude;

pub mod answer_bank;
pub mod audit;
pub mod binder;
pub mod domain;
pub mod export;
pub mod questionnaire;
pub mod sop;
pub mod storage;
pub mod util;

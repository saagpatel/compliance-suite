-- 0009_binder_controls.sql

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS binder_control (
  control_id TEXT PRIMARY KEY,
  vault_id TEXT NOT NULL,
  framework TEXT NOT NULL,
  control_code TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT NULL,
  reporting_period TEXT NOT NULL,
  status TEXT NOT NULL,
  owner TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(vault_id) REFERENCES vault(vault_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_binder_control_unique
ON binder_control(vault_id, framework, control_code, reporting_period);

CREATE INDEX IF NOT EXISTS idx_binder_control_period
ON binder_control(reporting_period, status);

CREATE TABLE IF NOT EXISTS binder_control_evidence (
  control_id TEXT NOT NULL,
  evidence_id TEXT NOT NULL,
  linked_at TEXT NOT NULL,
  PRIMARY KEY(control_id, evidence_id),
  FOREIGN KEY(control_id) REFERENCES binder_control(control_id) ON DELETE CASCADE,
  FOREIGN KEY(evidence_id) REFERENCES evidence_item(evidence_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_binder_control_evidence_control
ON binder_control_evidence(control_id, linked_at ASC);

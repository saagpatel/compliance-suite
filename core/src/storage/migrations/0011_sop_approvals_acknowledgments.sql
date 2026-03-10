-- 0011_sop_approvals_acknowledgments.sql

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sop_approval_request (
  request_id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  version_id TEXT NOT NULL,
  requested_by TEXT NOT NULL,
  status TEXT NOT NULL,
  requested_at TEXT NOT NULL,
  FOREIGN KEY(document_id) REFERENCES sop_document(document_id) ON DELETE CASCADE,
  FOREIGN KEY(version_id) REFERENCES sop_version(version_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sop_approval_request_document
ON sop_approval_request(document_id, requested_at DESC);

CREATE TABLE IF NOT EXISTS sop_approval_step (
  step_id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL,
  approver TEXT NOT NULL,
  status TEXT NOT NULL,
  decided_at TEXT NULL,
  notes TEXT NULL,
  requested_at TEXT NOT NULL,
  FOREIGN KEY(request_id) REFERENCES sop_approval_request(request_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sop_approval_step_request
ON sop_approval_step(request_id, approver ASC);

CREATE TABLE IF NOT EXISTS sop_acknowledgment (
  acknowledgment_id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  version_id TEXT NOT NULL,
  recipient TEXT NOT NULL,
  status TEXT NOT NULL,
  acknowledged_at TEXT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(document_id) REFERENCES sop_document(document_id) ON DELETE CASCADE,
  FOREIGN KEY(version_id) REFERENCES sop_version(version_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sop_acknowledgment_unique
ON sop_acknowledgment(document_id, version_id, recipient);

CREATE INDEX IF NOT EXISTS idx_sop_acknowledgment_document
ON sop_acknowledgment(document_id, created_at DESC);

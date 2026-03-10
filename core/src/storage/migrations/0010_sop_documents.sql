-- 0010_sop_documents.sql

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS sop_document (
  document_id TEXT PRIMARY KEY,
  vault_id TEXT NOT NULL,
  title TEXT NOT NULL,
  slug TEXT NOT NULL,
  owner TEXT NOT NULL,
  status TEXT NOT NULL,
  published_version_id TEXT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(vault_id) REFERENCES vault(vault_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sop_document_slug
ON sop_document(vault_id, slug);

CREATE INDEX IF NOT EXISTS idx_sop_document_status
ON sop_document(status, updated_at DESC);

CREATE TABLE IF NOT EXISTS sop_version (
  version_id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  version_number INTEGER NOT NULL,
  body_markdown TEXT NOT NULL,
  change_summary TEXT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(document_id) REFERENCES sop_document(document_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sop_version_unique
ON sop_version(document_id, version_number);

CREATE INDEX IF NOT EXISTS idx_sop_version_document
ON sop_version(document_id, version_number DESC);

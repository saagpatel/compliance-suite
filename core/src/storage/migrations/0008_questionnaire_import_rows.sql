-- 0008_questionnaire_import_rows.sql

PRAGMA foreign_keys = ON;

ALTER TABLE questionnaire_review ADD COLUMN source_row_ordinal INTEGER NULL;

CREATE TABLE IF NOT EXISTS questionnaire_import_row (
  import_id TEXT NOT NULL,
  row_ordinal INTEGER NOT NULL,
  question_text TEXT NOT NULL,
  answer_text TEXT NULL,
  notes_text TEXT NULL,
  PRIMARY KEY(import_id, row_ordinal),
  FOREIGN KEY(import_id) REFERENCES questionnaire_import(import_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_qna_import_row_import
ON questionnaire_import_row(import_id, row_ordinal ASC);

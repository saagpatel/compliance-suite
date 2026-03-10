-- 0007_questionnaire_review.sql

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS questionnaire_review (
  review_id TEXT PRIMARY KEY,
  import_id TEXT NOT NULL,
  vault_id TEXT NOT NULL,
  question_text TEXT NOT NULL,
  normalized_question TEXT NOT NULL,
  answer_bank_entry_id TEXT NULL,
  suggested_score REAL NULL,
  confidence_explanation TEXT NULL,
  final_answer TEXT NOT NULL,
  notes TEXT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(import_id) REFERENCES questionnaire_import(import_id) ON DELETE CASCADE,
  FOREIGN KEY(vault_id) REFERENCES vault(vault_id),
  FOREIGN KEY(answer_bank_entry_id) REFERENCES answer_bank(entry_id)
);

CREATE INDEX IF NOT EXISTS idx_questionnaire_review_import
ON questionnaire_review(import_id, updated_at DESC, review_id ASC);

CREATE INDEX IF NOT EXISTS idx_questionnaire_review_vault
ON questionnaire_review(vault_id, updated_at DESC, review_id ASC);

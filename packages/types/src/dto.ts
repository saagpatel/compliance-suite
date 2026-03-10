// DTO definitions mirrored from Rust. This is intentionally minimal in Phase 0/1.

export type VaultDto = {
  vault_id: string;
  name: string;
  root_path: string;
  created_at: string;
  encryption_mode: 'none' | 'passphrase';
  schema_version: number;
};

export type EvidenceDto = {
  evidence_id: string;
  vault_id: string;
  filename: string;
  relative_path: string;
  content_type: string;
  byte_size: number;
  sha256: string;
  source: 'manual_import' | 'generated' | 'extracted';
  tags: string[];
  created_at: string;
  notes?: string;
};

export type LicenseStatusDto = {
  installed: boolean;
  valid: boolean;
  license_id?: string;
  features: string[];
  verification_status?: string;
};

// Phase 2 (Questionnaire Autopilot)
export type ColumnMapDto = {
  question: string;
  answer: string;
  notes?: string;
};

export type QuestionnaireImportDto = {
  import_id: string;
  vault_id: string;
  source_filename: string;
  source_sha256: string;
  imported_at: string;
  format: 'csv' | 'xlsx';
  status: 'imported' | 'mapped' | string;
  column_map?: ColumnMapDto;
};

export type QuestionnaireImportRowDto = {
  import_id: string;
  row_ordinal: number;
  question_text: string;
  answer_text?: string;
  notes_text?: string;
};

export type ColumnMapValidationIssueDto = {
  code: string;
  message: string;
  field?: 'question' | 'answer' | 'notes' | string;
};

export type ColumnMapValidationDto = {
  ok: boolean;
  issues: ColumnMapValidationIssueDto[];
};

// Phase 2.3 (Answer Bank)
export type AnswerBankEntryDto = {
  entry_id: string;
  vault_id: string;
  question_canonical: string;
  answer_short: string;
  answer_long: string;
  notes?: string;
  evidence_links: string[];
  owner: string;
  last_reviewed_at?: string;
  tags: string[];
  source: 'manual' | 'import' | 'match' | string;
  content_hash: string;
  created_at: string;
  updated_at: string;
};

export type AnswerBankCreateInputDto = {
  question_canonical: string;
  answer_short: string;
  answer_long: string;
  notes?: string;
  evidence_links: string[];
  owner: string;
  last_reviewed_at?: string;
  tags: string[];
  source: 'manual' | 'import' | 'match' | string;
};

export type AnswerBankUpdatePatchDto = {
  question_canonical?: string;
  answer_short?: string;
  answer_long?: string;
  notes?: string | null;
  evidence_links?: string[];
  owner?: string;
  last_reviewed_at?: string | null;
  tags?: string[];
  source?: 'manual' | 'import' | 'match' | string;
};

export type AnswerBankListParamsDto = {
  limit: number;
  offset: number;
};

// Phase 2.4 (Matching Algorithm)
export type MatchSuggestionDto = {
  answer_bank_entry_id: string;
  score: number;           // 0.0 - 1.0
  question_canonical: string;
  answer_short: string;
  answer_long: string;
  notes?: string;
  normalized_question: string;
  normalized_answer: string;
  confidence_explanation: string;
};

export type QuestionnaireReviewStatusDto =
  | 'draft'
  | 'accepted_suggestion'
  | 'edited_answer';

export type QuestionnaireReviewDto = {
  review_id: string;
  import_id: string;
  vault_id: string;
  source_row_ordinal?: number;
  question_text: string;
  normalized_question: string;
  answer_bank_entry_id?: string;
  suggested_score?: number;
  confidence_explanation?: string;
  final_answer: string;
  notes?: string;
  status: QuestionnaireReviewStatusDto;
  created_at: string;
  updated_at: string;
};

export type QuestionnaireReviewUpsertDto = {
  review_id?: string;
  import_id: string;
  source_row_ordinal?: number;
  question_text: string;
  answer_bank_entry_id?: string;
  suggested_score?: number;
  confidence_explanation?: string;
  final_answer: string;
  notes?: string;
  status: QuestionnaireReviewStatusDto;
};

export type MatchingInputDto = {
  question: string;
  vault_id: string;
  top_n?: number;
};

// Phase 2 (Extended): Column profiling for import
export type ColumnProfileDto = {
  column_index: number;
  inferred_type: 'question' | 'answer' | 'notes' | 'unknown';
  sample_values: string[];
  validation_issues?: string[];
};

export type QuestionnaireImportWithProfilesDto = QuestionnaireImportDto & {
  column_count: number;
  question_count: number;
  column_profiles: ColumnProfileDto[];
};

// Export pack DTOs
export type ExportPackDto = {
  zip_path: string;
  manifest_version: number;
  file_count: number;
  included_paths: string[];
};

export type ExportFilterDto = {
  include_evidence?: boolean;
  include_audit_trail?: boolean;
  questionnaire_id?: string;
};

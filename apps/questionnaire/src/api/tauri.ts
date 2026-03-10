import { invoke } from "@tauri-apps/api/core";
import type {
  AnswerBankCreateInputDto,
  AnswerBankEntryDto,
  AnswerBankUpdatePatchDto,
  ColumnMapDto,
  EvidenceDto,
  LicenseStatusDto,
  MatchSuggestionDto,
  QuestionnaireImportDto,
  QuestionnaireImportRowDto,
  QuestionnaireReviewDto,
  QuestionnaireReviewUpsertDto,
  VaultDto,
} from "@packages/types";

export interface TauriAppErrorShape {
  code?: string;
  message: string;
  details?: string;
  retryable?: boolean;
  user_action?: string;
}

export class TauriAppError extends Error {
  code?: string;
  details?: string;
  retryable?: boolean;
  userAction?: string;

  constructor(payload: TauriAppErrorShape) {
    super(payload.message);
    this.name = "TauriAppError";
    this.code = payload.code;
    this.details = payload.details;
    this.retryable = payload.retryable;
    this.userAction = payload.user_action;
  }
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (err) {
    throw normalizeTauriError(err);
  }
}

function normalizeTauriError(err: unknown): TauriAppError {
  if (err instanceof TauriAppError) {
    return err;
  }

  if (err instanceof Error) {
    const parsed = tryParseAppError(err.message);
    if (parsed) {
      return new TauriAppError(parsed);
    }
    return new TauriAppError({ message: err.message });
  }

  const asString = String(err);
  const parsed = tryParseAppError(asString);
  if (parsed) {
    return new TauriAppError(parsed);
  }

  return new TauriAppError({ message: asString });
}

function tryParseAppError(value: string): TauriAppErrorShape | null {
  try {
    const parsed = JSON.parse(value) as TauriAppErrorShape;
    if (parsed && typeof parsed === "object" && typeof parsed.message === "string") {
      return parsed;
    }
  } catch {
    // Ignore parse failures and fall back to raw string messaging.
  }
  return null;
}

// ============================================================================
// VAULT COMMANDS
// ============================================================================

export async function invokeVaultCreate(path: string, name: string): Promise<VaultDto> {
  return invokeCommand("vault_create", { path, name });
}

export async function invokeVaultOpen(path: string): Promise<VaultDto> {
  return invokeCommand("vault_open", { path });
}

export async function invokeVaultClose(): Promise<void> {
  return invokeCommand("vault_close");
}

export async function invokeVaultLock(): Promise<void> {
  return invokeCommand("vault_lock");
}

// ============================================================================
// QUESTIONNAIRE COMMANDS
// ============================================================================

export interface ColumnProfileDto {
  col_ref: string;
  ordinal: number;
  label: string;
  non_empty_count: number;
  sample: string[];
}

export async function invokeImportQuestionnaire(filePath: string): Promise<QuestionnaireImportDto> {
  return invokeCommand("import_questionnaire", { file_path: filePath });
}

export async function invokeGetColumnProfiles(importId: string): Promise<ColumnProfileDto[]> {
  return invokeCommand("get_column_profiles", { import_id: importId });
}

export async function invokeSaveColumnMapping(
  importId: string,
  columnMap: ColumnMapDto
): Promise<QuestionnaireImportDto> {
  return invokeCommand("save_column_mapping", {
    import_id: importId,
    column_map: columnMap,
  });
}

export async function invokeListImportRows(importId: string): Promise<QuestionnaireImportRowDto[]> {
  return invokeCommand("list_import_rows", { import_id: importId });
}

// ============================================================================
// ANSWER BANK COMMANDS
// ============================================================================

export async function invokeAnswerBankCreate(
  input: AnswerBankCreateInputDto
): Promise<AnswerBankEntryDto> {
  return invokeCommand("answer_bank_create", { input });
}

export async function invokeAnswerBankUpdate(
  entryId: string,
  patch: AnswerBankUpdatePatchDto
): Promise<AnswerBankEntryDto> {
  return invokeCommand("answer_bank_update", { entry_id: entryId, patch });
}

export async function invokeAnswerBankDelete(entryId: string): Promise<void> {
  return invokeCommand("answer_bank_delete", { entry_id: entryId });
}

export async function invokeAnswerBankList(
  limit: number,
  offset: number
): Promise<AnswerBankEntryDto[]> {
  return invokeCommand("answer_bank_list", { limit, offset });
}

export async function invokeAnswerBankSearch(
  query: string,
  limit: number,
  offset: number
): Promise<AnswerBankEntryDto[]> {
  return invokeCommand("answer_bank_search", { query, limit, offset });
}

export async function invokeAnswerBankLinkEvidence(
  entryId: string,
  evidenceId: string
): Promise<AnswerBankEntryDto> {
  return invokeCommand("answer_bank_link_evidence", {
    entry_id: entryId,
    evidence_id: evidenceId,
  });
}

export async function invokeListEvidence(): Promise<EvidenceDto[]> {
  return invokeCommand("list_evidence");
}

export async function invokeImportEvidence(filePath: string): Promise<EvidenceDto> {
  return invokeCommand("import_evidence", { file_path: filePath });
}

// ============================================================================
// MATCHING AND REVIEW COMMANDS
// ============================================================================

export async function invokeGetMatchingSuggestions(
  question: string,
  topN?: number
): Promise<MatchSuggestionDto[]> {
  return invokeCommand("get_matching_suggestions", {
    question,
    top_n: topN ?? 5,
  });
}

export async function invokeListQuestionnaireReviews(
  importId: string
): Promise<QuestionnaireReviewDto[]> {
  return invokeCommand("list_questionnaire_reviews", { import_id: importId });
}

export async function invokeSaveQuestionnaireReview(
  input: QuestionnaireReviewUpsertDto
): Promise<QuestionnaireReviewDto> {
  return invokeCommand("save_questionnaire_review", { input });
}

export async function invokeDeleteQuestionnaireReview(reviewId: string): Promise<void> {
  return invokeCommand("delete_questionnaire_review", { review_id: reviewId });
}

// ============================================================================
// EXPORT COMMANDS
// ============================================================================

export interface ExportPackDto {
  zip_path: string;
  manifest_version: number;
  file_count: number;
  included_paths: string[];
}

export async function invokeGenerateExportPack(
  outputPath: string,
  importId: string
): Promise<ExportPackDto> {
  return invokeCommand("generate_export_pack", { output_path: outputPath, import_id: importId });
}

// ============================================================================
// LICENSE COMMANDS
// ============================================================================

export async function invokeCheckLicenseStatus(): Promise<LicenseStatusDto> {
  return invokeCommand("check_license_status");
}

export async function invokeInstallLicense(licensePath: string): Promise<LicenseStatusDto> {
  return invokeCommand("install_license", { license_path: licensePath });
}

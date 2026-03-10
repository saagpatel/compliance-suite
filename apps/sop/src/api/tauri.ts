import { invoke } from "@tauri-apps/api/core";

export type VaultDto = {
  vault_id: string;
  name: string;
  root_path: string;
  created_at: string;
  encryption_mode: string;
  schema_version: number;
};

export type SopDocumentDto = {
  document_id: string;
  vault_id: string;
  title: string;
  slug: string;
  owner: string;
  status: string;
  published_version_id?: string;
  latest_version_number: number;
  latest_body_markdown: string;
  latest_change_summary?: string;
  created_at: string;
  updated_at: string;
};

export type SopVersionDto = {
  version_id: string;
  document_id: string;
  version_number: number;
  body_markdown: string;
  change_summary?: string;
  created_at: string;
};

export type SopApprovalStepDto = {
  step_id: string;
  request_id: string;
  document_id: string;
  version_id: string;
  approver: string;
  request_status: string;
  status: string;
  decided_at?: string;
  notes?: string;
  requested_at: string;
};

export type SopAcknowledgmentDto = {
  acknowledgment_id: string;
  document_id: string;
  version_id: string;
  recipient: string;
  status: string;
  acknowledged_at?: string;
  created_at: string;
};

export type SopDocumentCreateInputDto = {
  title: string;
  slug: string;
  owner: string;
  body_markdown: string;
  change_summary?: string;
};

export type SopDocumentUpdateInputDto = {
  body_markdown: string;
  change_summary?: string;
};

export type SopApprovalSubmitInputDto = {
  approvers: string[];
};

export type SopApprovalDecisionInputDto = {
  decision: "approved" | "changes_requested";
  notes?: string;
};

export type SopAcknowledgmentAssignInputDto = {
  recipients: string[];
};

export type LicenseStatusDto = {
  installed: boolean;
  valid: boolean;
  license_id?: string;
  features: string[];
  verification_status?: string;
};

export type ExportPackDto = {
  zip_path: string;
  manifest_version: number;
  file_count: number;
  included_paths: string[];
};

type AppErrorShape = {
  code?: string;
  message: string;
  details?: string;
  retryable?: boolean;
  user_action?: string;
};

export class SopAppError extends Error {
  code?: string;
  details?: string;
  retryable?: boolean;
  userAction?: string;

  constructor(payload: AppErrorShape) {
    super(payload.message);
    this.name = "SopAppError";
    this.code = payload.code;
    this.details = payload.details;
    this.retryable = payload.retryable;
    this.userAction = payload.user_action;
  }
}

function parseError(value: unknown): SopAppError {
  if (value instanceof SopAppError) {
    return value;
  }
  if (value instanceof Error) {
    try {
      const parsed = JSON.parse(value.message) as AppErrorShape;
      if (parsed && typeof parsed.message === "string") {
        return new SopAppError(parsed);
      }
    } catch {
      return new SopAppError({ message: value.message });
    }
    return new SopAppError({ message: value.message });
  }
  return new SopAppError({ message: String(value) });
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw parseError(error);
  }
}

export async function createVault(path: string, name: string): Promise<VaultDto> {
  return invokeCommand("vault_create", { path, name });
}

export async function openVault(path: string): Promise<VaultDto> {
  return invokeCommand("vault_open", { path });
}

export async function closeVault(): Promise<void> {
  return invokeCommand("vault_close");
}

export async function checkLicenseStatus(): Promise<LicenseStatusDto> {
  return invokeCommand("check_license_status");
}

export async function installLicense(licensePath: string): Promise<LicenseStatusDto> {
  return invokeCommand("install_license", { license_path: licensePath });
}

export async function createSopDocument(
  input: SopDocumentCreateInputDto
): Promise<SopDocumentDto> {
  return invokeCommand("sop_create_document", { input });
}

export async function listSopDocuments(): Promise<SopDocumentDto[]> {
  return invokeCommand("sop_list_documents");
}

export async function updateSopDocument(
  documentId: string,
  input: SopDocumentUpdateInputDto
): Promise<SopDocumentDto> {
  return invokeCommand("sop_update_document", {
    document_id: documentId,
    input
  });
}

export async function publishSopDocument(documentId: string): Promise<SopDocumentDto> {
  return invokeCommand("sop_publish_document", {
    document_id: documentId
  });
}

export async function submitSopForApproval(
  documentId: string,
  input: SopApprovalSubmitInputDto
): Promise<SopDocumentDto> {
  return invokeCommand("sop_submit_for_approval", {
    document_id: documentId,
    input
  });
}

export async function listSopApprovalSteps(
  documentId: string
): Promise<SopApprovalStepDto[]> {
  return invokeCommand("sop_list_approval_steps", {
    document_id: documentId
  });
}

export async function decideSopApproval(
  stepId: string,
  input: SopApprovalDecisionInputDto
): Promise<SopDocumentDto> {
  return invokeCommand("sop_decide_approval", {
    step_id: stepId,
    input
  });
}

export async function listSopVersions(documentId: string): Promise<SopVersionDto[]> {
  return invokeCommand("sop_list_versions", {
    document_id: documentId
  });
}

export async function assignSopAcknowledgments(
  documentId: string,
  input: SopAcknowledgmentAssignInputDto
): Promise<SopAcknowledgmentDto[]> {
  return invokeCommand("sop_assign_acknowledgments", {
    document_id: documentId,
    input
  });
}

export async function listSopAcknowledgments(
  documentId: string
): Promise<SopAcknowledgmentDto[]> {
  return invokeCommand("sop_list_acknowledgments", {
    document_id: documentId
  });
}

export async function recordSopAcknowledgment(
  acknowledgmentId: string
): Promise<SopAcknowledgmentDto> {
  return invokeCommand("sop_record_acknowledgment", {
    acknowledgment_id: acknowledgmentId
  });
}

export async function generateExportPack(outputPath: string): Promise<ExportPackDto> {
  return invokeCommand("generate_export_pack", { output_path: outputPath });
}

import { invoke } from "@tauri-apps/api/core";

export type VaultDto = {
  vault_id: string;
  name: string;
  root_path: string;
  created_at: string;
  encryption_mode: string;
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
  source: string;
  tags: string[];
  created_at: string;
  notes?: string;
};

export type BinderControlDto = {
  control_id: string;
  vault_id: string;
  framework: string;
  control_code: string;
  title: string;
  description?: string;
  reporting_period: string;
  status: "draft" | "collecting_evidence" | "reviewing" | "ready" | string;
  owner: string;
  evidence_links: string[];
  created_at: string;
  updated_at: string;
};

export type BinderStatusSummaryDto = {
  reporting_period: string;
  total_controls: number;
  ready_controls: number;
  controls_with_evidence: number;
  controls_without_evidence: number;
};

export type BinderControlCreateInputDto = {
  framework: string;
  control_code: string;
  title: string;
  description?: string;
  reporting_period: string;
  status: "draft" | "collecting_evidence" | "reviewing" | "ready";
  owner: string;
  evidence_links: string[];
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

export class BinderAppError extends Error {
  code?: string;
  details?: string;
  retryable?: boolean;
  userAction?: string;

  constructor(payload: AppErrorShape) {
    super(payload.message);
    this.name = "BinderAppError";
    this.code = payload.code;
    this.details = payload.details;
    this.retryable = payload.retryable;
    this.userAction = payload.user_action;
  }
}

function parseError(value: unknown): BinderAppError {
  if (value instanceof BinderAppError) {
    return value;
  }
  if (value instanceof Error) {
    try {
      const parsed = JSON.parse(value.message) as AppErrorShape;
      if (parsed && typeof parsed.message === "string") {
        return new BinderAppError(parsed);
      }
    } catch {
      return new BinderAppError({ message: value.message });
    }
    return new BinderAppError({ message: value.message });
  }
  return new BinderAppError({ message: String(value) });
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

export async function listEvidence(): Promise<EvidenceDto[]> {
  return invokeCommand("list_evidence");
}

export async function importEvidence(filePath: string): Promise<EvidenceDto> {
  return invokeCommand("import_evidence", { file_path: filePath });
}

export async function createBinderControl(
  input: BinderControlCreateInputDto
): Promise<BinderControlDto> {
  return invokeCommand("binder_create_control", { input });
}

export async function listBinderControls(
  reportingPeriod?: string
): Promise<BinderControlDto[]> {
  return invokeCommand("binder_list_controls", {
    reporting_period: reportingPeriod || null
  });
}

export async function linkBinderEvidence(
  controlId: string,
  evidenceId: string
): Promise<BinderControlDto> {
  return invokeCommand("binder_link_evidence", {
    control_id: controlId,
    evidence_id: evidenceId
  });
}

export async function setBinderControlStatus(
  controlId: string,
  status: BinderControlDto["status"]
): Promise<BinderControlDto> {
  return invokeCommand("binder_set_control_status", {
    control_id: controlId,
    status
  });
}

export async function getBinderSummary(
  reportingPeriod?: string
): Promise<BinderStatusSummaryDto[]> {
  return invokeCommand("binder_status_summary", {
    reporting_period: reportingPeriod || null
  });
}

export async function generateExportPack(outputPath: string): Promise<ExportPackDto> {
  return invokeCommand("generate_export_pack", { output_path: outputPath });
}

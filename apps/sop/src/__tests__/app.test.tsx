import { render, screen, waitFor } from "@testing-library/react";
import App from "../App";
import { SOP_LAST_VAULT_KEY } from "../state";

const openDialogMock = vi.hoisted(() => vi.fn());
const saveDialogMock = vi.hoisted(() => vi.fn());
const api = vi.hoisted(() => ({
  assignSopAcknowledgments: vi.fn(),
  checkLicenseStatus: vi.fn(),
  closeVault: vi.fn(),
  createSopDocument: vi.fn(),
  createVault: vi.fn(),
  decideSopApproval: vi.fn(),
  generateExportPack: vi.fn(),
  installLicense: vi.fn(),
  listSopAcknowledgments: vi.fn(),
  listSopApprovalSteps: vi.fn(),
  listSopDocuments: vi.fn(),
  listSopVersions: vi.fn(),
  openVault: vi.fn(),
  publishSopDocument: vi.fn(),
  recordSopAcknowledgment: vi.fn(),
  submitSopForApproval: vi.fn(),
  updateSopDocument: vi.fn()
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: openDialogMock,
  save: saveDialogMock
}));

vi.mock("../api", async () => {
  const actual = await vi.importActual<typeof import("../api")>("../api");
  return {
    ...actual,
    assignSopAcknowledgments: api.assignSopAcknowledgments,
    checkLicenseStatus: api.checkLicenseStatus,
    closeVault: api.closeVault,
    createSopDocument: api.createSopDocument,
    createVault: api.createVault,
    decideSopApproval: api.decideSopApproval,
    generateExportPack: api.generateExportPack,
    installLicense: api.installLicense,
    listSopAcknowledgments: api.listSopAcknowledgments,
    listSopApprovalSteps: api.listSopApprovalSteps,
    listSopDocuments: api.listSopDocuments,
    listSopVersions: api.listSopVersions,
    openVault: api.openVault,
    publishSopDocument: api.publishSopDocument,
    recordSopAcknowledgment: api.recordSopAcknowledgment,
    submitSopForApproval: api.submitSopForApproval,
    updateSopDocument: api.updateSopDocument
  };
});

describe("sop app", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
    openDialogMock.mockResolvedValue(null);
    saveDialogMock.mockResolvedValue(null);
    api.assignSopAcknowledgments.mockResolvedValue([]);
    api.checkLicenseStatus.mockResolvedValue({
      installed: false,
      valid: false,
      features: []
    });
    api.closeVault.mockResolvedValue(undefined);
    api.createSopDocument.mockResolvedValue(undefined);
    api.createVault.mockResolvedValue(undefined);
    api.decideSopApproval.mockResolvedValue(undefined);
    api.generateExportPack.mockResolvedValue(undefined);
    api.installLicense.mockResolvedValue(undefined);
    api.listSopAcknowledgments.mockResolvedValue([]);
    api.listSopApprovalSteps.mockResolvedValue([]);
    api.listSopDocuments.mockResolvedValue([]);
    api.listSopVersions.mockResolvedValue([]);
    api.openVault.mockResolvedValue(undefined);
    api.publishSopDocument.mockResolvedValue(undefined);
    api.recordSopAcknowledgment.mockResolvedValue(undefined);
    api.submitSopForApproval.mockResolvedValue(undefined);
    api.updateSopDocument.mockResolvedValue(undefined);
  });

  it("renders the SOP welcome flow when no vault is available", async () => {
    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /SOP workspace for controlled procedures/i })
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Create SOP Vault/i })).toBeDisabled();
    expect(api.openVault).not.toHaveBeenCalled();
  });

  it("loads the SOP workspace from the remembered vault path", async () => {
    localStorage.setItem(SOP_LAST_VAULT_KEY, "/tmp/sop-vault");
    api.openVault.mockResolvedValue({
      vault_id: "vault_1",
      name: "SOP Vault",
      root_path: "/tmp/sop-vault",
      created_at: "2026-03-10T00:00:00Z",
      encryption_mode: "none",
      schema_version: 10
    });
    api.checkLicenseStatus.mockResolvedValue({
      installed: true,
      valid: true,
      features: ["EXPORT_PACKS"],
      license_id: "lic_1",
      verification_status: "verified"
    });
    api.listSopDocuments.mockResolvedValue([
      {
        document_id: "doc_1",
        vault_id: "vault_1",
        title: "Access Review Procedure",
        slug: "access-review-procedure",
        owner: "operations",
        status: "published",
        published_version_id: "version_2",
        latest_version_number: 2,
        latest_body_markdown: "# Access review",
        latest_change_summary: "Added manager approval step",
        created_at: "2026-03-10T00:00:00Z",
        updated_at: "2026-03-10T00:00:00Z"
      }
    ]);
    api.listSopVersions.mockResolvedValue([
      {
        version_id: "version_2",
        document_id: "doc_1",
        version_number: 2,
        body_markdown: "# Access review",
        change_summary: "Added manager approval step",
        created_at: "2026-03-10T00:00:00Z"
      }
    ]);
    api.listSopApprovalSteps.mockResolvedValue([
      {
        step_id: "step_1",
        request_id: "request_1",
        document_id: "doc_1",
        version_id: "version_2",
        approver: "Quality lead",
        request_status: "approved",
        status: "approved",
        decided_at: "2026-03-10T00:00:00Z",
        requested_at: "2026-03-10T00:00:00Z"
      }
    ]);
    api.listSopAcknowledgments.mockResolvedValue([
      {
        acknowledgment_id: "ack_1",
        document_id: "doc_1",
        version_id: "version_2",
        recipient: "Team lead",
        status: "pending",
        created_at: "2026-03-10T00:00:00Z"
      }
    ]);

    render(<App />);

    expect(await screen.findByRole("heading", { name: /SOP Vault/i })).toBeInTheDocument();
    expect(screen.getByText(/Access Review Procedure/i)).toBeInTheDocument();
    expect(screen.getByText(/Latest change: Added manager approval step/i)).toBeInTheDocument();
    expect(screen.getAllByText(/^Published$/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Export-ready/i).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: /Generate SOP Pack/i })).toBeEnabled();
    expect(screen.getByText(/Approval flow/i)).toBeInTheDocument();
    expect(screen.getAllByText(/Acknowledgments/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Quality lead/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Team lead/i).length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(api.openVault).toHaveBeenCalledWith("/tmp/sop-vault");
      expect(api.checkLicenseStatus).toHaveBeenCalled();
      expect(api.listSopDocuments).toHaveBeenCalled();
      expect(api.listSopVersions).toHaveBeenCalledWith("doc_1");
      expect(api.listSopApprovalSteps).toHaveBeenCalledWith("doc_1");
      expect(api.listSopAcknowledgments).toHaveBeenCalledWith("doc_1");
    });
  });
});

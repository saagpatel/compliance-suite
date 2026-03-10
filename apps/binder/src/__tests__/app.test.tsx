import { render, screen, waitFor } from "@testing-library/react";
import App from "../App";
import { BINDER_LAST_VAULT_KEY } from "../state";

const openDialogMock = vi.hoisted(() => vi.fn());
const api = vi.hoisted(() => ({
  checkLicenseStatus: vi.fn(),
  closeVault: vi.fn(),
  createBinderControl: vi.fn(),
  createVault: vi.fn(),
  generateExportPack: vi.fn(),
  getBinderSummary: vi.fn(),
  importEvidence: vi.fn(),
  installLicense: vi.fn(),
  linkBinderEvidence: vi.fn(),
  listBinderControls: vi.fn(),
  listEvidence: vi.fn(),
  openVault: vi.fn(),
  setBinderControlStatus: vi.fn()
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: openDialogMock
}));

vi.mock("../api", async () => {
  const actual = await vi.importActual<typeof import("../api")>("../api");
  return {
    ...actual,
    checkLicenseStatus: api.checkLicenseStatus,
    closeVault: api.closeVault,
    createBinderControl: api.createBinderControl,
    createVault: api.createVault,
    generateExportPack: api.generateExportPack,
    getBinderSummary: api.getBinderSummary,
    importEvidence: api.importEvidence,
    installLicense: api.installLicense,
    linkBinderEvidence: api.linkBinderEvidence,
    listBinderControls: api.listBinderControls,
    listEvidence: api.listEvidence,
    openVault: api.openVault,
    setBinderControlStatus: api.setBinderControlStatus
  };
});

describe("binder app", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
    openDialogMock.mockResolvedValue(null);
    api.checkLicenseStatus.mockResolvedValue({
      installed: false,
      valid: false,
      features: []
    });
    api.closeVault.mockResolvedValue(undefined);
    api.createBinderControl.mockResolvedValue(undefined);
    api.createVault.mockResolvedValue(undefined);
    api.generateExportPack.mockResolvedValue(undefined);
    api.getBinderSummary.mockResolvedValue([]);
    api.importEvidence.mockResolvedValue(undefined);
    api.installLicense.mockResolvedValue(undefined);
    api.linkBinderEvidence.mockResolvedValue(undefined);
    api.listBinderControls.mockResolvedValue([]);
    api.listEvidence.mockResolvedValue([]);
    api.openVault.mockResolvedValue(undefined);
    api.setBinderControlStatus.mockResolvedValue(undefined);
  });

  it("renders the Binder welcome flow when no vault is available", async () => {
    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /Binder workspace for control evidence/i })
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Create Binder Vault/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Open Binder Vault/i })).toBeDisabled();
    expect(api.openVault).not.toHaveBeenCalled();
  });

  it("loads the Binder workspace from the remembered vault path", async () => {
    localStorage.setItem(BINDER_LAST_VAULT_KEY, "/tmp/binder-vault");
    api.openVault.mockResolvedValue({
      vault_id: "vault_1",
      name: "Binder Vault",
      root_path: "/tmp/binder-vault",
      created_at: "2026-03-10T00:00:00Z",
      encryption_mode: "none",
      schema_version: 9
    });
    api.getBinderSummary.mockResolvedValue([
      {
        reporting_period: "2026-Q1",
        total_controls: 3,
        ready_controls: 1,
        controls_with_evidence: 2,
        controls_without_evidence: 1
      }
    ]);
    api.checkLicenseStatus.mockResolvedValue({
      installed: true,
      valid: true,
      features: ["EXPORT_PACKS"],
      license_id: "lic_1",
      verification_status: "verified"
    });
    api.listBinderControls.mockResolvedValue([
      {
        control_id: "control_1",
        vault_id: "vault_1",
        framework: "SOC 2",
        control_code: "CC6.1",
        title: "Access control review",
        description: "Demonstrate periodic review of privileged access.",
        reporting_period: "2026-Q1",
        status: "collecting_evidence",
        owner: "security",
        evidence_links: ["evidence_1"],
        created_at: "2026-03-10T00:00:00Z",
        updated_at: "2026-03-10T00:00:00Z"
      }
    ]);
    api.listEvidence.mockResolvedValue([
      {
        evidence_id: "evidence_1",
        vault_id: "vault_1",
        filename: "access-review.pdf",
        relative_path: "evidence/access-review.pdf",
        content_type: "application/pdf",
        byte_size: 1024,
        sha256: "abc123",
        source: "manual_import",
        tags: [],
        created_at: "2026-03-10T00:00:00Z"
      }
    ]);

    render(<App />);

    expect(await screen.findByRole("heading", { name: /Binder Vault/i })).toBeInTheDocument();
    expect(screen.getByText(/1\/3/i)).toBeInTheDocument();
    expect(screen.getAllByText(/Access control review/i).length).toBeGreaterThan(0);
    expect(screen.getByText(/evidence\/access-review\.pdf/i)).toBeInTheDocument();
    expect(screen.getByText(/Export-ready/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Generate Binder Pack/i })).toBeEnabled();
    expect(screen.getByText(/1 controls still need work/i)).toBeInTheDocument();
    expect(
      screen.getByText(/Next action: review the evidence set and mark this control Ready\./i)
    ).toBeInTheDocument();

    await waitFor(() => {
      expect(api.openVault).toHaveBeenCalledWith("/tmp/binder-vault");
      expect(api.checkLicenseStatus).toHaveBeenCalled();
      expect(api.listBinderControls).toHaveBeenCalled();
      expect(api.getBinderSummary).toHaveBeenCalled();
      expect(api.listEvidence).toHaveBeenCalled();
    });
  });
});

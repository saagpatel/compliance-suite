import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import AnswerBankTable from "../components/features/AnswerBankTable";

const { importEvidence, loadEvidence, dialogOpen } = vi.hoisted(() => ({
  importEvidence: vi.fn().mockResolvedValue({
    evidence_id: "ev_2",
    vault_id: "vault_1",
    filename: "new-evidence.pdf",
    relative_path: "evidence/cd/new-evidence.pdf",
    content_type: "application/pdf",
    byte_size: 1024,
    sha256: "sha_2",
    source: "manual_import",
    tags: [],
    created_at: "2026-03-10T00:00:00Z",
  }),
  loadEvidence: vi.fn(),
  dialogOpen: vi.fn().mockResolvedValue("/tmp/new-evidence.pdf"),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: dialogOpen,
}));

vi.mock("../hooks/useAnswerBank", () => ({
  useAnswerBank: () => ({
    entries: [
      {
        entry_id: "entry_1",
        vault_id: "vault_1",
        question_canonical: "Do you encrypt data?",
        answer_short: "Yes",
        answer_long: "Yes, all customer data is encrypted at rest.",
        notes: "Reviewed by security",
        evidence_links: ["ev_1"],
        owner: "security",
        last_reviewed_at: "2026-03-10T00:00:00Z",
        tags: ["security"],
        source: "manual",
        content_hash: "hash_1",
        created_at: "2026-03-10T00:00:00Z",
        updated_at: "2026-03-10T00:00:00Z",
      },
    ],
    selectedEntry: null,
    total: 1,
    limit: 50,
    offset: 0,
    searchQuery: "",
    loading: false,
    error: null,
    loadEntries: vi.fn(),
    searchEntries: vi.fn(),
    createEntry: vi.fn(),
    updateEntry: vi.fn(),
    deleteEntry: vi.fn(),
    linkEvidence: vi.fn(),
    setSelectedEntry: vi.fn(),
    setLimit: vi.fn(),
    setOffset: vi.fn(),
  }),
}));

vi.mock("../hooks/useEvidence", () => ({
  useEvidence: () => ({
    evidence: [
      {
        evidence_id: "ev_1",
        vault_id: "vault_1",
        filename: "soc2-report.pdf",
        relative_path: "evidence/ab/soc2-report.pdf",
        content_type: "application/pdf",
        byte_size: 2048,
        sha256: "sha_1",
        source: "manual_import",
        tags: [],
        created_at: "2026-03-10T00:00:00Z",
      },
    ],
    loading: false,
    error: null,
    loadEvidence,
    importEvidence,
  }),
}));

describe("AnswerBankTable", () => {
  beforeEach(() => {
    importEvidence.mockClear();
    loadEvidence.mockClear();
    dialogOpen.mockClear();
  });

  it("shows vault evidence context for linked answer bank entries", () => {
    render(<AnswerBankTable />);

    expect(screen.getByText(/Vault evidence browser/i)).toBeInTheDocument();
    expect(screen.getByText(/1 evidence files available/i)).toBeInTheDocument();
    expect(screen.getAllByText(/soc2-report\.pdf/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/evidence\/ab\/soc2-report\.pdf/i).length).toBeGreaterThan(0);
    expect(screen.getAllByText("ev_1").length).toBeGreaterThan(0);
  });

  it("imports evidence files into the vault browser", async () => {
    render(<AnswerBankTable />);

    fireEvent.click(screen.getByRole("button", { name: /Import Evidence Files/i }));

    await waitFor(() => {
      expect(dialogOpen).toHaveBeenCalled();
      expect(importEvidence).toHaveBeenCalledWith("/tmp/new-evidence.pdf");
    });
  });
});

import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { Header } from "../components/layout/Header";
import { Sidebar } from "../components/layout/Sidebar";
import { useImportStore } from "../state/importStore";
import { useVaultStore } from "../state/vaultStore";

describe("questionnaire layout shell", () => {
  beforeEach(() => {
    useVaultStore.setState({
      currentVault: {
        vault_id: "vault_1",
        name: "Operations Vault",
        root_path: "/tmp/operations-vault",
        created_at: "2026-03-10T00:00:00Z",
        encryption_mode: "none",
        schema_version: 1,
      },
      lastVaultPath: "/tmp/operations-vault",
      loading: false,
      error: null,
      initializing: false,
      initialized: true,
    });
    useImportStore.setState({
      currentImport: {
        import_id: "import_1",
        vault_id: "vault_1",
        source_filename: "questionnaire.xlsx",
        source_sha256: "abc123",
        imported_at: "2026-03-10T00:00:00Z",
        format: "xlsx",
        status: "mapped",
        column_map: {
          question: "A",
          answer: "B",
        },
      },
      columnProfiles: [],
      columnMap: {
        question: "A",
        answer: "B",
      },
      loading: false,
      error: null,
      step: "review",
    });
  });

  it("renders the sidebar navigation for the active questionnaire flow", () => {
    render(
      <MemoryRouter initialEntries={["/review"]}>
        <Sidebar />
      </MemoryRouter>,
    );

    expect(screen.getByText("Questionnaire Autopilot")).toBeInTheDocument();
    expect(screen.getByText("Operations Vault")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Import/i })).toHaveAttribute("href", "/");
    expect(screen.getByRole("link", { name: /Review/i })).toHaveAttribute("href", "/review");
  });

  it("renders page title and subtitle in the header", () => {
    render(
      <MemoryRouter>
        <Header title="Import Questionnaire" subtitle="Step 1: Import your questionnaire file" />
      </MemoryRouter>,
    );

    expect(screen.getByRole("heading", { name: /Import Questionnaire/i })).toBeInTheDocument();
    expect(screen.getByText(/Step 1: Import your questionnaire file/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /toggle sidebar/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /switch vault/i })).toBeInTheDocument();
  });
});

import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import ExportPage from "../routes/Export";

vi.mock("../hooks/useImport", () => ({
  useImport: () => ({
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
  }),
}));

vi.mock("../hooks/useLicenseStatus", () => ({
  useLicenseStatus: () => ({
    status: {
      installed: false,
      valid: false,
      features: [],
      verification_status: undefined,
      license_id: undefined,
    },
    loading: false,
    error: null,
  }),
}));

vi.mock("../hooks/useQuestionnaireReview", () => ({
  useQuestionnaireReview: () => ({
    reviews: [
      {
        review_id: "review_1",
        import_id: "import_1",
        vault_id: "vault_1",
        question_text: "Do you encrypt data?",
        normalized_question: "do you encrypt data",
        final_answer: "Yes",
        status: "accepted_suggestion",
        created_at: "2026-03-10T00:00:00Z",
        updated_at: "2026-03-10T00:00:00Z",
      },
    ],
    loading: false,
  }),
}));

describe("ExportPage", () => {
  it("shows blocked export messaging when license requirements are not met", () => {
    render(
      <MemoryRouter>
        <ExportPage />
      </MemoryRouter>
    );

    expect(screen.getByText(/Export is currently blocked/i)).toBeInTheDocument();
    expect(
      screen.getByText(/Install a valid license with the EXPORT_PACKS feature before exporting/i)
    ).toBeInTheDocument();
    expect(screen.getByText(/1 saved review entries/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Generate Export Pack/i })).toBeDisabled();
  });
});

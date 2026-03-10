import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import MatchingResults from "../components/features/MatchingResults";

const getMatches = vi.fn();
const clearSuggestions = vi.fn();
const saveReview = vi.fn().mockResolvedValue({
  review_id: "review_1",
  import_id: "import_1",
  vault_id: "vault_1",
  source_row_ordinal: 2,
  question_text: "Do you encrypt data?",
  normalized_question: "do you encrypt data",
  final_answer: "Yes, all customer data is encrypted at rest.",
  status: "accepted_suggestion",
  created_at: "2026-03-10T00:00:00Z",
  updated_at: "2026-03-10T00:00:00Z",
});
const deleteReview = vi.fn();

vi.mock("../hooks/useMatching", () => ({
  useMatching: () => ({
    suggestions: [
      {
        answer_bank_entry_id: "entry_1",
        score: 0.91,
        question_canonical: "Do you encrypt data?",
        answer_short: "Yes",
        answer_long: "Yes, all customer data is encrypted at rest.",
        notes: "Reviewed by security",
        normalized_question: "do you encrypt data",
        normalized_answer: "yes all customer data is encrypted at rest",
        confidence_explanation: "Strong overlap on encrypt and data",
      },
    ],
    loading: false,
    error: null,
    getMatches,
    clearSuggestions,
  }),
}));

vi.mock("../hooks/useQuestionnaireReview", () => ({
  useQuestionnaireReview: () => ({
    reviews: [],
    loading: false,
    saving: false,
    error: null,
    saveReview,
    deleteReview,
  }),
}));

vi.mock("../hooks/useQuestionnaireImportRows", () => ({
  useQuestionnaireImportRows: () => ({
    rows: [
      {
        import_id: "import_1",
        row_ordinal: 2,
        question_text: "Do you encrypt data?",
        answer_text: "Yes",
        notes_text: "Imported from source questionnaire",
      },
      {
        import_id: "import_1",
        row_ordinal: 3,
        question_text: "Do you monitor access logs?",
        answer_text: "Yes, all access is logged.",
        notes_text: "Second imported row",
      },
    ],
    loading: false,
    error: null,
  }),
}));

describe("MatchingResults", () => {
  beforeEach(() => {
    getMatches.mockReset();
    clearSuggestions.mockReset();
    saveReview.mockClear();
    deleteReview.mockClear();
  });

  it("saves an accepted suggestion into the persisted review workspace", async () => {
    render(<MatchingResults importId="import_1" />);

    fireEvent.click(screen.getAllByRole("button", { name: /^Review$/i })[0]);
    fireEvent.click(screen.getByRole("button", { name: /Use/i }));
    fireEvent.click(screen.getByRole("button", { name: /Save Review Entry/i }));

    await waitFor(() => {
      expect(saveReview).toHaveBeenCalledWith(
        expect.objectContaining({
          import_id: "import_1",
          source_row_ordinal: 2,
          question_text: "Do you encrypt data?",
          answer_bank_entry_id: "entry_1",
          status: "accepted_suggestion",
        })
      );
    });

    await waitFor(() => {
      expect(screen.getByDisplayValue("Do you monitor access logs?")).toBeInTheDocument();
    });
  });
});

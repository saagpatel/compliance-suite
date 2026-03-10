import { useCallback, useMemo, useState } from "react";
import type {
  MatchSuggestionDto,
  QuestionnaireImportRowDto,
  QuestionnaireReviewDto,
} from "@packages/types";
import { useMatching } from "../../hooks/useMatching";
import { useQuestionnaireImportRows } from "../../hooks/useQuestionnaireImportRows";
import { useQuestionnaireReview } from "../../hooks/useQuestionnaireReview";
import Button from "../ui/Button";
import Input from "../ui/Input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../ui/Table";

interface MatchingResultsProps {
  importId: string;
}

function buildSavedSuggestion(review?: QuestionnaireReviewDto | null): MatchSuggestionDto | null {
  if (!review?.answer_bank_entry_id) {
    return null;
  }

  return {
    answer_bank_entry_id: review.answer_bank_entry_id,
    score: review.suggested_score ?? 0,
    question_canonical: review.question_text,
    answer_short: review.final_answer,
    answer_long: review.final_answer,
    notes: review.notes,
    normalized_question: review.normalized_question,
    normalized_answer: review.final_answer,
    confidence_explanation: review.confidence_explanation ?? "Previously accepted suggestion",
  };
}

function summarizeText(value?: string, maxLength = 96) {
  if (!value) {
    return "No value provided";
  }

  const normalized = value.replace(/\s+/g, " ").trim();
  if (normalized.length <= maxLength) {
    return normalized;
  }

  return `${normalized.slice(0, maxLength - 3)}...`;
}

export default function MatchingResults({ importId }: MatchingResultsProps) {
  const { suggestions, loading, error, getMatches, clearSuggestions } = useMatching();
  const { rows: importRows, loading: importRowsLoading, error: importRowsError } =
    useQuestionnaireImportRows(importId);
  const { reviews, loading: reviewsLoading, saving, deleteReview, saveReview } =
    useQuestionnaireReview(importId);
  const [question, setQuestion] = useState("");
  const [finalAnswer, setFinalAnswer] = useState("");
  const [notes, setNotes] = useState("");
  const [editingReviewId, setEditingReviewId] = useState<string | undefined>(undefined);
  const [selectedSuggestion, setSelectedSuggestion] = useState<MatchSuggestionDto | null>(null);
  const [selectedRowOrdinal, setSelectedRowOrdinal] = useState<number | undefined>(undefined);
  const [rowFilter, setRowFilter] = useState<"all" | "pending" | "reviewed">("all");

  const selectedSuggestionAnswer = useMemo(() => {
    if (!selectedSuggestion) {
      return "";
    }
    return selectedSuggestion.answer_long.trim() || selectedSuggestion.answer_short.trim();
  }, [selectedSuggestion]);

  const reviewsByRow = useMemo(() => {
    const next = new Map<number, QuestionnaireReviewDto>();

    for (const review of reviews) {
      if (review.source_row_ordinal != null) {
        next.set(review.source_row_ordinal, review);
      }
    }

    return next;
  }, [reviews]);

  const reviewedRowCount = useMemo(() => reviewsByRow.size, [reviewsByRow]);
  const completionPercent = useMemo(() => {
    if (importRows.length === 0) {
      return 0;
    }
    return Math.round((reviewedRowCount / importRows.length) * 100);
  }, [importRows.length, reviewedRowCount]);

  const selectedImportRow = useMemo(
    () => importRows.find((row) => row.row_ordinal === selectedRowOrdinal),
    [importRows, selectedRowOrdinal]
  );
  const nextPendingRow = useMemo(
    () => importRows.find((row) => !reviewsByRow.has(row.row_ordinal)) ?? null,
    [importRows, reviewsByRow]
  );
  const visibleRows = useMemo(() => {
    switch (rowFilter) {
      case "pending":
        return importRows.filter((row) => !reviewsByRow.has(row.row_ordinal));
      case "reviewed":
        return importRows.filter((row) => reviewsByRow.has(row.row_ordinal));
      default:
        return importRows;
    }
  }, [importRows, reviewsByRow, rowFilter]);

  const syncWorkspaceFromRow = useCallback(
    (row: QuestionnaireImportRowDto, review?: QuestionnaireReviewDto) => {
      setSelectedRowOrdinal(row.row_ordinal);
      setEditingReviewId(review?.review_id);
      setQuestion(review?.question_text ?? row.question_text);
      setFinalAnswer(review?.final_answer ?? row.answer_text ?? "");
      setNotes(review?.notes ?? row.notes_text ?? "");
      setSelectedSuggestion(buildSavedSuggestion(review ?? null));
    },
    []
  );

  const loadSuggestionsForQuestion = useCallback(
    async (nextQuestion: string, review?: QuestionnaireReviewDto) => {
      clearSuggestions();

      const savedSuggestion = buildSavedSuggestion(review ?? null);
      setSelectedSuggestion(savedSuggestion);

      if (!nextQuestion.trim()) {
        return;
      }

      try {
        const nextSuggestions = await getMatches(nextQuestion, 10);
        if (!review?.answer_bank_entry_id) {
          return;
        }

        const matchedSuggestion = nextSuggestions.find(
          (suggestion) => suggestion.answer_bank_entry_id === review.answer_bank_entry_id
        );
        setSelectedSuggestion(matchedSuggestion ?? savedSuggestion);
      } catch (err) {
        console.error("Matching failed:", err);
      }
    },
    [clearSuggestions, getMatches]
  );

  const startRowReview = useCallback(
    async (row: QuestionnaireImportRowDto, review?: QuestionnaireReviewDto) => {
      syncWorkspaceFromRow(row, review);
      await loadSuggestionsForQuestion(review?.question_text ?? row.question_text, review);
    },
    [loadSuggestionsForQuestion, syncWorkspaceFromRow]
  );

  const loadManualReview = useCallback(
    async (review: QuestionnaireReviewDto) => {
      setSelectedRowOrdinal(review.source_row_ordinal ?? undefined);
      setEditingReviewId(review.review_id);
      setQuestion(review.question_text);
      setFinalAnswer(review.final_answer);
      setNotes(review.notes ?? "");
      await loadSuggestionsForQuestion(review.question_text, review);
    },
    [loadSuggestionsForQuestion]
  );

  const handleSearch = async () => {
    if (!question.trim()) {
      return;
    }

    setSelectedSuggestion(null);
    try {
      await getMatches(question, 10);
    } catch (err) {
      console.error("Matching failed:", err);
    }
  };

  const handleSelectSuggestion = (suggestion: MatchSuggestionDto) => {
    setSelectedSuggestion(suggestion);
    setFinalAnswer(suggestion.answer_long || suggestion.answer_short);
  };

  const handleSave = async () => {
    if (!question.trim() || !finalAnswer.trim()) {
      return;
    }

    const currentRowOrdinal = selectedRowOrdinal;
    const acceptedSuggestion =
      selectedSuggestion && finalAnswer.trim() === selectedSuggestionAnswer.trim();

    await saveReview({
      review_id: editingReviewId,
      import_id: importId,
      source_row_ordinal: selectedRowOrdinal,
      question_text: question,
      answer_bank_entry_id: selectedSuggestion?.answer_bank_entry_id,
      suggested_score: selectedSuggestion?.score,
      confidence_explanation: selectedSuggestion?.confidence_explanation,
      final_answer: finalAnswer,
      notes: notes.trim() || undefined,
      status: acceptedSuggestion ? "accepted_suggestion" : "edited_answer",
    });

    if (currentRowOrdinal != null) {
      const completedOrdinals = new Set(reviewsByRow.keys());
      completedOrdinals.add(currentRowOrdinal);

      const nextRow =
        importRows.find(
          (row) => row.row_ordinal > currentRowOrdinal && !completedOrdinals.has(row.row_ordinal)
        ) ?? importRows.find((row) => !completedOrdinals.has(row.row_ordinal));

      if (nextRow) {
        await startRowReview(nextRow, reviewsByRow.get(nextRow.row_ordinal));
        return;
      }
    }

    setEditingReviewId(undefined);
    setSelectedRowOrdinal(undefined);
    setQuestion("");
    setFinalAnswer("");
    setNotes("");
    setSelectedSuggestion(null);
    clearSuggestions();
  };

  const handleEdit = async (review: QuestionnaireReviewDto) => {
    if (review.source_row_ordinal != null) {
      const row = importRows.find((item) => item.row_ordinal === review.source_row_ordinal);
      if (row) {
        await startRowReview(row, review);
        return;
      }
    }

    await loadManualReview(review);
  };

  const handleStartFresh = () => {
    setEditingReviewId(undefined);
    setSelectedRowOrdinal(undefined);
    setQuestion("");
    setFinalAnswer("");
    setNotes("");
    setSelectedSuggestion(null);
    clearSuggestions();
  };

  const handleDelete = async (reviewId: string) => {
    if (confirm("Remove this saved review entry?")) {
      await deleteReview(reviewId);
    }
  };

  const handleKeyPress = (event: React.KeyboardEvent) => {
    if (event.key === "Enter") {
      void handleSearch();
    }
  };

  const getScoreColor = (score: number) => {
    if (score >= 0.7) return "text-green-600 font-semibold";
    if (score >= 0.4) return "text-yellow-600";
    return "text-muted-foreground";
  };

  return (
    <div className="space-y-8">
      <div className="rounded-xl border border-border bg-secondary/20 p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="mb-2 text-lg font-semibold">Review the imported questionnaire queue</h2>
            <p className="text-sm text-muted-foreground">
              Start from the mapped questionnaire rows saved in the vault, then confirm or refine
              the final answer that should be carried into export.
            </p>
          </div>
          <div className="rounded-lg border border-border bg-background px-4 py-3 text-sm">
            <p className="font-medium text-foreground">
              {reviewedRowCount} of {importRows.length || 0} mapped rows reviewed
            </p>
            <p className="text-muted-foreground">{reviews.length} saved review entries in vault</p>
          </div>
        </div>

        <div className="mt-5 rounded-xl border border-border bg-background p-4">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <p className="text-sm font-medium text-foreground">Review progress</p>
              <p className="text-sm text-muted-foreground">
                {completionPercent}% complete across mapped questionnaire rows
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button
                size="sm"
                variant={rowFilter === "all" ? "default" : "outline"}
                onClick={() => setRowFilter("all")}
              >
                All Rows
              </Button>
              <Button
                size="sm"
                variant={rowFilter === "pending" ? "default" : "outline"}
                onClick={() => setRowFilter("pending")}
              >
                Pending
              </Button>
              <Button
                size="sm"
                variant={rowFilter === "reviewed" ? "default" : "outline"}
                onClick={() => setRowFilter("reviewed")}
              >
                Reviewed
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => nextPendingRow && void startRowReview(nextPendingRow, reviewsByRow.get(nextPendingRow.row_ordinal))}
                disabled={!nextPendingRow}
              >
                {nextPendingRow ? `Jump to Row ${nextPendingRow.row_ordinal}` : "All Rows Reviewed"}
              </Button>
            </div>
          </div>
          <div className="mt-4 h-2 overflow-hidden rounded-full bg-secondary">
            <div
              className="h-full rounded-full bg-primary transition-all"
              style={{ width: `${completionPercent}%` }}
            />
          </div>
        </div>

        <div className="mt-6 space-y-4 rounded-xl border border-border bg-background p-5">
          <div className="flex items-start justify-between gap-4">
            <div>
              <h3 className="text-base font-semibold text-foreground">Imported questionnaire rows</h3>
              <p className="text-sm text-muted-foreground">
                Choose a row to preload the original question, imported answer, notes, and review
                history for this import.
              </p>
            </div>
            <div className="space-y-2">
              {selectedImportRow && (
                <div className="rounded-lg border border-primary/20 bg-primary/5 px-3 py-2 text-sm">
                  <p className="font-medium text-foreground">
                    Working from row {selectedImportRow.row_ordinal}
                  </p>
                  <p className="text-muted-foreground">
                    {reviewsByRow.has(selectedImportRow.row_ordinal)
                      ? "Existing review loaded"
                      : "Ready for first review"}
                  </p>
                </div>
              )}
              <p className="text-right text-xs text-muted-foreground">
                Showing {visibleRows.length} of {importRows.length} rows
              </p>
            </div>
          </div>

          {importRowsError && (
            <div className="rounded-md border border-destructive/20 bg-destructive/10 p-4">
              <p className="text-sm text-destructive">{importRowsError}</p>
            </div>
          )}

          {importRowsLoading ? (
            <div className="rounded-md border px-4 py-10 text-center text-sm text-muted-foreground">
              Loading imported questionnaire rows...
            </div>
          ) : importRows.length === 0 ? (
            <div className="rounded-md border border-dashed border-border px-4 py-10 text-center">
              <p className="text-sm text-muted-foreground">
                No mapped questionnaire rows are available yet. Save a valid column mapping to build
                the review queue.
              </p>
            </div>
          ) : visibleRows.length === 0 ? (
            <div className="rounded-md border border-dashed border-border px-4 py-10 text-center">
              <p className="text-sm text-muted-foreground">
                No rows match the current filter. Switch filters to continue reviewing the import.
              </p>
            </div>
          ) : (
            <div className="overflow-hidden rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[72px]">Row</TableHead>
                    <TableHead>Question</TableHead>
                    <TableHead>Imported Answer</TableHead>
                    <TableHead className="w-[140px]">Status</TableHead>
                    <TableHead className="w-[120px]">Action</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {visibleRows.map((row) => {
                    const rowReview = reviewsByRow.get(row.row_ordinal);
                    const isSelected = selectedRowOrdinal === row.row_ordinal;

                    return (
                      <TableRow
                        key={row.row_ordinal}
                        data-state={isSelected ? "selected" : undefined}
                      >
                        <TableCell className="font-medium text-foreground">
                          {row.row_ordinal}
                        </TableCell>
                        <TableCell>
                          <p className="font-medium text-foreground">
                            {summarizeText(row.question_text, 120)}
                          </p>
                          {row.notes_text && (
                            <p className="mt-1 text-xs text-muted-foreground">
                              Notes: {summarizeText(row.notes_text, 90)}
                            </p>
                          )}
                        </TableCell>
                        <TableCell className="text-sm text-muted-foreground">
                          {summarizeText(row.answer_text, 100)}
                        </TableCell>
                        <TableCell>
                          <span className="rounded-full bg-secondary px-2 py-1 text-xs uppercase tracking-wide text-muted-foreground">
                            {rowReview ? rowReview.status.replace("_", " ") : "pending"}
                          </span>
                        </TableCell>
                        <TableCell>
                          <Button
                            size="sm"
                            variant={isSelected ? "default" : "outline"}
                            onClick={() => void startRowReview(row, rowReview)}
                          >
                            {rowReview ? "Edit" : "Review"}
                          </Button>
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </div>
          )}
        </div>

        <div className="mt-6 grid gap-4 lg:grid-cols-[minmax(0,1fr)_220px]">
          <Input
            label="Question to review"
            value={question}
            onChange={(event) => setQuestion(event.target.value)}
            onKeyDown={handleKeyPress}
            placeholder="Select an imported row or paste a questionnaire prompt"
          />
          <div className="flex items-end gap-3">
            <Button
              className="flex-1"
              onClick={() => void handleSearch()}
              disabled={loading || !question.trim()}
            >
              {loading ? "Searching..." : "Refresh Matches"}
            </Button>
            <Button variant="outline" onClick={handleStartFresh} disabled={saving}>
              Reset
            </Button>
          </div>
        </div>

        {error && (
          <div className="mt-4 rounded-md border border-destructive/20 bg-destructive/10 p-4">
            <p className="text-sm text-destructive">{error}</p>
          </div>
        )}

        <div className="mt-6 grid gap-6 xl:grid-cols-[minmax(0,1.3fr)_minmax(0,1fr)]">
          <div className="space-y-4">
            <div>
              <h3 className="text-base font-semibold text-foreground">Suggestions</h3>
              <p className="text-sm text-muted-foreground">
                Choose a suggestion to seed the final answer, or keep the imported answer and edit
                it manually.
              </p>
            </div>

            {suggestions.length > 0 ? (
              <div className="overflow-hidden rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[90px]">Score</TableHead>
                      <TableHead>Matched Question</TableHead>
                      <TableHead>Answer Preview</TableHead>
                      <TableHead className="w-[120px]">Action</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {suggestions.map((suggestion) => {
                      const isSelected =
                        selectedSuggestion?.answer_bank_entry_id === suggestion.answer_bank_entry_id;

                      return (
                        <TableRow key={suggestion.answer_bank_entry_id}>
                          <TableCell>
                            <span className={getScoreColor(suggestion.score)}>
                              {(suggestion.score * 100).toFixed(0)}%
                            </span>
                          </TableCell>
                          <TableCell className="font-medium">
                            {suggestion.question_canonical}
                          </TableCell>
                          <TableCell className="text-sm text-muted-foreground">
                            {summarizeText(
                              suggestion.answer_long || suggestion.answer_short,
                              120
                            )}
                          </TableCell>
                          <TableCell>
                            <Button
                              size="sm"
                              variant={isSelected ? "default" : "outline"}
                              onClick={() => handleSelectSuggestion(suggestion)}
                            >
                              {isSelected ? "Selected" : "Use"}
                            </Button>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
            ) : (
              <div className="rounded-md border border-dashed border-border px-4 py-10 text-center">
                <p className="text-sm text-muted-foreground">
                  Load an imported question or search manually to populate suggestions here.
                </p>
              </div>
            )}
          </div>

          <div className="space-y-4 rounded-xl border border-border bg-background p-5">
            <div>
              <h3 className="text-base font-semibold text-foreground">
                {editingReviewId ? "Update review entry" : "Save review entry"}
              </h3>
              <p className="text-sm text-muted-foreground">
                The final answer below is what will be written into the review workspace export.
              </p>
            </div>

            {selectedImportRow && (
              <div className="rounded-lg border border-border bg-secondary/30 p-4 text-sm">
                <p className="font-medium text-foreground">
                  Source row {selectedImportRow.row_ordinal}
                </p>
                <p className="mt-1 text-muted-foreground">
                  Imported answer: {summarizeText(selectedImportRow.answer_text, 140)}
                </p>
              </div>
            )}

            {selectedSuggestion && (
              <div className="rounded-lg border border-primary/20 bg-primary/5 p-4 text-sm">
                <p className="font-medium text-foreground">Selected suggestion</p>
                <p className="mt-1 text-muted-foreground">
                  {selectedSuggestion.question_canonical} ·{" "}
                  {(selectedSuggestion.score * 100).toFixed(0)}% match
                </p>
                <p className="mt-2 text-muted-foreground">
                  {selectedSuggestion.confidence_explanation}
                </p>
              </div>
            )}

            <div>
              <label className="mb-2 block text-sm font-medium text-foreground">Final answer</label>
              <textarea
                className="min-h-[180px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={finalAnswer}
                onChange={(event) => setFinalAnswer(event.target.value)}
                placeholder="Write or refine the answer that should be carried forward"
              />
            </div>

            <div>
              <label className="mb-2 block text-sm font-medium text-foreground">Reviewer notes</label>
              <textarea
                className="min-h-[110px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={notes}
                onChange={(event) => setNotes(event.target.value)}
                placeholder="Optional notes for context, follow-up, or confidence"
              />
            </div>

            <Button
              className="w-full"
              onClick={() => void handleSave()}
              disabled={saving || !question.trim() || !finalAnswer.trim()}
            >
              {saving ? "Saving..." : editingReviewId ? "Update Review Entry" : "Save Review Entry"}
            </Button>
          </div>
        </div>
      </div>

      <section className="space-y-4">
        <div>
          <h3 className="text-base font-semibold text-foreground">Saved review entries</h3>
          <p className="text-sm text-muted-foreground">
            These saved entries are what the export pack will include for the current questionnaire
            import.
          </p>
        </div>

        {reviewsLoading ? (
          <div className="rounded-md border px-4 py-10 text-center text-sm text-muted-foreground">
            Loading saved review entries...
          </div>
        ) : reviews.length === 0 ? (
          <div className="rounded-md border border-dashed border-border px-4 py-10 text-center">
            <p className="text-sm text-muted-foreground">
              No review entries have been saved yet. Select an imported row, confirm the final
              answer, then save it here.
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {reviews.map((review) => (
              <article
                key={review.review_id}
                className="rounded-xl border border-border bg-background p-5"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="space-y-2">
                    <div className="flex flex-wrap items-center gap-2">
                      <h4 className="font-semibold text-foreground">{review.question_text}</h4>
                      {review.source_row_ordinal != null && (
                        <span className="rounded-full border border-border px-2 py-1 text-xs text-muted-foreground">
                          Row {review.source_row_ordinal}
                        </span>
                      )}
                      <span className="rounded-full bg-secondary px-2 py-1 text-xs uppercase tracking-wide text-muted-foreground">
                        {review.status.replace("_", " ")}
                      </span>
                    </div>
                    {review.confidence_explanation && (
                      <p className="text-sm text-muted-foreground">
                        {review.confidence_explanation}
                      </p>
                    )}
                    <p className="whitespace-pre-wrap text-sm text-foreground">
                      {review.final_answer}
                    </p>
                    {review.notes && (
                      <p className="text-sm text-muted-foreground">Notes: {review.notes}</p>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <Button variant="outline" size="sm" onClick={() => void handleEdit(review)}>
                      Edit
                    </Button>
                    <Button
                      variant="destructive"
                      size="sm"
                      onClick={() => void handleDelete(review.review_id)}
                      disabled={saving}
                    >
                      Remove
                    </Button>
                  </div>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

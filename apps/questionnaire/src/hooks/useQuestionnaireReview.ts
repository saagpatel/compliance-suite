import { useCallback, useEffect, useState } from "react";
import type { QuestionnaireReviewDto, QuestionnaireReviewUpsertDto } from "@packages/types";
import {
  invokeDeleteQuestionnaireReview,
  invokeListQuestionnaireReviews,
  invokeSaveQuestionnaireReview,
} from "../api/tauri";
import { useUiStore } from "../state/uiStore";

export function useQuestionnaireReview(importId?: string) {
  const [reviews, setReviews] = useState<QuestionnaireReviewDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const addToast = useUiStore((state) => state.addToast);

  const loadReviews = useCallback(async () => {
    if (!importId) {
      setReviews([]);
      return [];
    }

    setLoading(true);
    setError(null);
    try {
      const result = await invokeListQuestionnaireReviews(importId);
      setReviews(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      addToast({
        title: "Failed to Load Reviews",
        description: message,
        variant: "destructive",
      });
      throw err;
    } finally {
      setLoading(false);
    }
  }, [addToast, importId]);

  useEffect(() => {
    void loadReviews();
  }, [loadReviews]);

  const saveReview = useCallback(
    async (input: QuestionnaireReviewUpsertDto) => {
      setSaving(true);
      setError(null);
      try {
        const saved = await invokeSaveQuestionnaireReview(input);
        setReviews((current) => {
          const next = current.filter((review) => review.review_id !== saved.review_id);
          next.unshift(saved);
          return next;
        });
        addToast({
          title: "Review Saved",
          description: "The questionnaire review workspace has been updated.",
          variant: "success",
        });
        return saved;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        addToast({
          title: "Failed to Save Review",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        setSaving(false);
      }
    },
    [addToast]
  );

  const deleteReview = useCallback(
    async (reviewId: string) => {
      setSaving(true);
      setError(null);
      try {
        await invokeDeleteQuestionnaireReview(reviewId);
        setReviews((current) => current.filter((review) => review.review_id !== reviewId));
        addToast({
          title: "Review Removed",
          description: "The saved review entry has been removed.",
          variant: "default",
        });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        addToast({
          title: "Failed to Remove Review",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        setSaving(false);
      }
    },
    [addToast]
  );

  return {
    reviews,
    loading,
    saving,
    error,
    loadReviews,
    saveReview,
    deleteReview,
  };
}

import { useCallback, useEffect, useState } from "react";
import type { QuestionnaireImportRowDto } from "@packages/types";
import { invokeListImportRows } from "../api/tauri";
import { useUiStore } from "../state/uiStore";

export function useQuestionnaireImportRows(importId?: string) {
  const [rows, setRows] = useState<QuestionnaireImportRowDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const addToast = useUiStore((state) => state.addToast);

  const loadRows = useCallback(async () => {
    if (!importId) {
      setRows([]);
      return [];
    }

    setLoading(true);
    setError(null);
    try {
      const result = await invokeListImportRows(importId);
      setRows(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      addToast({
        title: "Failed to Load Imported Questions",
        description: message,
        variant: "destructive",
      });
      throw err;
    } finally {
      setLoading(false);
    }
  }, [addToast, importId]);

  useEffect(() => {
    void loadRows();
  }, [loadRows]);

  return {
    rows,
    loading,
    error,
    loadRows,
  };
}

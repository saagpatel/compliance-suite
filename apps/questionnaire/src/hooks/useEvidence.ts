import { useCallback, useEffect, useState } from "react";
import type { EvidenceDto } from "@packages/types";
import { invokeImportEvidence, invokeListEvidence } from "../api/tauri";
import { useUiStore } from "../state/uiStore";

export function useEvidence() {
  const [evidence, setEvidence] = useState<EvidenceDto[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const addToast = useUiStore((state) => state.addToast);

  const loadEvidence = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invokeListEvidence();
      setEvidence(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      addToast({
        title: "Failed to Load Evidence",
        description: message,
        variant: "destructive",
      });
      throw err;
    } finally {
      setLoading(false);
    }
  }, [addToast]);

  useEffect(() => {
    void loadEvidence();
  }, [loadEvidence]);

  const importEvidence = useCallback(
    async (filePath: string) => {
      setLoading(true);
      setError(null);
      try {
        const saved = await invokeImportEvidence(filePath);
        setEvidence((current) => [saved, ...current.filter((item) => item.evidence_id !== saved.evidence_id)]);
        addToast({
          title: "Evidence Imported",
          description: `${saved.filename} is now available in the vault evidence browser.`,
          variant: "success",
        });
        return saved;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        addToast({
          title: "Failed to Import Evidence",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [addToast]
  );

  return {
    evidence,
    loading,
    error,
    loadEvidence,
    importEvidence,
  };
}

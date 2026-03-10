import { useCallback, useEffect, useState } from "react";
import type { LicenseStatusDto } from "@packages/types";
import { invokeCheckLicenseStatus } from "../api/tauri";
import { useUiStore } from "../state/uiStore";

export function useLicenseStatus(enabled = true) {
  const [status, setStatus] = useState<LicenseStatusDto | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const addToast = useUiStore((state) => state.addToast);

  const loadStatus = useCallback(async () => {
    if (!enabled) {
      setStatus(null);
      return null;
    }

    setLoading(true);
    setError(null);
    try {
      const result = await invokeCheckLicenseStatus();
      setStatus(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      addToast({
        title: "Failed to Check License",
        description: message,
        variant: "destructive",
      });
      throw err;
    } finally {
      setLoading(false);
    }
  }, [addToast, enabled]);

  useEffect(() => {
    void loadStatus();
  }, [loadStatus]);

  return {
    status,
    loading,
    error,
    refresh: loadStatus,
  };
}

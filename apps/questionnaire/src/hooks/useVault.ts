import { useCallback } from "react";
import { invokeVaultClose, invokeVaultCreate, invokeVaultOpen } from "../api/tauri";
import { useUiStore } from "../state/uiStore";
import { useVaultStore } from "../state/vaultStore";

export function useVault() {
  const currentVault = useVaultStore((state) => state.currentVault);
  const lastVaultPath = useVaultStore((state) => state.lastVaultPath);
  const loading = useVaultStore((state) => state.loading);
  const error = useVaultStore((state) => state.error);
  const initializing = useVaultStore((state) => state.initializing);
  const initialized = useVaultStore((state) => state.initialized);
  const setCurrentVault = useVaultStore((state) => state.setCurrentVault);
  const setLastVaultPath = useVaultStore((state) => state.setLastVaultPath);
  const setLoading = useVaultStore((state) => state.setLoading);
  const setError = useVaultStore((state) => state.setError);
  const setInitializing = useVaultStore((state) => state.setInitializing);
  const setInitialized = useVaultStore((state) => state.setInitialized);
  const clearCurrentVault = useVaultStore((state) => state.clearCurrentVault);
  const addToast = useUiStore((state) => state.addToast);

  const createVault = useCallback(
    async (path: string, name: string) => {
      setLoading(true);
      setError(null);
      try {
        const vault = await invokeVaultCreate(path, name);
        setCurrentVault(vault);
        setLastVaultPath(path);
        setInitialized(true);
        addToast({
          title: "Vault Created",
          description: `Vault "${name}" created successfully`,
          variant: "success",
        });
        return vault;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        addToast({
          title: "Failed to Create Vault",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [addToast, setCurrentVault, setError, setInitialized, setLastVaultPath, setLoading]
  );

  const openVault = useCallback(
    async (path: string) => {
      setLoading(true);
      setError(null);
      try {
        const vault = await invokeVaultOpen(path);
        setCurrentVault(vault);
        setLastVaultPath(path);
        setInitialized(true);
        addToast({
          title: "Vault Opened",
          description: `Vault "${vault.name}" opened successfully`,
          variant: "success",
        });
        return vault;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        setError(message);
        addToast({
          title: "Failed to Open Vault",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [addToast, setCurrentVault, setError, setInitialized, setLastVaultPath, setLoading]
  );

  const closeVault = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await invokeVaultClose();
      clearCurrentVault();
      addToast({
        title: "Vault Closed",
        description: "Vault closed successfully",
        variant: "default",
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      addToast({
        title: "Failed to Close Vault",
        description: message,
        variant: "destructive",
      });
      throw err;
    } finally {
      setLoading(false);
    }
  }, [addToast, clearCurrentVault, setError, setLoading]);

  const bootstrapVault = useCallback(async () => {
    const state = useVaultStore.getState();

    if (state.currentVault || state.initializing || state.initialized) {
      if (!state.initialized) {
        setInitialized(true);
      }
      return state.currentVault;
    }

    setInitializing(true);
    setError(null);

    try {
      if (!state.lastVaultPath) {
        return null;
      }

      const vault = await invokeVaultOpen(state.lastVaultPath);
      setCurrentVault(vault);
      addToast({
        title: "Recent Vault Reopened",
        description: `Reconnected to "${vault.name}" automatically`,
        variant: "success",
      });
      return vault;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      clearCurrentVault();
      setLastVaultPath(null);
      setError(message);
      addToast({
        title: "Recent Vault Needs Attention",
        description: "The last vault could not be reopened. Choose a vault to continue.",
        variant: "destructive",
      });
      return null;
    } finally {
      setInitializing(false);
      setInitialized(true);
    }
  }, [
    addToast,
    clearCurrentVault,
    setCurrentVault,
    setError,
    setInitialized,
    setInitializing,
    setLastVaultPath,
  ]);

  return {
    currentVault,
    lastVaultPath,
    loading,
    error,
    initializing,
    initialized,
    createVault,
    openVault,
    closeVault,
    bootstrapVault,
  };
}

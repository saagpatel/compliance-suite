import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import type { VaultDto } from "@packages/types";

interface VaultState {
  currentVault: VaultDto | null;
  lastVaultPath: string | null;
  loading: boolean;
  error: string | null;
  initializing: boolean;
  initialized: boolean;
  setCurrentVault: (vault: VaultDto | null) => void;
  setLastVaultPath: (path: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setInitializing: (initializing: boolean) => void;
  setInitialized: (initialized: boolean) => void;
  clearCurrentVault: () => void;
}

export const useVaultStore = create<VaultState>()(
  persist(
    (set) => ({
      currentVault: null,
      lastVaultPath: null,
      loading: false,
      error: null,
      initializing: false,
      initialized: false,

      setCurrentVault: (currentVault) => set({ currentVault }),
      setLastVaultPath: (lastVaultPath) => set({ lastVaultPath }),
      setLoading: (loading) => set({ loading }),
      setError: (error) => set({ error }),
      setInitializing: (initializing) => set({ initializing }),
      setInitialized: (initialized) => set({ initialized }),
      clearCurrentVault: () => set({ currentVault: null }),
    }),
    {
      name: "questionnaire-vault",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        lastVaultPath: state.lastVaultPath,
      }),
    }
  )
);

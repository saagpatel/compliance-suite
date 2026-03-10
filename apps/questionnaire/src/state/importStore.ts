import { create } from "zustand";
import type { QuestionnaireImportDto, ColumnMapDto } from "@packages/types";
import type { ColumnProfileDto } from "../api/tauri";

interface ImportState {
  // Import data
  currentImport: QuestionnaireImportDto | null;
  columnProfiles: ColumnProfileDto[];
  columnMap: ColumnMapDto | null;

  // UI state
  loading: boolean;
  error: string | null;
  step: "import" | "map" | "answer-bank" | "review" | "export";

  // Actions
  setCurrentImport: (importData: QuestionnaireImportDto | null) => void;
  setColumnProfiles: (profiles: ColumnProfileDto[]) => void;
  setColumnMap: (map: ColumnMapDto | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setStep: (step: "import" | "map" | "answer-bank" | "review" | "export") => void;
  reset: () => void;
}

const initialState = {
  currentImport: null,
  columnProfiles: [],
  columnMap: null,
  loading: false,
  error: null,
  step: "import" as const,
};

export const useImportStore = create<ImportState>((set) => ({
  ...initialState,

  setCurrentImport: (currentImport) => set({ currentImport }),
  setColumnProfiles: (columnProfiles) => set({ columnProfiles }),
  setColumnMap: (columnMap) => set({ columnMap }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
  setStep: (step) => set({ step }),
  reset: () => set(initialState),
}));

import { create } from "zustand";
import type { AnswerBankEntryDto } from "@packages/types";

interface AnswerBankState {
  // Answer bank entries
  entries: AnswerBankEntryDto[];
  selectedEntry: AnswerBankEntryDto | null;

  // Pagination
  total: number;
  limit: number;
  offset: number;
  searchQuery: string;

  // UI state
  loading: boolean;
  error: string | null;

  // Actions
  setEntries: (entries: AnswerBankEntryDto[]) => void;
  addEntry: (entry: AnswerBankEntryDto) => void;
  updateEntry: (entryId: string, updates: Partial<AnswerBankEntryDto>) => void;
  removeEntry: (entryId: string) => void;
  setSelectedEntry: (entry: AnswerBankEntryDto | null) => void;
  setTotal: (total: number) => void;
  setLimit: (limit: number) => void;
  setOffset: (offset: number) => void;
  setSearchQuery: (query: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialState = {
  entries: [],
  selectedEntry: null,
  total: 0,
  limit: 50,
  offset: 0,
  searchQuery: "",
  loading: false,
  error: null,
};

export const useAnswerBankStore = create<AnswerBankState>((set) => ({
  ...initialState,

  setEntries: (entries) => set({ entries }),
  addEntry: (entry) => set((state) => ({ entries: [...state.entries, entry] })),
  updateEntry: (entryId, updates) =>
    set((state) => ({
      entries: state.entries.map((e) => (e.entry_id === entryId ? { ...e, ...updates } : e)),
    })),
  removeEntry: (entryId) =>
    set((state) => ({
      entries: state.entries.filter((e) => e.entry_id !== entryId),
    })),
  setSelectedEntry: (selectedEntry) => set({ selectedEntry }),
  setTotal: (total) => set({ total }),
  setLimit: (limit) => set({ limit }),
  setOffset: (offset) => set({ offset }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
  reset: () => set(initialState),
}));

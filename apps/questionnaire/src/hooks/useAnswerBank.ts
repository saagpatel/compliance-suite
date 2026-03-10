import { useCallback } from "react";
import { useAnswerBankStore } from "../state/answerBankStore";
import { useUiStore } from "../state/uiStore";
import {
  invokeAnswerBankCreate,
  invokeAnswerBankDelete,
  invokeAnswerBankLinkEvidence,
  invokeAnswerBankList,
  invokeAnswerBankSearch,
  invokeAnswerBankUpdate,
} from "../api/tauri";
import type { AnswerBankCreateInputDto, AnswerBankUpdatePatchDto } from "@packages/types";

export function useAnswerBank() {
  const store = useAnswerBankStore();
  const addToast = useUiStore((state) => state.addToast);

  const loadEntries = useCallback(
    async (limit?: number, offset?: number) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const entries = await invokeAnswerBankList(limit ?? store.limit, offset ?? store.offset);
        store.setEntries(entries);
        store.setSearchQuery("");
        return entries;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        addToast({
          title: "Failed to Load Entries",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, addToast]
  );

  const searchEntries = useCallback(
    async (query: string, limit?: number, offset?: number) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const entries = await invokeAnswerBankSearch(
          query,
          limit ?? store.limit,
          offset ?? store.offset
        );
        store.setEntries(entries);
        store.setSearchQuery(query);
        return entries;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        addToast({
          title: "Failed to Search Entries",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, addToast]
  );

  const createEntry = useCallback(
    async (input: AnswerBankCreateInputDto) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const entry = await invokeAnswerBankCreate(input);
        store.addEntry(entry);
        addToast({
          title: "Entry Created",
          description: "Answer bank entry created successfully",
          variant: "success",
        });
        return entry;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        addToast({
          title: "Failed to Create Entry",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, addToast]
  );

  const updateEntry = useCallback(
    async (entryId: string, patch: AnswerBankUpdatePatchDto) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const entry = await invokeAnswerBankUpdate(entryId, patch);
        store.updateEntry(entryId, entry);
        addToast({
          title: "Entry Updated",
          description: "Answer bank entry updated successfully",
          variant: "success",
        });
        return entry;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        addToast({
          title: "Failed to Update Entry",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, addToast]
  );

  const deleteEntry = useCallback(
    async (entryId: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        await invokeAnswerBankDelete(entryId);
        store.removeEntry(entryId);
        addToast({
          title: "Entry Deleted",
          description: "Answer bank entry deleted successfully",
          variant: "success",
        });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        addToast({
          title: "Failed to Delete Entry",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, addToast]
  );

  const linkEvidence = useCallback(
    async (entryId: string, evidenceId: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const entry = await invokeAnswerBankLinkEvidence(entryId, evidenceId);
        store.updateEntry(entryId, entry);
        addToast({
          title: "Evidence Linked",
          description: `Evidence ${evidenceId} linked to the entry.`,
          variant: "success",
        });
        return entry;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        addToast({
          title: "Failed to Link Evidence",
          description: message,
          variant: "destructive",
        });
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, addToast]
  );

  return {
    entries: store.entries,
    selectedEntry: store.selectedEntry,
    total: store.total,
    limit: store.limit,
    offset: store.offset,
    searchQuery: store.searchQuery,
    loading: store.loading,
    error: store.error,
    loadEntries,
    searchEntries,
    createEntry,
    updateEntry,
    deleteEntry,
    linkEvidence,
    setSelectedEntry: store.setSelectedEntry,
    setLimit: store.setLimit,
    setOffset: store.setOffset,
  };
}

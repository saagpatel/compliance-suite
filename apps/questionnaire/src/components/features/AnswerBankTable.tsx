import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import type { AnswerBankCreateInputDto, AnswerBankEntryDto, AnswerBankUpdatePatchDto } from "@packages/types";
import { useAnswerBank } from "../../hooks/useAnswerBank";
import { useEvidence } from "../../hooks/useEvidence";
import Button from "../ui/Button";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "../ui/Dialog";
import Form, { FormField } from "../ui/Form";
import Input from "../ui/Input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../ui/Table";

type EntryFormState = {
  question_canonical: string;
  answer_short: string;
  answer_long: string;
  notes: string;
  tags: string;
  owner: string;
  evidence_links: string;
};

const emptyForm: EntryFormState = {
  question_canonical: "",
  answer_short: "",
  answer_long: "",
  notes: "",
  tags: "",
  owner: "user",
  evidence_links: "",
};

function buildFormState(entry?: AnswerBankEntryDto | null): EntryFormState {
  if (!entry) {
    return emptyForm;
  }

  return {
    question_canonical: entry.question_canonical,
    answer_short: entry.answer_short,
    answer_long: entry.answer_long,
    notes: entry.notes ?? "",
    tags: entry.tags.join(", "),
    owner: entry.owner,
    evidence_links: entry.evidence_links.join(", "),
  };
}

function parseCommaSeparated(value: string) {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

export default function AnswerBankTable() {
  const {
    entries,
    loading,
    error,
    loadEntries,
    searchEntries,
    createEntry,
    updateEntry,
    deleteEntry,
    linkEvidence,
    setOffset,
    offset,
    limit,
    searchQuery,
  } = useAnswerBank();
  const {
    evidence,
    loading: evidenceLoading,
    error: evidenceError,
    importEvidence,
    loadEvidence,
  } = useEvidence();
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [editingEntry, setEditingEntry] = useState<AnswerBankEntryDto | null>(null);
  const [formData, setFormData] = useState<EntryFormState>(emptyForm);
  const [searchInput, setSearchInput] = useState("");
  const [linkingEvidenceId, setLinkingEvidenceId] = useState<Record<string, string>>({});
  const [importingEvidence, setImportingEvidence] = useState(false);

  useEffect(() => {
    void loadEntries();
  }, [loadEntries]);

  useEffect(() => {
    setSearchInput(searchQuery);
  }, [searchQuery]);

  const dialogTitle = useMemo(
    () => (editingEntry ? "Edit Answer Bank Entry" : "Create Answer Bank Entry"),
    [editingEntry]
  );
  const evidenceById = useMemo(
    () => new Map(evidence.map((item) => [item.evidence_id, item])),
    [evidence]
  );
  const recentEvidence = useMemo(() => evidence.slice(0, 6), [evidence]);

  const resetDialog = () => {
    setEditingEntry(null);
    setFormData(emptyForm);
    setIsCreateDialogOpen(false);
  };

  const openCreateDialog = () => {
    setEditingEntry(null);
    setFormData(emptyForm);
    setIsCreateDialogOpen(true);
  };

  const openEditDialog = (entry: AnswerBankEntryDto) => {
    setEditingEntry(entry);
    setFormData(buildFormState(entry));
    setIsCreateDialogOpen(true);
  };

  const handleSubmit = async () => {
    const common = {
      question_canonical: formData.question_canonical,
      answer_short: formData.answer_short,
      answer_long: formData.answer_long,
      notes: formData.notes || undefined,
      owner: formData.owner,
      tags: parseCommaSeparated(formData.tags),
    };

      try {
      if (editingEntry) {
        const patch: AnswerBankUpdatePatchDto = {
          question_canonical: formData.question_canonical,
          answer_short: formData.answer_short,
          answer_long: formData.answer_long,
          notes: formData.notes ? formData.notes : null,
          owner: formData.owner,
          tags: parseCommaSeparated(formData.tags),
          evidence_links: parseCommaSeparated(formData.evidence_links),
        };
        await updateEntry(editingEntry.entry_id, patch);
      } else {
        const input: AnswerBankCreateInputDto = {
          ...common,
          evidence_links: parseCommaSeparated(formData.evidence_links),
          last_reviewed_at: undefined,
          source: "manual",
        };
        await createEntry(input);
      }

      resetDialog();
    } catch (err) {
      console.error("Failed to save answer bank entry:", err);
    }
  };

  const handleDelete = async (entryId: string) => {
    if (confirm("Are you sure you want to delete this entry?")) {
      try {
        await deleteEntry(entryId);
      } catch (err) {
        console.error("Failed to delete entry:", err);
      }
    }
  };

  const handleSearch = async () => {
    setOffset(0);
    if (!searchInput.trim()) {
      await loadEntries(limit, 0);
      return;
    }
    await searchEntries(searchInput.trim(), limit, 0);
  };

  const handleNextPage = async () => {
    const nextOffset = offset + limit;
    setOffset(nextOffset);
    if (searchQuery.trim()) {
      await searchEntries(searchQuery, limit, nextOffset);
      return;
    }
    await loadEntries(limit, nextOffset);
  };

  const handlePreviousPage = async () => {
    const newOffset = Math.max(0, offset - limit);
    setOffset(newOffset);
    if (searchQuery.trim()) {
      await searchEntries(searchQuery, limit, newOffset);
      return;
    }
    await loadEntries(limit, newOffset);
  };

  const handleLinkEvidence = async (entryId: string) => {
    const evidenceId = linkingEvidenceId[entryId]?.trim();
    if (!evidenceId) {
      return;
    }

    try {
      await linkEvidence(entryId, evidenceId);
      setLinkingEvidenceId((current) => ({ ...current, [entryId]: "" }));
    } catch (err) {
      console.error("Failed to link evidence:", err);
    }
  };

  const handleImportEvidence = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: [
        {
          name: "Evidence Files",
          extensions: ["pdf", "png", "jpg", "jpeg", "csv", "xlsx", "txt", "docx"],
        },
      ],
    });

    const selectedPaths =
      typeof selected === "string" ? [selected] : Array.isArray(selected) ? selected : [];

    if (selectedPaths.length === 0) {
      return;
    }

    setImportingEvidence(true);
    try {
      for (const filePath of selectedPaths) {
        await importEvidence(filePath);
      }
    } catch (err) {
      console.error("Failed to import evidence:", err);
    } finally {
      setImportingEvidence(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h2 className="text-lg font-semibold">Answer Bank</h2>
          <p className="text-sm text-muted-foreground">
            Search, edit, and link evidence to the entries that power review suggestions.
          </p>
        </div>
        <div className="flex flex-1 flex-col gap-3 lg:max-w-2xl lg:flex-row">
          <Input
            value={searchInput}
            onChange={(event) => setSearchInput(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                void handleSearch();
              }
            }}
            placeholder="Search questions and answers"
            className="flex-1"
          />
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => void handleSearch()} disabled={loading}>
              Search
            </Button>
            <Button variant="outline" onClick={() => {
              setSearchInput("");
              setOffset(0);
              void loadEntries(limit, 0);
            }} disabled={loading && !searchQuery}>
              Clear
            </Button>
            <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
              <DialogTrigger asChild>
                <Button onClick={openCreateDialog}>Add Entry</Button>
              </DialogTrigger>
              <DialogContent className="max-w-2xl">
                <DialogHeader>
                  <DialogTitle>{dialogTitle}</DialogTitle>
                </DialogHeader>
                <Form className="space-y-4" onSubmit={(event) => event.preventDefault()}>
                  <FormField label="Question" required>
                    <Input
                      value={formData.question_canonical}
                      onChange={(event) =>
                        setFormData({ ...formData, question_canonical: event.target.value })
                      }
                      placeholder="Enter the canonical question"
                    />
                  </FormField>
                  <FormField label="Short Answer" required>
                    <Input
                      value={formData.answer_short}
                      onChange={(event) =>
                        setFormData({ ...formData, answer_short: event.target.value })
                      }
                      placeholder="Brief answer"
                    />
                  </FormField>
                  <FormField label="Long Answer" required>
                    <textarea
                      className="min-h-[120px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                      value={formData.answer_long}
                      onChange={(event) =>
                        setFormData({ ...formData, answer_long: event.target.value })
                      }
                      placeholder="Detailed answer"
                    />
                  </FormField>
                  <div className="grid gap-4 md:grid-cols-2">
                    <FormField label="Owner">
                      <Input
                        value={formData.owner}
                        onChange={(event) => setFormData({ ...formData, owner: event.target.value })}
                        placeholder="user"
                      />
                    </FormField>
                    <FormField label="Tags (comma-separated)">
                      <Input
                        value={formData.tags}
                        onChange={(event) => setFormData({ ...formData, tags: event.target.value })}
                        placeholder="security, privacy"
                      />
                    </FormField>
                  </div>
                  <FormField label="Evidence IDs (comma-separated)">
                    <Input
                      value={formData.evidence_links}
                      onChange={(event) =>
                        setFormData({ ...formData, evidence_links: event.target.value })
                      }
                      placeholder="ev_123, ev_456"
                    />
                  </FormField>
                  <FormField label="Notes">
                    <textarea
                      className="min-h-[90px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                      value={formData.notes}
                      onChange={(event) => setFormData({ ...formData, notes: event.target.value })}
                      placeholder="Optional notes"
                    />
                  </FormField>
                  <div className="flex gap-3 pt-2">
                    <Button onClick={() => void handleSubmit()} disabled={loading}>
                      {loading
                        ? "Saving..."
                        : editingEntry
                          ? "Save Changes"
                          : "Create Entry"}
                    </Button>
                    <Button variant="outline" onClick={resetDialog} disabled={loading}>
                      Cancel
                    </Button>
                  </div>
                </Form>
              </DialogContent>
            </Dialog>
          </div>
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-destructive/20 bg-destructive/10 p-4">
          <p className="text-sm text-destructive">{error}</p>
        </div>
      )}

      <section className="rounded-xl border border-border bg-secondary/20 p-5">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h3 className="text-base font-semibold text-foreground">Vault evidence browser</h3>
            <p className="text-sm text-muted-foreground">
              Use real evidence records when linking answer bank entries so reviewers can understand
              what a linked ID actually points to.
            </p>
          </div>
          <div className="rounded-lg border border-border bg-background px-4 py-3 text-sm">
            <p className="font-medium text-foreground">
              {evidenceLoading ? "Loading evidence..." : `${evidence.length} evidence files available`}
            </p>
            <p className="text-muted-foreground">Most recent vault evidence is listed below</p>
          </div>
        </div>

        <div className="mt-4 flex flex-wrap gap-3">
          <Button onClick={() => void handleImportEvidence()} disabled={importingEvidence || loading}>
            {importingEvidence ? "Importing Evidence..." : "Import Evidence Files"}
          </Button>
          <Button variant="outline" onClick={() => void loadEvidence()} disabled={evidenceLoading}>
            {evidenceLoading ? "Refreshing..." : "Refresh Evidence List"}
          </Button>
        </div>

        {evidenceError && (
          <div className="mt-4 rounded-md border border-destructive/20 bg-destructive/10 p-4">
            <p className="text-sm text-destructive">{evidenceError}</p>
          </div>
        )}

        {evidence.length === 0 ? (
          <div className="mt-4 rounded-md border border-dashed border-border px-4 py-8 text-center">
            <p className="text-sm text-muted-foreground">
              No evidence files are available in this vault yet. Linked evidence IDs will resolve
              here once evidence has been imported.
            </p>
          </div>
        ) : (
          <div className="mt-4 grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
            {recentEvidence.map((item) => (
              <article key={item.evidence_id} className="rounded-lg border border-border bg-background p-4">
                <div className="space-y-2">
                  <div>
                    <p className="font-medium text-foreground">{item.filename}</p>
                    <p className="text-xs text-muted-foreground">{item.relative_path}</p>
                  </div>
                  <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
                    <span className="rounded bg-secondary px-2 py-1">{item.evidence_id}</span>
                    <span>{item.content_type}</span>
                    <span>{formatBytes(item.byte_size)}</span>
                  </div>
                  {item.notes && <p className="text-xs text-muted-foreground">Notes: {item.notes}</p>}
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Question</TableHead>
              <TableHead>Answer</TableHead>
              <TableHead>Tags</TableHead>
              <TableHead>Evidence</TableHead>
              <TableHead>Owner</TableHead>
              <TableHead className="w-[220px]">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading && entries.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="py-8 text-center text-muted-foreground">
                  Loading...
                </TableCell>
              </TableRow>
            ) : entries.length === 0 ? (
              <TableRow>
                <TableCell colSpan={6} className="py-8 text-center text-muted-foreground">
                  No entries found. Add an entry or broaden your search.
                </TableCell>
              </TableRow>
            ) : (
              entries.map((entry) => (
                <TableRow key={entry.entry_id}>
                  <TableCell className="font-medium">
                    <div className="space-y-1">
                      <p>{entry.question_canonical}</p>
                      {entry.notes && (
                        <p className="text-xs text-muted-foreground">Notes: {entry.notes}</p>
                      )}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="space-y-1">
                      <p>{entry.answer_short}</p>
                      <p className="line-clamp-2 text-xs text-muted-foreground">
                        {entry.answer_long}
                      </p>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap gap-1">
                      {entry.tags.length === 0 ? (
                        <span className="text-xs text-muted-foreground">No tags</span>
                      ) : (
                        entry.tags.map((tag) => (
                          <span key={tag} className="rounded bg-secondary px-2 py-1 text-xs">
                            {tag}
                          </span>
                        ))
                      )}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="space-y-2">
                      <div className="flex flex-wrap gap-1">
                        {entry.evidence_links.length === 0 ? (
                          <span className="text-xs text-muted-foreground">No evidence linked</span>
                        ) : (
                          entry.evidence_links.map((evidenceId) => (
                            <div
                              key={evidenceId}
                              className="rounded-md border border-border bg-secondary/30 px-2 py-1 text-xs"
                            >
                              <p className="font-medium text-foreground">{evidenceId}</p>
                              {evidenceById.has(evidenceId) ? (
                                <>
                                  <p className="text-muted-foreground">
                                    {evidenceById.get(evidenceId)?.filename}
                                  </p>
                                  <p className="text-muted-foreground">
                                    {evidenceById.get(evidenceId)?.relative_path}
                                  </p>
                                </>
                              ) : (
                                <p className="text-muted-foreground">
                                  Linked ID is not present in the current vault evidence list.
                                </p>
                              )}
                            </div>
                          ))
                        )}
                      </div>
                      <div className="flex gap-2">
                        <select
                          value={linkingEvidenceId[entry.entry_id] ?? ""}
                          onChange={(event) =>
                            setLinkingEvidenceId((current) => ({
                              ...current,
                              [entry.entry_id]: event.target.value,
                            }))
                          }
                          className="h-9 flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm"
                        >
                          <option value="">Select vault evidence</option>
                          {evidence.map((item) => (
                            <option key={item.evidence_id} value={item.evidence_id}>
                              {item.evidence_id} - {item.filename}
                            </option>
                          ))}
                        </select>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => void handleLinkEvidence(entry.entry_id)}
                          disabled={loading || evidence.length === 0}
                        >
                          Link
                        </Button>
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>{entry.owner}</TableCell>
                  <TableCell>
                    <div className="flex gap-2">
                      <Button variant="outline" size="sm" onClick={() => openEditDialog(entry)}>
                        Edit
                      </Button>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => void handleDelete(entry.entry_id)}
                      >
                        Delete
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Showing {entries.length === 0 ? 0 : offset + 1} - {Math.min(offset + limit, offset + entries.length)}
          {searchQuery ? ` for "${searchQuery}"` : ""}
        </p>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => void handlePreviousPage()} disabled={offset === 0}>
            Previous
          </Button>
          <Button variant="outline" onClick={() => void handleNextPage()} disabled={entries.length < limit}>
            Next
          </Button>
        </div>
      </div>
    </div>
  );
}

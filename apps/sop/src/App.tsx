import { useEffect, useMemo, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  assignSopAcknowledgments,
  checkLicenseStatus,
  closeVault,
  createSopDocument,
  createVault,
  decideSopApproval,
  generateExportPack,
  installLicense,
  listSopAcknowledgments,
  listSopApprovalSteps,
  listSopDocuments,
  listSopVersions,
  openVault,
  publishSopDocument,
  recordSopAcknowledgment,
  submitSopForApproval,
  type SopAcknowledgmentDto,
  type SopApprovalStepDto,
  type SopApprovalSubmitInputDto,
  type ExportPackDto,
  type LicenseStatusDto,
  SopAppError,
  type SopDocumentCreateInputDto,
  type SopDocumentDto,
  type SopVersionDto,
  type VaultDto,
  updateSopDocument
} from "./api";
import { SOP_LAST_VAULT_KEY } from "./state";

type DocumentFormState = {
  title: string;
  slug: string;
  owner: string;
  body_markdown: string;
  change_summary: string;
};

type VersionFormState = {
  body_markdown: string;
  change_summary: string;
};

const emptyDocumentForm: DocumentFormState = {
  title: "",
  slug: "",
  owner: "operations",
  body_markdown: "# Procedure\n\nDescribe the operating procedure here.",
  change_summary: "Initial draft"
};

const emptyVersionForm: VersionFormState = {
  body_markdown: "",
  change_summary: ""
};

function toSlug(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-|-$/g, "");
}

function splitPeopleList(value: string) {
  return value
    .split(/[,\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

export default function App() {
  const [vault, setVault] = useState<VaultDto | null>(null);
  const [documents, setDocuments] = useState<SopDocumentDto[]>([]);
  const [versions, setVersions] = useState<SopVersionDto[]>([]);
  const [approvalSteps, setApprovalSteps] = useState<SopApprovalStepDto[]>([]);
  const [acknowledgments, setAcknowledgments] = useState<SopAcknowledgmentDto[]>([]);
  const [licenseStatus, setLicenseStatus] = useState<LicenseStatusDto | null>(null);
  const [lastExport, setLastExport] = useState<ExportPackDto | null>(null);
  const [selectedDocumentId, setSelectedDocumentId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createPath, setCreatePath] = useState("");
  const [createName, setCreateName] = useState("SOP Vault");
  const [openPath, setOpenPath] = useState("");
  const [documentForm, setDocumentForm] = useState<DocumentFormState>(emptyDocumentForm);
  const [versionForm, setVersionForm] = useState<VersionFormState>(emptyVersionForm);
  const [approvalInput, setApprovalInput] = useState("Quality lead, Operations owner");
  const [approvalDecisionNotes, setApprovalDecisionNotes] = useState("");
  const [acknowledgmentInput, setAcknowledgmentInput] = useState("Team lead, New hire");

  const selectedDocument = useMemo(
    () => documents.find((document) => document.document_id === selectedDocumentId) ?? null,
    [documents, selectedDocumentId]
  );
  const publishedCount = documents.filter((document) => document.status === "published").length;
  const approvedCount = documents.filter((document) => document.status === "approved").length;
  const exportReady = licenseStatus?.valid === true && licenseStatus.features.includes("EXPORT_PACKS");
  const canGenerateExport = exportReady && documents.length > 0;
  const pendingApprovalSteps = approvalSteps.filter((step) => step.status === "pending");
  const pendingAcknowledgments = acknowledgments.filter(
    (acknowledgment) => acknowledgment.status !== "acknowledged"
  );

  const loadSelectedDocumentState = async (
    documentId: string,
    nextDocuments: SopDocumentDto[]
  ) => {
    const [nextVersions, nextApprovalSteps, nextAcknowledgments] = await Promise.all([
      listSopVersions(documentId),
      listSopApprovalSteps(documentId),
      listSopAcknowledgments(documentId)
    ]);
    setVersions(nextVersions);
    setApprovalSteps(nextApprovalSteps);
    setAcknowledgments(nextAcknowledgments);

    const activeDocument =
      nextDocuments.find((document) => document.document_id === documentId) ?? null;
    if (activeDocument) {
      setVersionForm({
        body_markdown: activeDocument.latest_body_markdown,
        change_summary: activeDocument.latest_change_summary ?? ""
      });
    }
  };

  const loadWorkspace = async (preferredDocumentId?: string | null) => {
    const [nextDocuments, nextLicenseStatus] = await Promise.all([
      listSopDocuments(),
      checkLicenseStatus()
    ]);
    setDocuments(nextDocuments);
    setLicenseStatus(nextLicenseStatus);

    const resolvedDocumentId =
      preferredDocumentId && nextDocuments.some((document) => document.document_id === preferredDocumentId)
        ? preferredDocumentId
        : nextDocuments[0]?.document_id ?? null;

    setSelectedDocumentId(resolvedDocumentId);

    if (resolvedDocumentId) {
      await loadSelectedDocumentState(resolvedDocumentId, nextDocuments);
    } else {
      setVersions([]);
      setApprovalSteps([]);
      setAcknowledgments([]);
      setVersionForm(emptyVersionForm);
    }
  };

  useEffect(() => {
    const bootstrap = async () => {
      const lastVaultPath = localStorage.getItem(SOP_LAST_VAULT_KEY);
      if (!lastVaultPath) {
        setLoading(false);
        return;
      }

      try {
        const opened = await openVault(lastVaultPath);
        setVault(opened);
        setCreatePath(lastVaultPath);
        setOpenPath(lastVaultPath);
        await loadWorkspace();
      } catch (nextError) {
        localStorage.removeItem(SOP_LAST_VAULT_KEY);
        const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
        setError(message);
      } finally {
        setLoading(false);
      }
    };

    void bootstrap();
    // Bootstrap from remembered local vault state only once per mount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const chooseDirectory = async (setter: (value: string) => void) => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: openPath || createPath || undefined
    });
    if (typeof selected === "string") {
      setter(selected);
    }
  };

  const openOrCreateVault = async (mode: "create" | "open") => {
    setWorking(true);
    setError(null);
    try {
      const nextVault =
        mode === "create"
          ? await createVault(createPath.trim(), createName.trim() || "SOP Vault")
          : await openVault(openPath.trim());
      const rememberedPath = mode === "create" ? createPath.trim() : openPath.trim();
      localStorage.setItem(SOP_LAST_VAULT_KEY, rememberedPath);
      setVault(nextVault);
      setLastExport(null);
      await loadWorkspace();
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setLoading(false);
      setWorking(false);
    }
  };

  const handleCreateDocument = async () => {
    setWorking(true);
    setError(null);
    try {
      const input: SopDocumentCreateInputDto = {
        title: documentForm.title,
        slug: documentForm.slug,
        owner: documentForm.owner,
        body_markdown: documentForm.body_markdown,
        change_summary: documentForm.change_summary || undefined
      };
      const created = await createSopDocument(input);
      setDocumentForm(emptyDocumentForm);
      await loadWorkspace(created.document_id);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleSaveVersion = async () => {
    if (!selectedDocument) {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      await updateSopDocument(selectedDocument.document_id, {
        body_markdown: versionForm.body_markdown,
        change_summary: versionForm.change_summary || undefined
      });
      await loadWorkspace(selectedDocument.document_id);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handlePublish = async () => {
    if (!selectedDocument) {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      await publishSopDocument(selectedDocument.document_id);
      await loadWorkspace(selectedDocument.document_id);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleSubmitForApproval = async () => {
    if (!selectedDocument) {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      const input: SopApprovalSubmitInputDto = {
        approvers: splitPeopleList(approvalInput)
      };
      await submitSopForApproval(selectedDocument.document_id, input);
      await loadWorkspace(selectedDocument.document_id);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleDecideApproval = async (
    stepId: string,
    decision: "approved" | "changes_requested"
  ) => {
    if (!selectedDocument) {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      await decideSopApproval(stepId, {
        decision,
        notes: approvalDecisionNotes || undefined
      });
      setApprovalDecisionNotes("");
      await loadWorkspace(selectedDocument.document_id);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleInstallLicense = async () => {
    const selected = await open({
      directory: false,
      multiple: false,
      filters: [
        {
          name: "License Files",
          extensions: ["json", "lic"]
        }
      ]
    });

    if (typeof selected !== "string") {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      const nextStatus = await installLicense(selected);
      setLicenseStatus(nextStatus);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleGenerateExport = async () => {
    if (!vault) {
      return;
    }

    const suggestedPath = `${vault.root_path}/${vault.name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "")}-sop-pack.zip`;
    const destination = await save({
      defaultPath: suggestedPath,
      filters: [
        {
          name: "Zip Archive",
          extensions: ["zip"]
        }
      ]
    });

    if (typeof destination !== "string") {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      const exportPack = await generateExportPack(destination);
      setLastExport(exportPack);
      await loadWorkspace(selectedDocumentId);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleSelectDocument = async (documentId: string) => {
    setSelectedDocumentId(documentId);
    setWorking(true);
    setError(null);
    try {
      await loadSelectedDocumentState(documentId, documents);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleAssignAcknowledgments = async () => {
    if (!selectedDocument) {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      await assignSopAcknowledgments(selectedDocument.document_id, {
        recipients: splitPeopleList(acknowledgmentInput)
      });
      await loadWorkspace(selectedDocument.document_id);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleRecordAcknowledgment = async (acknowledgmentId: string) => {
    if (!selectedDocument) {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      await recordSopAcknowledgment(acknowledgmentId);
      await loadWorkspace(selectedDocument.document_id);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleCloseVault = async () => {
    setWorking(true);
    setError(null);
    try {
      await closeVault();
      localStorage.removeItem(SOP_LAST_VAULT_KEY);
      setVault(null);
      setDocuments([]);
      setVersions([]);
      setApprovalSteps([]);
      setAcknowledgments([]);
      setLicenseStatus(null);
      setLastExport(null);
      setSelectedDocumentId(null);
      setVersionForm(emptyVersionForm);
    } catch (nextError) {
      const message = nextError instanceof SopAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-background px-6">
        <div className="max-w-md space-y-3 text-center">
          <p className="text-sm font-semibold uppercase tracking-[0.24em] text-primary">
            Compliance Suite
          </p>
          <h1 className="text-3xl font-bold text-foreground">Preparing SOP workspace</h1>
          <p className="text-sm text-muted-foreground">
            Reconnecting to the last SOP vault and loading its document library.
          </p>
        </div>
      </div>
    );
  }

  if (!vault) {
    return (
      <div className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(26,127,145,0.18),_transparent_42%),linear-gradient(180deg,_#f1f9f8_0%,_#e8f1f0_100%)] px-6 py-10">
        <div className="mx-auto max-w-6xl space-y-8">
          <div className="max-w-3xl space-y-3">
            <p className="text-sm font-semibold uppercase tracking-[0.24em] text-primary">
              Compliance Suite
            </p>
            <h1 className="text-4xl font-bold text-foreground">SOP workspace for controlled procedures</h1>
            <p className="max-w-2xl text-base text-muted-foreground">
              Create or open a local vault to draft procedures, track versions, and publish the
              current approved working copy.
            </p>
          </div>

          {error && (
            <div className="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
              {error}
            </div>
          )}

          <div className="grid gap-6 lg:grid-cols-2">
            <section className="rounded-3xl border border-border bg-card/90 p-6 shadow-sm">
              <div className="space-y-2">
                <h2 className="text-xl font-semibold">Create new SOP vault</h2>
                <p className="text-sm text-muted-foreground">
                  Start a local procedure workspace with version history baked in from day one.
                </p>
              </div>
              <div className="mt-6 space-y-4">
                <label className="block space-y-2">
                  <span className="text-sm font-medium">Vault name</span>
                  <input
                    className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                    value={createName}
                    onChange={(event) => setCreateName(event.target.value)}
                    placeholder="SOP Vault"
                  />
                </label>
                <div className="flex gap-3">
                  <label className="block flex-1 space-y-2">
                    <span className="text-sm font-medium">Vault folder</span>
                    <input
                      className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                      value={createPath}
                      onChange={(event) => setCreatePath(event.target.value)}
                      placeholder="/path/to/vault"
                    />
                  </label>
                  <button
                    className="mt-8 h-11 rounded-xl border px-4 text-sm"
                    onClick={() => void chooseDirectory(setCreatePath)}
                    disabled={working}
                  >
                    Browse
                  </button>
                </div>
                <button
                  className="h-11 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground"
                  onClick={() => void openOrCreateVault("create")}
                  disabled={working || !createPath.trim()}
                >
                  {working ? "Creating..." : "Create SOP Vault"}
                </button>
              </div>
            </section>

            <section className="rounded-3xl border border-border bg-card/90 p-6 shadow-sm">
              <div className="space-y-2">
                <h2 className="text-xl font-semibold">Open existing vault</h2>
                <p className="text-sm text-muted-foreground">
                  Reconnect to the procedure library that already lives on this machine.
                </p>
              </div>
              <div className="mt-6 space-y-4">
                <div className="flex gap-3">
                  <label className="block flex-1 space-y-2">
                    <span className="text-sm font-medium">Existing vault folder</span>
                    <input
                      className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                      value={openPath}
                      onChange={(event) => setOpenPath(event.target.value)}
                      placeholder="/path/to/existing-vault"
                    />
                  </label>
                  <button
                    className="mt-8 h-11 rounded-xl border px-4 text-sm"
                    onClick={() => void chooseDirectory(setOpenPath)}
                    disabled={working}
                  >
                    Browse
                  </button>
                </div>
                <button
                  className="h-11 rounded-xl bg-foreground px-4 text-sm font-semibold text-background"
                  onClick={() => void openOrCreateVault("open")}
                  disabled={working || !openPath.trim()}
                >
                  {working ? "Opening..." : "Open SOP Vault"}
                </button>
              </div>
            </section>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[linear-gradient(180deg,_#f4faf9_0%,_#e5efee_100%)] px-6 py-8">
      <div className="mx-auto max-w-7xl space-y-6">
        <header className="rounded-3xl border border-border bg-card/95 p-6 shadow-sm">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-2">
              <p className="text-sm font-semibold uppercase tracking-[0.24em] text-primary">
                SOP
              </p>
              <h1 className="text-3xl font-bold text-foreground">{vault.name}</h1>
              <p className="max-w-3xl text-sm text-muted-foreground">
                Draft procedures, keep version history visible, and publish the current operating
                copy from one local workspace.
              </p>
            </div>
            <div className="flex flex-wrap gap-3">
              <button
                className="h-11 rounded-xl border px-4 text-sm"
                onClick={() => void loadWorkspace(selectedDocumentId)}
                disabled={working}
              >
                Refresh Workspace
              </button>
              <button
                className="h-11 rounded-xl border px-4 text-sm"
                onClick={() => void handleCloseVault()}
                disabled={working}
              >
                Close Vault
              </button>
            </div>
          </div>
          {error && (
            <div className="mt-4 rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
              {error}
            </div>
          )}
        </header>

        <section className="grid gap-4 lg:grid-cols-5">
          <article className="rounded-2xl border border-border bg-card p-5 shadow-sm">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              Documents
            </p>
            <p className="mt-3 text-3xl font-bold text-foreground">{documents.length}</p>
            <p className="mt-1 text-sm text-muted-foreground">tracked procedures in this vault</p>
          </article>
          <article className="rounded-2xl border border-border bg-card p-5 shadow-sm">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              Published
            </p>
            <p className="mt-3 text-3xl font-bold text-foreground">{publishedCount}</p>
            <p className="mt-1 text-sm text-muted-foreground">documents currently published</p>
          </article>
          <article className="rounded-2xl border border-border bg-card p-5 shadow-sm">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              Approved
            </p>
            <p className="mt-3 text-3xl font-bold text-foreground">{approvedCount}</p>
            <p className="mt-1 text-sm text-muted-foreground">documents ready to publish</p>
          </article>
          <article className="rounded-2xl border border-border bg-card p-5 shadow-sm">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              Latest version
            </p>
            <p className="mt-3 text-3xl font-bold text-foreground">
              {selectedDocument ? `v${selectedDocument.latest_version_number}` : "No doc"}
            </p>
            <p className="mt-1 text-sm text-muted-foreground">current focus in the editor</p>
          </article>
          <article className="rounded-2xl border border-border bg-card p-5 shadow-sm">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              Export status
            </p>
            <p className="mt-3 text-2xl font-bold text-foreground">
              {documents.length === 0
                ? "Add docs"
                : exportReady
                  ? "Export-ready"
                  : licenseStatus?.installed
                    ? "License gap"
                    : "No license"}
            </p>
            <p className="mt-1 text-sm text-muted-foreground">
              {documents.length === 0
                ? "Create at least one SOP before generating a pack."
                : exportReady
                  ? "The SOP pack can be generated from this vault."
                  : "Install a valid local license with export access."}
            </p>
          </article>
        </section>

        <section className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
          <article className="rounded-3xl border border-border bg-card p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold">Release readiness</h2>
              <p className="text-sm text-muted-foreground">
                SOP exports are gated by local license state so the package only ships when the
                same release rules used across the suite are satisfied.
              </p>
            </div>
            <div className="mt-6 space-y-4">
              <div className="rounded-2xl bg-secondary/50 p-4">
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  License status
                </p>
                <p className="mt-2 text-lg font-semibold text-foreground">
                  {licenseStatus?.installed
                    ? exportReady
                      ? "Export-ready"
                      : "Installed but missing export access"
                    : "No license installed"}
                </p>
                <p className="mt-2 text-sm text-muted-foreground">
                  {documents.length === 0
                    ? "Create at least one SOP document before generating a pack."
                    : licenseStatus?.installed
                      ? licenseStatus.valid
                        ? `Active features: ${licenseStatus.features.join(", ") || "none"}`
                        : "The current license failed validation and export will stay blocked."
                      : "Install a local license file to unlock SOP export packs."}
                </p>
              </div>
              <div className="flex flex-wrap gap-3">
                <button
                  className="h-11 rounded-xl border px-4 text-sm"
                  onClick={() => void handleInstallLicense()}
                  disabled={working}
                >
                  Install License
                </button>
                <button
                  className="h-11 rounded-xl bg-foreground px-4 text-sm font-semibold text-background disabled:cursor-not-allowed disabled:opacity-50"
                  onClick={() => void handleGenerateExport()}
                  disabled={working || !canGenerateExport}
                >
                  {working ? "Working..." : "Generate SOP Pack"}
                </button>
              </div>
            </div>
          </article>

          <article className="rounded-3xl border border-border bg-card p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold">SOP export contents</h2>
              <p className="text-sm text-muted-foreground">
                The current SOP pack carries the document library, version history, license state,
                and audit trail needed to understand what was published from this vault.
              </p>
            </div>
            <div className="mt-6 grid gap-3 md:grid-cols-2">
              {[
                "sop/documents.json",
                "sop/versions.json",
                "sop/approvals.json",
                "sop/acknowledgments.json",
                "license/status.json",
                "audit/events.json"
              ].map((path) => (
                <div key={path} className="rounded-2xl bg-secondary/40 px-4 py-3 text-sm">
                  <p className="font-medium text-foreground">{path}</p>
                </div>
              ))}
            </div>
            {lastExport && (
              <div className="mt-6 rounded-2xl border border-border bg-background px-4 py-4">
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  Last export
                </p>
                <p className="mt-2 text-sm font-medium text-foreground">{lastExport.zip_path}</p>
                <p className="mt-2 text-sm text-muted-foreground">
                  {lastExport.file_count} files included in manifest v{lastExport.manifest_version}.
                </p>
              </div>
            )}
          </article>
        </section>

        <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
          <section className="rounded-3xl border border-border bg-card p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold">Create SOP document</h2>
              <p className="text-sm text-muted-foreground">
                Start with a procedure title, owner, and draft body. The first version is created
                automatically.
              </p>
            </div>

            <div className="mt-6 grid gap-4">
              <label className="space-y-2">
                <span className="text-sm font-medium">Title</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={documentForm.title}
                  onChange={(event) =>
                    setDocumentForm((current) => {
                      const title = event.target.value;
                      return {
                        ...current,
                        title,
                        slug: current.slug || toSlug(title)
                      };
                    })
                  }
                  placeholder="Access review procedure"
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Slug</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={documentForm.slug}
                  onChange={(event) =>
                    setDocumentForm((current) => ({ ...current, slug: toSlug(event.target.value) }))
                  }
                  placeholder="access-review-procedure"
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Owner</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={documentForm.owner}
                  onChange={(event) =>
                    setDocumentForm((current) => ({ ...current, owner: event.target.value }))
                  }
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Initial change summary</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={documentForm.change_summary}
                  onChange={(event) =>
                    setDocumentForm((current) => ({
                      ...current,
                      change_summary: event.target.value
                    }))
                  }
                  placeholder="Initial draft"
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Draft body</span>
                <textarea
                  className="min-h-[220px] w-full rounded-xl border bg-background px-3 py-2 text-sm"
                  value={documentForm.body_markdown}
                  onChange={(event) =>
                    setDocumentForm((current) => ({
                      ...current,
                      body_markdown: event.target.value
                    }))
                  }
                />
              </label>
            </div>

            <div className="mt-6 flex gap-3">
              <button
                className="h-11 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground"
                onClick={() => void handleCreateDocument()}
                disabled={
                  working ||
                  !documentForm.title.trim() ||
                  !documentForm.slug.trim() ||
                  !documentForm.body_markdown.trim()
                }
              >
                {working ? "Saving..." : "Create SOP"}
              </button>
              <button
                className="h-11 rounded-xl border px-4 text-sm"
                onClick={() => setDocumentForm(emptyDocumentForm)}
                disabled={working}
              >
                Reset
              </button>
            </div>
          </section>

          <section className="space-y-6">
            <div className="rounded-3xl border border-border bg-card p-6 shadow-sm">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-xl font-semibold">Document library</h2>
                  <p className="text-sm text-muted-foreground">
                    Pick a procedure to inspect version history, draft the next revision, or publish
                    the latest copy.
                  </p>
                </div>
                <div className="rounded-xl bg-secondary px-3 py-2 text-sm">
                  <p className="font-semibold text-foreground">{documents.length} documents</p>
                </div>
              </div>

              <div className="mt-4 space-y-3">
                {documents.length === 0 ? (
                  <p className="rounded-xl border border-dashed px-4 py-6 text-sm text-muted-foreground">
                    No SOP documents yet. Create your first procedure to start a version history.
                  </p>
                ) : (
                  documents.map((document) => (
                    <button
                      key={document.document_id}
                      className={`w-full rounded-2xl border p-4 text-left transition ${
                        document.document_id === selectedDocumentId
                          ? "border-primary bg-primary/5"
                          : "border-border bg-secondary/25"
                      }`}
                      onClick={() => void handleSelectDocument(document.document_id)}
                    >
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-muted-foreground">
                          {document.slug}
                        </span>
                        <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-muted-foreground">
                          v{document.latest_version_number}
                        </span>
                        <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-muted-foreground">
                          {document.status}
                        </span>
                      </div>
                      <h3 className="mt-3 text-lg font-semibold text-foreground">{document.title}</h3>
                      <p className="mt-1 text-sm text-muted-foreground">Owner: {document.owner}</p>
                      {document.latest_change_summary && (
                        <p className="mt-2 text-sm text-muted-foreground">
                          Latest change: {document.latest_change_summary}
                        </p>
                      )}
                    </button>
                  ))
                )}
              </div>
            </div>

            <div className="rounded-3xl border border-border bg-card p-6 shadow-sm">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-xl font-semibold">Revision workspace</h2>
                  <p className="text-sm text-muted-foreground">
                    Move the selected SOP from draft to approval, publication, and acknowledgment
                    without leaving this workspace.
                  </p>
                </div>
                {selectedDocument && (
                  <div className="rounded-xl bg-secondary px-3 py-2 text-sm">
                    <p className="font-semibold text-foreground">
                      {selectedDocument.status === "published"
                        ? "Published"
                        : selectedDocument.status === "approved"
                          ? "Approved"
                          : selectedDocument.status === "in_review"
                            ? "In review"
                            : "Drafting"}
                    </p>
                  </div>
                )}
              </div>

              {!selectedDocument ? (
                <p className="mt-4 rounded-xl border border-dashed px-4 py-6 text-sm text-muted-foreground">
                  Select a document from the library to work on its next version.
                </p>
              ) : (
                <div className="mt-6 space-y-4">
                  <div className="rounded-2xl bg-secondary/30 p-4">
                    <p className="text-sm font-semibold text-foreground">Lifecycle status</p>
                    <p className="mt-2 text-sm text-muted-foreground">
                      {selectedDocument.status === "draft" &&
                        "Save changes, then submit this version for approval before publishing."}
                      {selectedDocument.status === "in_review" &&
                        "Approval is in progress. Review each approver decision below."}
                      {selectedDocument.status === "approved" &&
                        "This version is approved and ready to publish as the working copy."}
                      {selectedDocument.status === "published" &&
                        "This version is published. Assign and record acknowledgments for the people who must confirm it."}
                    </p>
                    <div className="mt-3 flex flex-wrap gap-2 text-xs text-muted-foreground">
                      <span className="rounded bg-background px-2 py-1">
                        Pending approvals: {pendingApprovalSteps.length}
                      </span>
                      <span className="rounded bg-background px-2 py-1">
                        Pending acknowledgments: {pendingAcknowledgments.length}
                      </span>
                    </div>
                  </div>
                  <label className="space-y-2">
                    <span className="text-sm font-medium">Change summary</span>
                    <input
                      className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                      value={versionForm.change_summary}
                      onChange={(event) =>
                        setVersionForm((current) => ({
                          ...current,
                          change_summary: event.target.value
                        }))
                      }
                      placeholder="Describe what changed in this revision"
                    />
                  </label>
                  <label className="space-y-2">
                    <span className="text-sm font-medium">Document body</span>
                    <textarea
                      className="min-h-[220px] w-full rounded-xl border bg-background px-3 py-2 text-sm"
                      value={versionForm.body_markdown}
                      onChange={(event) =>
                        setVersionForm((current) => ({
                          ...current,
                          body_markdown: event.target.value
                        }))
                      }
                    />
                  </label>
                  <div className="flex flex-wrap gap-3">
                    <button
                      className="h-11 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground"
                      onClick={() => void handleSaveVersion()}
                      disabled={working || !versionForm.body_markdown.trim()}
                    >
                      {working ? "Saving..." : "Save New Version"}
                    </button>
                    <button
                      className="h-11 rounded-xl border px-4 text-sm"
                      onClick={() => void handleSubmitForApproval()}
                      disabled={working || splitPeopleList(approvalInput).length === 0}
                    >
                      Submit for Approval
                    </button>
                    <button
                      className="h-11 rounded-xl border px-4 text-sm"
                      onClick={() => void handlePublish()}
                      disabled={working || selectedDocument.status !== "approved"}
                    >
                      Publish Current Version
                    </button>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    Publishing is only available after every current approval step is marked
                    approved.
                  </p>

                  <div className="rounded-2xl bg-secondary/30 p-4">
                    <p className="text-sm font-semibold text-foreground">Approval flow</p>
                    <p className="mt-2 text-sm text-muted-foreground">
                      Add approvers for the current version, then record whether each reviewer
                      approved it or sent it back for changes.
                    </p>
                    <label className="mt-4 block space-y-2">
                      <span className="text-sm font-medium">Approvers</span>
                      <textarea
                        className="min-h-[90px] w-full rounded-xl border bg-background px-3 py-2 text-sm"
                        value={approvalInput}
                        onChange={(event) => setApprovalInput(event.target.value)}
                        placeholder="Quality lead, Operations owner"
                      />
                    </label>
                    <label className="mt-4 block space-y-2">
                      <span className="text-sm font-medium">Decision notes (optional)</span>
                      <textarea
                        className="min-h-[80px] w-full rounded-xl border bg-background px-3 py-2 text-sm"
                        value={approvalDecisionNotes}
                        onChange={(event) => setApprovalDecisionNotes(event.target.value)}
                        placeholder="Use this when approving or requesting changes."
                      />
                    </label>
                    <div className="mt-4 space-y-3">
                      {approvalSteps.length === 0 ? (
                        <p className="rounded-2xl border border-dashed px-4 py-4 text-sm text-muted-foreground">
                          No approval request has been sent for this SOP version yet.
                        </p>
                      ) : (
                        approvalSteps.map((step) => (
                          <article key={step.step_id} className="rounded-2xl bg-background px-4 py-4">
                            <div className="flex flex-wrap items-center gap-2">
                              <span className="rounded bg-secondary px-2 py-1 text-xs font-semibold text-muted-foreground">
                                {step.approver}
                              </span>
                              <span className="rounded bg-secondary px-2 py-1 text-xs font-semibold text-muted-foreground">
                                {step.status}
                              </span>
                              <span className="rounded bg-secondary px-2 py-1 text-xs font-semibold text-muted-foreground">
                                request {step.request_status}
                              </span>
                            </div>
                            <p className="mt-2 text-sm text-muted-foreground">
                              Requested at {step.requested_at}
                            </p>
                            {step.notes && (
                              <p className="mt-2 text-sm text-muted-foreground">{step.notes}</p>
                            )}
                            {step.status === "pending" && (
                              <div className="mt-3 flex flex-wrap gap-3">
                                <button
                                  className="h-10 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground"
                                  onClick={() => void handleDecideApproval(step.step_id, "approved")}
                                  disabled={working}
                                >
                                  Approve
                                </button>
                                <button
                                  className="h-10 rounded-xl border px-4 text-sm"
                                  onClick={() =>
                                    void handleDecideApproval(step.step_id, "changes_requested")
                                  }
                                  disabled={working}
                                >
                                  Request Changes
                                </button>
                              </div>
                            )}
                          </article>
                        ))
                      )}
                    </div>
                  </div>

                  <div className="rounded-2xl bg-secondary/30 p-4">
                    <p className="text-sm font-semibold text-foreground">Version history</p>
                    <div className="mt-3 space-y-3">
                      {versions.map((version) => (
                        <article key={version.version_id} className="rounded-2xl bg-background px-4 py-3">
                          <div className="flex flex-wrap items-center gap-2">
                            <span className="rounded bg-secondary px-2 py-1 text-xs font-semibold text-muted-foreground">
                              v{version.version_number}
                            </span>
                            <span className="text-xs text-muted-foreground">{version.created_at}</span>
                          </div>
                          {version.change_summary && (
                            <p className="mt-2 text-sm text-muted-foreground">{version.change_summary}</p>
                          )}
                        </article>
                      ))}
                    </div>
                  </div>

                  <div className="rounded-2xl bg-secondary/30 p-4">
                    <p className="text-sm font-semibold text-foreground">Acknowledgments</p>
                    <p className="mt-2 text-sm text-muted-foreground">
                      After publication, track which recipients still need to acknowledge the active
                      SOP version.
                    </p>
                    <label className="mt-4 block space-y-2">
                      <span className="text-sm font-medium">Recipients</span>
                      <textarea
                        className="min-h-[90px] w-full rounded-xl border bg-background px-3 py-2 text-sm"
                        value={acknowledgmentInput}
                        onChange={(event) => setAcknowledgmentInput(event.target.value)}
                        placeholder="Team lead, New hire"
                      />
                    </label>
                    <div className="mt-4 flex flex-wrap gap-3">
                      <button
                        className="h-11 rounded-xl border px-4 text-sm"
                        onClick={() => void handleAssignAcknowledgments()}
                        disabled={
                          working ||
                          !selectedDocument.published_version_id ||
                          splitPeopleList(acknowledgmentInput).length === 0
                        }
                      >
                        Assign Acknowledgments
                      </button>
                    </div>
                    <div className="mt-4 space-y-3">
                      {acknowledgments.length === 0 ? (
                        <p className="rounded-2xl border border-dashed px-4 py-4 text-sm text-muted-foreground">
                          No acknowledgments are being tracked for this SOP yet.
                        </p>
                      ) : (
                        acknowledgments.map((acknowledgment) => (
                          <article
                            key={acknowledgment.acknowledgment_id}
                            className="rounded-2xl bg-background px-4 py-4"
                          >
                            <div className="flex flex-wrap items-center gap-2">
                              <span className="rounded bg-secondary px-2 py-1 text-xs font-semibold text-muted-foreground">
                                {acknowledgment.recipient}
                              </span>
                              <span className="rounded bg-secondary px-2 py-1 text-xs font-semibold text-muted-foreground">
                                {acknowledgment.status}
                              </span>
                            </div>
                            <p className="mt-2 text-sm text-muted-foreground">
                              Created at {acknowledgment.created_at}
                            </p>
                            {acknowledgment.acknowledged_at && (
                              <p className="mt-1 text-sm text-muted-foreground">
                                Acknowledged at {acknowledgment.acknowledged_at}
                              </p>
                            )}
                            {acknowledgment.status !== "acknowledged" && (
                              <button
                                className="mt-3 h-10 rounded-xl border px-4 text-sm"
                                onClick={() =>
                                  void handleRecordAcknowledgment(
                                    acknowledgment.acknowledgment_id
                                  )
                                }
                                disabled={working}
                              >
                                Mark Acknowledged
                              </button>
                            )}
                          </article>
                        ))
                      )}
                    </div>
                  </div>
                </div>
              )}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

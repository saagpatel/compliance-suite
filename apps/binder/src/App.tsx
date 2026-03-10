import { useEffect, useMemo, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  type BinderControlCreateInputDto,
  type BinderControlDto,
  type BinderStatusSummaryDto,
  type EvidenceDto,
  type ExportPackDto,
  type LicenseStatusDto,
  type VaultDto,
  BinderAppError,
  checkLicenseStatus,
  closeVault,
  createBinderControl,
  createVault,
  generateExportPack,
  getBinderSummary,
  importEvidence,
  installLicense,
  linkBinderEvidence,
  listBinderControls,
  listEvidence,
  openVault,
  setBinderControlStatus
} from "./api";
import { BINDER_LAST_VAULT_KEY } from "./state";

type ControlFormState = {
  framework: string;
  control_code: string;
  title: string;
  description: string;
  reporting_period: string;
  status: "draft" | "collecting_evidence" | "reviewing" | "ready";
  owner: string;
  evidence_links: string[];
};

type ControlFocusFilter = "needs_attention" | "all";

const emptyForm: ControlFormState = {
  framework: "SOC 2",
  control_code: "",
  title: "",
  description: "",
  reporting_period: "2026-Q1",
  status: "draft",
  owner: "security",
  evidence_links: []
};

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

export default function App() {
  const [vault, setVault] = useState<VaultDto | null>(null);
  const [controls, setControls] = useState<BinderControlDto[]>([]);
  const [summary, setSummary] = useState<BinderStatusSummaryDto[]>([]);
  const [evidence, setEvidence] = useState<EvidenceDto[]>([]);
  const [licenseStatus, setLicenseStatus] = useState<LicenseStatusDto | null>(null);
  const [lastExport, setLastExport] = useState<ExportPackDto | null>(null);
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createPath, setCreatePath] = useState("");
  const [createName, setCreateName] = useState("Binder Vault");
  const [openPath, setOpenPath] = useState("");
  const [formState, setFormState] = useState<ControlFormState>(emptyForm);
  const [selectedEvidenceByControl, setSelectedEvidenceByControl] = useState<Record<string, string>>({});
  const [focusFilter, setFocusFilter] = useState<ControlFocusFilter>("needs_attention");

  const evidenceById = useMemo(
    () => new Map(evidence.map((item) => [item.evidence_id, item])),
    [evidence]
  );
  const needsAttentionControls = useMemo(
    () =>
      controls.filter(
        (control) => control.status !== "ready" || control.evidence_links.length === 0
      ),
    [controls]
  );
  const visibleControls = focusFilter === "needs_attention" ? needsAttentionControls : controls;
  const nextAttentionControl = needsAttentionControls[0] ?? null;

  const exportReady =
    licenseStatus?.valid === true && licenseStatus.features.includes("EXPORT_PACKS");

  const loadWorkspace = async () => {
    const [nextControls, nextSummary, nextEvidence, nextLicenseStatus] = await Promise.all([
      listBinderControls(),
      getBinderSummary(),
      listEvidence(),
      checkLicenseStatus()
    ]);
    setControls(nextControls);
    setSummary(nextSummary);
    setEvidence(nextEvidence);
    setLicenseStatus(nextLicenseStatus);
  };

  useEffect(() => {
    const bootstrap = async () => {
      const lastVaultPath = localStorage.getItem(BINDER_LAST_VAULT_KEY);
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
        localStorage.removeItem(BINDER_LAST_VAULT_KEY);
        const message =
          nextError instanceof BinderAppError ? nextError.message : String(nextError);
        setError(message);
      } finally {
        setLoading(false);
      }
    };

    void bootstrap();
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
          ? await createVault(createPath.trim(), createName.trim() || "Binder Vault")
          : await openVault(openPath.trim());
      const rememberedPath = mode === "create" ? createPath.trim() : openPath.trim();
      localStorage.setItem(BINDER_LAST_VAULT_KEY, rememberedPath);
      setVault(nextVault);
      await loadWorkspace();
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setLoading(false);
      setWorking(false);
    }
  };

  const handleCreateControl = async () => {
    setWorking(true);
    setError(null);
    try {
      const input: BinderControlCreateInputDto = {
        framework: formState.framework,
        control_code: formState.control_code,
        title: formState.title,
        description: formState.description || undefined,
        reporting_period: formState.reporting_period,
        status: formState.status,
        owner: formState.owner,
        evidence_links: formState.evidence_links
      };
      await createBinderControl(input);
      setFormState(emptyForm);
      await loadWorkspace();
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleImportEvidence = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: [
        {
          name: "Evidence Files",
          extensions: ["pdf", "png", "jpg", "jpeg", "csv", "xlsx", "txt", "docx"]
        }
      ]
    });
    const selectedPaths =
      typeof selected === "string" ? [selected] : Array.isArray(selected) ? selected : [];

    if (selectedPaths.length === 0) {
      return;
    }

    setWorking(true);
    setError(null);
    try {
      for (const filePath of selectedPaths) {
        await importEvidence(filePath);
      }
      await loadWorkspace();
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleLinkEvidence = async (controlId: string) => {
    const evidenceId = selectedEvidenceByControl[controlId];
    if (!evidenceId) {
      return;
    }
    setWorking(true);
    setError(null);
    try {
      await linkBinderEvidence(controlId, evidenceId);
      await loadWorkspace();
      setSelectedEvidenceByControl((current) => ({ ...current, [controlId]: "" }));
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleSetStatus = async (controlId: string, status: BinderControlDto["status"]) => {
    setWorking(true);
    setError(null);
    try {
      await setBinderControlStatus(controlId, status);
      await loadWorkspace();
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
      setError(message);
    } finally {
      setWorking(false);
    }
  };

  const handleRefreshWorkspace = async () => {
    setWorking(true);
    setError(null);
    try {
      await loadWorkspace();
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
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
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
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
      .replace(/^-|-$/g, "")}-binder-pack.zip`;
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
      await loadWorkspace();
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
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
      localStorage.removeItem(BINDER_LAST_VAULT_KEY);
      setVault(null);
      setControls([]);
      setSummary([]);
      setEvidence([]);
      setLicenseStatus(null);
      setLastExport(null);
      setFocusFilter("needs_attention");
    } catch (nextError) {
      const message =
        nextError instanceof BinderAppError ? nextError.message : String(nextError);
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
          <h1 className="text-3xl font-bold text-foreground">Preparing Binder workspace</h1>
          <p className="text-sm text-muted-foreground">
            Reconnecting to the last Binder vault and loading its control summary.
          </p>
        </div>
      </div>
    );
  }

  if (!vault) {
    return (
      <div className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(174,97,46,0.18),_transparent_42%),linear-gradient(180deg,_#fbf7f1_0%,_#f4ede2_100%)] px-6 py-10">
        <div className="mx-auto max-w-6xl space-y-8">
          <div className="max-w-3xl space-y-3">
            <p className="text-sm font-semibold uppercase tracking-[0.24em] text-primary">
              Compliance Suite
            </p>
            <h1 className="text-4xl font-bold text-foreground">Binder workspace for control evidence</h1>
            <p className="max-w-2xl text-base text-muted-foreground">
              Create or open a local vault to track controls by reporting period, attach evidence,
              and monitor Binder readiness in one place.
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
                <h2 className="text-xl font-semibold">Create new Binder vault</h2>
                <p className="text-sm text-muted-foreground">
                  Start a new local workspace for control tracking and evidence mapping.
                </p>
              </div>
              <div className="mt-6 space-y-4">
                <label className="block space-y-2">
                  <span className="text-sm font-medium">Vault name</span>
                  <input
                    className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                    value={createName}
                    onChange={(event) => setCreateName(event.target.value)}
                    placeholder="Binder Vault"
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
                  {working ? "Creating..." : "Create Binder Vault"}
                </button>
              </div>
            </section>

            <section className="rounded-3xl border border-border bg-card/90 p-6 shadow-sm">
              <div className="space-y-2">
                <h2 className="text-xl font-semibold">Open existing vault</h2>
                <p className="text-sm text-muted-foreground">
                  Reconnect to a Binder workspace that already lives on this machine.
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
                  {working ? "Opening..." : "Open Binder Vault"}
                </button>
              </div>
            </section>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[linear-gradient(180deg,_#faf4ea_0%,_#efe6d8_100%)] px-6 py-8">
      <div className="mx-auto max-w-7xl space-y-6">
        <header className="rounded-3xl border border-border bg-card/95 p-6 shadow-sm">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div className="space-y-2">
              <p className="text-sm font-semibold uppercase tracking-[0.24em] text-primary">
                Binder
              </p>
              <h1 className="text-3xl font-bold text-foreground">{vault.name}</h1>
              <p className="max-w-3xl text-sm text-muted-foreground">
                Track controls by period, attach supporting evidence, and keep a simple running view
                of what is ready for your Binder pack.
              </p>
            </div>
            <div className="flex flex-wrap gap-3">
              <button
                className="h-11 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground"
                onClick={() => void handleImportEvidence()}
                disabled={working}
              >
                Import Evidence
              </button>
              <button
                className="h-11 rounded-xl border px-4 text-sm"
                onClick={() => void handleRefreshWorkspace()}
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

        <section className="grid gap-4 lg:grid-cols-3">
          {summary.length === 0 ? (
            <article className="rounded-2xl border border-border bg-card p-5 text-sm text-muted-foreground lg:col-span-3">
              No control summaries yet. Add your first Binder control below to begin tracking a reporting period.
            </article>
          ) : (
            summary.map((item) => (
              <article key={item.reporting_period} className="rounded-2xl border border-border bg-card p-5 shadow-sm">
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  {item.reporting_period}
                </p>
                <p className="mt-3 text-3xl font-bold text-foreground">{item.ready_controls}/{item.total_controls}</p>
                <p className="mt-1 text-sm text-muted-foreground">controls marked ready</p>
                <div className="mt-4 grid grid-cols-2 gap-3 text-sm">
                  <div className="rounded-xl bg-secondary px-3 py-2">
                    <p className="font-semibold text-foreground">{item.controls_with_evidence}</p>
                    <p className="text-muted-foreground">with evidence</p>
                  </div>
                  <div className="rounded-xl bg-secondary px-3 py-2">
                    <p className="font-semibold text-foreground">{item.controls_without_evidence}</p>
                    <p className="text-muted-foreground">without evidence</p>
                  </div>
                </div>
              </article>
            ))
          )}
        </section>

        <section className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
          <article className="rounded-3xl border border-border bg-card p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold">Release readiness</h2>
              <p className="text-sm text-muted-foreground">
                Binder exports are gated by the local license state so the generated pack matches
                the release rules already used elsewhere in the suite.
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
                  {licenseStatus?.installed
                    ? licenseStatus.valid
                      ? `Active features: ${licenseStatus.features.join(", ") || "none"}`
                      : "The current license failed validation and export will stay blocked."
                    : "Install a local license file to unlock Binder export packs."}
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
                  disabled={working || !exportReady}
                >
                  {working ? "Working..." : "Generate Binder Pack"}
                </button>
              </div>
            </div>
          </article>

          <article className="rounded-3xl border border-border bg-card p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold">Binder export contents</h2>
              <p className="text-sm text-muted-foreground">
                The current Binder pack carries the evidence tree plus the Binder-specific context
                reviewers need to understand control readiness.
              </p>
            </div>
            <div className="mt-6 grid gap-3 md:grid-cols-2">
              {[
                "binder/controls.json",
                "binder/summary.json",
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

        <div className="grid gap-6 xl:grid-cols-[minmax(0,1.1fr)_minmax(0,1.5fr)]">
          <section className="rounded-3xl border border-border bg-card p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold">Add Binder control</h2>
              <p className="text-sm text-muted-foreground">
                Start with one control, assign it to a reporting period, and attach any evidence you already have.
              </p>
            </div>

            <div className="mt-6 grid gap-4 md:grid-cols-2">
              <label className="space-y-2">
                <span className="text-sm font-medium">Framework</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={formState.framework}
                  onChange={(event) => setFormState((current) => ({ ...current, framework: event.target.value }))}
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Control code</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={formState.control_code}
                  onChange={(event) => setFormState((current) => ({ ...current, control_code: event.target.value }))}
                  placeholder="CC6.1"
                />
              </label>
              <label className="space-y-2 md:col-span-2">
                <span className="text-sm font-medium">Title</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={formState.title}
                  onChange={(event) => setFormState((current) => ({ ...current, title: event.target.value }))}
                  placeholder="Access controls are defined"
                />
              </label>
              <label className="space-y-2 md:col-span-2">
                <span className="text-sm font-medium">Description</span>
                <textarea
                  className="min-h-[110px] w-full rounded-xl border bg-background px-3 py-2 text-sm"
                  value={formState.description}
                  onChange={(event) => setFormState((current) => ({ ...current, description: event.target.value }))}
                  placeholder="What should this control demonstrate during review?"
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Reporting period</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={formState.reporting_period}
                  onChange={(event) => setFormState((current) => ({ ...current, reporting_period: event.target.value }))}
                  placeholder="2026-Q1"
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Owner</span>
                <input
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={formState.owner}
                  onChange={(event) => setFormState((current) => ({ ...current, owner: event.target.value }))}
                />
              </label>
              <label className="space-y-2">
                <span className="text-sm font-medium">Initial status</span>
                <select
                  className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                  value={formState.status}
                  onChange={(event) =>
                    setFormState((current) => ({
                      ...current,
                      status: event.target.value as ControlFormState["status"]
                    }))
                  }
                >
                  <option value="draft">Draft</option>
                  <option value="collecting_evidence">Collecting evidence</option>
                  <option value="reviewing">Reviewing</option>
                  <option value="ready">Ready</option>
                </select>
              </label>
              <label className="space-y-2 md:col-span-2">
                <span className="text-sm font-medium">Attach evidence now</span>
                <select
                  multiple
                  className="min-h-[140px] w-full rounded-xl border bg-background px-3 py-2 text-sm"
                  value={formState.evidence_links}
                  onChange={(event) => {
                    const selected = Array.from(event.target.selectedOptions).map((option) => option.value);
                    setFormState((current) => ({ ...current, evidence_links: selected }));
                  }}
                >
                  {evidence.map((item) => (
                    <option key={item.evidence_id} value={item.evidence_id}>
                      {item.evidence_id} - {item.filename}
                    </option>
                  ))}
                </select>
              </label>
            </div>

            <div className="mt-6 flex gap-3">
              <button
                className="h-11 rounded-xl bg-primary px-4 text-sm font-semibold text-primary-foreground"
                onClick={() => void handleCreateControl()}
                disabled={working || !formState.control_code.trim() || !formState.title.trim()}
              >
                {working ? "Saving..." : "Create Control"}
              </button>
              <button
                className="h-11 rounded-xl border px-4 text-sm"
                onClick={() => setFormState(emptyForm)}
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
                  <h2 className="text-xl font-semibold">Evidence in vault</h2>
                  <p className="text-sm text-muted-foreground">
                    Imported evidence is available for Binder control linkage and summary review.
                  </p>
                </div>
                <div className="rounded-xl bg-secondary px-3 py-2 text-sm">
                  <p className="font-semibold text-foreground">{evidence.length} files</p>
                </div>
              </div>
              <div className="mt-4 grid gap-3">
                {evidence.length === 0 ? (
                  <p className="rounded-xl border border-dashed px-4 py-6 text-sm text-muted-foreground">
                    No evidence has been imported yet. Use the import action above to start building your Binder.
                  </p>
                ) : (
                  evidence.slice(0, 8).map((item) => (
                    <article key={item.evidence_id} className="rounded-2xl border border-border bg-secondary/40 p-4">
                      <p className="font-medium text-foreground">{item.filename}</p>
                      <p className="mt-1 text-xs text-muted-foreground">{item.relative_path}</p>
                      <div className="mt-3 flex flex-wrap gap-2 text-xs text-muted-foreground">
                        <span className="rounded bg-background px-2 py-1">{item.evidence_id}</span>
                        <span>{item.content_type}</span>
                        <span>{formatBytes(item.byte_size)}</span>
                      </div>
                    </article>
                  ))
                )}
              </div>
            </div>

            <div className="rounded-3xl border border-border bg-card p-6 shadow-sm">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-xl font-semibold">Binder controls</h2>
                  <p className="text-sm text-muted-foreground">
                    Update control readiness, attach evidence, and keep the reporting-period view current.
                  </p>
                </div>
                <div className="rounded-xl bg-secondary px-3 py-2 text-sm">
                  <p className="font-semibold text-foreground">{controls.length} controls</p>
                </div>
              </div>

              <div className="mt-4 space-y-4">
                {controls.length === 0 ? (
                  <p className="rounded-xl border border-dashed px-4 py-6 text-sm text-muted-foreground">
                    No controls have been created yet. Add one from the form to begin tracking Binder readiness.
                  </p>
                ) : (
                  <>
                    <div className="rounded-2xl bg-secondary/35 p-4">
                      <div className="flex flex-wrap items-center justify-between gap-3">
                        <div className="space-y-1">
                          <p className="text-sm font-semibold text-foreground">
                            {needsAttentionControls.length} controls still need work
                          </p>
                          <p className="text-sm text-muted-foreground">
                            Start with controls that are missing evidence or are not marked ready.
                          </p>
                        </div>
                        <div className="flex flex-wrap gap-2">
                          <button
                            className={`h-10 rounded-xl px-4 text-sm font-medium ${
                              focusFilter === "needs_attention"
                                ? "bg-foreground text-background"
                                : "border border-border bg-background text-foreground"
                            }`}
                            onClick={() => setFocusFilter("needs_attention")}
                            disabled={working}
                          >
                            Needs attention ({needsAttentionControls.length})
                          </button>
                          <button
                            className={`h-10 rounded-xl px-4 text-sm font-medium ${
                              focusFilter === "all"
                                ? "bg-foreground text-background"
                                : "border border-border bg-background text-foreground"
                            }`}
                            onClick={() => setFocusFilter("all")}
                            disabled={working}
                          >
                            All controls ({controls.length})
                          </button>
                        </div>
                      </div>
                      {focusFilter === "needs_attention" && nextAttentionControl && (
                        <p className="mt-3 text-sm text-muted-foreground">
                          Next up: {nextAttentionControl.control_code} {nextAttentionControl.title}
                        </p>
                      )}
                    </div>

                    {visibleControls.length === 0 ? (
                      <p className="rounded-xl border border-dashed px-4 py-6 text-sm text-muted-foreground">
                        Everything in this view is complete. Switch to All controls to review the full Binder.
                      </p>
                    ) : (
                      visibleControls.map((control) => (
                    <article key={control.control_id} className="rounded-2xl border border-border bg-secondary/30 p-4">
                      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
                        <div className="space-y-2">
                          <div className="flex flex-wrap items-center gap-2">
                            <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-muted-foreground">
                              {control.framework}
                            </span>
                            <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-muted-foreground">
                              {control.control_code}
                            </span>
                            <span className="rounded bg-background px-2 py-1 text-xs font-semibold text-muted-foreground">
                              {control.reporting_period}
                            </span>
                          </div>
                          <h3 className="text-lg font-semibold text-foreground">{control.title}</h3>
                          {control.description && (
                            <p className="text-sm text-muted-foreground">{control.description}</p>
                          )}
                          <p className="text-sm text-muted-foreground">Owner: {control.owner}</p>
                          <p className="text-sm font-medium text-foreground">
                            {control.evidence_links.length === 0
                              ? "Next action: link at least one evidence file."
                              : control.status === "reviewing"
                                ? "Next action: complete the review and mark this control Ready if the evidence set is complete."
                              : control.status !== "ready"
                                ? "Next action: review the evidence set and mark this control Ready."
                                : "This control is marked ready."}
                          </p>
                          <div className="flex flex-wrap gap-2">
                            {control.evidence_links.length === 0 ? (
                              <span className="text-xs text-muted-foreground">No evidence linked yet</span>
                            ) : (
                              control.evidence_links.map((evidenceId) => (
                                <div key={evidenceId} className="rounded-xl border border-border bg-background px-2 py-1 text-xs">
                                  <p className="font-medium text-foreground">{evidenceId}</p>
                                  <p className="text-muted-foreground">
                                    {evidenceById.get(evidenceId)?.filename ?? "Unknown evidence"}
                                  </p>
                                </div>
                              ))
                            )}
                          </div>
                        </div>

                        <div className="w-full max-w-sm space-y-3">
                          <label className="block space-y-2">
                            <span className="text-sm font-medium">Status</span>
                            <select
                              className="h-11 w-full rounded-xl border bg-background px-3 text-sm"
                              value={control.status}
                              onChange={(event) =>
                                void handleSetStatus(
                                  control.control_id,
                                  event.target.value as BinderControlDto["status"]
                                )
                              }
                              disabled={working}
                            >
                              <option value="draft">Draft</option>
                              <option value="collecting_evidence">Collecting evidence</option>
                              <option value="reviewing">Reviewing</option>
                              <option value="ready">Ready</option>
                            </select>
                          </label>

                          <div className="space-y-2">
                            <span className="text-sm font-medium">Link more evidence</span>
                            <div className="flex gap-2">
                              <select
                                className="h-11 flex-1 rounded-xl border bg-background px-3 text-sm"
                                value={selectedEvidenceByControl[control.control_id] ?? ""}
                                onChange={(event) =>
                                  setSelectedEvidenceByControl((current) => ({
                                    ...current,
                                    [control.control_id]: event.target.value
                                  }))
                                }
                              >
                                <option value="">Select vault evidence</option>
                                {evidence.map((item) => (
                                  <option key={item.evidence_id} value={item.evidence_id}>
                                    {item.evidence_id} - {item.filename}
                                  </option>
                                ))}
                              </select>
                              <button
                                className="h-11 rounded-xl border px-4 text-sm"
                                onClick={() => void handleLinkEvidence(control.control_id)}
                                disabled={working || !selectedEvidenceByControl[control.control_id]}
                              >
                                Link
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>
                    </article>
                      ))
                    )}
                  </>
                )}
              </div>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

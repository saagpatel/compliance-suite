import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import ExportDialog from "../components/features/ExportDialog";
import { Toaster } from "../components/ui/Toast";
import { useImport } from "../hooks/useImport";
import { useLicenseStatus } from "../hooks/useLicenseStatus";
import { useQuestionnaireReview } from "../hooks/useQuestionnaireReview";

export default function ExportPage() {
  const { currentImport } = useImport();
  const { status, loading: licenseLoading, error: licenseError } = useLicenseStatus();
  const { reviews, loading: reviewsLoading } = useQuestionnaireReview(currentImport?.import_id);

  const licenseReady =
    status?.installed && status.valid && status.features.includes("EXPORT_PACKS");
  const disabledReason = !currentImport
    ? "Import a questionnaire before exporting."
    : licenseLoading
      ? "Checking license status..."
      : licenseError
        ? "License status could not be confirmed. Refresh the vault and try again."
        : !status?.installed
          ? "Install a valid license with the EXPORT_PACKS feature before exporting."
          : !status.valid
            ? "The installed license is invalid for export. Replace it before exporting."
            : !status.features.includes("EXPORT_PACKS")
              ? "The installed license does not include the EXPORT_PACKS feature."
              : null;

  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header title="Export" subtitle="Step 5: Generate your final export pack" />
        <main className="flex-1 overflow-auto p-8">
          <div className="mx-auto max-w-5xl space-y-6">
            <div>
              <h2 className="mb-2 text-lg font-semibold">Ready to export the current workspace</h2>
              <p className="text-sm text-muted-foreground">
                The export pack now reflects the real questionnaire workspace: import metadata,
                saved review decisions, answer bank snapshot, license status, audit events, and
                evidence files from the active vault.
              </p>
            </div>

            <div className="grid gap-4 lg:grid-cols-3">
              <section className="rounded-xl border border-border bg-secondary/20 p-5">
                <p className="text-xs font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                  Import
                </p>
                {currentImport ? (
                  <div className="mt-3 space-y-2">
                    <p className="font-medium text-foreground">{currentImport.source_filename}</p>
                    <p className="text-sm text-muted-foreground">
                      Format: {currentImport.format.toUpperCase()}
                    </p>
                    <p className="text-sm text-muted-foreground">Status: {currentImport.status}</p>
                  </div>
                ) : (
                  <p className="mt-3 text-sm text-muted-foreground">
                    No questionnaire import is available yet.
                  </p>
                )}
              </section>

              <section className="rounded-xl border border-border bg-secondary/20 p-5">
                <p className="text-xs font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                  Review Workspace
                </p>
                <div className="mt-3 space-y-2">
                  <p className="font-medium text-foreground">
                    {reviewsLoading ? "Loading..." : `${reviews.length} saved review entries`}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    These entries will be included as `questionnaire/reviews.json`.
                  </p>
                </div>
              </section>

              <section className="rounded-xl border border-border bg-secondary/20 p-5">
                <p className="text-xs font-semibold uppercase tracking-[0.16em] text-muted-foreground">
                  License
                </p>
                <div className="mt-3 space-y-2">
                  <p className="font-medium text-foreground">
                    {licenseLoading
                      ? "Checking..."
                      : status?.installed
                        ? status.valid
                          ? "Installed and valid"
                          : "Installed but invalid"
                        : "Not installed"}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {status?.features?.length
                      ? `Features: ${status.features.join(", ")}`
                      : "Export requires the EXPORT_PACKS feature."}
                  </p>
                </div>
              </section>
            </div>

            {disabledReason && (
              <div className="rounded-xl border border-destructive/20 bg-destructive/10 p-5">
                <p className="text-sm font-medium text-destructive">Export is currently blocked</p>
                <p className="mt-2 text-sm text-destructive">{disabledReason}</p>
              </div>
            )}

            <div className="rounded-xl border border-border bg-background p-6">
              <h3 className="mb-4 font-medium text-foreground">Export contents</h3>
              <ul className="space-y-2 text-sm text-muted-foreground">
                <li className="flex items-center gap-2">
                  <span className="text-primary">✓</span>
                  `index.md` and `manifest.json`
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-primary">✓</span>
                  `questionnaire/import.json` with import metadata, saved column mapping, and the
                  mapped row snapshot
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-primary">✓</span>
                  `questionnaire/reviews.json` with saved review decisions
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-primary">✓</span>
                  `questionnaire/answer_bank.json` snapshot for matching context
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-primary">✓</span>
                  `license/status.json` and `audit/events.json`
                </li>
                <li className="flex items-center gap-2">
                  <span className="text-primary">✓</span>
                  Evidence files under their vault-relative paths
                </li>
              </ul>
            </div>

            <div className="pt-2">
              {currentImport && (
                <ExportDialog
                  importId={currentImport.import_id}
                  disabled={!licenseReady}
                  disabledReason={disabledReason}
                />
              )}
              {!currentImport && (
                <p className="text-sm text-muted-foreground">
                  Import and review a questionnaire before generating an export pack.
                </p>
              )}
            </div>
          </div>
        </main>
      </div>
      <Toaster />
    </div>
  );
}

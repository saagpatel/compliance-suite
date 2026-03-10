import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import Button from "../components/ui/Button";
import Input from "../components/ui/Input";
import { useVault } from "../hooks/useVault";

export default function WelcomePage() {
  const navigate = useNavigate();
  const { createVault, openVault, lastVaultPath, loading, error } = useVault();
  const [createPath, setCreatePath] = useState(lastVaultPath ?? "");
  const [createName, setCreateName] = useState("Compliance Vault");
  const [openPath, setOpenPath] = useState(lastVaultPath ?? "");

  const recentVaultLabel = useMemo(() => {
    if (!lastVaultPath) {
      return null;
    }
    const parts = lastVaultPath.split("/");
    return parts[parts.length - 1] || lastVaultPath;
  }, [lastVaultPath]);

  const chooseDirectory = async (setter: (value: string) => void) => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: lastVaultPath ?? undefined,
    });

    if (typeof selected === "string") {
      setter(selected);
    }
  };

  const handleCreate = async () => {
    if (!createPath.trim()) {
      return;
    }

    await createVault(createPath.trim(), createName.trim() || "Compliance Vault");
    navigate("/", { replace: true });
  };

  const handleOpen = async (path = openPath) => {
    if (!path.trim()) {
      return;
    }

    await openVault(path.trim());
    navigate("/", { replace: true });
  };

  return (
    <div className="min-h-screen bg-background px-6 py-10">
      <div className="mx-auto flex max-w-6xl flex-col gap-8">
        <div className="max-w-3xl space-y-3">
          <p className="text-sm font-semibold uppercase tracking-[0.2em] text-primary">
            Compliance Suite
          </p>
          <h1 className="text-4xl font-bold text-foreground">Open or create a vault to begin</h1>
          <p className="max-w-2xl text-base text-muted-foreground">
            Questionnaire Autopilot needs a local vault before it can import files, use the answer
            bank, validate licensing, or generate exports. Start by creating a new vault or opening
            an existing one on this machine.
          </p>
        </div>

        {error && (
          <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
            {error}
          </div>
        )}

        {lastVaultPath && (
          <section className="rounded-2xl border border-border bg-secondary/20 p-6">
            <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
              <div className="space-y-1">
                <h2 className="text-lg font-semibold text-foreground">Resume recent vault</h2>
                <p className="text-sm text-muted-foreground">
                  Reopen <span className="font-medium text-foreground">{recentVaultLabel}</span> at{" "}
                  {lastVaultPath}
                </p>
              </div>
              <Button onClick={() => void handleOpen(lastVaultPath)} disabled={loading}>
                {loading ? "Opening..." : "Reopen Recent Vault"}
              </Button>
            </div>
          </section>
        )}

        <div className="grid gap-6 lg:grid-cols-2">
          <section className="rounded-2xl border border-border bg-background p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-foreground">Create new vault</h2>
              <p className="text-sm text-muted-foreground">
                Choose where the vault should live, give it a name, and start a fresh questionnaire
                workflow.
              </p>
            </div>

            <div className="mt-6 space-y-4">
              <Input
                label="Vault name"
                value={createName}
                onChange={(event) => setCreateName(event.target.value)}
                placeholder="Compliance Vault"
              />
              <div className="flex gap-3">
                <Input
                  label="Vault folder"
                  value={createPath}
                  onChange={(event) => setCreatePath(event.target.value)}
                  placeholder="/path/to/vault"
                  className="flex-1"
                />
                <Button
                  className="mt-8"
                  variant="outline"
                  onClick={() => void chooseDirectory(setCreatePath)}
                  disabled={loading}
                >
                  Browse
                </Button>
              </div>
              <Button onClick={() => void handleCreate()} disabled={loading || !createPath.trim()}>
                {loading ? "Creating..." : "Create Vault"}
              </Button>
            </div>
          </section>

          <section className="rounded-2xl border border-border bg-background p-6 shadow-sm">
            <div className="space-y-2">
              <h2 className="text-xl font-semibold text-foreground">Open existing vault</h2>
              <p className="text-sm text-muted-foreground">
                Point the app to an existing local vault folder to continue working with its data.
              </p>
            </div>

            <div className="mt-6 space-y-4">
              <div className="flex gap-3">
                <Input
                  label="Existing vault folder"
                  value={openPath}
                  onChange={(event) => setOpenPath(event.target.value)}
                  placeholder="/path/to/existing-vault"
                  className="flex-1"
                />
                <Button
                  className="mt-8"
                  variant="outline"
                  onClick={() => void chooseDirectory(setOpenPath)}
                  disabled={loading}
                >
                  Browse
                </Button>
              </div>
              <Button onClick={() => void handleOpen()} disabled={loading || !openPath.trim()}>
                {loading ? "Opening..." : "Open Vault"}
              </Button>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

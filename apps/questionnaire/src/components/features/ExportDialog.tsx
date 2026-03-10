import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import Button from "../ui/Button";
import Input from "../ui/Input";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "../ui/Dialog";
import { invokeGenerateExportPack } from "../../api/tauri";
import { useUiStore } from "../../state/uiStore";

interface ExportDialogProps {
  importId: string;
  disabled?: boolean;
  disabledReason?: string | null;
}

export default function ExportDialog({
  importId,
  disabled = false,
  disabledReason,
}: ExportDialogProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [exportPath, setExportPath] = useState<string>("");
  const addToast = useUiStore((state) => state.addToast);

  const handleSelectPath = async () => {
    try {
      const filePath = await save({
        filters: [
          {
            name: "Export Pack",
            extensions: ["zip"],
          },
        ],
        defaultPath: "questionnaire-export.zip",
      });

      if (filePath && typeof filePath === "string") {
        setExportPath(filePath);
      }
    } catch (err) {
      console.error("Path selection failed:", err);
    }
  };

  const handleExport = async () => {
    if (!exportPath || disabled) return;

    setLoading(true);
    try {
      const result = await invokeGenerateExportPack(exportPath, importId);
      addToast({
        title: "Export Successful",
        description: `Export pack created with ${result.file_count} files`,
        variant: "success",
      });
      setIsOpen(false);
      setExportPath("");
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      addToast({
        title: "Export Failed",
        description: message,
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button size="lg" disabled={disabled} title={disabledReason ?? undefined}>
          Generate Export Pack
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Generate Export Pack</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <p className="text-sm text-muted-foreground">
            This will create a ZIP file containing the current questionnaire import metadata and
            mapped row snapshot, saved review entries, answer bank snapshot, license status, audit
            events, and evidence files.
          </p>

          {disabledReason && (
            <div className="rounded-md border border-destructive/20 bg-destructive/10 p-4">
              <p className="text-sm text-destructive">{disabledReason}</p>
            </div>
          )}

          <div className="flex items-end gap-4">
            <Input
              label="Export Location"
              value={exportPath || "No path selected"}
              readOnly
              className="flex-1"
            />
            <Button onClick={() => void handleSelectPath()} variant="outline" disabled={loading}>
              Browse...
            </Button>
          </div>

          <div className="flex gap-4 pt-4">
            <Button onClick={() => void handleExport()} disabled={!exportPath || loading || disabled}>
              {loading ? "Exporting..." : "Generate Export"}
            </Button>
            <Button variant="outline" onClick={() => setIsOpen(false)} disabled={loading}>
              Cancel
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

import { Link, useLocation } from "react-router-dom";
import { useImportStore } from "../../state/importStore";
import { useUiStore } from "../../state/uiStore";
import { useVaultStore } from "../../state/vaultStore";

const navItems = [
  { path: "/", label: "Import", icon: "📥", stage: "import" },
  { path: "/map", label: "Map Columns", icon: "🗺️", stage: "map" },
  { path: "/answer-bank", label: "Answer Bank", icon: "📚", stage: "answer-bank" },
  { path: "/review", label: "Review", icon: "✓", stage: "review" },
  { path: "/export", label: "Export", icon: "📤", stage: "export" },
] as const;

export function Sidebar() {
  const location = useLocation();
  const sidebarOpen = useUiStore((state) => state.sidebarOpen);
  const currentVault = useVaultStore((state) => state.currentVault);
  const currentImport = useImportStore((state) => state.currentImport);
  const currentStep = useImportStore((state) => state.step);
  const columnMap = useImportStore((state) => state.columnMap ?? state.currentImport?.column_map);

  if (!sidebarOpen) {
    return null;
  }

  const getTarget = (stage: (typeof navItems)[number]["stage"]) => {
    if (!currentVault) {
      return "/welcome";
    }
    if (stage === "import") {
      return "/";
    }
    if (!currentImport) {
      return "/";
    }
    if (stage === "map") {
      return "/map";
    }
    if (!columnMap) {
      return "/map";
    }
    return navItems.find((item) => item.stage === stage)?.path ?? "/";
  };

  const getHint = (stage: (typeof navItems)[number]["stage"]) => {
    if (!currentVault) {
      return "Open or create a vault first";
    }
    if (stage === "import") {
      return "Ready";
    }
    if (!currentImport) {
      return "Import a questionnaire first";
    }
    if (stage === "map") {
      return "Ready";
    }
    if (!columnMap) {
      return "Save a column map first";
    }
    return "Ready";
  };

  return (
    <aside className="w-64 border-r border-border bg-secondary">
      <div className="p-6">
        <h1 className="mb-6 text-xl font-bold text-foreground">Questionnaire Autopilot</h1>
        <div className="mb-6 rounded-lg border border-border bg-background/70 p-4">
          <p className="text-xs font-semibold uppercase tracking-[0.16em] text-muted-foreground">
            Vault
          </p>
          {currentVault ? (
            <div className="mt-2 space-y-1">
              <p className="font-medium text-foreground">{currentVault.name}</p>
              <p className="break-all text-xs text-muted-foreground">{currentVault.root_path}</p>
            </div>
          ) : (
            <p className="mt-2 text-sm text-muted-foreground">
              No vault open yet. Start from the welcome screen.
            </p>
          )}
          <p className="mt-3 text-xs text-muted-foreground">
            Current step: {currentStep === "answer-bank" ? "answer bank" : currentStep}
          </p>
        </div>
        <nav className="space-y-2">
          {navItems.map((item) => {
            const isActive = location.pathname === item.path;
            const target = getTarget(item.stage);
            const ready = target === item.path;

            return (
              <Link
                key={item.path}
                to={target}
                aria-disabled={!ready}
                title={getHint(item.stage)}
                className={`flex items-center gap-3 rounded-md px-4 py-3 transition-colors ${
                  isActive
                    ? "bg-primary text-primary-foreground"
                    : ready
                      ? "text-foreground hover:bg-accent hover:text-accent-foreground"
                      : "text-muted-foreground hover:bg-accent/60"
                }`}
              >
                <span className="text-lg">{item.icon}</span>
                <div className="min-w-0">
                  <p className="font-medium">{item.label}</p>
                  {!ready && <p className="text-xs opacity-80">{getHint(item.stage)}</p>}
                </div>
              </Link>
            );
          })}
        </nav>
      </div>
    </aside>
  );
}

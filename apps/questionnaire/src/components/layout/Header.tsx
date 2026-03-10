import { useNavigate } from "react-router-dom";
import { useVault } from "../../hooks/useVault";
import { useUiStore } from "../../state/uiStore";
import { useVaultStore } from "../../state/vaultStore";
import Button from "../ui/Button";

interface HeaderProps {
  title: string;
  subtitle?: string;
}

export function Header({ title, subtitle }: HeaderProps) {
  const navigate = useNavigate();
  const toggleSidebar = useUiStore((state) => state.toggleSidebar);
  const sidebarOpen = useUiStore((state) => state.sidebarOpen);
  const currentVault = useVaultStore((state) => state.currentVault);
  const { closeVault, loading } = useVault();

  const handleCloseVault = async () => {
    await closeVault();
    navigate("/welcome", { replace: true });
  };

  return (
    <header className="border-b border-border bg-background px-6 py-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="sm" onClick={toggleSidebar} aria-label="Toggle sidebar">
            {sidebarOpen ? "◄" : "►"}
          </Button>
          <div>
            <h1 className="text-2xl font-bold text-foreground">{title}</h1>
            {subtitle && <p className="text-sm text-muted-foreground">{subtitle}</p>}
          </div>
        </div>
        <div className="flex items-center gap-3">
          {currentVault && (
            <div className="hidden rounded-full border border-border bg-secondary/60 px-3 py-1 text-sm text-foreground md:block">
              {currentVault.name}
            </div>
          )}
          {currentVault && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => void handleCloseVault()}
              disabled={loading}
            >
              {loading ? "Closing..." : "Switch Vault"}
            </Button>
          )}
        </div>
      </div>
    </header>
  );
}

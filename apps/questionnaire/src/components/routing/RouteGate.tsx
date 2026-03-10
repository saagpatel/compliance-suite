import type { ReactNode } from "react";
import { Navigate } from "react-router-dom";
import { useImportStore } from "../../state/importStore";
import { useVaultStore } from "../../state/vaultStore";

interface RouteGateProps {
  stage: "import" | "map" | "answer-bank" | "review" | "export";
  children: ReactNode;
}

export default function RouteGate({ stage, children }: RouteGateProps) {
  const currentVault = useVaultStore((state) => state.currentVault);
  const currentImport = useImportStore((state) => state.currentImport);
  const columnMap = useImportStore(
    (state) => state.columnMap ?? state.currentImport?.column_map ?? null
  );

  if (!currentVault) {
    return <Navigate to="/welcome" replace />;
  }

  if (stage === "import") {
    return <>{children}</>;
  }

  if (!currentImport) {
    return <Navigate to="/" replace />;
  }

  if (stage === "map") {
    return <>{children}</>;
  }

  if (!columnMap) {
    return <Navigate to="/map" replace />;
  }

  return <>{children}</>;
}

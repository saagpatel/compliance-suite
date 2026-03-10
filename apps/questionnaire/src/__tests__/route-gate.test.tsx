import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import RouteGate from "../components/routing/RouteGate";
import { useImportStore } from "../state/importStore";
import { useVaultStore } from "../state/vaultStore";

describe("RouteGate", () => {
  beforeEach(() => {
    localStorage.clear();
    useVaultStore.setState({
      currentVault: null,
      lastVaultPath: null,
      loading: false,
      error: null,
      initializing: false,
      initialized: true,
    });
    useImportStore.getState().reset();
  });

  it("redirects to welcome when no vault is open", () => {
    render(
      <MemoryRouter initialEntries={["/map"]}>
        <Routes>
          <Route
            path="/map"
            element={
              <RouteGate stage="map">
                <div>Map page</div>
              </RouteGate>
            }
          />
          <Route path="/welcome" element={<div>Welcome page</div>} />
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText("Welcome page")).toBeInTheDocument();
  });

  it("redirects to import when import prerequisites are missing", () => {
    useVaultStore.setState({
      currentVault: {
        vault_id: "vault_1",
        name: "Test Vault",
        root_path: "/tmp/test-vault",
        created_at: "2026-03-10T00:00:00Z",
        encryption_mode: "none",
        schema_version: 1,
      },
    });

    render(
      <MemoryRouter initialEntries={["/answer-bank"]}>
        <Routes>
          <Route path="/" element={<div>Import page</div>} />
          <Route
            path="/answer-bank"
            element={
              <RouteGate stage="answer-bank">
                <div>Answer bank page</div>
              </RouteGate>
            }
          />
        </Routes>
      </MemoryRouter>
    );

    expect(screen.getByText("Import page")).toBeInTheDocument();
  });
});

import { useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import RouteGate from "./components/routing/RouteGate";
import { useVault } from "./hooks/useVault";
import AnswerBankPage from "./routes/AnswerBank";
import ExportPage from "./routes/Export";
import ImportPage from "./routes/Import";
import MapPage from "./routes/Map";
import ReviewPage from "./routes/Review";
import WelcomePage from "./routes/Welcome";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

export default function App() {
  const { bootstrapVault, currentVault, initialized, initializing } = useVault();

  useEffect(() => {
    void bootstrapVault();
  }, [bootstrapVault]);

  if (!initialized || initializing) {
    return (
      <QueryClientProvider client={queryClient}>
        <div className="flex min-h-screen items-center justify-center bg-background px-6">
          <div className="max-w-md space-y-3 text-center">
            <p className="text-sm font-semibold uppercase tracking-[0.2em] text-primary">
              Compliance Suite
            </p>
            <h1 className="text-3xl font-bold text-foreground">Preparing your local workspace</h1>
            <p className="text-sm text-muted-foreground">
              Reconnecting to your last vault and loading the questionnaire workspace.
            </p>
          </div>
        </div>
      </QueryClientProvider>
    );
  }

  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route
            path="/welcome"
            element={currentVault ? <Navigate to="/" replace /> : <WelcomePage />}
          />
          <Route
            path="/"
            element={
              <RouteGate stage="import">
                <ImportPage />
              </RouteGate>
            }
          />
          <Route
            path="/map"
            element={
              <RouteGate stage="map">
                <MapPage />
              </RouteGate>
            }
          />
          <Route
            path="/answer-bank"
            element={
              <RouteGate stage="answer-bank">
                <AnswerBankPage />
              </RouteGate>
            }
          />
          <Route
            path="/review"
            element={
              <RouteGate stage="review">
                <ReviewPage />
              </RouteGate>
            }
          />
          <Route
            path="/export"
            element={
              <RouteGate stage="export">
                <ExportPage />
              </RouteGate>
            }
          />
          <Route path="*" element={<Navigate to={currentVault ? "/" : "/welcome"} replace />} />
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  );
}

import { useNavigate } from "react-router-dom";
import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import ImportForm from "../components/features/ImportForm";
import { Toaster } from "../components/ui/Toast";
import { useImport } from "../hooks/useImport";

export default function ImportPage() {
  const navigate = useNavigate();
  const { setStep } = useImport();

  const handleImportComplete = () => {
    setStep("map");
    navigate("/map");
  };

  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header title="Import Questionnaire" subtitle="Step 1: Import your questionnaire file" />
        <main className="flex-1 overflow-auto p-8">
          <div className="max-w-4xl mx-auto">
            <ImportForm onComplete={handleImportComplete} />
          </div>
        </main>
      </div>
      <Toaster />
    </div>
  );
}

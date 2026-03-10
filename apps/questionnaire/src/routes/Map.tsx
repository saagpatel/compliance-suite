import { useNavigate } from "react-router-dom";
import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import ColumnMapTable from "../components/features/ColumnMapTable";
import { Toaster } from "../components/ui/Toast";
import { useImport } from "../hooks/useImport";

export default function MapPage() {
  const navigate = useNavigate();
  const { setStep } = useImport();

  const handleMappingComplete = () => {
    setStep("answer-bank");
    navigate("/answer-bank");
  };

  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header
          title="Map Columns"
          subtitle="Step 2: Map your file columns to questionnaire fields"
        />
        <main className="flex-1 overflow-auto p-8">
          <div className="max-w-6xl mx-auto">
            <ColumnMapTable onComplete={handleMappingComplete} />
          </div>
        </main>
      </div>
      <Toaster />
    </div>
  );
}

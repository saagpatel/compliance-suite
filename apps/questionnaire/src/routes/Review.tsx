import { useNavigate } from "react-router-dom";
import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import MatchingResults from "../components/features/MatchingResults";
import Button from "../components/ui/Button";
import { Toaster } from "../components/ui/Toast";
import { useImport } from "../hooks/useImport";

export default function ReviewPage() {
  const navigate = useNavigate();
  const { currentImport, setStep } = useImport();

  const handleContinue = () => {
    setStep("export");
    navigate("/export");
  };

  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header
          title="Review & Match"
          subtitle="Step 4: Review mapped questionnaire rows and confirm final answers"
        />
        <main className="flex-1 overflow-auto p-8">
          <div className="max-w-6xl mx-auto space-y-6">
            {currentImport && <MatchingResults importId={currentImport.import_id} />}
            <div className="flex items-center justify-between rounded-lg border border-border bg-secondary/20 p-4">
              <p className="text-sm text-muted-foreground">
                Move forward when you are ready to package the current questionnaire work.
              </p>
              <Button onClick={handleContinue}>Continue to Export</Button>
            </div>
          </div>
        </main>
      </div>
      <Toaster />
    </div>
  );
}

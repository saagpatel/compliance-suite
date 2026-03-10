import { useNavigate } from "react-router-dom";
import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import AnswerBankTable from "../components/features/AnswerBankTable";
import Button from "../components/ui/Button";
import { Toaster } from "../components/ui/Toast";
import { useImport } from "../hooks/useImport";

export default function AnswerBankPage() {
  const navigate = useNavigate();
  const { setStep } = useImport();

  const handleContinue = () => {
    setStep("review");
    navigate("/review");
  };

  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header title="Answer Bank" subtitle="Step 3: Manage your answer bank entries" />
        <main className="flex-1 overflow-auto p-8">
          <div className="max-w-6xl mx-auto space-y-6">
            <AnswerBankTable />
            <div className="flex items-center justify-between rounded-lg border border-border bg-secondary/20 p-4">
              <p className="text-sm text-muted-foreground">
                Continue once your answer bank is ready for review suggestions.
              </p>
              <Button onClick={handleContinue}>Continue to Review</Button>
            </div>
          </div>
        </main>
      </div>
      <Toaster />
    </div>
  );
}

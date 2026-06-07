import { Routes, Route, Navigate } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import DashboardPage from "./pages/DashboardPage";
import ProblemsListPage from "./pages/ProblemsListPage";
import ProblemFormPage from "./pages/ProblemFormPage";
import ProblemDetailPage from "./pages/ProblemDetailPage";
import SubmissionsListPage from "./pages/SubmissionsListPage";
import KnowledgeListPage from "./pages/KnowledgeListPage";
import AnalysisPage from "./pages/AnalysisPage";
import SettingsPage from "./pages/SettingsPage";
import AppShell from "./components/layout/AppShell";
import { useAuth } from "./lib/stores/auth";

function Protected({ children }: { children: React.ReactNode }) {
  const user = useAuth((s) => s.user);
  const loading = useAuth((s) => s.loading);
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center text-muted-foreground">
        Loading…
      </div>
    );
  }
  if (!user) return <Navigate to="/login" replace />;
  return <>{children}</>;
}

export default function AppRouter() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route path="/register" element={<RegisterPage />} />
      <Route
        element={
          <Protected>
            <AppShell />
          </Protected>
        }
      >
        <Route path="/" element={<DashboardPage />} />
        <Route path="/problems" element={<ProblemsListPage />} />
        <Route path="/problems/new" element={<ProblemFormPage />} />
        <Route path="/problems/:id" element={<ProblemDetailPage />} />
        <Route path="/problems/:id/edit" element={<ProblemFormPage />} />
        <Route path="/submissions" element={<SubmissionsListPage />} />
        <Route path="/knowledge" element={<KnowledgeListPage />} />
        <Route path="/analysis" element={<AnalysisPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  );
}

import { Routes, Route, Navigate } from "react-router-dom";
import LoginPage from "./pages/LoginPage";
import RegisterPage from "./pages/RegisterPage";
import DashboardPage from "./pages/DashboardPage";
import ProblemsListPage from "./pages/ProblemsListPage";
import ProblemFormPage from "./pages/ProblemFormPage";
import ProblemDetailPage from "./pages/ProblemDetailPage";
import KnowledgeListPage from "./pages/KnowledgeListPage";
import KnowledgeDetailPage from "./pages/KnowledgeDetailPage";
import KnowledgeFormPage from "./pages/KnowledgeFormPage";
import AnalysisPage from "./pages/AnalysisPage";
import TemplatesListPage from "./pages/TemplatesListPage";
import TemplateDetailPage from "./pages/TemplateDetailPage";
import TemplateFormPage from "./pages/TemplateFormPage";
import TasksPage from "./pages/TasksPage";
import SettingsPage from "./pages/SettingsPage";
import NotFoundPage from "./pages/NotFoundPage";
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
        <Route path="/knowledge" element={<KnowledgeListPage />} />
        <Route path="/knowledge/new" element={<KnowledgeFormPage />} />
        <Route path="/knowledge/:id" element={<KnowledgeDetailPage />} />
        <Route path="/knowledge/:id/edit" element={<KnowledgeFormPage />} />
        <Route path="/analysis" element={<AnalysisPage />} />
        <Route path="/templates" element={<TemplatesListPage />} />
        <Route path="/templates/new" element={<TemplateFormPage />} />
        <Route path="/templates/:id" element={<TemplateDetailPage />} />
        <Route path="/templates/:id/edit" element={<TemplateFormPage />} />
        <Route path="/tasks" element={<TasksPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="*" element={<NotFoundPage />} />
      </Route>
    </Routes>
  );
}

import { Routes, Route } from "react-router-dom";
import { AppSidebar } from "./components/layout/AppSidebar";
import { DashboardPage } from "./pages/DashboardPage";
import { ProblemsPage } from "./pages/ProblemsPage";
import { ProblemDetailPage } from "./pages/ProblemDetailPage";
import { ReportsPage } from "./pages/ReportsPage";
import { KnowledgePage } from "./pages/KnowledgePage";
import { SettingsPage } from "./pages/SettingsPage";

function App() {
	return (
		<div className="flex h-screen overflow-hidden">
			<AppSidebar />
			<main className="flex-1 overflow-y-auto p-6">
				<Routes>
					<Route path="/" element={<DashboardPage />} />
					<Route path="/problems" element={<ProblemsPage />} />
					<Route path="/problems/:id" element={<ProblemDetailPage />} />
					<Route path="/reports" element={<ReportsPage />} />
					<Route path="/knowledge" element={<KnowledgePage />} />
					<Route path="/settings" element={<SettingsPage />} />
				</Routes>
			</main>
		</div>
	);
}

export default App;

import { Outlet, useLocation } from "react-router-dom";
import { AnimatePresence } from "motion/react";
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";
import PageTransition from "./PageTransition";

export default function AppShell() {
  const location = useLocation();
  return (
    <div className="min-h-screen flex">
      <Sidebar />
      <div className="flex-1 flex flex-col">
        <TopBar />
        <main className="flex-1 overflow-y-auto p-6">
          <AnimatePresence mode="wait">
            <PageTransition key={location.pathname}>
              <Outlet />
            </PageTransition>
          </AnimatePresence>
        </main>
      </div>
    </div>
  );
}

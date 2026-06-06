import { NavLink } from "react-router-dom";
import { Home, ListChecks, GitPullRequest, BookOpen, BarChart3, Settings } from "lucide-react";
import { cn } from "@/lib/utils";

const items = [
  { to: "/", label: "Dashboard", icon: Home },
  { to: "/problems", label: "Problems", icon: ListChecks },
  { to: "/submissions", label: "Submissions", icon: GitPullRequest },
  { to: "/knowledge", label: "Knowledge", icon: BookOpen },
  { to: "/analysis", label: "Analysis", icon: BarChart3 },
  { to: "/settings", label: "Settings", icon: Settings },
];

export default function Sidebar() {
  return (
    <aside className="w-56 border-r border-[var(--color-border)] bg-[var(--color-card)] p-4">
      <h1 className="text-xl font-bold mb-6">ACMind</h1>
      <nav className="space-y-1">
        {items.map((i) => (
          <NavLink
            key={i.to}
            to={i.to}
            end
            className={({ isActive }) =>
              cn(
                "flex items-center gap-2 px-3 py-2 rounded-md text-sm",
                isActive
                  ? "bg-[var(--color-accent)] text-[var(--color-accent-foreground)]"
                  : "hover:bg-[var(--color-accent)]",
              )
            }
          >
            <i.icon className="h-4 w-4" />
            {i.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}

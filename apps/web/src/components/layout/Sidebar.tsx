import { NavLink } from "react-router-dom";
import { motion } from "motion/react";
import { Home, ListChecks, BookOpen, BarChart3, Code2, Activity, Settings } from "lucide-react";
import { cn } from "@/lib/utils";

const items = [
  { to: "/", label: "仪表盘", icon: Home },
  { to: "/problems", label: "题目", icon: ListChecks },
  { to: "/templates", label: "模板库", icon: Code2 },
  { to: "/knowledge", label: "知识库", icon: BookOpen },
  { to: "/analysis", label: "数据分析", icon: BarChart3 },
  { to: "/tasks", label: "任务中心", icon: Activity },
  { to: "/settings", label: "设置", icon: Settings },
];

export default function Sidebar() {
  return (
    <aside className="w-56 border-r border-border bg-card p-4">
      <motion.h1
        initial={{ opacity: 0, x: -8 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.3, ease: "easeOut" }}
        className="text-xl font-bold mb-6"
      >
        ACMind
      </motion.h1>
      <nav className="space-y-1">
        {items.map((i, idx) => (
          <motion.div
            key={i.to}
            initial={{ opacity: 0, x: -8 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.25, delay: 0.05 + idx * 0.04, ease: "easeOut" }}
            whileHover={{ x: 2 }}
            whileTap={{ scale: 0.98 }}
          >
            <NavLink
              to={i.to}
              end
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors",
                  isActive
                    ? "bg-accent text-accent-foreground"
                    : "hover:bg-accent",
                )
              }
            >
              <i.icon className="h-4 w-4" />
              {i.label}
            </NavLink>
          </motion.div>
        ))}
      </nav>
    </aside>
  );
}

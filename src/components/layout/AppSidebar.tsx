import { NavLink, useLocation } from "react-router-dom";
import {
	LayoutDashboard,
	BookOpen,
	FileText,
	Brain,
	Settings,
	ChevronLeft,
	ChevronRight,
} from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

const navItems = [
	{ to: "/", icon: LayoutDashboard, labelKey: "nav.dashboard" },
	{ to: "/problems", icon: BookOpen, labelKey: "nav.problems" },
	{ to: "/reports", icon: FileText, labelKey: "nav.reports" },
	{ to: "/knowledge", icon: Brain, labelKey: "nav.knowledge" },
	{ to: "/settings", icon: Settings, labelKey: "nav.settings" },
];

export function AppSidebar() {
	const [collapsed, setCollapsed] = useState(false);
	const location = useLocation();
	const { t } = useTranslation();

	return (
		<aside
			className={cn(
				"flex flex-col border-r bg-sidebar text-sidebar-foreground transition-all duration-200",
				collapsed ? "w-16" : "w-56",
			)}
		>
			{/* App title */}
			<div className="flex h-14 items-center justify-between border-b border-sidebar-border px-4">
				{!collapsed && (
					<span className="text-lg font-bold tracking-tight">{t("app.name")}</span>
				)}
				<button
					onClick={() => setCollapsed(!collapsed)}
					className="rounded-md p-1.5 hover:bg-sidebar-accent"
					title={
						collapsed ? t("nav.expandSidebar") : t("nav.collapseSidebar")
					}
				>
					{collapsed ? (
						<ChevronRight className="h-4 w-4" />
					) : (
						<ChevronLeft className="h-4 w-4" />
					)}
				</button>
			</div>

			{/* Navigation */}
			<nav className="flex-1 space-y-1 p-2">
				{navItems.map((item) => {
					const isActive =
						item.to === "/"
							? location.pathname === "/"
							: location.pathname.startsWith(item.to);
					return (
						<NavLink
							key={item.to}
							to={item.to}
							className={cn(
								"flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
								isActive
									? "bg-sidebar-accent text-sidebar-accent-foreground"
									: "hover:bg-sidebar-accent/50",
							)}
						>
							<item.icon className="h-5 w-5 shrink-0" />
							{!collapsed && <span>{t(item.labelKey)}</span>}
						</NavLink>
					);
				})}
			</nav>

			{/* Footer */}
			<div className="border-t border-sidebar-border p-3">
				{!collapsed && <p className="text-xs text-muted-foreground">v0.1.0</p>}
			</div>
		</aside>
	);
}

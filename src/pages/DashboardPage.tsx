import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
	BookOpen,
	CheckCircle2,
	XCircle,
	Clock,
	AlertTriangle,
} from "lucide-react";

// TODO: Replace with actual invoke calls when running in Tauri
// import { invoke } from "@tauri-apps/api/core";

interface DashboardStats {
	total_problems: number;
	total_submissions: number;
	ac_count: number;
	wa_count: number;
	tle_count: number;
	re_count: number;
	other_count: number;
}

async function fetchDashboardStats(): Promise<DashboardStats> {
	try {
		const { invoke } = await import("@tauri-apps/api/core");
		return await invoke<DashboardStats>("get_dashboard_stats");
	} catch {
		// Return mock data for dev mode (browser, not Tauri)
		return {
			total_problems: 0,
			total_submissions: 0,
			ac_count: 0,
			wa_count: 0,
			tle_count: 0,
			re_count: 0,
			other_count: 0,
		};
	}
}

export function DashboardPage() {
	const { data, isLoading } = useQuery({
		queryKey: ["dashboard-stats"],
		queryFn: fetchDashboardStats,
	});

	const acRate =
		data && data.total_submissions > 0
			? ((data.ac_count / data.total_submissions) * 100).toFixed(1)
			: "—";

	const statCards = [
		{
			title: "Total Problems",
			value: data?.total_problems ?? "—",
			icon: BookOpen,
			color: "text-blue-500",
		},
		{
			title: "AC",
			value: data?.ac_count ?? "—",
			icon: CheckCircle2,
			color: "text-success",
		},
		{
			title: "WA",
			value: data?.wa_count ?? "—",
			icon: XCircle,
			color: "text-error",
		},
		{
			title: "TLE",
			value: data?.tle_count ?? "—",
			icon: Clock,
			color: "text-warning",
		},
		{
			title: "RE",
			value: data?.re_count ?? "—",
			icon: AlertTriangle,
			color: "text-destructive",
		},
		{
			title: "AC Rate",
			value: `${acRate}%`,
			icon: CheckCircle2,
			color: "text-green-500",
		},
	];

	return (
		<div className="space-y-6">
			<div>
				<h1 className="text-2xl font-bold">Dashboard</h1>
				<p className="text-muted-foreground">Your ACM training overview</p>
			</div>

			{/* Stats cards */}
			<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
				{statCards.map((stat) => (
					<Card key={stat.title}>
						<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
							<CardTitle className="text-sm font-medium text-muted-foreground">
								{stat.title}
							</CardTitle>
							<stat.icon className={`h-4 w-4 ${stat.color}`} />
						</CardHeader>
						<CardContent>
							{isLoading ? (
								<Skeleton className="h-8 w-16" />
							) : (
								<div className="text-2xl font-bold">{stat.value}</div>
							)}
						</CardContent>
					</Card>
				))}
			</div>

			{/* Getting started */}
			{(!data || data.total_problems === 0) && (
				<Card className="border-dashed">
					<CardHeader>
						<CardTitle>Getting Started</CardTitle>
					</CardHeader>
					<CardContent className="space-y-3">
						<p className="text-muted-foreground">
							Your ACM training database is empty. Start by adding your first
							problem!
						</p>
						<div className="flex flex-col gap-2 text-sm text-muted-foreground">
							<p>
								1. Go to <strong>Problems</strong> and click{" "}
								<strong>Add Problem</strong>
							</p>
							<p>2. Record your submissions (WA and AC codes)</p>
							<p>3. Add solution notes and AI analysis</p>
							<p>4. View your training reports here</p>
						</div>
					</CardContent>
				</Card>
			)}
		</div>
	);
}

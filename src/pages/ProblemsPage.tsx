import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { Plus, Trash2, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { Problem } from "@/lib/types";

async function fetchProblems(): Promise<Problem[]> {
	try {
		const { invoke } = await import("@tauri-apps/api/core");
		return await invoke<Problem[]>("list_problems");
	} catch {
		return [];
	}
}

const difficultyLabels: Record<
	number,
	{ label: string; variant: "success" | "warning" | "error" }
> = {
	800: { label: "800", variant: "success" },
	1200: { label: "1200", variant: "success" },
	1600: { label: "1600", variant: "warning" },
	2000: { label: "2000", variant: "warning" },
	2400: { label: "2400", variant: "error" },
	2800: { label: "2800", variant: "error" },
	3200: { label: "3200", variant: "error" },
};

function getDifficultyBadge(difficulty?: number) {
	if (!difficulty) return null;
	// Find the closest label
	const keys = Object.keys(difficultyLabels)
		.map(Number)
		.sort((a, b) => a - b);
	const closest = keys.reduce((prev, curr) =>
		Math.abs(curr - difficulty) < Math.abs(prev - difficulty) ? curr : prev,
	);
	const info = difficultyLabels[closest];
	return (
		<Badge variant={info.variant} className="text-xs">
			{difficulty}
		</Badge>
	);
}

export function ProblemsPage() {
	const queryClient = useQueryClient();
	const { t } = useTranslation();
	const { data: problems, isLoading } = useQuery({
		queryKey: ["problems"],
		queryFn: fetchProblems,
	});

	const [search, setSearch] = useState("");
	const [dialogOpen, setDialogOpen] = useState(false);
	const [form, setForm] = useState({
		title: "",
		source: "",
		source_problem_id: "",
		url: "",
		difficulty: "",
		tags: "",
		statement: "",
	});

	const createMutation = useMutation({
		mutationFn: async () => {
			const { invoke } = await import("@tauri-apps/api/core");
			return invoke("create_problem", {
				input: {
					source: form.source,
					source_problem_id: form.source_problem_id,
					title: form.title,
					url: form.url || undefined,
					difficulty: form.difficulty ? parseInt(form.difficulty) : undefined,
					tags: form.tags
						.split(",")
						.map((t) => t.trim())
						.filter(Boolean),
					statement: form.statement || undefined,
				},
			});
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["problems"] });
			setDialogOpen(false);
			setForm({
				title: "",
				source: "",
				source_problem_id: "",
				url: "",
				difficulty: "",
				tags: "",
				statement: "",
			});
		},
	});

	const deleteMutation = useMutation({
		mutationFn: async (id: string) => {
			const { invoke } = await import("@tauri-apps/api/core");
			return invoke("delete_problem", { id });
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["problems"] });
		},
	});

	const filtered = problems?.filter(
		(p) =>
			p.title.toLowerCase().includes(search.toLowerCase()) ||
			p.source.toLowerCase().includes(search.toLowerCase()) ||
			p.tags.some((t) => t.toLowerCase().includes(search.toLowerCase())),
	);

	return (
		<div className="space-y-6">
			<div className="flex items-center justify-between">
				<div>
					<h1 className="text-2xl font-bold">{t("problems.title")}</h1>
					<p className="text-muted-foreground">
						{t("problems.recorded", { count: problems?.length ?? 0 })}
					</p>
				</div>

				<div className="flex gap-2">
					<Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
						<DialogTrigger asChild>
							<Button>
								<Plus className="mr-2 h-4 w-4" />
								添加题目
							</Button>
						</DialogTrigger>
						<DialogContent className="sm:max-w-lg">
							<DialogHeader>
								<DialogTitle>{t("problems.addNew")}</DialogTitle>
							</DialogHeader>
							<div className="grid gap-4 py-4">
								<div className="grid gap-2">
									<Label htmlFor="title">{t("problems.problemTitle")}</Label>
									<Input
										id="title"
										value={form.title}
										onChange={(e) =>
											setForm({ ...form, title: e.target.value })
										}
										placeholder={t("problems.placeholderTitle")}
									/>
								</div>
								<div className="grid grid-cols-2 gap-4">
									<div className="grid gap-2">
										<Label htmlFor="source">{t("problems.source")}</Label>
										<Input
											id="source"
											value={form.source}
											onChange={(e) =>
												setForm({ ...form, source: e.target.value })
											}
										/>
									</div>
									<div className="grid gap-2">
										<Label htmlFor="source_id">{t("problems.problemId")}</Label>
										<Input
											id="source_id"
											value={form.source_problem_id}
											onChange={(e) =>
												setForm({ ...form, source_problem_id: e.target.value })
											}
											placeholder="e.g. 1900A"
										/>
									</div>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="url">URL</Label>
									<Input
										id="url"
										value={form.url}
										onChange={(e) => setForm({ ...form, url: e.target.value })}
										placeholder="https://vjudge.net/..."
									/>
								</div>
								<div className="grid grid-cols-2 gap-4">
									<div className="grid gap-2">
										<Label htmlFor="difficulty">
											{t("problems.difficultyRating")}
										</Label>
										<Input
											id="difficulty"
											type="number"
											value={form.difficulty}
											onChange={(e) =>
												setForm({ ...form, difficulty: e.target.value })
											}
											placeholder="e.g. 1600"
										/>
									</div>
									<div className="grid gap-2">
										<Label htmlFor="tags">{t("problems.tagsComma")}</Label>
										<Input
											id="tags"
											value={form.tags}
											onChange={(e) =>
												setForm({ ...form, tags: e.target.value })
											}
											placeholder="dp, greedy, graph"
										/>
									</div>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="statement">{t("problems.statement")}</Label>
									<Textarea
										id="statement"
										rows={4}
										value={form.statement}
										onChange={(e) =>
											setForm({ ...form, statement: e.target.value })
										}
										placeholder={t("problems.placeholderStatement")}
									/>
								</div>
							</div>
							<div className="flex justify-end gap-2">
								<Button variant="outline" onClick={() => setDialogOpen(false)}>
									{t("common.cancel")}
								</Button>
								<Button
									onClick={() => createMutation.mutate()}
									disabled={!form.title || createMutation.isPending}
								>
									{createMutation.isPending
										? t("problems.adding")
										: t("problems.add")}
								</Button>
							</div>
						</DialogContent>
					</Dialog>
				</div>
			</div>

			{/* Search */}
			<Input
				placeholder={t("problems.search")}
				value={search}
				onChange={(e) => setSearch(e.target.value)}
				className="max-w-sm"
			/>

			{/* Table */}
			{isLoading && !problems ? (
				<div className="space-y-2">
					{Array.from({ length: 5 }).map((_, i) => (
						<Skeleton key={i} className="h-12 w-full" />
					))}
				</div>
			) : filtered && filtered.length > 0 ? (
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead>{t("problems.problemTitle")}</TableHead>
							<TableHead>{t("problems.source")}</TableHead>
							<TableHead>{t("problems.difficulty")}</TableHead>
							<TableHead>{t("problems.tags")}</TableHead>
							<TableHead className="w-20">{t("problems.actions")}</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{filtered.map((problem) => (
							<TableRow key={problem.id}>
								<TableCell>
									<Link
										to={`/problems/${problem.id}`}
										className="font-medium hover:underline"
									>
										{problem.title}
									</Link>
								</TableCell>
								<TableCell className="text-muted-foreground">
									{problem.source} #{problem.source_problem_id}
								</TableCell>
								<TableCell>{getDifficultyBadge(problem.difficulty)}</TableCell>
								<TableCell>
									<div className="flex flex-wrap gap-1">
										{problem.tags.map((tag) => (
											<Badge key={tag} variant="secondary" className="text-xs">
												{tag}
											</Badge>
										))}
									</div>
								</TableCell>
								<TableCell>
									<div className="flex gap-1">
										{problem.url && (
											<a
												href={problem.url}
												target="_blank"
												rel="noopener noreferrer"
												className="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-muted"
											>
												<ExternalLink className="h-4 w-4" />
											</a>
										)}
										<button
											onClick={() => {
												if (confirm(t("problems.deleteConfirm"))) {
													deleteMutation.mutate(problem.id);
												}
											}}
											className="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-destructive/10 text-destructive"
										>
											<Trash2 className="h-4 w-4" />
										</button>
									</div>
								</TableCell>
							</TableRow>
						))}
					</TableBody>
				</Table>
			) : (
				<p className="text-muted-foreground">{t("problems.empty")}</p>
			)}
		</div>
	);
}

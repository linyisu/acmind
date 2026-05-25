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
	const { data: problems, isLoading } = useQuery({
		queryKey: ["problems"],
		queryFn: fetchProblems,
	});

	const [search, setSearch] = useState("");
	const [dialogOpen, setDialogOpen] = useState(false);
	const [form, setForm] = useState({
		title: "",
		source: "Codeforces",
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
				source: "Codeforces",
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
					<h1 className="text-2xl font-bold">Problems</h1>
					<p className="text-muted-foreground">
						{problems?.length ?? 0} problems recorded
					</p>
				</div>

				<Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
					<DialogTrigger asChild>
						<Button>
							<Plus className="mr-2 h-4 w-4" />
							Add Problem
						</Button>
					</DialogTrigger>
					<DialogContent className="sm:max-w-lg">
						<DialogHeader>
							<DialogTitle>Add New Problem</DialogTitle>
						</DialogHeader>
						<div className="grid gap-4 py-4">
							<div className="grid gap-2">
								<Label htmlFor="title">Title *</Label>
								<Input
									id="title"
									value={form.title}
									onChange={(e) => setForm({ ...form, title: e.target.value })}
									placeholder="Problem title"
								/>
							</div>
							<div className="grid grid-cols-2 gap-4">
								<div className="grid gap-2">
									<Label htmlFor="source">Source</Label>
									<Input
										id="source"
										value={form.source}
										onChange={(e) =>
											setForm({ ...form, source: e.target.value })
										}
									/>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="source_id">Problem ID</Label>
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
									placeholder="https://codeforces.com/..."
								/>
							</div>
							<div className="grid grid-cols-2 gap-4">
								<div className="grid gap-2">
									<Label htmlFor="difficulty">Difficulty (rating)</Label>
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
									<Label htmlFor="tags">Tags (comma-separated)</Label>
									<Input
										id="tags"
										value={form.tags}
										onChange={(e) => setForm({ ...form, tags: e.target.value })}
										placeholder="dp, greedy, graph"
									/>
								</div>
							</div>
							<div className="grid gap-2">
								<Label htmlFor="statement">
									Statement (Markdown, optional)
								</Label>
								<Textarea
									id="statement"
									rows={4}
									value={form.statement}
									onChange={(e) =>
										setForm({ ...form, statement: e.target.value })
									}
									placeholder="Paste the problem statement here..."
								/>
							</div>
						</div>
						<div className="flex justify-end gap-2">
							<Button variant="outline" onClick={() => setDialogOpen(false)}>
								Cancel
							</Button>
							<Button
								onClick={() => createMutation.mutate()}
								disabled={!form.title || createMutation.isPending}
							>
								{createMutation.isPending ? "Adding..." : "Add Problem"}
							</Button>
						</div>
					</DialogContent>
				</Dialog>
			</div>

			{/* Search */}
			<Input
				placeholder="Search by title, source, or tags..."
				value={search}
				onChange={(e) => setSearch(e.target.value)}
				className="max-w-sm"
			/>

			{/* Table */}
			{isLoading ? (
				<div className="space-y-2">
					{Array.from({ length: 5 }).map((_, i) => (
						<Skeleton key={i} className="h-12 w-full" />
					))}
				</div>
			) : filtered && filtered.length > 0 ? (
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead>Title</TableHead>
							<TableHead>Source</TableHead>
							<TableHead>Difficulty</TableHead>
							<TableHead>Tags</TableHead>
							<TableHead className="w-20">Actions</TableHead>
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
												if (confirm("Delete this problem?")) {
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
				<p className="text-muted-foreground">No problems found.</p>
			)}
		</div>
	);
}

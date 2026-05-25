import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
	ArrowLeft,
	Plus,
	Trash2,
	Code,
	FileText,
	Brain,
	Sparkles,
	Loader2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Skeleton } from "@/components/ui/skeleton";
import {
	Dialog,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { useState } from "react";
import type {
	Problem,
	Submission,
	SolutionNote,
	ErrorAnalysis,
} from "@/lib/types";

// API helpers
async function api<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
	const { invoke } = await import("@tauri-apps/api/core");
	return invoke<T>(cmd, args);
}

const statusColors: Record<
	string,
	"success" | "error" | "warning" | "secondary"
> = {
	AC: "success",
	WA: "error",
	TLE: "warning",
	RE: "error",
	MLE: "warning",
	CE: "secondary",
};

export function ProblemDetailPage() {
	const { id } = useParams<{ id: string }>();
	const navigate = useNavigate();
	const queryClient = useQueryClient();

	const { data: problem, isLoading } = useQuery({
		queryKey: ["problem", id],
		queryFn: () => api<Problem>("get_problem", { id }),
		enabled: !!id,
	});

	const { data: submissions } = useQuery({
		queryKey: ["submissions", id],
		queryFn: () =>
			api<Submission[]>("list_submissions_by_problem", { problemId: id }),
		enabled: !!id,
	});

	const { data: notes } = useQuery({
		queryKey: ["notes", id],
		queryFn: () =>
			api<SolutionNote[]>("list_notes_by_problem", { problemId: id }),
		enabled: !!id,
	});

	const { data: errors } = useQuery({
		queryKey: ["errors", id],
		queryFn: () =>
			api<ErrorAnalysis[]>("list_error_analyses_by_problem", { problemId: id }),
		enabled: !!id,
	});

	// Add submission dialog
	const [subDialogOpen, setSubDialogOpen] = useState(false);
	const [subForm, setSubForm] = useState({
		status: "AC",
		language: "C++",
		code_text: "",
		runtime: "",
		memory: "",
		note: "",
	});

	const createSub = useMutation({
		mutationFn: () =>
			api("create_submission", {
				input: {
					problem_id: id,
					status: subForm.status,
					language: subForm.language,
					code_text: subForm.code_text,
					runtime: subForm.runtime ? parseInt(subForm.runtime) : undefined,
					memory: subForm.memory ? parseInt(subForm.memory) : undefined,
					note: subForm.note || undefined,
				},
			}),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["submissions", id] });
			queryClient.invalidateQueries({ queryKey: ["dashboard-stats"] });
			setSubDialogOpen(false);
			setSubForm({
				status: "AC",
				language: "C++",
				code_text: "",
				runtime: "",
				memory: "",
				note: "",
			});
		},
	});

	const deleteSub = useMutation({
		mutationFn: (subId: string) => api("delete_submission", { id: subId }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["submissions", id] });
			queryClient.invalidateQueries({ queryKey: ["dashboard-stats"] });
		},
	});

	// Add note dialog
	const [noteDialogOpen, setNoteDialogOpen] = useState(false);
	const [noteForm, setNoteForm] = useState({
		note_type: "self",
		content: "",
		source_url: "",
	});

	const [analyzing, setAnalyzing] = useState(false);
	const [analysisError, setAnalysisError] = useState("");

	const analyzeMutation = useMutation({
		mutationFn: () => api("analyze_problem", { problemId: id }),
		onMutate: () => {
			setAnalyzing(true);
			setAnalysisError("");
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["errors", id] });
			queryClient.invalidateQueries({ queryKey: ["notes", id] });
			setAnalyzing(false);
		},
		onError: (err) => {
			// Tauri errors come as strings, not Error objects
			setAnalysisError(typeof err === "string" ? err : err instanceof Error ? err.message : "Analysis failed");
			setAnalyzing(false);
		},
	});

	const createNote = useMutation({
		mutationFn: () =>
			api("create_note", {
				input: {
					problem_id: id,
					note_type: noteForm.note_type,
					content: noteForm.content,
					source_url: noteForm.source_url || undefined,
				},
			}),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["notes", id] });
			setNoteDialogOpen(false);
			setNoteForm({ note_type: "self", content: "", source_url: "" });
		},
	});

	if (isLoading) {
		return (
			<div className="space-y-4">
				<Skeleton className="h-8 w-64" />
				<Skeleton className="h-4 w-32" />
				<Skeleton className="h-64 w-full" />
			</div>
		);
	}

	if (!problem) {
		return <p>Problem not found.</p>;
	}

	return (
		<div className="space-y-6">
			{/* Back + Title */}
			<div className="flex items-start justify-between">
				<div>
					<button
						onClick={() => navigate("/problems")}
						className="mb-2 flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
					>
						<ArrowLeft className="h-4 w-4" />
						Back to Problems
					</button>
					<h1 className="text-2xl font-bold">{problem.title}</h1>
					<p className="text-muted-foreground">
						{problem.source} #{problem.source_problem_id}
						{problem.difficulty && (
							<span className="ml-2">
								<Badge variant="warning">{problem.difficulty}</Badge>
							</span>
						)}
					</p>
				</div>
				<div className="flex gap-2">
					<Dialog open={subDialogOpen} onOpenChange={setSubDialogOpen}>
						<DialogTrigger asChild>
							<Button size="sm">
								<Plus className="mr-2 h-4 w-4" />
								Add Submission
							</Button>
						</DialogTrigger>
						<DialogContent className="sm:max-w-lg">
							<DialogHeader>
								<DialogTitle>Add Submission</DialogTitle>
							</DialogHeader>
							<div className="grid gap-4 py-4">
								<div className="grid grid-cols-2 gap-4">
									<div className="grid gap-2">
										<Label>Status</Label>
										<select
											className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
											value={subForm.status}
											onChange={(e) =>
												setSubForm({ ...subForm, status: e.target.value })
											}
										>
											<option value="AC">AC</option>
											<option value="WA">WA</option>
											<option value="TLE">TLE</option>
											<option value="RE">RE</option>
											<option value="MLE">MLE</option>
											<option value="CE">CE</option>
										</select>
									</div>
									<div className="grid gap-2">
										<Label>Language</Label>
										<select
											className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
											value={subForm.language}
											onChange={(e) =>
												setSubForm({ ...subForm, language: e.target.value })
											}
										>
											<option value="C++">C++</option>
											<option value="Python">Python</option>
											<option value="Java">Java</option>
											<option value="Rust">Rust</option>
										</select>
									</div>
								</div>
								<div className="grid gap-2">
									<Label>Code *</Label>
									<Textarea
										rows={8}
										className="font-mono text-sm"
										value={subForm.code_text}
										onChange={(e) =>
											setSubForm({ ...subForm, code_text: e.target.value })
										}
										placeholder="Paste your code here..."
									/>
								</div>
								<div className="grid grid-cols-2 gap-4">
									<div className="grid gap-2">
										<Label>Runtime (ms)</Label>
										<Input
											type="number"
											value={subForm.runtime}
											onChange={(e) =>
												setSubForm({ ...subForm, runtime: e.target.value })
											}
										/>
									</div>
									<div className="grid gap-2">
										<Label>Memory (KB)</Label>
										<Input
											type="number"
											value={subForm.memory}
											onChange={(e) =>
												setSubForm({ ...subForm, memory: e.target.value })
											}
										/>
									</div>
								</div>
								<div className="grid gap-2">
									<Label>Note</Label>
									<Input
										value={subForm.note}
										onChange={(e) =>
											setSubForm({ ...subForm, note: e.target.value })
										}
										placeholder="Optional note about this submission"
									/>
								</div>
							</div>
							<div className="flex justify-end gap-2">
								<Button
									variant="outline"
									onClick={() => setSubDialogOpen(false)}
								>
									Cancel
								</Button>
								<Button
									onClick={() => createSub.mutate()}
									disabled={!subForm.code_text || createSub.isPending}
								>
									Save
								</Button>
							</div>
						</DialogContent>
					</Dialog>

					<Dialog open={noteDialogOpen} onOpenChange={setNoteDialogOpen}>
						<DialogTrigger asChild>
							<Button variant="outline" size="sm">
								<FileText className="mr-2 h-4 w-4" />
								Add Note
							</Button>
						</DialogTrigger>
						<DialogContent className="sm:max-w-lg">
							<DialogHeader>
								<DialogTitle>Add Note</DialogTitle>
							</DialogHeader>
							<div className="grid gap-4 py-4">
								<div className="grid gap-2">
									<Label>Type</Label>
									<select
										className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
										value={noteForm.note_type}
										onChange={(e) =>
											setNoteForm({ ...noteForm, note_type: e.target.value })
										}
									>
										<option value="self">Self Review</option>
										<option value="official">Official Solution</option>
										<option value="community">Community Solution</option>
										<option value="ai">AI Analysis</option>
									</select>
								</div>
								<div className="grid gap-2">
									<Label>Content (Markdown)</Label>
									<Textarea
										rows={6}
										value={noteForm.content}
										onChange={(e) =>
											setNoteForm({ ...noteForm, content: e.target.value })
										}
										placeholder="Write your notes in Markdown..."
									/>
								</div>
								<div className="grid gap-2">
									<Label>Source URL</Label>
									<Input
										value={noteForm.source_url}
										onChange={(e) =>
											setNoteForm({ ...noteForm, source_url: e.target.value })
										}
										placeholder="https://..."
									/>
								</div>
							</div>
							<div className="flex justify-end gap-2">
								<Button
									variant="outline"
									onClick={() => setNoteDialogOpen(false)}
								>
									Cancel
								</Button>
								<Button
									onClick={() => createNote.mutate()}
									disabled={!noteForm.content || createNote.isPending}
								>
									Save
								</Button>
							</div>
						</DialogContent>
					</Dialog>
				</div>
			</div>

			{/* Tags */}
			<div className="flex flex-wrap gap-1">
				{problem.tags.map((tag) => (
					<Badge key={tag} variant="secondary">
						{tag}
					</Badge>
				))}
			</div>

			<Tabs defaultValue="submissions">
				<TabsList>
					<TabsTrigger value="submissions">
						<Code className="mr-2 h-4 w-4" />
						Submissions ({submissions?.length ?? 0})
					</TabsTrigger>
					<TabsTrigger value="notes">
						<FileText className="mr-2 h-4 w-4" />
						Notes ({notes?.length ?? 0})
					</TabsTrigger>
					<TabsTrigger value="errors">
						<Brain className="mr-2 h-4 w-4" />
						Error Analysis ({errors?.length ?? 0})
					</TabsTrigger>
				</TabsList>

				{/* Submissions tab */}
				<TabsContent value="submissions" className="space-y-4">
					{submissions && submissions.length > 0 ? (
						<Table>
							<TableHeader>
								<TableRow>
									<TableHead>Status</TableHead>
									<TableHead>Language</TableHead>
									<TableHead>Runtime</TableHead>
									<TableHead>Memory</TableHead>
									<TableHead>Note</TableHead>
									<TableHead>Time</TableHead>
									<TableHead className="w-16" />
								</TableRow>
							</TableHeader>
							<TableBody>
								{submissions.map((sub) => (
									<TableRow key={sub.id}>
										<TableCell>
											<Badge variant={statusColors[sub.status] ?? "secondary"}>
												{sub.status}
											</Badge>
										</TableCell>
										<TableCell>{sub.language}</TableCell>
										<TableCell>
											{sub.runtime ? `${sub.runtime}ms` : "—"}
										</TableCell>
										<TableCell>
											{sub.memory ? `${sub.memory}KB` : "—"}
										</TableCell>
										<TableCell className="max-w-48 truncate">
											{sub.note ?? "—"}
										</TableCell>
										<TableCell className="text-muted-foreground text-sm">
											{new Date(sub.submitted_at).toLocaleDateString()}
										</TableCell>
										<TableCell>
											<button
												onClick={() => {
													if (confirm("Delete this submission?")) {
														deleteSub.mutate(sub.id);
													}
												}}
												className="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-destructive/10 text-destructive"
											>
												<Trash2 className="h-4 w-4" />
											</button>
										</TableCell>
									</TableRow>
								))}
							</TableBody>
						</Table>
					) : (
						<p className="text-muted-foreground text-sm py-4">
							No submissions yet. Add your first submission above.
						</p>
					)}

					{/* WA → AC comparison -> AI Analysis */}
					{submissions && submissions.length >= 2 && (
						<Card className="border-dashed">
							<CardHeader>
								<CardTitle className="text-sm flex items-center gap-2">
									<Sparkles className="h-4 w-4" />
									AI Analysis
								</CardTitle>
							</CardHeader>
							<CardContent className="space-y-3">
								<p className="text-sm text-muted-foreground">
									You have both WA and AC submissions! Let AI analyze your
									mistakes and suggest improvements.
								</p>
								{analysisError && (
									<div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
										{analysisError}
									</div>
								)}
								<Button
									size="sm"
									onClick={() => analyzeMutation.mutate()}
									disabled={analyzing}
								>
									{analyzing ? (
										<>
											<Loader2 className="mr-2 h-4 w-4 animate-spin" />
											Analyzing...
										</>
									) : (
										<>
											<Sparkles className="mr-2 h-4 w-4" />
											Analyze Problem
										</>
									)}
								</Button>
							</CardContent>
						</Card>
					)}
				</TabsContent>

				{/* Notes tab */}
				<TabsContent value="notes" className="space-y-4">
					{notes && notes.length > 0 ? (
						notes.map((note) => (
							<Card key={note.id}>
								<CardHeader>
									<CardTitle className="text-sm flex items-center gap-2">
										<Badge variant="outline">{note.note_type}</Badge>
										<span className="text-muted-foreground font-normal">
											{new Date(note.created_at).toLocaleDateString()}
										</span>
									</CardTitle>
								</CardHeader>
								<CardContent>
									<div className="prose prose-sm max-w-none dark:prose-invert">
										{note.content}
									</div>
									{note.source_url && (
										<a
											href={note.source_url}
											target="_blank"
											rel="noopener noreferrer"
											className="text-sm text-primary hover:underline mt-2 inline-block"
										>
											Source →
										</a>
									)}
								</CardContent>
							</Card>
						))
					) : (
						<p className="text-muted-foreground text-sm py-4">
							No notes yet. Add solution notes or AI analysis above.
						</p>
					)}
				</TabsContent>

				{/* Errors tab */}
				<TabsContent value="errors" className="space-y-4">
					{errors && errors.length > 0 ? (
						errors.map((err) => (
							<Card key={err.id}>
								<CardHeader>
									<CardTitle className="text-sm">
										<Badge variant="error" className="mr-2">
											{err.error_type}
										</Badge>
										{new Date(err.created_at).toLocaleDateString()}
									</CardTitle>
								</CardHeader>
								<CardContent className="space-y-3 text-sm">
									<div>
										<strong>Root Cause:</strong>
										<p className="text-muted-foreground mt-1">
											{err.root_cause}
										</p>
									</div>
									<div>
										<strong>Fix:</strong>
										<p className="text-muted-foreground mt-1">
											{err.fix_summary}
										</p>
									</div>
									<div>
										<strong>Related Knowledge:</strong>
										<div className="flex flex-wrap gap-1 mt-1">
											{err.related_knowledge.map((k) => (
												<Badge key={k} variant="secondary" className="text-xs">
													{k}
												</Badge>
											))}
										</div>
									</div>
								</CardContent>
							</Card>
						))
					) : (
						<p className="text-muted-foreground text-sm py-4">
							No error analyses yet. Use the AI analysis feature to generate
							them.
						</p>
					)}
				</TabsContent>
			</Tabs>
		</div>
	);
}

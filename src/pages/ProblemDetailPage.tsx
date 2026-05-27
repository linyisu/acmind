import { Channel, invoke } from "@tauri-apps/api/core";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
	ArrowLeft,
	Plus,
	Trash2,
	Code,
	FileText,
	Sparkles,
	Loader2,
	Eye,
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
import ReactMarkdown from "react-markdown";
import rehypeKatex from "rehype-katex";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import type { Problem, Submission, SolutionNote } from "@/lib/types";

// API helpers
async function api<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
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

function markdownComponents() {
	return {
		pre: ({ children }: { children?: React.ReactNode }) => (
			<pre className="overflow-x-auto rounded-lg bg-muted p-3 text-sm">
				{children}
			</pre>
		),
		code: ({ children }: { children?: React.ReactNode }) => (
			<code className="rounded bg-muted px-1 py-0.5 font-mono text-sm">
				{children}
			</code>
		),
		a: ({ href, children }: { href?: string; children?: React.ReactNode }) => (
			<a
				href={href}
				target="_blank"
				rel="noopener noreferrer"
				className="text-primary hover:underline"
			>
				{children}
			</a>
		),
		img: ({ src, alt }: { src?: string; alt?: string }) => (
			<img
				src={src}
				alt={alt ?? ""}
				className="my-3 max-w-full rounded-md border bg-background"
				loading="lazy"
			/>
		),
	};
}

export function ProblemDetailPage() {
	const { id } = useParams<{ id: string }>();
	const navigate = useNavigate();
	const queryClient = useQueryClient();

	const { data: problem, isLoading } = useQuery({
		queryKey: ["problem", id],
		queryFn: () => api<Problem>("get_problem", { id }),
		enabled: !!id,
	});

	const { data: statement } = useQuery({
		queryKey: ["problem-statement", id, problem?.statement_path],
		queryFn: () => api<string | null>("get_problem_statement", { id }),
		enabled: !!id && !!problem,
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

	const currentNote = notes?.[0];

	// Code viewer state
	const [codeViewerOpen, setCodeViewerOpen] = useState(false);
	const [codeViewerSubId, setCodeViewerSubId] = useState<string | null>(null);

	const { data: viewedSubmission, isFetching: codeLoading } = useQuery({
		queryKey: ["submission-code", codeViewerSubId],
		queryFn: () =>
			api<{ code_text?: string; language: string; status: string }>(
				"get_submission",
				{ id: codeViewerSubId },
			),
		enabled: !!codeViewerSubId && codeViewerOpen,
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
	const [statementDialogOpen, setStatementDialogOpen] = useState(false);
	const [statementDraft, setStatementDraft] = useState("");

	const [analyzing, setAnalyzing] = useState(false);
	const [formatError, setFormatError] = useState("");
	const [analysisError, setAnalysisError] = useState("");
	const [analysisStream, setAnalysisStream] = useState("");

	const analyzeMutation = useMutation({
		mutationFn: async () => {
			const channel = new Channel<string>((chunk) => {
				setAnalysisStream((current) => current + chunk);
			});

			return api("analyze_problem_streaming", {
				problemId: id,
				channel,
			});
		},
		onMutate: () => {
			setAnalyzing(true);
			setAnalysisError("");
			setAnalysisStream("");
		},
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["errors", id] });
			queryClient.invalidateQueries({ queryKey: ["notes", id] });
			setAnalyzing(false);
		},
		onError: (err) => {
			// Tauri errors come as strings, not Error objects
			setAnalysisStream("");
			setAnalysisError(
				typeof err === "string"
					? err
					: err instanceof Error
						? err.message
						: "Analysis failed",
			);
			setAnalyzing(false);
		},
	});

	const saveNote = useMutation({
		mutationFn: () =>
			currentNote
				? api("update_note", {
						id: currentNote.id,
						content: noteForm.content,
					})
				: api("create_note", {
						input: {
							problem_id: id,
							note_type: "self",
							content: noteForm.content,
							source_url: undefined,
						},
					}),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["notes", id] });
			setNoteDialogOpen(false);
			setNoteForm({ note_type: "self", content: "", source_url: "" });
		},
	});

	const deleteNote = useMutation({
		mutationFn: (noteId: string) => api("delete_note", { id: noteId }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["notes", id] });
		},
	});

	const updateStatement = useMutation({
		mutationFn: () =>
			api("update_problem", {
				id,
				input: {
					statement: statementDraft,
				},
			}),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["problem", id] });
			queryClient.invalidateQueries({ queryKey: ["problem-statement", id] });
			setStatementDialogOpen(false);
		},
	});

	const formatStatement = useMutation({
		mutationFn: () =>
			api<string>("format_problem_statement", {
				rawText: statementDraft,
			}),
		onMutate: () => setFormatError(""),
		onSuccess: (formatted) => setStatementDraft(formatted),
		onError: (err) => {
			setFormatError(
				typeof err === "string"
					? err
					: err instanceof Error
						? err.message
						: "整理题面失败",
			);
		},
	});

	if (isLoading && !problem) {
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
							<Button
								variant="outline"
								size="sm"
								onClick={() =>
									setNoteForm({
										note_type: currentNote?.note_type ?? "self",
										content: currentNote?.content ?? "",
										source_url: currentNote?.source_url ?? "",
									})
								}
							>
								<FileText className="mr-2 h-4 w-4" />
								{currentNote ? "编辑笔记" : "添加笔记"}
							</Button>
						</DialogTrigger>
						<DialogContent className="sm:max-w-3xl">
							<DialogHeader>
								<DialogTitle>
									{currentNote ? "编辑笔记" : "添加笔记"}
								</DialogTitle>
							</DialogHeader>
							<div className="grid gap-4 py-4">
								<div className="grid gap-2">
									<Label>笔记内容（Markdown）</Label>
									<Textarea
										rows={14}
										className="font-mono text-sm"
										value={noteForm.content}
										onChange={(e) =>
											setNoteForm({ ...noteForm, content: e.target.value })
										}
										placeholder="记录这道题的思路、坑点、复盘。支持 Markdown 和 LaTeX。"
									/>
								</div>
							</div>
							<div className="flex justify-between gap-2">
								<div>
									{currentNote && (
										<Button
											variant="destructive"
											onClick={() => {
												if (confirm("确定删除这条笔记吗？")) {
													deleteNote.mutate(currentNote.id);
													setNoteDialogOpen(false);
												}
											}}
											disabled={deleteNote.isPending}
										>
											删除笔记
										</Button>
									)}
								</div>
								<div className="flex gap-2">
									<Button
										variant="outline"
										onClick={() => setNoteDialogOpen(false)}
									>
										取消
									</Button>
									<Button
										onClick={() => saveNote.mutate()}
										disabled={!noteForm.content.trim() || saveNote.isPending}
									>
										保存笔记
									</Button>
								</div>
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

			<Card>
				<CardHeader className="flex flex-row items-center justify-between gap-4">
					<CardTitle className="text-base">题面</CardTitle>
					<Button
						variant="outline"
						size="sm"
						onClick={() => {
							setStatementDraft(statement ?? "");
							setStatementDialogOpen(true);
						}}
					>
						<FileText className="mr-2 h-4 w-4" />
						{statement ? "编辑题面" : "导入题面"}
					</Button>
				</CardHeader>
				<CardContent>
					{statement ? (
						<div className="prose prose-sm max-w-none dark:prose-invert">
							<ReactMarkdown
								remarkPlugins={[remarkGfm, remarkMath]}
								rehypePlugins={[rehypeKatex]}
								components={markdownComponents()}
							>
								{statement}
							</ReactMarkdown>
						</div>
					) : (
						<p className="text-sm text-muted-foreground">
							还没有题面。可以从 Firefox 扩展复制 Markdown 后导入。
						</p>
					)}
				</CardContent>
			</Card>

			<Dialog open={statementDialogOpen} onOpenChange={setStatementDialogOpen}>
				<DialogContent className="sm:max-w-3xl">
					<DialogHeader>
						<DialogTitle>导入题面</DialogTitle>
					</DialogHeader>
					<div className="grid gap-3 py-4">
						<div className="flex items-center justify-between gap-3">
							<Label>Markdown 题面</Label>
							<Button
								variant="outline"
								size="sm"
								onClick={() => formatStatement.mutate()}
								disabled={!statementDraft.trim() || formatStatement.isPending}
							>
								{formatStatement.isPending ? (
									<>
										<Loader2 className="mr-2 h-4 w-4 animate-spin" />
										AI 整理中...
									</>
								) : (
									<>
										<Sparkles className="mr-2 h-4 w-4" />
										AI 整理
									</>
								)}
							</Button>
						</div>
						{formatError && (
							<p className="rounded-md bg-destructive/10 p-2 text-sm text-destructive">
								{formatError}
							</p>
						)}
						<Textarea
							rows={18}
							className="font-mono text-sm"
							value={statementDraft}
							onChange={(event) => setStatementDraft(event.target.value)}
							placeholder="把 ACMind Firefox 扩展复制的 Markdown 粘贴到这里..."
						/>
					</div>
					<div className="flex justify-end gap-2">
						<Button
							variant="outline"
							onClick={() => setStatementDialogOpen(false)}
						>
							取消
						</Button>
						<Button
							onClick={() => updateStatement.mutate()}
							disabled={!statementDraft.trim() || updateStatement.isPending}
						>
							保存题面
						</Button>
					</div>
				</DialogContent>
			</Dialog>

			<Tabs defaultValue="submissions">
				<TabsList>
					<TabsTrigger value="submissions">
						<Code className="mr-2 h-4 w-4" />
						提交记录 ({submissions?.length ?? 0})
					</TabsTrigger>
					<TabsTrigger value="notes">
						<FileText className="mr-2 h-4 w-4" />
						笔记 ({currentNote ? 1 : 0})
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
											<div className="flex items-center gap-1">
												<button
													onClick={() => {
														setCodeViewerSubId(sub.id);
														setCodeViewerOpen(true);
													}}
													className="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-muted"
													title="查看代码"
												>
													<Eye className="h-4 w-4" />
												</button>
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
											</div>
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
									<div className="rounded-md bg-destructive/10 p-3 text-sm text-destructive space-y-2">
										<p>{analysisError}</p>
										<p className="text-xs text-muted-foreground">
											Check ~/.local/share/acmind/acmind.log for details.
										</p>
									</div>
								)}
								{analysisStream && (
									<div className="max-h-64 overflow-auto rounded-md border bg-muted/40 p-3">
										<p className="mb-2 text-xs font-medium text-muted-foreground">
											AI response stream
										</p>
										<pre className="whitespace-pre-wrap break-words text-xs leading-relaxed text-muted-foreground">
											{analysisStream}
										</pre>
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
					{currentNote ? (
						<Card>
							<CardHeader className="flex flex-row items-center justify-between gap-4">
								<CardTitle className="text-sm flex items-center gap-2">
									<Badge variant="outline">
										{currentNote.note_type === "ai" ? "AI 分析" : "复盘笔记"}
									</Badge>
									<span className="text-muted-foreground font-normal">
										{new Date(currentNote.created_at).toLocaleDateString()}
									</span>
								</CardTitle>
								<div className="flex gap-2">
									<Button
										variant="outline"
										size="sm"
										onClick={() => {
											setNoteForm({
												note_type: currentNote.note_type,
												content: currentNote.content,
												source_url: currentNote.source_url ?? "",
											});
											setNoteDialogOpen(true);
										}}
									>
										编辑
									</Button>
									<Button
										variant="destructive"
										size="sm"
										onClick={() => {
											if (confirm("确定删除这条笔记吗？")) {
												deleteNote.mutate(currentNote.id);
											}
										}}
										disabled={deleteNote.isPending}
									>
										删除
									</Button>
								</div>
							</CardHeader>
							<CardContent>
								<div className="prose prose-sm max-w-none dark:prose-invert">
									<ReactMarkdown
										remarkPlugins={[remarkGfm, remarkMath]}
										rehypePlugins={[rehypeKatex]}
										components={markdownComponents()}
									>
										{currentNote.content}
									</ReactMarkdown>
								</div>
							</CardContent>
						</Card>
					) : (
						<p className="text-muted-foreground text-sm py-4">
							还没有笔记。每道题只保留一条复盘笔记，AI 分析也会覆盖写入这里。
						</p>
					)}
				</TabsContent>
			</Tabs>

			{/* Code viewer dialog */}
			<Dialog open={codeViewerOpen} onOpenChange={setCodeViewerOpen}>
				<DialogContent className="sm:max-w-3xl max-h-[80vh] flex flex-col">
					<DialogHeader>
						<DialogTitle>
							源码
							{viewedSubmission && (
								<span className="ml-2 text-sm font-normal text-muted-foreground">
									{viewedSubmission.language}
									<Badge
										variant={
											statusColors[viewedSubmission.status] ?? "secondary"
										}
										className="ml-2"
									>
										{viewedSubmission.status}
									</Badge>
								</span>
							)}
						</DialogTitle>
					</DialogHeader>
					<div className="flex-1 overflow-auto">
						{codeLoading ? (
							<div className="flex items-center justify-center py-12">
								<Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
							</div>
						) : viewedSubmission?.code_text ? (
							<pre className="overflow-x-auto rounded-lg bg-muted p-4 text-sm font-mono">
								<code>{viewedSubmission.code_text}</code>
							</pre>
						) : (
							<p className="text-sm text-muted-foreground py-8 text-center">
								暂无源码。请先在设置页填写 VJudge Cookie 后重新同步。
							</p>
						)}
					</div>
				</DialogContent>
			</Dialog>
		</div>
	);
}

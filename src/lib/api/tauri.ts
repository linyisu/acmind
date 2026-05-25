import { invoke } from "@tauri-apps/api/core";
import type {
	Problem,
	Submission,
	SolutionNote,
	ErrorAnalysis,
	KnowledgePoint,
	Report,
} from "../types";

// -- Problems --
export const problemApi = {
	list: () => invoke<Problem[]>("list_problems"),
	get: (id: string) => invoke<Problem>("get_problem", { id }),
	create: (input: CreateProblemInput) =>
		invoke<Problem>("create_problem", { input }),
	update: (id: string, input: Partial<CreateProblemInput>) =>
		invoke<Problem>("update_problem", { id, input }),
	delete: (id: string) => invoke<void>("delete_problem", { id }),
};

export interface CreateProblemInput {
	source: string;
	source_problem_id: string;
	title: string;
	url?: string;
	difficulty?: number;
	tags: string[];
	statement?: string;
}

// -- Submissions --
export const submissionApi = {
	listByProblem: (problemId: string) =>
		invoke<Submission[]>("list_submissions_by_problem", { problemId }),
	get: (id: string) => invoke<Submission>("get_submission", { id }),
	create: (input: CreateSubmissionInput) =>
		invoke<Submission>("create_submission", { input }),
	delete: (id: string) => invoke<void>("delete_submission", { id }),
};

export interface CreateSubmissionInput {
	problem_id: string;
	status: string;
	language: string;
	code_text: string;
	runtime?: number;
	memory?: number;
	note?: string;
}

// -- Notes --
export const noteApi = {
	listByProblem: (problemId: string) =>
		invoke<SolutionNote[]>("list_notes_by_problem", { problemId }),
	create: (input: CreateNoteInput) =>
		invoke<SolutionNote>("create_note", { input }),
	update: (id: string, content: string) =>
		invoke<SolutionNote>("update_note", { id, content }),
	delete: (id: string) => invoke<void>("delete_note", { id }),
};

export interface CreateNoteInput {
	problem_id: string;
	note_type: string;
	content: string;
	source_url?: string;
}

// -- Error Analysis --
export const errorApi = {
	listByProblem: (problemId: string) =>
		invoke<ErrorAnalysis[]>("list_error_analyses_by_problem", { problemId }),
	create: (input: CreateErrorInput) =>
		invoke<ErrorAnalysis>("create_error_analysis", { input }),
};

export interface CreateErrorInput {
	problem_id: string;
	submission_id: string;
	error_type: string;
	root_cause: string;
	fix_summary: string;
	related_knowledge: string[];
}

// -- Knowledge Points --
export const knowledgeApi = {
	listAll: () => invoke<KnowledgePoint[]>("list_knowledge_points"),
	create: (input: CreateKnowledgeInput) =>
		invoke<KnowledgePoint>("create_knowledge_point", { input }),
};

export interface CreateKnowledgeInput {
	name: string;
	category: string;
	parent_id?: string;
}

// -- Reports --
export const reportApi = {
	listAll: () => invoke<Report[]>("list_reports"),
	generate: (input: GenerateReportInput) =>
		invoke<Report>("generate_report", { input }),
};

export interface GenerateReportInput {
	report_type: string;
	title: string;
	start_date: string;
	end_date: string;
}

// -- AI Analysis --
export const aiApi = {
	analyzeProblem: (problemId: string) =>
		invoke<string>("analyze_problem", { problemId }),
	generateReport: (input: GenerateReportInput) =>
		invoke<string>("generate_ai_report", { input }),
};

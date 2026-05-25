export interface Problem {
	id: string;
	source: string;
	source_problem_id: string;
	title: string;
	url?: string;
	difficulty?: number;
	tags: string[];
	created_at: string;
}

export type SubmissionStatus = "AC" | "WA" | "TLE" | "RE" | "MLE" | "CE";

export interface Submission {
	id: string;
	problem_id: string;
	status: SubmissionStatus;
	language: string;
	code_path: string;
	code_text?: string;
	submitted_at: string;
	runtime?: number;
	memory?: number;
	note?: string;
}

export interface SolutionNote {
	id: string;
	problem_id: string;
	note_type: "official" | "community" | "self" | "ai";
	content: string;
	source_url?: string;
	created_at: string;
}

export type ErrorCategory =
	| "logic"
	| "boundary"
	| "overflow"
	| "index"
	| "initialization"
	| "complexity"
	| "template"
	| "misread"
	| "other";

export interface ErrorAnalysis {
	id: string;
	problem_id: string;
	submission_id: string;
	error_type: ErrorCategory;
	root_cause: string;
	fix_summary: string;
	related_knowledge: string[];
	created_at: string;
}

export interface KnowledgePoint {
	id: string;
	name: string;
	category: string;
	parent_id?: string;
}

export interface ProblemKnowledgeMap {
	problem_id: string;
	knowledge_point_id: string;
	confidence: number;
}

export interface Report {
	id: string;
	report_type: "weekly" | "monthly" | "custom";
	title: string;
	content: string;
	start_date: string;
	end_date: string;
	created_at: string;
}

import { api } from "./client";
import type {
  AnalysisSummary,
  CreateKnowledgeRequest,
  CreateProblemRequest,
  CreateSubmissionRequest,
  CreateTagRequest,
  CreateTemplateRequest,
  DifficultyBucket,
  Knowledge,
  Problem,
  Submission,
  Tag,
  Template,
  TemplateCategory,
  TemplateStats,
  TimelinePoint,
  UpdateKnowledgeRequest,
  UpdateProblemRequest,
  UpdateTemplateRequest,
} from "@acmind/shared";

export interface AnalysisResult {
  algorithm_type: string;
  sub_type: string;
  tags: string[];
  summary: string;
  template_snippet?: string | null;
  error_analysis?: string | null;
  suggested_difficulty?: number | null;
}

export interface AnalysisResp {
  id: number;
  target_type: string;
  target_id: number;
  result: AnalysisResult;
  created_at: string;
}

export interface ProblemAnalysisResp {
  analysis: AnalysisResp;
  extracted_templates: number;
  extracted_errors: number;
  extracted_knowledge: number;
  submissions_analyzed: number;
  knowledge_merged: number;
}

export const problemsApi = {
  list: (tagId?: number) => {
    const q = tagId ? `?tag_id=${tagId}` : "";
    return api.get<Problem[]>(`/api/v1/problems${q}`);
  },
  get: (id: number) => api.get<Problem>(`/api/v1/problems/${id}`),
  create: (req: CreateProblemRequest) => api.post<Problem>("/api/v1/problems", req),
  update: (id: number, req: UpdateProblemRequest) =>
    api.patch<Problem>(`/api/v1/problems/${id}`, req),
  delete: (id: number) => api.delete<void>(`/api/v1/problems/${id}`),
};

export const submissionsApi = {
  list: (problemId?: number) => {
    const q = problemId ? `?problem_id=${problemId}` : "";
    return api.get<Submission[]>(`/api/v1/submissions${q}`);
  },
  get: (id: number) => api.get<Submission>(`/api/v1/submissions/${id}`),
  create: (req: CreateSubmissionRequest) => api.post<Submission>("/api/v1/submissions", req),
  delete: (id: number) => api.delete<void>(`/api/v1/submissions/${id}`),
};

export const knowledgeApi = {
  list: () => api.get<Knowledge[]>("/api/v1/knowledge"),
  get: (id: number) => api.get<Knowledge>(`/api/v1/knowledge/${id}`),
  create: (req: CreateKnowledgeRequest) => api.post<Knowledge>("/api/v1/knowledge", req),
  update: (id: number, req: UpdateKnowledgeRequest) =>
    api.patch<Knowledge>(`/api/v1/knowledge/${id}`, req),
  delete: (id: number) => api.delete<void>(`/api/v1/knowledge/${id}`),
};

export const tagsApi = {
  list: () => api.get<Tag[]>("/api/v1/tags"),
  create: (req: CreateTagRequest) => api.post<Tag>("/api/v1/tags", req),
  delete: (id: number) => api.delete<void>(`/api/v1/tags/${id}`),
};

export const templatesApi = {
  list: (params?: {
    category?: TemplateCategory;
    language?: string;
    tag_id?: number;
    problem_id?: number;
    search?: string;
    sort?: string;
  }) => {
    const sp = new URLSearchParams();
    if (params?.category) sp.set("category", params.category);
    if (params?.language) sp.set("language", params.language);
    if (params?.tag_id) sp.set("tag_id", String(params.tag_id));
    if (params?.problem_id) sp.set("problem_id", String(params.problem_id));
    if (params?.search) sp.set("search", params.search);
    if (params?.sort) sp.set("sort", params.sort);
    const q = sp.toString() ? `?${sp.toString()}` : "";
    return api.get<Template[]>(`/api/v1/templates${q}`);
  },
  get: (id: number) => api.get<Template>(`/api/v1/templates/${id}`),
  create: (req: CreateTemplateRequest) =>
    api.post<Template>("/api/v1/templates", req),
  update: (id: number, req: UpdateTemplateRequest) =>
    api.patch<Template>(`/api/v1/templates/${id}`, req),
  delete: (id: number) => api.delete<void>(`/api/v1/templates/${id}`),
  linkProblem: (templateId: number, problemId: number) =>
    api.post<void>(`/api/v1/templates/${templateId}/problems/${problemId}`),
  unlinkProblem: (templateId: number, problemId: number) =>
    api.delete<void>(`/api/v1/templates/${templateId}/problems/${problemId}`),
  stats: () => api.get<TemplateStats>("/api/v1/templates/stats"),
};

export const aiApi = {
  analyze: (submissionId: number) =>
    api.post<AnalysisResp>(`/api/v1/ai/analyze/${submissionId}`),
  analyzeProblem: (problemId: number) =>
    api.post<Task>(`/api/v1/ai/analyze-problem/${problemId}`),
  list: () => api.get<AnalysisResp[]>("/api/v1/ai/analyses"),
  test: () => api.get<{ ok: boolean; message: string }>("/api/v1/ai/test"),
};

export const analysisApi = {
  summary: () => api.get<AnalysisSummary>("/api/v1/analysis/submissions/summary"),
  timeline: (from?: string, to?: string) => {
    const params = new URLSearchParams();
    if (from) params.set("from", from);
    if (to) params.set("to", to);
    const q = params.toString() ? `?${params.toString()}` : "";
    return api.get<TimelinePoint[]>(`/api/v1/analysis/submissions/timeline${q}`);
  },
  difficultyDistribution: () =>
    api.get<DifficultyBucket[]>("/api/v1/analysis/problems/difficulty-distribution"),
};

export interface AgentStep {
  label: string;
  status: string;
  detail: string;
}

export interface AgentProgress {
  id: string;
  name: string;
  icon: string;
  status: string;
  message: string;
  steps: AgentStep[];
}

export interface TaskProgress {
  agents: AgentProgress[];
}

export interface Task {
  id: number;
  kind: string;
  status: string;
  target_type: string;
  target_id: number;
  progress: TaskProgress;
  result: Record<string, unknown> | null;
  error: string | null;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
}

export const tasksApi = {
  list: () => api.get<Task[]>("/api/v1/tasks"),
  get: (id: number) => api.get<Task>(`/api/v1/tasks/${id}`),
  cancel: (id: number) => api.delete<boolean>(`/api/v1/tasks/${id}`),
};

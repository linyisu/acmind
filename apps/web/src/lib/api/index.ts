import { api } from "./client";
import type {
  AnalysisSummary,
  CreateKnowledgeRequest,
  CreateProblemRequest,
  CreateSubmissionRequest,
  CreateTagRequest,
  DifficultyBucket,
  Knowledge,
  Problem,
  Submission,
  Tag,
  TimelinePoint,
  UpdateKnowledgeRequest,
  UpdateProblemRequest,
} from "@acmind/shared";

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

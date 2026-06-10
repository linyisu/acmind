export type TemplateCategory =
  | "data_structure"
  | "graph"
  | "dp"
  | "string"
  | "math"
  | "geometry"
  | "greedy"
  | "search"
  | "sort"
  | "binary_search"
  | "other";

export const TEMPLATE_CATEGORIES: {
  value: TemplateCategory;
  label: string;
}[] = [
  { value: "data_structure", label: "数据结构" },
  { value: "graph", label: "图论" },
  { value: "dp", label: "动态规划" },
  { value: "string", label: "字符串" },
  { value: "math", label: "数学" },
  { value: "geometry", label: "计算几何" },
  { value: "greedy", label: "贪心" },
  { value: "search", label: "搜索" },
  { value: "sort", label: "排序" },
  { value: "binary_search", label: "二分" },
  { value: "other", label: "其他" },
];

export const TEMPLATE_LANGUAGES = [
  "cpp",
  "python",
  "java",
  "rust",
  "go",
  "other",
] as const;

export interface Template {
  id: number;
  user_id: number;
  title: string;
  category: TemplateCategory;
  language: string;
  code: string;
  description: string;
  summary: string;
  time_complexity: string | null;
  space_complexity: string | null;
  source: string;
  source_problem_id: number | null;
  difficulty: number | null;
  usage_count: number;
  tag_ids: number[];
  problem_ids: number[];
  created_at: string;
  updated_at: string;
}

export interface CreateTemplateRequest {
  title: string;
  category: TemplateCategory;
  language: string;
  code: string;
  description: string;
  summary?: string;
  time_complexity?: string;
  space_complexity?: string;
  difficulty?: number;
  tag_ids: number[];
  problem_ids: number[];
}

export interface UpdateTemplateRequest {
  title?: string;
  category?: TemplateCategory;
  language?: string;
  code?: string;
  description?: string;
  summary?: string;
  time_complexity?: string;
  space_complexity?: string;
  difficulty?: number;
  tag_ids?: number[];
}

export interface TemplateStats {
  total: number;
  by_category: { category: TemplateCategory; count: number }[];
  by_language: { language: string; count: number }[];
}

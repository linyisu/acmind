export interface Problem {
  id: number;
  user_id: number;
  source: string;
  external_id: string | null;
  title: string;
  url: string | null;
  difficulty: number | null;
  statement: string | null;
  tag_ids: number[];
  created_at: string;
  updated_at: string;
}

export interface CreateProblemRequest {
  source: string;
  external_id?: string;
  title: string;
  url?: string;
  difficulty?: number;
  statement?: string;
  tag_ids: number[];
}

export interface UpdateProblemRequest {
  source?: string;
  external_id?: string;
  title?: string;
  url?: string;
  difficulty?: number;
  statement?: string;
  tag_ids?: number[];
}

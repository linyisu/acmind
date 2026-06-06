export type KnowledgeKind = "template" | "technique" | "note" | "snippet";

export interface Knowledge {
  id: number;
  user_id: number;
  problem_id: number | null;
  kind: KnowledgeKind;
  title: string;
  content: string;
  tag_ids: number[];
  created_at: string;
  updated_at: string;
}

export interface CreateKnowledgeRequest {
  problem_id?: number;
  kind: KnowledgeKind;
  title: string;
  content: string;
  tag_ids: number[];
}

export interface UpdateKnowledgeRequest {
  problem_id?: number;
  kind?: KnowledgeKind;
  title?: string;
  content?: string;
  tag_ids?: number[];
}

export interface AnalysisSummary {
  total: number;
  by_verdict: Record<string, number>;
  ac_rate: number;
}

export interface TimelinePoint {
  date: string;
  count: number;
  ac_count: number;
}

export interface DifficultyBucket {
  difficulty: number;
  count: number;
  ac_count: number;
}

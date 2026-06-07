use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde::Serialize;
use std::collections::HashMap;

pub struct AnalysisService<'a> {
    pub state: &'a AppState,
}

#[derive(Serialize, Debug)]
pub struct SummaryResp {
    pub total: i64,
    pub by_verdict: HashMap<String, i64>,
    pub ac_rate: f64,
}

#[derive(Serialize, Debug)]
pub struct TimelinePoint {
    pub date: String,
    pub count: i64,
    pub ac_count: i64,
}

#[derive(Serialize, Debug)]
pub struct DifficultyBucket {
    pub difficulty: i32,
    pub count: i64,
    pub ac_count: i64,
}

impl<'a> AnalysisService<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Aggregates submission counts by verdict for the given user.
    pub async fn submissions_summary(&self, user_id: i64) -> AppResult<SummaryResp> {
        let stmt = Statement::from_string(
            DbBackend::Postgres,
            format!(
                "SELECT verdict, COUNT(*) AS cnt FROM submission \
                 WHERE user_id = {user_id} GROUP BY verdict"
            ),
        );
        let rows = self.state.db.query_all(stmt).await?;

        let mut by_verdict: HashMap<String, i64> = HashMap::new();
        let mut total: i64 = 0;
        for r in rows {
            let verdict: String = r
                .try_get_by("verdict")
                .map_err(|e| AppError::Internal(format!("verdict column: {e}")))?;
            let cnt: i64 = r
                .try_get_by("cnt")
                .map_err(|e| AppError::Internal(format!("cnt column: {e}")))?;
            total += cnt;
            by_verdict.insert(verdict, cnt);
        }

        let ac = *by_verdict.get("AC").unwrap_or(&0);
        let ac_rate = if total == 0 {
            0.0
        } else {
            ac as f64 / total as f64
        };

        Ok(SummaryResp {
            total,
            by_verdict,
            ac_rate,
        })
    }

    /// Daily submission counts (total + AC) for the given user, optionally filtered by date range.
    pub async fn submissions_timeline(
        &self,
        user_id: i64,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> AppResult<Vec<TimelinePoint>> {
        let mut filters = format!("WHERE user_id = {user_id}");
        if let Some(f) = from {
            filters.push_str(&format!(" AND submitted_at >= '{}'", f.to_rfc3339()));
        }
        if let Some(t) = to {
            filters.push_str(&format!(" AND submitted_at <= '{}'", t.to_rfc3339()));
        }

        let stmt = Statement::from_string(
            DbBackend::Postgres,
            format!(
                "SELECT DATE(submitted_at) AS date, \
                        COUNT(*) AS cnt, \
                        SUM(CASE WHEN verdict = 'AC' THEN 1 ELSE 0 END) AS ac_cnt \
                 FROM submission {filters} \
                 GROUP BY DATE(submitted_at) ORDER BY date"
            ),
        );
        let rows = self.state.db.query_all(stmt).await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let date: chrono::NaiveDate = r
                .try_get_by("date")
                .map_err(|e| AppError::Internal(format!("date column: {e}")))?;
            let cnt: i64 = r
                .try_get_by("cnt")
                .map_err(|e| AppError::Internal(format!("cnt column: {e}")))?;
            let ac_cnt: i64 = r
                .try_get_by("ac_cnt")
                .map_err(|e| AppError::Internal(format!("ac_cnt column: {e}")))?;
            out.push(TimelinePoint {
                date: date.to_string(),
                count: cnt,
                ac_count: ac_cnt,
            });
        }
        Ok(out)
    }

    /// Submission counts bucketed by problem difficulty for the given user.
    pub async fn difficulty_distribution(&self, user_id: i64) -> AppResult<Vec<DifficultyBucket>> {
        let stmt = Statement::from_string(
            DbBackend::Postgres,
            format!(
                "SELECT p.difficulty, COUNT(s.*) AS cnt, \
                        SUM(CASE WHEN s.verdict = 'AC' THEN 1 ELSE 0 END) AS ac_cnt \
                 FROM submission s \
                 JOIN problem p ON p.id = s.problem_id \
                 WHERE s.user_id = {user_id} AND p.difficulty IS NOT NULL \
                 GROUP BY p.difficulty \
                 ORDER BY p.difficulty"
            ),
        );
        let rows = self.state.db.query_all(stmt).await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let diff: Option<i32> = r.try_get_by("difficulty").ok().flatten();
            let cnt: Option<i64> = r.try_get_by("cnt").ok().flatten();
            let ac_cnt: Option<i64> = r.try_get_by("ac_cnt").ok().flatten();
            if let Some(d) = diff {
                out.push(DifficultyBucket {
                    difficulty: d,
                    count: cnt.unwrap_or(0),
                    ac_count: ac_cnt.unwrap_or(0),
                });
            }
        }
        Ok(out)
    }
}

use crate::{
    analysis::datafusion_ctx::{make_session_with_submissions, SubmissionRow},
    error::{AppError, AppResult},
    state::AppState,
    submission::repo,
};
use chrono::{DateTime, Utc};
use datafusion::arrow::record_batch::RecordBatch;
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

    pub async fn submissions_summary(&self, user_id: i64) -> AppResult<SummaryResp> {
        let rows = fetch_user_submissions(&self.state.db, user_id).await?;
        let total = rows.len() as i64;
        let mut by_verdict: HashMap<String, i64> = HashMap::new();
        for r in &rows {
            *by_verdict.entry(r.verdict.clone()).or_insert(0) += 1;
        }
        let ac = *by_verdict.get("AC").unwrap_or(&0);
        let ac_rate = if total == 0 { 0.0 } else { ac as f64 / total as f64 };
        Ok(SummaryResp {
            total,
            by_verdict,
            ac_rate,
        })
    }

    pub async fn submissions_timeline(
        &self,
        user_id: i64,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> AppResult<Vec<TimelinePoint>> {
        let mut rows = fetch_user_submissions(&self.state.db, user_id).await?;
        if let Some(f) = from {
            let f = f.to_rfc3339();
            rows.retain(|r| r.submitted_at >= f);
        }
        if let Some(t) = to {
            let t = t.to_rfc3339();
            rows.retain(|r| r.submitted_at <= t);
        }
        let ctx = make_session_with_submissions(rows)
            .await
            .map_err(|e| AppError::Internal(format!("datafusion ctx: {e}")))?;
        let df = ctx
            .sql(
                "SELECT substr(submitted_at, 1, 10) AS date, \
                        COUNT(*) AS count, \
                        SUM(CASE WHEN verdict = 'AC' THEN 1 ELSE 0 END) AS ac_count \
                 FROM submissions GROUP BY date ORDER BY date",
            )
            .await
            .map_err(|e| AppError::Internal(format!("datafusion sql: {e}")))?;
        let batches = df
            .collect()
            .await
            .map_err(|e| AppError::Internal(format!("datafusion collect: {e}")))?;
        Ok(records_to_timeline(&batches))
    }

    pub async fn difficulty_distribution(&self, user_id: i64) -> AppResult<Vec<DifficultyBucket>> {
        // Bucket by problem.difficulty, with submission count and AC count per bucket.
        let stmt = Statement::from_string(
            DbBackend::Postgres,
            format!(
                r#"SELECT p.difficulty, COUNT(s.*) AS count,
                          SUM(CASE WHEN s.verdict = 'AC' THEN 1 ELSE 0 END) AS ac_count
                   FROM submission s
                   JOIN problem p ON p.id = s.problem_id
                   WHERE s.user_id = {} AND p.difficulty IS NOT NULL
                   GROUP BY p.difficulty
                   ORDER BY p.difficulty"#,
                user_id
            ),
        );
        let rows = self.state.db.query_all(stmt).await?;
        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            let diff: Option<i32> = r.try_get_by::<Option<i32>, _>("difficulty").ok().flatten();
            let count: Option<i64> = r.try_get_by::<Option<i64>, _>("count").ok().flatten();
            let ac: Option<i64> = r.try_get_by::<Option<i64>, _>("ac_count").ok().flatten();
            if let Some(d) = diff {
                out.push(DifficultyBucket {
                    difficulty: d,
                    count: count.unwrap_or(0),
                    ac_count: ac.unwrap_or(0),
                });
            }
        }
        Ok(out)
    }
}

async fn fetch_user_submissions(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
) -> AppResult<Vec<SubmissionRow>> {
    let submission_rows = repo::list_by_user(db, user_id, None).await?;
    Ok(submission_rows
        .into_iter()
        .map(|r| SubmissionRow {
            id: r.id,
            user_id: r.user_id,
            problem_id: r.problem_id,
            language: r.language,
            verdict: r.verdict,
            runtime_ms: r.runtime_ms,
            memory_kb: r.memory_kb,
            submitted_at: r.submitted_at.to_rfc3339(),
        })
        .collect())
}

fn records_to_timeline(batches: &[RecordBatch]) -> Vec<TimelinePoint> {
    let mut out = Vec::new();
    for batch in batches {
        let date_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<datafusion::arrow::array::StringArray>()
            .expect("date column is Utf8");
        let count_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<datafusion::arrow::array::Int64Array>()
            .expect("count column is Int64");
        let ac_col = batch
            .column(2)
            .as_any()
            .downcast_ref::<datafusion::arrow::array::Int64Array>()
            .expect("ac_count column is Int64");
        for i in 0..batch.num_rows() {
            out.push(TimelinePoint {
                date: date_col.value(i).to_string(),
                count: count_col.value(i),
                ac_count: ac_col.value(i),
            });
        }
    }
    out
}

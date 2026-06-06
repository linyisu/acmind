use datafusion::arrow::{
    array::{Int64Array, RecordBatch, StringArray},
    datatypes::{DataType, Field, Schema, SchemaRef},
};
use datafusion::error::Result as DfResult;
use datafusion::prelude::*;
use std::sync::Arc;

pub fn submissions_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("user_id", DataType::Int64, false),
        Field::new("problem_id", DataType::Int64, false),
        Field::new("language", DataType::Utf8, false),
        Field::new("verdict", DataType::Utf8, false),
        Field::new("runtime_ms", DataType::Int64, true),
        Field::new("memory_kb", DataType::Int64, true),
        Field::new("submitted_at", DataType::Utf8, false),
    ]))
}

#[derive(Debug, Clone)]
pub struct SubmissionRow {
    pub id: i64,
    pub user_id: i64,
    pub problem_id: i64,
    pub language: String,
    pub verdict: String,
    pub runtime_ms: Option<i32>,
    pub memory_kb: Option<i32>,
    pub submitted_at: String,
}

pub fn build_record_batch(rows: &[SubmissionRow]) -> DfResult<RecordBatch> {
    let ids: Int64Array = rows.iter().map(|r| Some(r.id)).collect();
    let user_ids: Int64Array = rows.iter().map(|r| Some(r.user_id)).collect();
    let problem_ids: Int64Array = rows.iter().map(|r| Some(r.problem_id)).collect();
    let languages: StringArray = rows.iter().map(|r| Some(r.language.as_str())).collect();
    let verdicts: StringArray = rows.iter().map(|r| Some(r.verdict.as_str())).collect();
    let runtimes: Int64Array = rows.iter().map(|r| r.runtime_ms.map(|v| v as i64)).collect();
    let memories: Int64Array = rows.iter().map(|r| r.memory_kb.map(|v| v as i64)).collect();
    let submitted_ats: StringArray = rows.iter().map(|r| Some(r.submitted_at.as_str())).collect();

    Ok(RecordBatch::try_new(
        submissions_schema(),
        vec![
            Arc::new(ids),
            Arc::new(user_ids),
            Arc::new(problem_ids),
            Arc::new(languages),
            Arc::new(verdicts),
            Arc::new(runtimes),
            Arc::new(memories),
            Arc::new(submitted_ats),
        ],
    )?)
}

pub async fn make_session_with_submissions(rows: Vec<SubmissionRow>) -> DfResult<SessionContext> {
    let ctx = SessionContext::new();
    let batch = build_record_batch(&rows)?;
    ctx.register_batch("submissions", batch)?;
    Ok(ctx)
}

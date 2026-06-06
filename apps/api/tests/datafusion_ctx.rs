use acmind_api::analysis::datafusion_ctx::{
    make_session_with_submissions, SubmissionRow,
};

fn row(id: i64, problem: i64, language: &str, verdict: &str) -> SubmissionRow {
    SubmissionRow {
        id,
        user_id: 1,
        problem_id: problem,
        language: language.into(),
        verdict: verdict.into(),
        runtime_ms: Some(10),
        memory_kb: Some(1024),
        submitted_at: "2025-01-01T00:00:00+00:00".into(),
    }
}

#[tokio::test]
async fn count_submissions_by_verdict() {
    let rows = vec![
        row(1, 1, "rust", "AC"),
        row(2, 2, "cpp", "WA"),
        row(3, 3, "rust", "AC"),
        row(4, 4, "py", "AC"),
    ];
    let ctx = make_session_with_submissions(rows).await.unwrap();
    let df = ctx
        .sql("SELECT verdict, COUNT(*) AS c FROM submissions GROUP BY verdict ORDER BY c DESC")
        .await
        .unwrap();
    let batches = df.collect().await.unwrap();
    assert_eq!(batches.len(), 1);
    let batch = &batches[0];
    assert_eq!(batch.num_rows(), 2); // AC and WA

    let verdicts = batch
        .column(0)
        .as_any()
        .downcast_ref::<datafusion::arrow::array::StringArray>()
        .unwrap();
    let counts = batch
        .column(1)
        .as_any()
        .downcast_ref::<datafusion::arrow::array::Int64Array>()
        .unwrap();
    // First row should be the higher count: AC=3
    assert_eq!(verdicts.value(0), "AC");
    assert_eq!(counts.value(0), 3);
    assert_eq!(verdicts.value(1), "WA");
    assert_eq!(counts.value(1), 1);
}

#[tokio::test]
async fn empty_table_returns_empty_result() {
    let rows: Vec<SubmissionRow> = vec![];
    let ctx = make_session_with_submissions(rows).await.unwrap();
    let df = ctx
        .sql("SELECT COUNT(*) AS c FROM submissions")
        .await
        .unwrap();
    let batches = df.collect().await.unwrap();
    let total: i64 = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<datafusion::arrow::array::Int64Array>()
        .unwrap()
        .value(0);
    assert_eq!(total, 0);
}

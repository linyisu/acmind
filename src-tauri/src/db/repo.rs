use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use super::models::*;

// -- Problems --

pub async fn list_problems(pool: &SqlitePool) -> Result<Vec<Problem>, sqlx::Error> {
    sqlx::query_as::<_, Problem>("SELECT * FROM problems ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_problem(pool: &SqlitePool, id: &str) -> Result<Problem, sqlx::Error> {
    sqlx::query_as::<_, Problem>("SELECT * FROM problems WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn create_problem(
    pool: &SqlitePool,
    input: &CreateProblemInput,
    statement_path: Option<&str>,
) -> Result<Problem, sqlx::Error> {
    let id = new_id();
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_default();

    sqlx::query(
        "INSERT INTO problems (id, source, source_problem_id, title, url, difficulty, tags, statement_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind(&id)
    .bind(&input.source)
    .bind(&input.source_problem_id)
    .bind(&input.title)
    .bind(&input.url)
    .bind(input.difficulty)
    .bind(&tags_json)
    .bind(statement_path)
    .execute(pool)
    .await?;

    get_problem(pool, &id).await
}

pub async fn update_problem(
    pool: &SqlitePool,
    id: &str,
    input: &UpdateProblemInput,
    statement_path: Option<&str>,
) -> Result<Problem, sqlx::Error> {
    let existing = get_problem(pool, id).await?;
    let tags_json = input
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default());

    sqlx::query(
        "UPDATE problems SET source = ?1, source_problem_id = ?2, title = ?3, url = ?4, difficulty = ?5, tags = COALESCE(?6, tags), statement_path = COALESCE(?7, statement_path), updated_at = datetime('now') WHERE id = ?8",
    )
    .bind(input.source.as_deref().unwrap_or(&existing.source))
    .bind(
        input
            .source_problem_id
            .as_deref()
            .unwrap_or(&existing.source_problem_id),
    )
    .bind(input.title.as_deref().unwrap_or(&existing.title))
    .bind(input.url.as_ref().or(existing.url.as_ref()).map(|s| s.as_str()))
    .bind(input.difficulty.or(existing.difficulty))
    .bind(tags_json.as_deref())
    .bind(statement_path.or(existing.statement_path.as_deref()))
    .bind(id)
    .execute(pool)
    .await?;

    get_problem(pool, id).await
}

pub async fn delete_problem(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM problems WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// -- Submissions --

pub async fn list_submissions_by_problem(
    pool: &SqlitePool,
    problem_id: &str,
) -> Result<Vec<Submission>, sqlx::Error> {
    sqlx::query_as::<_, Submission>(
        "SELECT * FROM submissions WHERE problem_id = ?1 ORDER BY submitted_at DESC",
    )
    .bind(problem_id)
    .fetch_all(pool)
    .await
}

pub async fn get_submission(pool: &SqlitePool, id: &str) -> Result<Submission, sqlx::Error> {
    sqlx::query_as::<_, Submission>("SELECT * FROM submissions WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn create_submission(
    pool: &SqlitePool,
    input: &CreateSubmissionInput,
    code_path: &str,
) -> Result<Submission, sqlx::Error> {
    let id = new_id();

    sqlx::query(
        "INSERT INTO submissions (id, problem_id, status, language, code_path, runtime, memory, note, external_run_id, submitted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, COALESCE(?10, datetime('now')))",
    )
    .bind(&id)
    .bind(&input.problem_id)
    .bind(&input.status)
    .bind(&input.language)
    .bind(code_path)
    .bind(input.runtime)
    .bind(input.memory)
    .bind(&input.note)
    .bind(&input.external_run_id)
    .bind(input.submitted_at)
    .execute(pool)
    .await?;

    get_submission(pool, &id).await
}

pub async fn delete_submission(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM submissions WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_submission_code_path(
    pool: &SqlitePool,
    id: &str,
    code_path: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE submissions SET code_path = ?1 WHERE id = ?2")
        .bind(code_path)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_problem_by_source_id(
    pool: &SqlitePool,
    source: &str,
    source_problem_id: &str,
) -> Result<Option<Problem>, sqlx::Error> {
    sqlx::query_as::<_, Problem>(
        "SELECT * FROM problems WHERE source = ?1 AND source_problem_id = ?2 LIMIT 1",
    )
    .bind(source)
    .bind(source_problem_id)
    .fetch_optional(pool)
    .await
}

pub async fn submission_by_external_run_id(
    pool: &SqlitePool,
    external_run_id: &str,
) -> Result<Option<Submission>, sqlx::Error> {
    sqlx::query_as::<_, Submission>("SELECT * FROM submissions WHERE external_run_id = ?1 LIMIT 1")
        .bind(external_run_id)
        .fetch_optional(pool)
        .await
}

// -- Solution Notes --

pub async fn list_notes_by_problem(
    pool: &SqlitePool,
    problem_id: &str,
) -> Result<Vec<SolutionNote>, sqlx::Error> {
    sqlx::query_as::<_, SolutionNote>(
        "SELECT * FROM solution_notes WHERE problem_id = ?1 ORDER BY created_at DESC",
    )
    .bind(problem_id)
    .fetch_all(pool)
    .await
}

pub async fn create_note(
    pool: &SqlitePool,
    input: &CreateNoteInput,
) -> Result<SolutionNote, sqlx::Error> {
    let id = new_id();

    sqlx::query(
        "INSERT INTO solution_notes (id, problem_id, note_type, content, source_url) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&id)
    .bind(&input.problem_id)
    .bind(&input.note_type)
    .bind(&input.content)
    .bind(&input.source_url)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, SolutionNote>("SELECT * FROM solution_notes WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn update_note(
    pool: &SqlitePool,
    id: &str,
    content: &str,
) -> Result<SolutionNote, sqlx::Error> {
    sqlx::query("UPDATE solution_notes SET content = ?1 WHERE id = ?2")
        .bind(content)
        .bind(id)
        .execute(pool)
        .await?;

    sqlx::query_as::<_, SolutionNote>("SELECT * FROM solution_notes WHERE id = ?1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn delete_note(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM solution_notes WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_notes_by_problem_except(
    pool: &SqlitePool,
    problem_id: &str,
    keep_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM solution_notes WHERE problem_id = ?1 AND id != ?2")
        .bind(problem_id)
        .bind(keep_id)
        .execute(pool)
        .await?;
    Ok(())
}

// -- Error Analyses --

pub async fn list_error_analyses_by_problem(
    pool: &SqlitePool,
    problem_id: &str,
) -> Result<Vec<ErrorAnalysis>, sqlx::Error> {
    sqlx::query_as::<_, ErrorAnalysis>(
        "SELECT * FROM error_analyses WHERE problem_id = ?1 ORDER BY created_at DESC",
    )
    .bind(problem_id)
    .fetch_all(pool)
    .await
}

pub async fn create_error_analysis(
    pool: &SqlitePool,
    input: &CreateErrorInput,
) -> Result<ErrorAnalysis, sqlx::Error> {
    let id = new_id();
    let related = serde_json::to_string(&input.related_knowledge).unwrap_or_default();

    sqlx::query(
        "INSERT INTO error_analyses (id, problem_id, submission_id, error_type, root_cause, fix_summary, related_knowledge) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&id)
    .bind(&input.problem_id)
    .bind(&input.submission_id)
    .bind(&input.error_type)
    .bind(&input.root_cause)
    .bind(&input.fix_summary)
    .bind(&related)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, ErrorAnalysis>("SELECT * FROM error_analyses WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await
}

pub async fn delete_error_analyses_by_problem(
    pool: &SqlitePool,
    problem_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM error_analyses WHERE problem_id = ?1")
        .bind(problem_id)
        .execute(pool)
        .await?;
    Ok(())
}

// -- Knowledge Points --

pub async fn list_knowledge_points(pool: &SqlitePool) -> Result<Vec<KnowledgePoint>, sqlx::Error> {
    sqlx::query_as::<_, KnowledgePoint>("SELECT * FROM knowledge_points ORDER BY category, name")
        .fetch_all(pool)
        .await
}

pub async fn create_knowledge_point(
    pool: &SqlitePool,
    input: &CreateKnowledgeInput,
) -> Result<KnowledgePoint, sqlx::Error> {
    let id = new_id();

    sqlx::query(
        "INSERT INTO knowledge_points (id, name, category, parent_id) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.category)
    .bind(&input.parent_id)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, KnowledgePoint>("SELECT * FROM knowledge_points WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await
}

// -- Reports --

pub async fn list_reports(pool: &SqlitePool) -> Result<Vec<Report>, sqlx::Error> {
    sqlx::query_as::<_, Report>("SELECT * FROM reports ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn save_report(
    pool: &SqlitePool,
    input: &GenerateReportInput,
    content: &str,
) -> Result<Report, sqlx::Error> {
    let id = new_id();

    sqlx::query(
        "INSERT INTO reports (id, report_type, title, content, start_date, end_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(&id)
    .bind(&input.report_type)
    .bind(&input.title)
    .bind(content)
    .bind(&input.start_date)
    .bind(&input.end_date)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, Report>("SELECT * FROM reports WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await
}

// -- Stats queries --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_problems: i64,
    pub total_submissions: i64,
    pub ac_count: i64,
    pub wa_count: i64,
    pub tle_count: i64,
    pub re_count: i64,
    pub other_count: i64,
}

pub async fn get_dashboard_stats(pool: &SqlitePool) -> Result<DashboardStats, sqlx::Error> {
    let total_problems: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM problems")
        .fetch_one(pool)
        .await?;

    let status_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) as cnt FROM submissions GROUP BY status")
            .fetch_all(pool)
            .await?;

    let total_submissions: i64 = status_counts.iter().map(|(_, c)| c).sum();
    let ac_count = status_counts
        .iter()
        .find(|(s, _)| s == "AC")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let wa_count = status_counts
        .iter()
        .find(|(s, _)| s == "WA")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let tle_count = status_counts
        .iter()
        .find(|(s, _)| s == "TLE")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let re_count = status_counts
        .iter()
        .find(|(s, _)| s == "RE")
        .map(|(_, c)| *c)
        .unwrap_or(0);
    let other_count = total_submissions - ac_count - wa_count - tle_count - re_count;

    Ok(DashboardStats {
        total_problems: total_problems.0,
        total_submissions,
        ac_count,
        wa_count,
        tle_count,
        re_count,
        other_count,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ErrorTypeStat {
    pub error_type: String,
    pub count: i64,
}

pub async fn get_error_type_stats(pool: &SqlitePool) -> Result<Vec<ErrorTypeStat>, sqlx::Error> {
    sqlx::query_as::<_, ErrorTypeStat>(
        "SELECT error_type, COUNT(*) as count FROM error_analyses GROUP BY error_type ORDER BY count DESC",
    )
    .fetch_all(pool)
    .await
}

// -- Settings --

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_setting(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(result.map(|r| r.0))
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now')) ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_all_settings(pool: &SqlitePool) -> Result<Vec<AppSetting>, sqlx::Error> {
    sqlx::query_as::<_, AppSetting>("SELECT key, value, updated_at FROM settings ORDER BY key")
        .fetch_all(pool)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect("sqlite::memory:")
            .await
            .expect("failed to create test pool");

        // Run schema
        sqlx::query(include_str!("../../migrations/001_init.sql"))
            .execute(&pool)
            .await
            .expect("failed to init test schema");

        sqlx::query(include_str!("../../migrations/002_settings.sql"))
            .execute(&pool)
            .await
            .expect("failed to init settings schema");

        sqlx::query("ALTER TABLE submissions ADD COLUMN external_run_id TEXT")
            .execute(&pool)
            .await
            .expect("failed to init external submissions column");

        sqlx::query(include_str!(
            "../../migrations/003_external_submissions.sql"
        ))
        .execute(&pool)
        .await
        .expect("failed to init external submissions schema");

        pool
    }

    async fn create_test_problem(pool: &SqlitePool, title: &str, tags: Vec<&str>) -> Problem {
        let input = CreateProblemInput {
            source: "TestOJ".into(),
            source_problem_id: "T001".into(),
            title: title.into(),
            url: None,
            difficulty: Some(1500),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            statement: None,
        };
        create_problem(pool, &input, None).await.unwrap()
    }

    // -- Problem CRUD --

    #[tokio::test]
    async fn test_create_and_get_problem() {
        let pool = test_pool().await;
        let problem = create_test_problem(&pool, "Two Sum", vec!["array", "hash"]).await;

        assert_eq!(problem.title, "Two Sum");
        assert_eq!(problem.source, "TestOJ");
        assert_eq!(problem.difficulty, Some(1500));
        assert_eq!(
            problem.tags,
            serde_json::to_string(&["array", "hash"]).unwrap()
        );
        assert!(!problem.id.is_empty());
    }

    #[tokio::test]
    async fn test_list_problems_empty() {
        let pool = test_pool().await;
        let problems = list_problems(&pool).await.unwrap();
        assert!(problems.is_empty());
    }

    #[tokio::test]
    async fn test_list_problems_multiple() {
        let pool = test_pool().await;
        create_test_problem(&pool, "Problem A", vec!["dp"]).await;
        create_test_problem(&pool, "Problem B", vec!["graph"]).await;

        let problems = list_problems(&pool).await.unwrap();
        assert_eq!(problems.len(), 2);
    }

    #[tokio::test]
    async fn test_get_problem_not_found() {
        let pool = test_pool().await;
        let result = get_problem(&pool, "does-not-exist").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_problem() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "Old Title", vec!["old"]).await;

        let input = UpdateProblemInput {
            source: None,
            source_problem_id: None,
            title: Some("New Title".into()),
            url: None,
            difficulty: Some(2000),
            tags: Some(vec!["greedy".into()]),
            statement: None,
        };

        let updated = update_problem(&pool, &p.id, &input, None).await.unwrap();
        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.difficulty, Some(2000));
        assert_eq!(updated.tags, serde_json::to_string(&["greedy"]).unwrap());
    }

    #[tokio::test]
    async fn test_delete_problem() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "To Delete", vec![]).await;

        delete_problem(&pool, &p.id).await.unwrap();
        let result = get_problem(&pool, &p.id).await;
        assert!(result.is_err());
    }

    // -- Submission CRUD --

    #[tokio::test]
    async fn test_create_and_get_submission() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "Test", vec![]).await;

        let input = CreateSubmissionInput {
            problem_id: p.id.clone(),
            status: "WA".into(),
            language: "Rust".into(),
            code_text: "fn main() {}".into(),
            runtime: Some(200),
            memory: Some(32000),
            note: Some("forgot modulo".into()),
            external_run_id: None,
            submitted_at: None,
        };

        let sub = create_submission(&pool, &input, "/tmp/test.rs")
            .await
            .unwrap();
        assert_eq!(sub.status, "WA");
        assert_eq!(sub.language, "Rust");
        assert_eq!(sub.code_path, "/tmp/test.rs");
        assert_eq!(sub.runtime, Some(200));
    }

    #[tokio::test]
    async fn test_list_submissions_by_problem() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "MultiSub", vec![]).await;

        for (i, status) in ["WA", "AC", "TLE"].iter().enumerate() {
            let input = CreateSubmissionInput {
                problem_id: p.id.clone(),
                status: status.to_string(),
                language: "C++".into(),
                code_text: format!("code{}", i),
                runtime: None,
                memory: None,
                note: None,
                external_run_id: None,
                submitted_at: None,
            };
            create_submission(&pool, &input, &format!("/tmp/{}.cpp", i))
                .await
                .unwrap();
        }

        let subs = list_submissions_by_problem(&pool, &p.id).await.unwrap();
        assert_eq!(subs.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_submission() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "DelSub", vec![]).await;

        let input = CreateSubmissionInput {
            problem_id: p.id.clone(),
            status: "AC".into(),
            language: "C++".into(),
            code_text: "code".into(),
            runtime: None,
            memory: None,
            note: None,
            external_run_id: None,
            submitted_at: None,
        };
        let sub = create_submission(&pool, &input, "/tmp/x.cpp")
            .await
            .unwrap();

        delete_submission(&pool, &sub.id).await.unwrap();
        let result = get_submission(&pool, &sub.id).await;
        assert!(result.is_err());
    }

    // -- Solution Notes --

    #[tokio::test]
    async fn test_create_and_list_notes() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "NoteTest", vec![]).await;

        let input = CreateNoteInput {
            problem_id: p.id.clone(),
            note_type: "self".into(),
            content: "Use hashmap for O(n)".into(),
            source_url: Some("https://blog.example.com".into()),
        };

        let note = create_note(&pool, &input).await.unwrap();
        assert_eq!(note.content, "Use hashmap for O(n)");
        assert_eq!(note.note_type, "self");

        let notes = list_notes_by_problem(&pool, &p.id).await.unwrap();
        assert_eq!(notes.len(), 1);
    }

    #[tokio::test]
    async fn test_update_note() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "UpdateNote", vec![]).await;
        let input = CreateNoteInput {
            problem_id: p.id.clone(),
            note_type: "self".into(),
            content: "initial".into(),
            source_url: None,
        };
        let note = create_note(&pool, &input).await.unwrap();

        let updated = update_note(&pool, &note.id, "updated content")
            .await
            .unwrap();
        assert_eq!(updated.content, "updated content");
    }

    // -- Error Analyses --

    #[tokio::test]
    async fn test_create_and_list_error_analysis() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "ErrTest", vec![]).await;

        // Need a submission first
        let sub_input = CreateSubmissionInput {
            problem_id: p.id.clone(),
            status: "WA".into(),
            language: "C++".into(),
            code_text: "wa code".into(),
            runtime: None,
            memory: None,
            note: None,
            external_run_id: None,
            submitted_at: None,
        };
        let sub = create_submission(&pool, &sub_input, "/tmp/wa.cpp")
            .await
            .unwrap();

        let input = CreateErrorInput {
            problem_id: p.id.clone(),
            submission_id: sub.id.clone(),
            error_type: "OffByOne".into(),
            root_cause: "Forgot to initialize dp[0]".into(),
            fix_summary: "dp[0] = 1 before loop".into(),
            related_knowledge: vec!["DP".into()],
        };

        let err = create_error_analysis(&pool, &input).await.unwrap();
        assert_eq!(err.error_type, "OffByOne");
        assert_eq!(
            err.related_knowledge,
            serde_json::to_string(&["DP"]).unwrap()
        );

        let errors = list_error_analyses_by_problem(&pool, &p.id).await.unwrap();
        assert_eq!(errors.len(), 1);
    }

    // -- Knowledge Points --

    #[tokio::test]
    async fn test_create_knowledge_point() {
        let pool = test_pool().await;
        let input = CreateKnowledgeInput {
            name: "Binary Search".into(),
            category: "Search".into(),
            parent_id: None,
        };

        let kp = create_knowledge_point(&pool, &input).await.unwrap();
        assert_eq!(kp.name, "Binary Search");
        assert_eq!(kp.category, "Search");
    }

    #[tokio::test]
    async fn test_list_knowledge_points() {
        let pool = test_pool().await;
        // Seed knowledge points are already inserted by the migration
        let points = list_knowledge_points(&pool).await.unwrap();
        // Should have the 9 seeded categories
        assert!(
            points.len() >= 9,
            "expected at least 9 seed points, got {}",
            points.len()
        );
    }

    // -- Reports --

    #[tokio::test]
    async fn test_save_and_list_reports() {
        let pool = test_pool().await;
        let input = GenerateReportInput {
            report_type: "weekly".into(),
            title: "Week 1 Report".into(),
            start_date: "2025-01-01".into(),
            end_date: "2025-01-07".into(),
        };

        let report = save_report(&pool, &input, "# Week 1\nContent")
            .await
            .unwrap();
        assert_eq!(report.title, "Week 1 Report");
        assert_eq!(report.content, "# Week 1\nContent");

        let reports = list_reports(&pool).await.unwrap();
        assert_eq!(reports.len(), 1);
    }

    // -- Dashboard Stats --

    #[tokio::test]
    async fn test_dashboard_stats_empty() {
        let pool = test_pool().await;
        let stats = get_dashboard_stats(&pool).await.unwrap();
        assert_eq!(stats.total_problems, 0);
        assert_eq!(stats.total_submissions, 0);
        assert_eq!(stats.ac_count, 0);
    }

    #[tokio::test]
    async fn test_dashboard_stats_with_data() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "StatsTest", vec!["dp"]).await;

        for status in ["AC", "WA", "WA", "TLE", "AC", "RE"] {
            let input = CreateSubmissionInput {
                problem_id: p.id.clone(),
                status: status.into(),
                language: "C++".into(),
                code_text: "x".into(),
                runtime: None,
                memory: None,
                note: None,
                external_run_id: None,
                submitted_at: None,
            };
            create_submission(&pool, &input, &format!("/tmp/{}.cpp", status))
                .await
                .unwrap();
        }

        let stats = get_dashboard_stats(&pool).await.unwrap();
        assert_eq!(stats.total_problems, 1);
        assert_eq!(stats.total_submissions, 6);
        assert_eq!(stats.ac_count, 2);
        assert_eq!(stats.wa_count, 2);
        assert_eq!(stats.tle_count, 1);
        assert_eq!(stats.re_count, 1);
        assert_eq!(stats.other_count, 0);
    }

    #[tokio::test]
    async fn test_error_type_stats() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "ErrTypeTest", vec![]).await;

        let sub = {
            let input = CreateSubmissionInput {
                problem_id: p.id.clone(),
                status: "WA".into(),
                language: "C++".into(),
                code_text: "x".into(),
                runtime: None,
                memory: None,
                note: None,
                external_run_id: None,
                submitted_at: None,
            };
            create_submission(&pool, &input, "/tmp/x.cpp")
                .await
                .unwrap()
        };

        for error_type in ["OffByOne", "Logic", "OffByOne", "Overflow"] {
            let input = CreateErrorInput {
                problem_id: p.id.clone(),
                submission_id: sub.id.clone(),
                error_type: error_type.into(),
                root_cause: "test".into(),
                fix_summary: "test".into(),
                related_knowledge: vec![],
            };
            create_error_analysis(&pool, &input).await.unwrap();
        }

        let stats = get_error_type_stats(&pool).await.unwrap();
        assert_eq!(stats.len(), 3);
        // OffByOne should be first with count 2
        assert_eq!(stats[0].error_type, "OffByOne");
        assert_eq!(stats[0].count, 2);
    }

    // -- Edge cases --

    #[tokio::test]
    async fn test_create_problem_empty_tags() {
        let pool = test_pool().await;
        let input = CreateProblemInput {
            source: "OJ".into(),
            source_problem_id: "P1".into(),
            title: "Empty Tags".into(),
            url: None,
            difficulty: None,
            tags: vec![],
            statement: None,
        };
        let problem = create_problem(&pool, &input, None).await.unwrap();
        assert_eq!(problem.tags, "[]");
    }

    #[tokio::test]
    async fn test_delete_problem_cascades() {
        let pool = test_pool().await;
        let p = create_test_problem(&pool, "Cascade", vec![]).await;

        let sub_input = CreateSubmissionInput {
            problem_id: p.id.clone(),
            status: "AC".into(),
            language: "C++".into(),
            code_text: "x".into(),
            runtime: None,
            memory: None,
            note: None,
            external_run_id: None,
            submitted_at: None,
        };
        let sub = create_submission(&pool, &sub_input, "/tmp/x.cpp")
            .await
            .unwrap();

        let err_input = CreateErrorInput {
            problem_id: p.id.clone(),
            submission_id: sub.id.clone(),
            error_type: "Bug".into(),
            root_cause: "x".into(),
            fix_summary: "y".into(),
            related_knowledge: vec![],
        };
        create_error_analysis(&pool, &err_input).await.unwrap();

        // Delete the problem — cascade should delete submissions and errors too
        delete_problem(&pool, &p.id).await.unwrap();

        let subs = list_submissions_by_problem(&pool, &p.id).await.unwrap();
        assert!(subs.is_empty());

        let errors = list_error_analyses_by_problem(&pool, &p.id).await.unwrap();
        assert!(errors.is_empty());
    }

    // -- Settings --

    #[tokio::test]
    async fn test_get_setting_default() {
        let pool = test_pool().await;
        let val = get_setting(&pool, "ai_provider").await.unwrap();
        assert_eq!(val, Some("openai".into()));
    }

    #[tokio::test]
    async fn test_get_setting_not_found() {
        let pool = test_pool().await;
        let val = get_setting(&pool, "nonexistent").await.unwrap();
        assert_eq!(val, None);
    }

    #[tokio::test]
    async fn test_set_and_get_setting() {
        let pool = test_pool().await;
        set_setting(&pool, "ai_provider", "anthropic")
            .await
            .unwrap();
        let val = get_setting(&pool, "ai_provider").await.unwrap();
        assert_eq!(val, Some("anthropic".into()));
    }

    #[tokio::test]
    async fn test_get_all_settings() {
        let pool = test_pool().await;
        let settings = get_all_settings(&pool).await.unwrap();
        // Should have default settings from 002_settings.sql
        assert!(settings.iter().any(|s| s.key == "ai_provider"));
        assert!(settings.iter().any(|s| s.key == "ai_model"));
    }

    #[tokio::test]
    async fn test_set_setting_new_key() {
        let pool = test_pool().await;
        set_setting(&pool, "custom_key", "custom_value")
            .await
            .unwrap();
        let val = get_setting(&pool, "custom_key").await.unwrap();
        assert_eq!(val, Some("custom_value".into()));
    }
}

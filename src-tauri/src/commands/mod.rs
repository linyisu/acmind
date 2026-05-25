use crate::db::models::*;
use crate::db::repo;
use crate::error::AppError;
use crate::storage::Storage;
use sqlx::SqlitePool;
use tauri::State;
use tracing::info;

// -- Problems --

#[tauri::command(rename_all = "camelCase")]
pub async fn list_problems(pool: State<'_, SqlitePool>) -> Result<Vec<Problem>, AppError> {
    Ok(repo::list_problems(&pool).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_problem(pool: State<'_, SqlitePool>, id: String) -> Result<Problem, AppError> {
    Ok(repo::get_problem(&pool, &id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_problem(
    pool: State<'_, SqlitePool>,
    storage: State<'_, Storage>,
    input: CreateProblemInput,
) -> Result<Problem, AppError> {
    let statement_path = if let Some(ref statement) = input.statement {
        // Create a temporary id to save the file; the real id is generated in the repo
        let temp_id = new_id();
        Some(storage.save_statement(&temp_id, statement)?)
    } else {
        None
    };

    let mut problem = repo::create_problem(&pool, &input, statement_path.as_deref()).await?;

    // If statement was saved with a temp id, move the file to the real problem dir
    if let Some(ref tmp_path) = statement_path {
        let new_path = storage.save_statement(&problem.id, &storage.read_file(tmp_path)?)?;
        // Update the path in DB
        repo::update_problem(
            &pool,
            &problem.id,
            &UpdateProblemInput {
                source: None,
                source_problem_id: None,
                title: None,
                url: None,
                difficulty: None,
                tags: None,
                statement: None,
            },
            Some(&new_path),
        )
        .await?;
        // Clean up temp
        storage.delete_file(tmp_path).ok();
        // Re-fetch for correct data
        problem = repo::get_problem(&pool, &problem.id).await?;
    }

    Ok(problem)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_problem(
    pool: State<'_, SqlitePool>,
    storage: State<'_, Storage>,
    id: String,
    input: UpdateProblemInput,
) -> Result<Problem, AppError> {
    let statement_path = if let Some(ref statement) = input.statement {
        Some(storage.save_statement(&id, statement)?)
    } else {
        None
    };

    Ok(repo::update_problem(&pool, &id, &input, statement_path.as_deref()).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_problem(
    pool: State<'_, SqlitePool>,
    storage: State<'_, Storage>,
    id: String,
) -> Result<(), AppError> {
    repo::delete_problem(&pool, &id).await?;
    storage.delete_problem_dir(&id).ok();
    Ok(())
}

// -- Submissions --

#[tauri::command(rename_all = "camelCase")]
pub async fn list_submissions_by_problem(
    pool: State<'_, SqlitePool>,
    problem_id: String,
) -> Result<Vec<Submission>, AppError> {
    Ok(repo::list_submissions_by_problem(&pool, &problem_id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_submission(
    pool: State<'_, SqlitePool>,
    storage: State<'_, Storage>,
    id: String,
) -> Result<serde_json::Value, AppError> {
    let sub = repo::get_submission(&pool, &id).await?;
    let code = storage.read_code(&sub.code_path).ok();

    let mut value = serde_json::to_value(&sub)?;
    if let Some(code) = code {
        value["code_text"] = serde_json::Value::String(code);
    }

    Ok(value)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_submission(
    pool: State<'_, SqlitePool>,
    storage: State<'_, Storage>,
    input: CreateSubmissionInput,
) -> Result<Submission, AppError> {
    // Save code to file first with a temp id
    let temp_id = new_id();
    let code_path = storage.save_submission(
        &input.problem_id,
        &temp_id,
        &input.status,
        &input.language,
        &input.code_text,
    )?;

    let sub = repo::create_submission(&pool, &input, &code_path).await?;

    // Rename file to use real submission id
    let new_code_path = storage.save_submission(
        &input.problem_id,
        &sub.id,
        &input.status,
        &input.language,
        &input.code_text,
    )?;

    // Update code_path in DB to the new filename
    sqlx::query("UPDATE submissions SET code_path = ?1 WHERE id = ?2")
        .bind(&new_code_path)
        .bind(&sub.id)
        .execute(&*pool)
        .await?;

    // Clean up old temp file
    storage.delete_file(&code_path).ok();

    Ok(repo::get_submission(&pool, &sub.id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_submission(
    pool: State<'_, SqlitePool>,
    storage: State<'_, Storage>,
    id: String,
) -> Result<(), AppError> {
    let sub = repo::get_submission(&pool, &id).await?;
    storage.delete_file(&sub.code_path).ok();
    repo::delete_submission(&pool, &id).await?;
    Ok(())
}

// -- Notes --

#[tauri::command(rename_all = "camelCase")]
pub async fn list_notes_by_problem(
    pool: State<'_, SqlitePool>,
    problem_id: String,
) -> Result<Vec<SolutionNote>, AppError> {
    Ok(repo::list_notes_by_problem(&pool, &problem_id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_note(
    pool: State<'_, SqlitePool>,
    input: CreateNoteInput,
) -> Result<SolutionNote, AppError> {
    Ok(repo::create_note(&pool, &input).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn update_note(
    pool: State<'_, SqlitePool>,
    id: String,
    content: String,
) -> Result<SolutionNote, AppError> {
    Ok(repo::update_note(&pool, &id, &content).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_note(pool: State<'_, SqlitePool>, id: String) -> Result<(), AppError> {
    repo::delete_note(&pool, &id).await?;
    Ok(())
}

// -- Error Analyses --

#[tauri::command(rename_all = "camelCase")]
pub async fn list_error_analyses_by_problem(
    pool: State<'_, SqlitePool>,
    problem_id: String,
) -> Result<Vec<ErrorAnalysis>, AppError> {
    Ok(repo::list_error_analyses_by_problem(&pool, &problem_id).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_error_analysis(
    pool: State<'_, SqlitePool>,
    input: CreateErrorInput,
) -> Result<ErrorAnalysis, AppError> {
    Ok(repo::create_error_analysis(&pool, &input).await?)
}

// -- Knowledge Points --

#[tauri::command(rename_all = "camelCase")]
pub async fn list_knowledge_points(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<KnowledgePoint>, AppError> {
    Ok(repo::list_knowledge_points(&pool).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn create_knowledge_point(
    pool: State<'_, SqlitePool>,
    input: CreateKnowledgeInput,
) -> Result<KnowledgePoint, AppError> {
    Ok(repo::create_knowledge_point(&pool, &input).await?)
}

// -- Reports --

#[tauri::command(rename_all = "camelCase")]
pub async fn list_reports(pool: State<'_, SqlitePool>) -> Result<Vec<Report>, AppError> {
    Ok(repo::list_reports(&pool).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn generate_report(
    pool: State<'_, SqlitePool>,
    input: GenerateReportInput,
) -> Result<Report, AppError> {
    let stats = repo::get_dashboard_stats(&pool).await?;
    let error_stats = repo::get_error_type_stats(&pool).await?;

    // Build a simple markdown report (AI integration to be added later)
    let content = build_report_content(&input, &stats, &error_stats);
    Ok(repo::save_report(&pool, &input, &content).await?)
}

fn build_report_content(
    input: &GenerateReportInput,
    stats: &repo::DashboardStats,
    error_stats: &[repo::ErrorTypeStat],
) -> String {
    let ac_rate = if stats.total_submissions > 0 {
        (stats.ac_count as f64 / stats.total_submissions as f64) * 100.0
    } else {
        0.0
    };

    let mut content = format!(
        "# {}\n\n**Period:** {} to {}\n\n",
        input.title, input.start_date, input.end_date
    );

    content.push_str("## Overview\n\n");
    content.push_str(&format!("- Total Problems: {}\n", stats.total_problems));
    content.push_str(&format!(
        "- Total Submissions: {}\n",
        stats.total_submissions
    ));
    content.push_str(&format!("- AC Rate: {:.1}%\n", ac_rate));

    content.push_str("\n## Submission Breakdown\n\n");
    content.push_str(&format!("- AC: {}\n", stats.ac_count));
    content.push_str(&format!("- WA: {}\n", stats.wa_count));
    content.push_str(&format!("- TLE: {}\n", stats.tle_count));
    content.push_str(&format!("- RE: {}\n", stats.re_count));
    content.push_str(&format!("- Other: {}\n", stats.other_count));

    if !error_stats.is_empty() {
        content.push_str("\n## Common Error Types\n\n");
        for stat in error_stats {
            content.push_str(&format!("- {}: {}\n", stat.error_type, stat.count));
        }
    }

    content
}

// -- AI Analysis --

#[tauri::command(rename_all = "camelCase")]
pub async fn analyze_problem(
    pool: State<'_, SqlitePool>,
    problem_id: String,
) -> Result<crate::ai::AnalysisResult, AppError> {
    info!("analyze_problem command called for problem {}", problem_id);
    let analysis = crate::ai::analyze_problem(&pool, &problem_id).await?;

    // Save error analysis to DB (attach to first WA submission)
    let submissions = repo::list_submissions_by_problem(&pool, &problem_id).await?;
    if let Some(wa_sub) = submissions.iter().find(|s| s.status == "WA") {
        let error_input = crate::db::models::CreateErrorInput {
            problem_id: problem_id.clone(),
            submission_id: wa_sub.id.clone(),
            error_type: analysis.error_type.clone(),
            root_cause: analysis.root_cause.clone(),
            fix_summary: analysis.fix_summary.clone(),
            related_knowledge: analysis.knowledge_points.clone(),
        };
        repo::create_error_analysis(&pool, &error_input).await?;
    }

    // Save suggestions as an AI note
    let note_content = format!(
        "# AI Analysis\n\n## Root Cause\n{}\n\n## Fix\n{}\n\n## Suggestions\n{}",
        analysis.root_cause,
        analysis.fix_summary,
        analysis
            .suggestions
            .iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let note_input = crate::db::models::CreateNoteInput {
        problem_id,
        note_type: "ai".into(),
        content: note_content,
        source_url: None,
    };
    repo::create_note(&pool, &note_input).await?;

    Ok(analysis)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn analyze_problem_streaming(
    pool: State<'_, SqlitePool>,
    problem_id: String,
    channel: tauri::ipc::Channel<String>,
) -> Result<crate::ai::AnalysisResult, AppError> {
    let pid = problem_id.clone();
    let full_response = crate::ai::analyze_problem_streaming(&pool, &problem_id, |chunk| {
        let _ = channel.send(chunk.to_string());
    })
    .await?;

    // Parse the full response into AnalysisResult
    let analysis = crate::ai::parse_analysis_response(&full_response)?;

    // Save to DB (same as analyze_problem)
    let submissions = repo::list_submissions_by_problem(&pool, &pid).await?;
    if let Some(wa_sub) = submissions.iter().find(|s| s.status == "WA") {
        let error_input = crate::db::models::CreateErrorInput {
            problem_id: pid.clone(),
            submission_id: wa_sub.id.clone(),
            error_type: analysis.error_type.clone(),
            root_cause: analysis.root_cause.clone(),
            fix_summary: analysis.fix_summary.clone(),
            related_knowledge: analysis.knowledge_points.clone(),
        };
        repo::create_error_analysis(&pool, &error_input).await?;
    }

    let note_content = format!(
        "# AI Analysis\n\n## Root Cause\n{}\n\n## Fix\n{}\n\n## Suggestions\n{}",
        analysis.root_cause,
        analysis.fix_summary,
        analysis
            .suggestions
            .iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let note_input = crate::db::models::CreateNoteInput {
        problem_id: pid,
        note_type: "ai".into(),
        content: note_content,
        source_url: None,
    };
    repo::create_note(&pool, &note_input).await?;

    Ok(analysis)
}

// -- Dashboard --

#[tauri::command(rename_all = "camelCase")]
pub async fn get_dashboard_stats(
    pool: State<'_, SqlitePool>,
) -> Result<repo::DashboardStats, AppError> {
    Ok(repo::get_dashboard_stats(&pool).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_error_type_stats(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<repo::ErrorTypeStat>, AppError> {
    Ok(repo::get_error_type_stats(&pool).await?)
}

// -- Settings --

#[tauri::command(rename_all = "camelCase")]
pub async fn get_setting(
    pool: State<'_, SqlitePool>,
    key: String,
) -> Result<Option<String>, AppError> {
    Ok(repo::get_setting(&pool, &key).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn set_setting(
    pool: State<'_, SqlitePool>,
    key: String,
    value: String,
) -> Result<(), AppError> {
    Ok(repo::set_setting(&pool, &key, &value).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_all_settings(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<repo::AppSetting>, AppError> {
    Ok(repo::get_all_settings(&pool).await?)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_log_path(storage: State<'_, Storage>) -> Result<String, AppError> {
    Ok(storage
        .base_dir()
        .join("acmind.log")
        .to_string_lossy()
        .to_string())
}

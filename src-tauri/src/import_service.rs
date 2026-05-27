use crate::db::models::{CreateProblemInput, CreateSubmissionInput, UpdateProblemInput};
use crate::db::repo;
use crate::storage::Storage;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::runtime::Handle;

const VJUDGE_CDN_ORIGIN: &str = "https://cdn.vjudge.net.cn";

pub type ImportNotifier = Arc<dyn Fn(&str, &str) + Send + Sync>;

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_problems: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_synced: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ImportSubmissionsPayload {
    pub username: String,
    pub items: Vec<SubmissionItem>,
    #[serde(default)]
    pub include_source: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionItem {
    #[serde(alias = "runId")]
    pub run_id: i64,
    pub oj: String,
    #[serde(alias = "probNum")]
    pub prob_num: String,
    pub status: String,
    pub language: String,
    #[serde(default)]
    pub runtime: Option<i32>,
    #[serde(default)]
    pub memory: Option<i32>,
    #[allow(dead_code)]
    pub time: i64,
    #[serde(default)]
    #[serde(alias = "sourceProblemId")]
    pub source_problem_id: String,
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProblemPayload {
    #[serde(alias = "sourceProblemId")]
    pub source_problem_id: String,
    #[allow(dead_code)]
    pub oj: String,
    #[serde(alias = "probNum")]
    #[allow(dead_code)]
    pub prob_num: String,
    pub title: String,
    pub url: Option<String>,
    pub statement: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSubmissionPayload {
    #[serde(alias = "runId")]
    pub run_id: i64,
    pub oj: String,
    #[serde(alias = "probNum")]
    pub prob_num: String,
    pub status: String,
    pub language: String,
    pub code: Option<String>,
    pub runtime: Option<i32>,
    pub memory: Option<i32>,
    #[serde(alias = "submitTime")]
    pub submit_time: Option<i64>,
}

pub struct ImportState {
    pub pool: SqlitePool,
    pub storage: Storage,
    pub rt: Handle,
    pub notify: ImportNotifier,
}

impl ImportState {
    pub fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
        self.rt.block_on(f)
    }

    pub fn notify_imported(&self, action: &str, detail: &str) {
        (self.notify)(action, detail);
    }
}

pub struct ImportResult {
    pub created: bool,
    pub problem_created: bool,
    pub source_synced: bool,
}

struct VJudgeSubmissionDraft {
    run_id: i64,
    oj: String,
    #[allow(dead_code)]
    prob_num: String,
    source_problem_id: String,
    status: String,
    language: String,
    runtime: Option<i32>,
    memory: Option<i32>,
    code: String,
    submitted_at: chrono::DateTime<chrono::Utc>,
    note: Option<String>,
}

impl VJudgeSubmissionDraft {
    fn from_item(item: &SubmissionItem) -> Self {
        let source_problem_id = if item.source_problem_id.is_empty() {
            format!("{}-{}", item.oj, item.prob_num)
        } else {
            item.source_problem_id.clone()
        };

        Self {
            run_id: item.run_id,
            oj: item.oj.clone(),
            prob_num: item.prob_num.clone(),
            source_problem_id,
            status: item.status.clone(),
            language: item.language.clone(),
            runtime: item.runtime,
            memory: item.memory,
            code: item.code.clone().unwrap_or_default(),
            submitted_at: chrono::Utc::now(),
            note: None,
        }
    }

    fn from_payload(payload: &ImportSubmissionPayload) -> Self {
        Self {
            run_id: payload.run_id,
            oj: payload.oj.clone(),
            prob_num: payload.prob_num.clone(),
            source_problem_id: format!("{}-{}", payload.oj, payload.prob_num),
            status: payload.status.clone(),
            language: payload.language.clone(),
            runtime: payload.runtime,
            memory: payload.memory,
            code: payload.code.clone().unwrap_or_default(),
            submitted_at: payload
                .submit_time
                .and_then(|t| crate::vjudge::timestamp_millis(t).ok())
                .unwrap_or_else(chrono::Utc::now),
            note: Some(format!("VJudge run #{}，从浏览器扩展导入", payload.run_id)),
        }
    }
}

pub fn import_submissions(
    state: &ImportState,
    payload: &ImportSubmissionsPayload,
) -> Result<(usize, usize, usize), String> {
    if payload.items.is_empty() {
        return Ok((0, 0, 0));
    }

    if let Err(e) = state.block_on(repo::set_setting(
        &state.pool,
        "vjudge_username",
        &payload.username,
    )) {
        tracing::warn!(target: "app_lib::import_server", "Failed to save username: {}", e);
    }

    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut created_problems = 0usize;

    for item in &payload.items {
        let draft = VJudgeSubmissionDraft::from_item(item);
        match upsert_vjudge_submission(state, &draft) {
            Ok(result) => {
                if result.created {
                    imported += 1;
                } else {
                    skipped += 1;
                }
                if result.problem_created {
                    created_problems += 1;
                }
            }
            Err(e) => {
                tracing::warn!(target: "app_lib::import_server", "Failed to import submission run#{}: {}", item.run_id, e);
                skipped += 1;
            }
        }
    }

    state.notify_imported(
        "submissions",
        &format!(
            "{} imported ({} new, {} skipped)",
            payload.items.len(),
            imported,
            skipped
        ),
    );

    Ok((imported, skipped, created_problems))
}

pub fn import_problem(state: &ImportState, payload: &ImportProblemPayload) -> Result<bool, String> {
    let source_problem_id = payload.source_problem_id.clone();

    let existing = state
        .block_on(repo::find_problem_by_source_id(
            &state.pool,
            "VJudge",
            &source_problem_id,
        ))
        .map_err(|e| format!("DB error: {}", e))?;

    if let Some(problem) = existing {
        if let Some(ref statement) = payload.statement {
            let statement = normalize_statement_asset_urls(statement);
            let statement_path = state
                .storage
                .save_statement(&problem.id, &statement)
                .map_err(|e| format!("Failed to save problem statement: {}", e))?;
            state
                .block_on(repo::update_problem(
                    &state.pool,
                    &problem.id,
                    &UpdateProblemInput {
                        source: None,
                        source_problem_id: None,
                        title: Some(payload.title.clone()),
                        url: payload.url.clone(),
                        difficulty: None,
                        tags: Some(payload.tags.clone()),
                        statement: None,
                    },
                    Some(&statement_path),
                ))
                .map_err(|e| format!("Failed to update problem statement: {}", e))?;
        }
        state.notify_imported("problem", &format!("Updated {}", source_problem_id));
        return Ok(false);
    }

    let statement_path = if let Some(ref statement) = payload.statement {
        let statement = normalize_statement_asset_urls(statement);
        Some(
            state
                .storage
                .save_statement(&source_problem_id, &statement)
                .map_err(|e| format!("Failed to save problem statement: {}", e))?,
        )
    } else {
        None
    };

    state
        .block_on(repo::create_problem(
            &state.pool,
            &CreateProblemInput {
                source: "VJudge".into(),
                source_problem_id: source_problem_id.clone(),
                title: payload.title.clone(),
                url: payload.url.clone(),
                difficulty: None,
                tags: payload.tags.clone(),
                statement: None,
            },
            statement_path.as_deref(),
        ))
        .map_err(|e| format!("Failed to create problem: {}", e))?;

    state.notify_imported("problem", &format!("Created {}", source_problem_id));

    Ok(true)
}

fn normalize_statement_asset_urls(statement: &str) -> String {
    statement.replace("CDN_BASE_URL/", &format!("{}/", VJUDGE_CDN_ORIGIN))
}

pub fn import_submission(
    state: &ImportState,
    payload: &ImportSubmissionPayload,
) -> Result<ImportResult, String> {
    let draft = VJudgeSubmissionDraft::from_payload(payload);
    let result = upsert_vjudge_submission(state, &draft)?;

    if result.created {
        state.notify_imported(
            "submission",
            &format!(
                "#{} {}-{} {}",
                payload.run_id, payload.oj, payload.prob_num, payload.status
            ),
        );
    } else if result.source_synced {
        state.notify_imported("submission", &format!("#{} source synced", payload.run_id));
    }

    Ok(result)
}

fn upsert_vjudge_submission(
    state: &ImportState,
    draft: &VJudgeSubmissionDraft,
) -> Result<ImportResult, String> {
    let external_run_id = format!("vjudge:{}", draft.run_id);
    let status = normalize_status(&draft.status);

    if let Some(existing) = state
        .block_on(repo::submission_by_external_run_id(
            &state.pool,
            &external_run_id,
        ))
        .map_err(|e| format!("DB error: {}", e))?
    {
        let source_synced = sync_missing_source(state, &existing.id, &existing.problem_id, draft)?;
        return Ok(ImportResult {
            created: false,
            problem_created: false,
            source_synced,
        });
    }

    let (problem_id, problem_created) = find_or_create_problem(state, draft)?;
    let code_path = save_source_if_present(state, &problem_id, draft)?;

    state
        .block_on(repo::create_submission(
            &state.pool,
            &CreateSubmissionInput {
                problem_id: problem_id.clone(),
                status: status.into(),
                language: draft.language.clone(),
                code_text: draft.code.clone(),
                runtime: draft.runtime,
                memory: draft.memory,
                note: draft.note.clone(),
                external_run_id: Some(external_run_id),
                submitted_at: Some(draft.submitted_at),
            },
            &code_path,
        ))
        .map_err(|e| format!("Failed to create submission: {}", e))?;

    Ok(ImportResult {
        created: true,
        problem_created,
        source_synced: !code_path.is_empty(),
    })
}

fn find_or_create_problem(
    state: &ImportState,
    draft: &VJudgeSubmissionDraft,
) -> Result<(String, bool), String> {
    match state.block_on(repo::find_problem_by_source_id(
        &state.pool,
        "VJudge",
        &draft.source_problem_id,
    )) {
        Ok(Some(problem)) => Ok((problem.id, false)),
        Ok(None) => {
            let created = state
                .block_on(repo::create_problem(
                    &state.pool,
                    &CreateProblemInput {
                        source: "VJudge".into(),
                        source_problem_id: draft.source_problem_id.clone(),
                        title: draft.source_problem_id.clone(),
                        url: Some(format!(
                            "https://vjudge.net/problem/{}",
                            draft.source_problem_id
                        )),
                        difficulty: None,
                        tags: vec![draft.oj.clone()],
                        statement: None,
                    },
                    None,
                ))
                .map_err(|e| format!("Failed to create problem: {}", e))?;
            Ok((created.id, true))
        }
        Err(e) => Err(format!("DB error: {}", e)),
    }
}

fn sync_missing_source(
    state: &ImportState,
    submission_id: &str,
    problem_id: &str,
    draft: &VJudgeSubmissionDraft,
) -> Result<bool, String> {
    if draft.code.is_empty() {
        return Ok(false);
    }

    let temp_id = uuid::Uuid::new_v4().to_string();
    let code_path = state
        .storage
        .save_submission(
            problem_id,
            &temp_id,
            normalize_status(&draft.status),
            &draft.language,
            &draft.code,
        )
        .map_err(|e| format!("Failed to save submission source: {}", e))?;

    state
        .block_on(repo::set_submission_code_path(
            &state.pool,
            submission_id,
            &code_path,
        ))
        .map_err(|e| format!("Failed to sync submission source: {}", e))?;

    Ok(true)
}

fn save_source_if_present(
    state: &ImportState,
    problem_id: &str,
    draft: &VJudgeSubmissionDraft,
) -> Result<String, String> {
    if draft.code.is_empty() {
        return Ok(String::new());
    }

    let temp_id = uuid::Uuid::new_v4().to_string();
    state
        .storage
        .save_submission(
            problem_id,
            &temp_id,
            normalize_status(&draft.status),
            &draft.language,
            &draft.code,
        )
        .map_err(|e| format!("Failed to save submission source: {}", e))
}

fn normalize_status(status: &str) -> &str {
    match status {
        "Accepted" => "AC",
        "Wrong answer" | "Wrong Answer" => "WA",
        "Time limit exceeded" | "Time Limit Exceeded" | "Time Limit Exceed" => "TLE",
        "Runtime error" | "Runtime Error" => "RE",
        "Memory limit exceeded" | "Memory Limit Exceeded" | "Memory Limit Exceed" => "MLE",
        "Compile error" | "Compilation error" | "Compile Error" => "CE",
        _ => "WA",
    }
}

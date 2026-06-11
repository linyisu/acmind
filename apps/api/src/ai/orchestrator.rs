use crate::{
    ai::{
        agents::{
            classifier::ClassifierAgent,
            error::ErrorAgent,
            knowledge::KnowledgeAgent,
            template::{TemplateAgent, TemplateAgentOutput},
        },
        context::{collect_context, TemplateBrief},
        repo as ai_repo,
    },
    error::{AppError, AppResult},
    knowledge::repo as knowledge_repo,
    state::AppState,
    task::{
        model::{AgentProgress, AgentStep, TaskProgress},
        repo as task_repo,
    },
    template::model::ListTemplatesQuery,
    template::repo as template_repo,
};
use serde_json::{json, Value};

// ─── Agent definitions ──────────────────────────────────────────────

struct AgentDef {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
}

const AGENT_CLASSIFIER: AgentDef = AgentDef {
    id: "classifier",
    name: "题目分析官",
    icon: "🔍",
};
const AGENT_TEMPLATE: AgentDef = AgentDef {
    id: "template",
    name: "模板提取官",
    icon: "🔧",
};
const AGENT_ERROR: AgentDef = AgentDef {
    id: "error",
    name: "错误诊断官",
    icon: "🔎",
};
const AGENT_KNOWLEDGE: AgentDef = AgentDef {
    id: "knowledge",
    name: "知识梳理官",
    icon: "📚",
};
const AGENT_SAVER: AgentDef = AgentDef {
    id: "saver",
    name: "数据存档官",
    icon: "💾",
};

fn initial_progress() -> TaskProgress {
    TaskProgress {
        agents: vec![
            AgentProgress {
                id: AGENT_CLASSIFIER.id.into(),
                name: AGENT_CLASSIFIER.name.into(),
                icon: AGENT_CLASSIFIER.icon.into(),
                status: "pending".into(),
                message: String::new(),
                steps: vec![],
            },
            AgentProgress {
                id: AGENT_TEMPLATE.id.into(),
                name: AGENT_TEMPLATE.name.into(),
                icon: AGENT_TEMPLATE.icon.into(),
                status: "pending".into(),
                message: String::new(),
                steps: vec![],
            },
            AgentProgress {
                id: AGENT_ERROR.id.into(),
                name: AGENT_ERROR.name.into(),
                icon: AGENT_ERROR.icon.into(),
                status: "pending".into(),
                message: String::new(),
                steps: vec![],
            },
            AgentProgress {
                id: AGENT_KNOWLEDGE.id.into(),
                name: AGENT_KNOWLEDGE.name.into(),
                icon: AGENT_KNOWLEDGE.icon.into(),
                status: "pending".into(),
                message: String::new(),
                steps: vec![],
            },
            AgentProgress {
                id: AGENT_SAVER.id.into(),
                name: AGENT_SAVER.name.into(),
                icon: AGENT_SAVER.icon.into(),
                status: "pending".into(),
                message: String::new(),
                steps: vec![],
            },
        ],
    }
}

// ─── Progress helpers ───────────────────────────────────────────────

async fn update_progress(db: &sea_orm::DatabaseConnection, task_id: i64, progress: &TaskProgress) {
    if let Ok(val) = serde_json::to_value(progress) {
        let _ = task_repo::update_progress(db, task_id, &val).await;
    }
}

async fn set_agent_status(
    db: &sea_orm::DatabaseConnection,
    task_id: i64,
    progress: &mut TaskProgress,
    agent_id: &str,
    status: &str,
    message: &str,
) {
    if let Some(agent) = progress.agents.iter_mut().find(|a| a.id == agent_id) {
        agent.status = status.to_string();
        agent.message = message.to_string();
    }
    update_progress(db, task_id, progress).await;
}

async fn set_agent_step(
    db: &sea_orm::DatabaseConnection,
    task_id: i64,
    progress: &mut TaskProgress,
    agent_id: &str,
    step_label: &str,
    status: &str,
    detail: &str,
) {
    if let Some(agent) = progress.agents.iter_mut().find(|a| a.id == agent_id) {
        if let Some(step) = agent.steps.iter_mut().find(|s| s.label == step_label) {
            step.status = status.to_string();
            step.detail = detail.to_string();
        } else {
            agent.steps.push(AgentStep {
                label: step_label.to_string(),
                status: status.to_string(),
                detail: detail.to_string(),
            });
        }
    }
    update_progress(db, task_id, progress).await;
}

/// Public helper for route.rs to create initial progress JSON.
pub fn new_initial_progress() -> Value {
    serde_json::to_value(initial_progress()).unwrap_or_default()
}

/// Check whether the current task has been cancelled by the user.
/// Call this periodically from long-running operations to bail out promptly.
async fn check_cancelled(db: &sea_orm::DatabaseConnection, task_id: i64) -> AppResult<bool> {
    Ok(task_repo::is_cancelled(db, task_id).await)
}

/// Retry an async operation up to `max_attempts` times on transient failures
/// (LLM call errors, parse errors). Returns the first success or last error.
async fn with_retry<F, Fut, T>(
    label: &str,
    task_id: i64,
    db: &sea_orm::DatabaseConnection,
    max_attempts: u32,
    mut op: F,
) -> AppResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = AppResult<T>>,
{
    let mut last_err: Option<crate::error::AppError> = None;
    for attempt in 1..=max_attempts {
        // Cancel check before each attempt
        if check_cancelled(db, task_id).await? {
            return Err(crate::error::AppError::BadRequest(format!(
                "任务 {} 已取消",
                task_id
            )));
        }
        match op().await {
            Ok(v) => {
                if attempt > 1 {
                    tracing::info!("[task-{}] {} 第 {} 次重试成功", task_id, label, attempt);
                }
                return Ok(v);
            }
            Err(e) => {
                tracing::warn!(
                    "[task-{}] {} 第 {}/{} 次失败: {}",
                    task_id,
                    label,
                    attempt,
                    max_attempts,
                    e
                );
                last_err = Some(e);
                if attempt < max_attempts {
                    let delay_ms = 500u64 * (1 << (attempt - 1)); // 0.5s, 1s, 2s
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                }
            }
        }
    }
    Err(last_err.unwrap_or_else(|| crate::error::AppError::Internal(format!("{} 重试耗尽", label))))
}

// ─── Main orchestrator ──────────────────────────────────────────────

pub async fn run_task(
    state: &AppState,
    task_id: i64,
    user_id: i64,
    problem_id: i64,
) -> AppResult<()> {
    tracing::info!("[task-{}] 开始全量分析 problem={}", task_id, problem_id);
    task_repo::mark_running(&state.db, task_id).await?;
    let mut progress = initial_progress();

    // ── Stage 0: 收集数据 ──────────────────────────────────────────
    set_agent_status(
        &state.db,
        task_id,
        &mut progress,
        "classifier",
        "running",
        "正在收集数据...",
    )
    .await;
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "classifier",
        "收集提交记录",
        "running",
        "",
    )
    .await;

    let ctx = collect_context(&state.db, user_id, problem_id).await?;
    if ctx.submissions.is_empty() {
        set_agent_step(
            &state.db,
            task_id,
            &mut progress,
            "classifier",
            "收集提交记录",
            "failed",
            "没有提交记录",
        )
        .await;
        set_agent_status(
            &state.db,
            task_id,
            &mut progress,
            "classifier",
            "failed",
            "没有提交记录",
        )
        .await;
        return Err(AppError::BadRequest("该题目没有提交记录".into()));
    }

    let ac_count = ctx.submissions.iter().filter(|s| s.verdict == "AC").count();
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "classifier",
        "收集提交记录",
        "completed",
        &format!("{} 条提交 ({} AC)", ctx.submissions.len(), ac_count),
    )
    .await;

    // ── Stage 1: 分类 ──────────────────────────────────────────────
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "classifier",
        "分析算法类型",
        "running",
        "",
    )
    .await;

    let classifier = ClassifierAgent;
    let llm = &*state.llm;
    let (brief, _) = with_retry("题目分类", task_id, &state.db, 2, || async {
        classifier.run(llm, &ctx).await
    })
    .await?;

    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "classifier",
        "分析算法类型",
        "completed",
        &format!("{} / {}", brief.algorithm_type, brief.sub_type),
    )
    .await;

    // Save classification to ai_analysis
    let analysis_json = json!({
        "algorithm_type": brief.algorithm_type,
        "sub_type": brief.sub_type,
        "tags": brief.tags,
        "summary": brief.summary,
        "difficulty_analysis": brief.difficulty_analysis,
        "progress_notes": brief.progress_notes,
        "suggested_difficulty": brief.suggested_difficulty,
    });
    let analysis_id =
        ai_repo::insert(&state.db, user_id, "problem", problem_id, &analysis_json).await?;

    set_agent_status(
        &state.db,
        task_id,
        &mut progress,
        "classifier",
        "completed",
        &format!("识别为 {} 算法", brief.algorithm_type),
    )
    .await;

    // ── Stage 1.5: 查现有模板 ──────────────────────────────────────
    let existing_templates: Vec<TemplateBrief> = match template_repo::list(
        &state.db,
        user_id,
        &ListTemplatesQuery {
            category: brief.algorithm_type_as_category(),
            ..Default::default()
        },
    )
    .await
    {
        Ok(templates) => templates
            .into_iter()
            .map(|t| TemplateBrief {
                id: t.id,
                title: t.title,
                summary: t.description.chars().take(100).collect(),
            })
            .collect(),
        Err(_) => vec![],
    };

    tracing::info!(
        "[task-{}] 找到 {} 个相关现有模板",
        task_id,
        existing_templates.len()
    );

    // ── Stage 2: 并行提取 ──────────────────────────────────────────
    let ac_codes: Vec<_> = ctx
        .submissions
        .iter()
        .filter(|s| s.verdict == "AC")
        .cloned()
        .collect();

    // Set up agent steps
    set_agent_status(
        &state.db,
        task_id,
        &mut progress,
        "template",
        "running",
        "正在提取代码模板...",
    )
    .await;
    set_agent_status(
        &state.db,
        task_id,
        &mut progress,
        "error",
        "running",
        "正在分析错误模式...",
    )
    .await;
    set_agent_status(
        &state.db,
        task_id,
        &mut progress,
        "knowledge",
        "running",
        "正在提取知识点...",
    )
    .await;

    if !existing_templates.is_empty() {
        set_agent_step(
            &state.db,
            task_id,
            &mut progress,
            "template",
            "匹配现有模板",
            "running",
            "",
        )
        .await;
    }
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "template",
        "分析 AC 代码",
        "running",
        "",
    )
    .await;
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "error",
        "分析非 AC 提交",
        "running",
        "",
    )
    .await;
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "knowledge",
        "提取知识点",
        "running",
        "",
    )
    .await;

    // Clone what the parallel tasks need
    let brief_clone = brief.clone();
    let ctx_clone = ctx.clone();

    // Pre-check cancel before launching parallel agents
    if check_cancelled(&state.db, task_id).await? {
        return Err(crate::error::AppError::BadRequest(format!(
            "任务 {} 已取消",
            task_id
        )));
    }

    let (template_res, error_res, knowledge_res) = tokio::join!(
        async {
            // Template agent: up to 2 retries on parse/network failure
            with_retry("模板提取", task_id, &state.db, 2, || async {
                TemplateAgent
                    .run(llm, &brief_clone, &ac_codes, &existing_templates)
                    .await
            })
            .await
        },
        async {
            with_retry("错误诊断", task_id, &state.db, 2, || async {
                ErrorAgent
                    .run(llm, &ctx_clone.submissions, &brief_clone)
                    .await
            })
            .await
        },
        async {
            with_retry("知识梳理", task_id, &state.db, 2, || async {
                KnowledgeAgent.run(llm, &ctx_clone, &brief_clone).await
            })
            .await
        },
    );

    // Process template results
    let template_output = match template_res {
        Ok(out) => {
            if !existing_templates.is_empty() {
                set_agent_step(
                    &state.db,
                    task_id,
                    &mut progress,
                    "template",
                    "匹配现有模板",
                    "completed",
                    &format!("匹配了 {} 个现有模板", out.matched_template_ids.len()),
                )
                .await;
            }
            set_agent_step(
                &state.db,
                task_id,
                &mut progress,
                "template",
                "分析 AC 代码",
                "completed",
                &format!("提取了 {} 个新模板", out.templates.len()),
            )
            .await;
            set_agent_status(
                &state.db,
                task_id,
                &mut progress,
                "template",
                "completed",
                &format!(
                    "提取了 {} 个模板 ({} 个复用现有)",
                    out.templates.len(),
                    out.matched_template_ids.len()
                ),
            )
            .await;
            out
        }
        Err(e) => {
            tracing::error!("[task-{}] 模板提取失败: {e}", task_id);
            set_agent_status(
                &state.db,
                task_id,
                &mut progress,
                "template",
                "failed",
                &format!("失败: {e}"),
            )
            .await;
            TemplateAgentOutput {
                templates: vec![],
                matched_template_ids: vec![],
            }
        }
    };

    // Process error results
    let errors = match error_res {
        Ok(e) => {
            set_agent_step(
                &state.db,
                task_id,
                &mut progress,
                "error",
                "分析非 AC 提交",
                "completed",
                &format!("发现 {} 个错误模式", e.len()),
            )
            .await;
            set_agent_status(
                &state.db,
                task_id,
                &mut progress,
                "error",
                "completed",
                &format!("发现 {} 个错误模式", e.len()),
            )
            .await;
            e
        }
        Err(e) => {
            tracing::error!("[task-{}] 错误分析失败: {e}", task_id);
            set_agent_status(
                &state.db,
                task_id,
                &mut progress,
                "error",
                "failed",
                &format!("失败: {e}"),
            )
            .await;
            vec![]
        }
    };

    // Process knowledge results
    let knowledge_points = match knowledge_res {
        Ok(k) => {
            set_agent_step(
                &state.db,
                task_id,
                &mut progress,
                "knowledge",
                "提取知识点",
                "completed",
                &format!("提取了 {} 个知识点", k.len()),
            )
            .await;
            set_agent_status(
                &state.db,
                task_id,
                &mut progress,
                "knowledge",
                "completed",
                &format!("提取了 {} 个知识点", k.len()),
            )
            .await;
            k
        }
        Err(e) => {
            tracing::error!("[task-{}] 知识点提取失败: {e}", task_id);
            set_agent_status(
                &state.db,
                task_id,
                &mut progress,
                "knowledge",
                "failed",
                &format!("失败: {e}"),
            )
            .await;
            vec![]
        }
    };

    // ── Stage 3: 保存 ──────────────────────────────────────────────
    set_agent_status(
        &state.db,
        task_id,
        &mut progress,
        "saver",
        "running",
        "正在保存结果...",
    )
    .await;
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "saver",
        "保存模板",
        "running",
        "",
    )
    .await;
    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "saver",
        "保存错误和知识点",
        "running",
        "",
    )
    .await;

    let mut saved = 0usize;

    // Save new templates
    for t in &template_output.templates {
        let category = if t.category.is_empty() {
            "other"
        } else {
            &t.category
        };
        let tc = if t.time_complexity.is_empty() {
            None
        } else {
            Some(t.time_complexity.as_str())
        };
        if template_repo::exists_by_identity(&state.db, user_id, category, "cpp", &t.title)
            .await
            .unwrap_or(false)
        {
            continue;
        }
        // Auto-generate summary from description (first 200 chars)
        let summary: String = t.description.chars().take(200).collect();
        if template_repo::insert(
            &state.db,
            user_id,
            &t.title,
            category,
            "cpp",
            &t.code,
            &t.description,
            &summary,
            tc,
            None,
            "ai_extracted",
            Some(problem_id),
            None,
            &[],
            &[problem_id],
        )
        .await
        .is_ok()
        {
            saved += 1;
        }
    }

    // Link matched existing templates to this problem
    for template_id in &template_output.matched_template_ids {
        let _ = template_repo::link_problem(&state.db, *template_id, problem_id).await;
    }

    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "saver",
        "保存模板",
        "completed",
        &format!(
            "保存了 {} 个新模板，关联了 {} 个现有模板",
            template_output.templates.len(),
            template_output.matched_template_ids.len()
        ),
    )
    .await;

    // Save errors and knowledge
    for e in &errors {
        if knowledge_exists_by_title(&state.db, user_id, problem_id, &e.title)
            .await
            .unwrap_or(true)
        {
            continue;
        }
        let content = format!(
            "**错误描述：** {}\n\n**修复建议：** {}",
            e.description, e.fix_suggestion
        );
        if knowledge_repo::insert(
            &state.db,
            user_id,
            Some(problem_id),
            "note",
            &e.title,
            &content,
            &[],
        )
        .await
        .is_ok()
        {
            saved += 1;
        }
    }
    for k in &knowledge_points {
        if knowledge_exists_by_title(&state.db, user_id, problem_id, &k.title)
            .await
            .unwrap_or(true)
        {
            continue;
        }
        if knowledge_repo::insert(
            &state.db,
            user_id,
            Some(problem_id),
            "technique",
            &k.title,
            &k.content,
            &[],
        )
        .await
        .is_ok()
        {
            saved += 1;
        }
    }

    set_agent_step(
        &state.db,
        task_id,
        &mut progress,
        "saver",
        "保存错误和知识点",
        "completed",
        &format!("保存了 {} 条", saved),
    )
    .await;
    set_agent_status(
        &state.db,
        task_id,
        &mut progress,
        "saver",
        "completed",
        &format!("共保存 {} 条知识条目", saved),
    )
    .await;

    // ── 完成 ───────────────────────────────────────────────────────
    let task_result = json!({
        "analysis_id": analysis_id,
        "algorithm_type": brief.algorithm_type,
        "sub_type": brief.sub_type,
        "tags": brief.tags,
        "summary": brief.summary,
        "extracted_templates": template_output.templates.len(),
        "matched_templates": template_output.matched_template_ids.len(),
        "extracted_errors": errors.len(),
        "extracted_knowledge": knowledge_points.len(),
        "saved": saved,
        "submissions_analyzed": ctx.submissions.len(),
    });
    task_repo::mark_completed(&state.db, task_id, &task_result).await?;
    tracing::info!("[task-{}] 全量分析完成: 保存 {} 条", task_id, saved);
    Ok(())
}

async fn knowledge_exists_by_title(
    db: &sea_orm::DatabaseConnection,
    user_id: i64,
    problem_id: i64,
    title: &str,
) -> AppResult<bool> {
    use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
    let count = crate::entity::knowledge::Entity::find()
        .filter(crate::entity::knowledge::Column::UserId.eq(user_id))
        .filter(crate::entity::knowledge::Column::ProblemId.eq(problem_id))
        .filter(crate::entity::knowledge::Column::Title.eq(title))
        .count(db)
        .await?;
    Ok(count > 0)
}

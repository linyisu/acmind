use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single progress step within a task (legacy format, kept for backward compat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStep {
    pub step: String,
    pub status: String,   // pending / running / completed / failed
    pub message: String,
}

/// A single step within an agent's progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub label: String,
    pub status: String,   // pending / running / completed / failed
    pub detail: String,
}

/// Agent-level progress tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProgress {
    pub id: String,       // classifier, template, error, knowledge, saver
    pub name: String,     // 题目分析官, 模板提取官, etc.
    pub icon: String,     // emoji
    pub status: String,   // pending / running / completed / failed
    pub message: String,
    pub steps: Vec<AgentStep>,
}

/// The new progress format: a list of agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub agents: Vec<AgentProgress>,
}

/// GET /api/v1/tasks response item.
#[derive(Debug, Serialize)]
pub struct TaskResp {
    pub id: i64,
    pub kind: String,
    pub status: String,
    pub target_type: String,
    pub target_id: i64,
    pub progress: TaskProgress,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl TaskResp {
    pub fn from_model(m: crate::entity::task::Model) -> Self {
        // Try new format first, fall back to legacy
        let progress: TaskProgress = serde_json::from_value(m.progress.clone())
            .unwrap_or_else(|_| {
                // Legacy format: convert flat steps to agents
                let legacy: Vec<ProgressStep> = serde_json::from_value(m.progress)
                    .unwrap_or_default();
                TaskProgress {
                    agents: legacy.into_iter().map(|s| AgentProgress {
                        id: s.step,
                        name: s.message.clone(),
                        icon: String::new(),
                        status: s.status,
                        message: s.message,
                        steps: vec![],
                    }).collect(),
                }
            });
        Self {
            id: m.id,
            kind: m.kind,
            status: m.status,
            target_type: m.target_type,
            target_id: m.target_id,
            progress,
            result: m.result,
            error: m.error,
            created_at: m.created_at.with_timezone(&Utc),
            started_at: m.started_at.map(|t| t.with_timezone(&Utc)),
            completed_at: m.completed_at.map(|t| t.with_timezone(&Utc)),
        }
    }
}

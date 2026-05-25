use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Custom serde module for JSON array fields stored as strings in DB.
/// When serializing (→ frontend), outputs a Vec<String>.
/// When deserializing (← frontend/DB), expects either a JSON string or a JSON array.
mod json_array {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(json_str: &str, s: S) -> Result<S::Ok, S::Error> {
        let arr: Vec<String> = serde_json::from_str(json_str).unwrap_or_default();
        arr.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        // Accept both a JSON string (from DB) or a JSON array (from frontend)
        let value: serde_json::Value = serde_json::Value::deserialize(d)?;
        match value {
            serde_json::Value::String(s) => {
                // Verify it's valid JSON array
                let _: Vec<String> = serde_json::from_str(&s).unwrap_or_default();
                Ok(s)
            }
            serde_json::Value::Array(arr) => Ok(serde_json::to_string(&arr).unwrap_or_default()),
            _ => Ok("[]".into()),
        }
    }
}

// -- Problem --
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Problem {
    pub id: String,
    pub source: String,
    pub source_problem_id: String,
    pub title: String,
    pub url: Option<String>,
    pub difficulty: Option<i32>,
    #[serde(with = "json_array")]
    pub tags: String, // JSON array stored as string, serialized as Vec<String>
    pub statement_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProblemInput {
    pub source: String,
    pub source_problem_id: String,
    pub title: String,
    pub url: Option<String>,
    pub difficulty: Option<i32>,
    pub tags: Vec<String>,
    pub statement: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProblemInput {
    pub source: Option<String>,
    pub source_problem_id: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub difficulty: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub statement: Option<String>,
}

// -- Submission --
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Submission {
    pub id: String,
    pub problem_id: String,
    pub status: String,
    pub language: String,
    pub code_path: String,
    pub runtime: Option<i32>,
    pub memory: Option<i32>,
    pub note: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmissionInput {
    pub problem_id: String,
    pub status: String,
    pub language: String,
    pub code_text: String,
    pub runtime: Option<i32>,
    pub memory: Option<i32>,
    pub note: Option<String>,
}

// -- Solution Note --
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SolutionNote {
    pub id: String,
    pub problem_id: String,
    pub note_type: String,
    pub content: String,
    pub source_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteInput {
    pub problem_id: String,
    pub note_type: String,
    pub content: String,
    pub source_url: Option<String>,
}

// -- Error Analysis --
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ErrorAnalysis {
    pub id: String,
    pub problem_id: String,
    pub submission_id: String,
    pub error_type: String,
    pub root_cause: String,
    pub fix_summary: String,
    #[serde(with = "json_array")]
    pub related_knowledge: String, // JSON array, serialized as Vec<String>
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateErrorInput {
    pub problem_id: String,
    pub submission_id: String,
    pub error_type: String,
    pub root_cause: String,
    pub fix_summary: String,
    pub related_knowledge: Vec<String>,
}

// -- Knowledge Point --
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KnowledgePoint {
    pub id: String,
    pub name: String,
    pub category: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateKnowledgeInput {
    pub name: String,
    pub category: String,
    pub parent_id: Option<String>,
}

// -- Problem-Knowledge Map --
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProblemKnowledgeMap {
    pub problem_id: String,
    pub knowledge_point_id: String,
    pub confidence: f64,
}

// -- Report --
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Report {
    pub id: String,
    pub report_type: String,
    pub title: String,
    pub content: String,
    pub start_date: String,
    pub end_date: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateReportInput {
    pub report_type: String,
    pub title: String,
    pub start_date: String,
    pub end_date: String,
}

// Helper: generate a new UUID v4 as a string
pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

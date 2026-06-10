use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemplateCategory {
    DataStructure,
    Graph,
    Dp,
    String,
    Math,
    Geometry,
    Greedy,
    Search,
    Sort,
    BinarySearch,
    Other,
}

impl TemplateCategory {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::DataStructure => "data_structure",
            Self::Graph => "graph",
            Self::Dp => "dp",
            Self::String => "string",
            Self::Math => "math",
            Self::Geometry => "geometry",
            Self::Greedy => "greedy",
            Self::Search => "search",
            Self::Sort => "sort",
            Self::BinarySearch => "binary_search",
            Self::Other => "other",
        }
    }

    pub fn label_zh(&self) -> &'static str {
        match self {
            Self::DataStructure => "数据结构",
            Self::Graph => "图论",
            Self::Dp => "动态规划",
            Self::String => "字符串",
            Self::Math => "数学",
            Self::Geometry => "计算几何",
            Self::Greedy => "贪心",
            Self::Search => "搜索",
            Self::Sort => "排序",
            Self::BinarySearch => "二分",
            Self::Other => "其他",
        }
    }
}

// ── Request DTOs ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTemplateReq {
    pub title: String,
    pub category: TemplateCategory,
    pub language: String,
    pub code: String,
    pub description: String,
    pub summary: Option<String>,
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub difficulty: Option<i32>,
    pub tag_ids: Vec<i64>,
    pub problem_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateReq {
    pub title: Option<String>,
    pub category: Option<TemplateCategory>,
    pub language: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub difficulty: Option<i32>,
    pub tag_ids: Option<Vec<i64>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ListTemplatesQuery {
    pub category: Option<TemplateCategory>,
    pub language: Option<String>,
    pub tag_id: Option<i64>,
    pub problem_id: Option<i64>,
    pub search: Option<String>,
    pub sort: Option<String>,
}

// ── Response DTOs ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct TemplateResp {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub category: String,
    pub language: String,
    pub code: String,
    pub description: String,
    pub summary: String,
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub source: String,
    pub source_problem_id: Option<i64>,
    pub difficulty: Option<i32>,
    pub usage_count: i32,
    pub tag_ids: Vec<i64>,
    pub problem_ids: Vec<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TemplateStats {
    pub total: i64,
    pub by_category: Vec<CategoryCount>,
    pub by_language: Vec<LanguageCount>,
}

#[derive(Debug, Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct LanguageCount {
    pub language: String,
    pub count: i64,
}

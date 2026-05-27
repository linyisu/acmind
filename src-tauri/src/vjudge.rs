use crate::db::models::{CreateProblemInput, CreateSubmissionInput};
use crate::db::repo;
use crate::error::AppError;
use crate::storage::Storage;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::time::Duration;

const VJUDGE_STATUS_DATA_URL: &str = "https://vjudge.net/status/data";
const VJUDGE_SOLUTION_DATA_URL: &str = "https://vjudge.net/solution/data";
const VJUDGE_PROBLEM_URL_PREFIX: &str = "https://vjudge.net/problem";
const VJUDGE_DESCRIPTION_URL_PREFIX: &str = "https://vjudge.net/problem/description";
const VJUDGE_PAGE_SIZE: usize = 20;
const VJUDGE_MAX_PAGES: usize = 200;
const VJUDGE_SOURCE_DELAY_MS: u64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VJudgeSyncSummary {
    pub username: String,
    pub fetched: usize,
    pub imported: usize,
    pub skipped: usize,
    pub created_problems: usize,
    pub source_synced: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VJudgeStatusResponse {
    data: Vec<VJudgeSubmissionItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VJudgeSubmissionItem {
    run_id: i64,
    time: i64,
    oj: String,
    prob_num: String,
    status: String,
    language: String,
    runtime: Option<i32>,
    memory: Option<i32>,
}

pub async fn sync_public_submissions(
    pool: &SqlitePool,
    storage: &Storage,
    username: &str,
) -> Result<VJudgeSyncSummary, AppError> {
    let username = username.trim();
    if username.is_empty() {
        return Err(AppError::InvalidInput("请填写 VJudge 用户名。".into()));
    }

    let client = reqwest::Client::builder()
        .user_agent("ACMind/0.1")
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| AppError::Other(format!("创建 VJudge 客户端失败：{}", err)))?;
    let cookie = repo::get_setting(pool, "vjudge_cookie").await?;
    let cookie = cookie
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let mut summary = VJudgeSyncSummary {
        username: username.to_string(),
        fetched: 0,
        imported: 0,
        skipped: 0,
        created_problems: 0,
        source_synced: 0,
    };

    for page in 0..VJUDGE_MAX_PAGES {
        let start = page * VJUDGE_PAGE_SIZE;
        let items = fetch_status_page(&client, username, start, cookie).await?;
        if items.is_empty() {
            break;
        }

        summary.fetched += items.len();
        for item in items {
            let imported = import_submission(pool, storage, &client, cookie, &item).await?;
            if imported.created_problem {
                summary.created_problems += 1;
            }
            if imported.created_submission {
                summary.imported += 1;
            } else {
                summary.skipped += 1;
            }
            if imported.source_synced {
                summary.source_synced += 1;
            }
        }
    }

    repo::set_setting(pool, "vjudge_username", username).await?;
    Ok(summary)
}

async fn fetch_status_page(
    client: &reqwest::Client,
    username: &str,
    start: usize,
    cookie: Option<&str>,
) -> Result<Vec<VJudgeSubmissionItem>, AppError> {
    fetch_status_page_filtered(client, username, start, "All", "", cookie).await
}

async fn fetch_status_page_filtered(
    client: &reqwest::Client,
    username: &str,
    start: usize,
    oj: &str,
    prob_num: &str,
    cookie: Option<&str>,
) -> Result<Vec<VJudgeSubmissionItem>, AppError> {
    let mut last_error = None;
    for _ in 0..3 {
        let mut request = client
            .get(VJUDGE_STATUS_DATA_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", "https://vjudge.net/status");

        if let Some(cookie) = cookie {
            request = request.header("Cookie", cookie);
        }

        let result = request
            .query(&[
                ("draw", "1"),
                ("start", &start.to_string()),
                ("length", &VJUDGE_PAGE_SIZE.to_string()),
                ("un", username),
                ("OJId", oj),
                ("probNum", prob_num),
                ("res", "all"),
                ("language", ""),
                ("onlyFollowee", "false"),
                ("orderBy", "run_id"),
            ])
            .send()
            .await;

        match result {
            Ok(response) => {
                if !response.status().is_success() {
                    return Err(AppError::Other(format!(
                        "VJudge 请求失败：{}",
                        response.status()
                    )));
                }
                let payload = response
                    .json::<VJudgeStatusResponse>()
                    .await
                    .map_err(|err| AppError::Other(format!("解析 VJudge 提交记录失败：{}", err)))?;
                return Ok(payload.data);
            }
            Err(err) => {
                last_error = Some(err.to_string());
            }
        }
    }

    Err(AppError::Other(format!(
        "请求 VJudge 提交记录失败：{}",
        last_error.unwrap_or_else(|| "未知错误".into())
    )))
}

struct ImportResult {
    created_problem: bool,
    created_submission: bool,
    source_synced: bool,
}

async fn import_problem_submissions(
    pool: &SqlitePool,
    storage: &Storage,
    client: &reqwest::Client,
    cookie: Option<&str>,
    username: &str,
    oj: &str,
    prob_num: &str,
) -> Result<(Option<crate::db::models::Submission>, bool), AppError> {
    let mut latest_submission = None;
    let mut source_synced = false;
    let items = fetch_status_page_filtered(client, username, 0, oj, prob_num, cookie).await?;
    for item in items {
        let result = import_submission(pool, storage, client, cookie, &item).await?;
        if result.source_synced {
            source_synced = true;
        }
        let external_run_id = format!("vjudge:{}", item.run_id);
        if let Some(submission) =
            repo::submission_by_external_run_id(pool, &external_run_id).await?
        {
            latest_submission.get_or_insert(submission);
        }
    }
    Ok((latest_submission, source_synced))
}

async fn import_submission(
    pool: &SqlitePool,
    storage: &Storage,
    client: &reqwest::Client,
    cookie: Option<&str>,
    item: &VJudgeSubmissionItem,
) -> Result<ImportResult, AppError> {
    let external_run_id = format!("vjudge:{}", item.run_id);
    if let Some(existing) = repo::submission_by_external_run_id(pool, &external_run_id).await? {
        let source_synced = if existing.code_path.is_empty() && cookie.is_some() {
            tokio::time::sleep(Duration::from_millis(VJUDGE_SOURCE_DELAY_MS)).await;
            match fetch_source_code(client, cookie, item.run_id).await {
                Ok(source) => match storage.save_submission(
                    &existing.problem_id,
                    &existing.id,
                    normalize_status(&item.status),
                    &item.language,
                    &source,
                ) {
                    Ok(path) if !path.is_empty() => {
                        repo::set_submission_code_path(pool, &existing.id, &path).await?;
                        true
                    }
                    _ => false,
                },
                Err(err) => {
                    tracing::warn!(target: "app_lib::vjudge", "抓取 VJudge 源码失败 run #{}: {}", item.run_id, err);
                    false
                }
            }
        } else {
            false
        };

        return Ok(ImportResult {
            created_problem: false,
            created_submission: false,
            source_synced,
        });
    }

    let source_problem_id = format!("{}-{}", item.oj, item.prob_num);
    let existing_problem =
        repo::find_problem_by_source_id(pool, "VJudge", &source_problem_id).await?;
    let (problem_id, created_problem) = match existing_problem {
        Some(problem) => (problem.id, false),
        None => {
            let problem = repo::create_problem(
                pool,
                &CreateProblemInput {
                    source: "VJudge".into(),
                    source_problem_id: source_problem_id.clone(),
                    title: source_problem_id.clone(),
                    url: Some(format!("https://vjudge.net/problem/{}", source_problem_id)),
                    difficulty: None,
                    tags: vec![item.oj.clone()],
                    statement: None,
                },
                None,
            )
            .await?;
            (problem.id, true)
        }
    };

    // Try to fetch source code if cookie is present.
    let (code_text, code_path, source_synced) = match cookie {
        Some(_) => {
            tokio::time::sleep(Duration::from_millis(VJUDGE_SOURCE_DELAY_MS)).await;
            match fetch_source_code(client, cookie, item.run_id).await {
                Ok(source) => {
                    let path = storage
                        .save_submission(
                            &problem_id,
                            &external_run_id,
                            normalize_status(&item.status),
                            &item.language,
                            &source,
                        )
                        .unwrap_or_default();
                    (source, path, true)
                }
                Err(err) => {
                    tracing::warn!(target: "app_lib::vjudge", "抓取 VJudge 源码失败 run #{}: {}", item.run_id, err);
                    (String::new(), String::new(), false)
                }
            }
        }
        None => (String::new(), String::new(), false),
    };

    let sub = repo::create_submission(
        pool,
        &CreateSubmissionInput {
            problem_id: problem_id.clone(),
            status: normalize_status(&item.status).into(),
            language: item.language.clone(),
            code_text,
            runtime: item.runtime,
            memory: item.memory,
            note: Some(format!(
                "VJudge run #{}，原始状态：{}",
                item.run_id, item.status
            )),
            external_run_id: Some(external_run_id),
            submitted_at: Some(timestamp_millis(item.time)?),
        },
        &code_path,
    )
    .await?;

    // If the code_path was written after the sub was created, patch it.
    if source_synced && !code_path.is_empty() && code_path != sub.code_path {
        let _ = repo::set_submission_code_path(pool, &sub.id, &code_path).await;
    }

    Ok(ImportResult {
        created_problem,
        created_submission: true,
        source_synced,
    })
}

/// Fetch the source code for a single VJudge submission.
async fn fetch_source_code(
    client: &reqwest::Client,
    cookie: Option<&str>,
    run_id: i64,
) -> Result<String, AppError> {
    let url = format!("{}/{}?inPage=true", VJUDGE_SOLUTION_DATA_URL, run_id);
    let mut request = client
        .post(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:150.0) Gecko/20100101 Firefox/150.0",
        )
        .header("Accept", "*/*")
        .header("Accept-Language", "zh,en-US;q=0.9,en;q=0.8")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("Origin", "https://vjudge.net")
        .header("Sec-GPC", "1")
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Referer", format!("https://vjudge.net/solution/{}", run_id))
        // Match the browser request exactly: the form value is empty.
        .body("shareCode=");

    if let Some(cookie) = cookie {
        request = request.header("Cookie", cookie);
    }

    let response = request
        .send()
        .await
        .map_err(|err| AppError::Other(format!("请求源码失败：{}", err)))?;

    if !response.status().is_success() {
        return Err(AppError::Other(format!(
            "源码接口返回 {}",
            response.status()
        )));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|err| AppError::Other(format!("解析源码 JSON 失败：{}", err)))?;

    match body.get("code").and_then(|v| v.as_str()) {
        Some(code) if !code.is_empty() => Ok(code.to_string()),
        _ => {
            let reason = body
                .get("codeAccessInfo")
                .and_then(|v| v.get("i18nKey"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            Err(AppError::Other(format!("未获取到源码：{}", reason)))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VJudgeProblemImportResult {
    pub problem: crate::db::models::Problem,
    pub submission: Option<crate::db::models::Submission>,
    pub statement_imported: bool,
    pub source_synced: bool,
}

#[derive(Debug)]
struct VJudgeProblemRef {
    oj: String,
    prob_num: String,
    run_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VJudgeProblemPageData {
    oj: String,
    prob: String,
    #[serde(default)]
    desc_briefs: Vec<VJudgeDescBrief>,
}

#[derive(Debug, Deserialize)]
struct VJudgeDescBrief {
    key: i64,
    version: i64,
}

#[derive(Debug, Deserialize)]
struct VJudgeDescriptionData {
    sections: Vec<VJudgeDescriptionSection>,
}

#[derive(Debug, Deserialize)]
struct VJudgeDescriptionSection {
    title: String,
    value: VJudgeDescriptionValue,
}

#[derive(Debug, Deserialize)]
struct VJudgeDescriptionValue {
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VJudgeSolutionData {
    code: Option<String>,
    run_id: i64,
    oj: String,
    prob_num: String,
    language: String,
    status: String,
    runtime: Option<i32>,
    memory: Option<i32>,
    submit_time: Option<i64>,
}

pub async fn import_problem_from_url(
    pool: &SqlitePool,
    storage: &Storage,
    input: &str,
) -> Result<VJudgeProblemImportResult, AppError> {
    let problem_ref = parse_vjudge_problem_ref(input)?;
    let client = vjudge_client()?;
    let cookie = repo::get_setting(pool, "vjudge_cookie").await?;
    let cookie = cookie
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let solution = match problem_ref.run_id {
        Some(run_id) => Some(fetch_solution_data(&client, cookie, run_id).await?),
        None => None,
    };

    let oj = solution
        .as_ref()
        .map(|item| item.oj.clone())
        .unwrap_or(problem_ref.oj);
    let prob_num = solution
        .as_ref()
        .map(|item| item.prob_num.clone())
        .unwrap_or(problem_ref.prob_num);
    let source_problem_id = format!("{}-{}", oj, prob_num);
    let page = fetch_problem_page(&client, cookie, &source_problem_id).await?;
    let title =
        extract_title(&page, &source_problem_id).unwrap_or_else(|| source_problem_id.clone());

    let page_data = extract_problem_page_data(&page)?;
    let oj = if page_data.oj.is_empty() {
        oj
    } else {
        page_data.oj.clone()
    };
    let prob_num = if page_data.prob.is_empty() {
        prob_num
    } else {
        page_data.prob.clone()
    };
    let source_problem_id = format!("{}-{}", oj, prob_num);
    let statement = if let Some(desc) = page_data.desc_briefs.first() {
        let desc_page = match fetch_description_page(
            &client,
            cookie,
            desc.key,
            desc.version,
            &source_problem_id,
        )
        .await
        {
            Ok(page) => page,
            Err(first_error) => {
                if let Some(path) = extract_description_path(&page) {
                    fetch_text(
                        &client,
                        cookie,
                        &format!("https://vjudge.net{}", path),
                        &format!("{}/{}", VJUDGE_PROBLEM_URL_PREFIX, source_problem_id),
                    )
                    .await
                    .map_err(|_| first_error)?
                } else {
                    return Err(first_error);
                }
            }
        };
        extract_statement_markdown(&desc_page)?
    } else if oj == "洛谷" {
        fetch_luogu_statement(&client, &prob_num).await?
    } else {
        return Err(AppError::Other(
            "VJudge 未提供该题题面描述，且当前只支持洛谷题面兜底。".into(),
        ));
    };

    let mut problem =
        match repo::find_problem_by_source_id(pool, "VJudge", &source_problem_id).await? {
            Some(existing) => existing,
            None => {
                repo::create_problem(
                    pool,
                    &CreateProblemInput {
                        source: "VJudge".into(),
                        source_problem_id: source_problem_id.clone(),
                        title: title.clone(),
                        url: Some(format!(
                            "{}/{}",
                            VJUDGE_PROBLEM_URL_PREFIX, source_problem_id
                        )),
                        difficulty: None,
                        tags: vec![oj.clone()],
                        statement: None,
                    },
                    None,
                )
                .await?
            }
        };

    let statement_path = storage.save_statement(&problem.id, &statement)?;
    problem = repo::update_problem(
        pool,
        &problem.id,
        &crate::db::models::UpdateProblemInput {
            source: None,
            source_problem_id: None,
            title: Some(title),
            url: Some(format!(
                "{}/{}",
                VJUDGE_PROBLEM_URL_PREFIX, source_problem_id
            )),
            difficulty: None,
            tags: Some(vec![oj.clone()]),
            statement: None,
        },
        Some(&statement_path),
    )
    .await?;

    let mut submission = None;
    let mut source_synced = false;
    if let Some(solution) = solution {
        let external_run_id = format!("vjudge:{}", solution.run_id);
        let code = solution.code.unwrap_or_default();
        let status = normalize_status(&solution.status);
        let submitted_at = match solution.submit_time {
            Some(value) => Some(timestamp_millis(value)?),
            None => None,
        };

        if let Some(existing) = repo::submission_by_external_run_id(pool, &external_run_id).await? {
            if !code.is_empty() && existing.code_path.is_empty() {
                let code_path = storage.save_submission(
                    &problem.id,
                    &existing.id,
                    status,
                    &solution.language,
                    &code,
                )?;
                repo::set_submission_code_path(pool, &existing.id, &code_path).await?;
                source_synced = true;
            }
            submission = Some(repo::get_submission(pool, &existing.id).await?);
        } else {
            let code_path = if code.is_empty() {
                String::new()
            } else {
                source_synced = true;
                storage.save_submission(
                    &problem.id,
                    &external_run_id,
                    status,
                    &solution.language,
                    &code,
                )?
            };
            submission = Some(
                repo::create_submission(
                    pool,
                    &CreateSubmissionInput {
                        problem_id: problem.id.clone(),
                        status: status.into(),
                        language: solution.language,
                        code_text: code,
                        runtime: solution.runtime,
                        memory: solution.memory,
                        note: Some(format!("VJudge run #{}，单题导入", solution.run_id)),
                        external_run_id: Some(external_run_id),
                        submitted_at,
                    },
                    &code_path,
                )
                .await?,
            );
        }
    }

    if let Some(username) = repo::get_setting(pool, "vjudge_username").await? {
        let username = username.trim().to_string();
        if !username.is_empty() {
            match import_problem_submissions(
                pool, storage, &client, cookie, &username, &oj, &prob_num,
            )
            .await
            {
                Ok((latest, synced)) => {
                    if submission.is_none() {
                        submission = latest;
                    }
                    source_synced = source_synced || synced;
                }
                Err(err) => {
                    tracing::warn!(target: "app_lib::vjudge", "导入该题提交记录失败: {}", err);
                }
            }
        }
    }

    Ok(VJudgeProblemImportResult {
        problem,
        submission,
        statement_imported: true,
        source_synced,
    })
}

fn vjudge_client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:150.0) Gecko/20100101 Firefox/150.0")
        .timeout(Duration::from_secs(20))
        .http1_only()
        .build()
        .map_err(|err| AppError::Other(format!("创建 VJudge 客户端失败：{}", err)))
}

fn parse_vjudge_problem_ref(input: &str) -> Result<VJudgeProblemRef, AppError> {
    let value = input.trim();
    if value.is_empty() {
        return Err(AppError::InvalidInput(
            "请填写 VJudge 题目或提交链接。".into(),
        ));
    }

    if let Some(index) = value.find("/solution/") {
        let run_part = &value[index + "/solution/".len()..];
        let run_id = run_part
            .split(|ch: char| !ch.is_ascii_digit())
            .next()
            .and_then(|part| part.parse::<i64>().ok())
            .ok_or_else(|| AppError::InvalidInput("无法解析 VJudge 提交编号。".into()))?;
        return Ok(VJudgeProblemRef {
            oj: String::new(),
            prob_num: String::new(),
            run_id: Some(run_id),
        });
    }

    let source_id = if let Some(index) = value.find("/problem/") {
        &value[index + "/problem/".len()..]
    } else {
        value
    }
    .split(['?', '#'])
    .next()
    .unwrap_or(value)
    .trim_matches('/');

    let source_id = percent_decode_utf8(source_id)?;
    let (oj, prob_num) = source_id
        .split_once('-')
        .ok_or_else(|| AppError::InvalidInput("题目格式应为 OJ-题号，例如 洛谷-P1540。".into()))?;

    Ok(VJudgeProblemRef {
        oj: oj.to_string(),
        prob_num: prob_num.to_string(),
        run_id: None,
    })
}

fn percent_decode_utf8(input: &str) -> Result<String, AppError> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                .map_err(|_| AppError::InvalidInput("VJudge 题目链接编码无效。".into()))?;
            let value = u8::from_str_radix(hex, 16)
                .map_err(|_| AppError::InvalidInput("VJudge 题目链接编码无效。".into()))?;
            output.push(value);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output)
        .map_err(|_| AppError::InvalidInput("VJudge 题目链接不是有效 UTF-8。".into()))
}

async fn fetch_problem_page(
    client: &reqwest::Client,
    cookie: Option<&str>,
    source_problem_id: &str,
) -> Result<String, AppError> {
    fetch_text(
        client,
        cookie,
        &format!("{}/{}", VJUDGE_PROBLEM_URL_PREFIX, source_problem_id),
        "https://vjudge.net/",
    )
    .await
}

async fn fetch_description_page(
    client: &reqwest::Client,
    cookie: Option<&str>,
    key: i64,
    version: i64,
    source_problem_id: &str,
) -> Result<String, AppError> {
    fetch_text(
        client,
        cookie,
        &format!("{}/{}?{}", VJUDGE_DESCRIPTION_URL_PREFIX, key, version),
        &format!("{}/{}", VJUDGE_PROBLEM_URL_PREFIX, source_problem_id),
    )
    .await
}

async fn fetch_luogu_statement(
    _client: &reqwest::Client,
    prob_num: &str,
) -> Result<String, AppError> {
    let url = format!("https://www.luogu.com.cn/problem/{}", prob_num);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:150.0) Gecko/20100101 Firefox/150.0")
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| AppError::Other(format!("创建洛谷客户端失败：{}", err)))?;
    let first = fetch_luogu_once(&client, &url, None).await?;
    let html = if first.status().is_redirection() {
        let cookie = first
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(str::to_string);
        fetch_luogu_once(&client, &url, cookie.as_deref())
            .await?
            .text()
            .await
            .map_err(|err| AppError::Other(format!("读取洛谷题面失败：{}", err)))?
    } else {
        first
            .text()
            .await
            .map_err(|err| AppError::Other(format!("读取洛谷题面失败：{}", err)))?
    };
    extract_luogu_statement_markdown(&html)
}

async fn fetch_luogu_once(
    client: &reqwest::Client,
    url: &str,
    cookie: Option<&str>,
) -> Result<reqwest::Response, AppError> {
    let mut request = client
        .get(url)
        .header("Accept", "text/html,*/*")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("x-luogu-type", "content-only");
    if let Some(cookie) = cookie {
        request = request.header("Cookie", cookie);
    }
    request
        .send()
        .await
        .map_err(|err| AppError::Other(format!("请求洛谷题面失败：{}", err)))
}

async fn fetch_solution_data(
    client: &reqwest::Client,
    cookie: Option<&str>,
    run_id: i64,
) -> Result<VJudgeSolutionData, AppError> {
    let url = format!("{}/{}?inPage=true", VJUDGE_SOLUTION_DATA_URL, run_id);
    let mut request = client
        .post(&url)
        .header("Accept", "*/*")
        .header("Accept-Language", "zh,en-US;q=0.9,en;q=0.8")
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("Origin", "https://vjudge.net")
        .header("Sec-GPC", "1")
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .header("Referer", format!("https://vjudge.net/solution/{}", run_id))
        .body("shareCode=");
    if let Some(cookie) = cookie {
        request = request.header("Cookie", cookie);
    }
    let response = request
        .send()
        .await
        .map_err(|err| AppError::Other(format!("请求 VJudge 提交失败：{}", err)))?;
    response
        .json::<VJudgeSolutionData>()
        .await
        .map_err(|err| AppError::Other(format!("解析 VJudge 提交失败：{}", err)))
}

async fn fetch_text(
    client: &reqwest::Client,
    cookie: Option<&str>,
    url: &str,
    referer: &str,
) -> Result<String, AppError> {
    let mut last_error = None;
    for _ in 0..3 {
        let mut request = client
            .get(url)
            .header("Accept", "text/html,application/json,*/*")
            .header("Accept-Language", "zh,en-US;q=0.9,en;q=0.8")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", referer);
        if let Some(cookie) = cookie {
            request = request.header("Cookie", cookie);
        }
        match request.send().await {
            Ok(response) if response.status().is_success() => {
                return response
                    .text()
                    .await
                    .map_err(|err| AppError::Other(format!("读取 VJudge 响应失败：{}", err)));
            }
            Ok(response) => last_error = Some(format!("HTTP {}", response.status())),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(Duration::from_millis(700)).await;
    }
    Err(AppError::Other(format!(
        "请求 VJudge 页面失败：{}",
        last_error.unwrap_or_else(|| "未知错误".into())
    )))
}

const VJUDGE_CDN_ORIGIN: &str = "https://cdn.vjudge.net.cn";

fn extract_problem_page_data(html: &str) -> Result<VJudgeProblemPageData, AppError> {
    let json = extract_textarea(html, "name=\"dataJson\"")
        .or_else(|| extract_textarea(html, "name='dataJson'"))
        .ok_or_else(|| AppError::Other("未找到 VJudge 题目信息。".into()))?;
    serde_json::from_str(&decode_html_entities(&json))
        .map_err(|err| AppError::Other(format!("解析 VJudge 题目信息失败：{}", err)))
}

fn extract_statement_markdown(html: &str) -> Result<String, AppError> {
    let json = extract_textarea(html, "data-json-container")
        .ok_or_else(|| AppError::Other("未找到 VJudge 题面内容。".into()))?;
    let data: VJudgeDescriptionData = serde_json::from_str(&decode_html_entities(&json))
        .map_err(|err| AppError::Other(format!("解析 VJudge 题面失败：{}", err)))?;
    let mut markdown = String::new();
    for section in data.sections {
        let content = html_fragment_to_markdown(&section.value.content);
        if content.trim().is_empty() {
            continue;
        }
        if !section.title.trim().is_empty() {
            markdown.push_str(&format!("## {}\n\n", section.title.trim()));
        }
        markdown.push_str(content.trim());
        markdown.push_str("\n\n");
    }
    Ok(markdown.trim().to_string())
}

fn extract_luogu_statement_markdown(html: &str) -> Result<String, AppError> {
    let article = extract_between(html, "<article", "</article>")
        .ok_or_else(|| AppError::Other("未找到洛谷题面内容。".into()))?;
    let title = extract_between(article, "<h1>", "</h1>")
        .map(|value| format!("# {}\n\n", decode_html_entities(value.trim())))
        .unwrap_or_default();
    let mut markdown = title;
    for section in extract_luogu_sections(article) {
        markdown.push_str(&section);
        markdown.push_str("\n\n");
    }
    let markdown = normalize_markdown(&markdown);
    if markdown.trim().is_empty() {
        return Err(AppError::Other("洛谷题面内容为空。".into()));
    }
    Ok(markdown)
}

fn extract_luogu_sections(article: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut rest = article;
    while let Some(start) = rest.find("<section>") {
        let after_start = &rest[start + "<section>".len()..];
        let Some(end) = after_start.find("</section>") else {
            break;
        };
        let section = &after_start[..end];
        let title = extract_between(section, "<h2>", "</h2>")
            .unwrap_or("")
            .trim();
        let content = extract_between(section, "<div>", "</div>").unwrap_or("");
        let content = html_fragment_to_markdown(content);
        if !title.is_empty() || !content.trim().is_empty() {
            let mut item = String::new();
            if !title.is_empty() {
                item.push_str(&format!("## {}\n\n", decode_html_entities(title)));
            }
            item.push_str(content.trim());
            sections.push(item);
        }
        rest = &after_start[end + "</section>".len()..];
    }
    sections
}

fn extract_between<'a>(input: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_index = input.find(start)?;
    let rest = &input[start_index..];
    let end_index = rest.find(end)?;
    Some(&rest[..end_index])
}

fn extract_textarea(html: &str, marker: &str) -> Option<String> {
    let marker_index = html.find(marker)?;
    let before = html[..marker_index].rfind("<textarea")?;
    let after = html[marker_index..].find("</textarea>")? + marker_index;
    let open_end = html[before..].find('>')? + before + 1;
    Some(html[open_end..after].trim().to_string())
}

fn extract_description_path(html: &str) -> Option<String> {
    let index = html.find("/problem/description/")?;
    let rest = &html[index..];
    let end = rest
        .find(['"', '\'', '<', '>', ' ', '\n', '\r', '\t'])
        .unwrap_or(rest.len());
    Some(rest[..end].to_string())
}

fn extract_title(html: &str, source_problem_id: &str) -> Option<String> {
    let title = html.split("<title>").nth(1)?.split("</title>").next()?;
    let title = decode_html_entities(title);
    Some(
        title
            .split(&format!(" - {}", source_problem_id))
            .next()
            .unwrap_or(&title)
            .trim()
            .to_string(),
    )
}

fn html_fragment_to_markdown(input: &str) -> String {
    let mut output = decode_html_entities(input);
    output = replace_images(&output);
    output = replace_pre_blocks(&output);
    output = replace_sample_table(&output);
    output = output
        .replace("<h1>", "# ")
        .replace("</h1>", "\n\n")
        .replace("<h2>", "## ")
        .replace("</h2>", "\n\n")
        .replace("<h3>", "### ")
        .replace("</h3>", "\n\n")
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    for tag in ["p", "div", "tr", "section"] {
        output = output.replace(&format!("</{}>", tag), "\n\n");
    }
    output = output.replace("<li>", "- ").replace("</li>", "\n");
    output = strip_html_tags(&output);
    normalize_markdown(&output)
}

fn replace_images(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("<img") {
        output.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('>') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let tag = &rest[start..start + end + 1];
        if let Some(src) = extract_attr(tag, "src") {
            let alt = extract_attr(tag, "alt").unwrap_or_else(|| "image".into());
            output.push_str(&format!("\n![{}]({})\n", alt, normalize_asset_url(&src)));
        }
        rest = &rest[start + end + 1..];
    }
    output.push_str(rest);
    output
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let marker = format!("{}={}", name, quote);
        if let Some(start) = tag.find(&marker) {
            let value_start = start + marker.len();
            let value_end = tag[value_start..].find(quote)? + value_start;
            return Some(tag[value_start..value_end].to_string());
        }
    }
    None
}

fn normalize_asset_url(src: &str) -> String {
    if let Some(path) = src.strip_prefix("CDN_BASE_URL/") {
        format!("{}/{}", VJUDGE_CDN_ORIGIN, path)
    } else if src.starts_with("http://") || src.starts_with("https://") {
        src.to_string()
    } else if src.starts_with("//") {
        format!("https:{}", src)
    } else if src.starts_with('/') {
        format!("https://vjudge.net{}", src)
    } else {
        format!("https://vjudge.net/{}", src)
    }
}

fn replace_sample_table(input: &str) -> String {
    let mut pre_values = Vec::new();
    let mut rest = input;
    while let Some(start) = rest.find("<pre>") {
        if let Some(end) = rest[start + 5..].find("</pre>") {
            pre_values.push(rest[start + 5..start + 5 + end].to_string());
            rest = &rest[start + 5 + end + 6..];
        } else {
            break;
        }
    }
    if pre_values.len() >= 2 && input.contains("vjudge_sample") {
        format!(
            "### Sample Input\n\n```text\n{}\n```\n\n### Sample Output\n\n```text\n{}\n```",
            decode_html_entities(pre_values[0].trim()),
            decode_html_entities(pre_values[1].trim())
        )
    } else {
        input.to_string()
    }
}

fn replace_pre_blocks(input: &str) -> String {
    let mut output = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("<pre>") {
        output.push_str(&rest[..start]);
        if let Some(end) = rest[start + 5..].find("</pre>") {
            let code = &rest[start + 5..start + 5 + end];
            output.push_str("\n```text\n");
            output.push_str(&decode_html_entities(code));
            output.push_str("\n```\n");
            rest = &rest[start + 5 + end + 6..];
        } else {
            output.push_str(&rest[start..]);
            rest = "";
        }
    }
    output.push_str(rest);
    output
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn decode_html_entities(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn normalize_markdown(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
        .replace("\n\n\n", "\n\n")
        .trim()
        .to_string()
}

fn normalize_status(status: &str) -> &'static str {
    match status {
        "Accepted" => "AC",
        "Wrong answer" => "WA",
        "Time limit exceeded" | "Time Limit Exceeded" | "Time Limit Exceed" => "TLE",
        "Runtime error" | "Runtime Error" => "RE",
        "Memory limit exceeded" | "Memory Limit Exceeded" | "Memory Limit Exceed" => "MLE",
        "Compile error" | "Compilation error" | "Compile Error" => "CE",
        _ => "WA",
    }
}

pub fn timestamp_millis(value: i64) -> Result<DateTime<Utc>, AppError> {
    Utc.timestamp_millis_opt(value)
        .single()
        .ok_or_else(|| AppError::InvalidInput(format!("无效的 VJudge 提交时间：{}", value)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_vjudge_statuses() {
        assert_eq!(normalize_status("Accepted"), "AC");
        assert_eq!(normalize_status("Wrong answer"), "WA");
        assert_eq!(normalize_status("Time limit exceeded"), "TLE");
        assert_eq!(normalize_status("Runtime error"), "RE");
        assert_eq!(normalize_status("Memory limit exceeded"), "MLE");
        assert_eq!(normalize_status("Compile error"), "CE");
    }
}

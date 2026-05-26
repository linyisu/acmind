mod ai;
mod commands;
mod db;
mod error;
mod import_server;
mod storage;
mod vjudge;

use db::init_db;
use std::path::PathBuf;
use storage::Storage;
use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn setup_tracing() {
    let data_dir = if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(dir).join("acmind")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/share/acmind")
    } else {
        PathBuf::from("acmind-data")
    };
    std::fs::create_dir_all(&data_dir).ok();

    let log_path = data_dir.join("acmind.log");
    let file_appender = tracing_appender::rolling::never(&data_dir, "acmind.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // Leak the guard so the file writer lives for the entire app lifetime
    Box::leak(Box::new(_guard));

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,acmind=debug,reqwest=warn"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stderr).pretty())
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .try_init()
        .ok();

    tracing::info!("Log file: {}", log_path.display());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = get_app_data_dir(app);
            tracing::info!("ACMind starting, data dir: {:?}", app_data_dir);

            let pool = tauri::async_runtime::block_on(init_db(&app_data_dir))
                .expect("Failed to initialize database");
            tracing::info!("Database initialized");

            let storage = Storage::new(app_data_dir.clone());

            // Start the import server for browser extension communication
            let _import_handle = import_server::start_import_server(
                pool.clone(),
                storage.clone(),
                app.handle().clone(),
            );

            app.manage(pool);
            app.manage(storage);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_problems,
            commands::get_problem,
            commands::create_problem,
            commands::get_problem_statement,
            commands::update_problem,
            commands::delete_problem,
            commands::list_submissions_by_problem,
            commands::get_submission,
            commands::create_submission,
            commands::delete_submission,
            commands::list_notes_by_problem,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::list_error_analyses_by_problem,
            commands::create_error_analysis,
            commands::list_knowledge_points,
            commands::create_knowledge_point,
            commands::list_reports,
            commands::generate_report,
            commands::analyze_problem,
            commands::analyze_problem_streaming,
            commands::format_problem_statement,
            commands::get_dashboard_stats,
            commands::get_error_type_stats,
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
            commands::sync_vjudge_submissions,
            commands::import_vjudge_problem,
            commands::get_log_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_app_data_dir(app: &tauri::App) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

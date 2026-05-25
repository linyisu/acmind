mod ai;
mod commands;
mod db;
mod error;
mod storage;

use db::init_db;
use storage::Storage;
use tauri::Manager;
use std::path::PathBuf;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Tracing MUST be initialized before any Tauri plugin.
    // tauri-plugin-log will detect an existing global subscriber and skip its own init.
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,acmind=debug,reqwest=warn")),
        )
        .with(fmt::layer().with_writer(std::io::stderr).pretty())
        .with(
            fmt::layer()
                .with_writer(std::io::stderr) // file layer added later in setup
                .with_ansi(false),
        )
        .try_init()
        .ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_data_dir = get_app_data_dir(app);
            tracing::info!("ACMind starting, data dir: {:?}", app_data_dir);

            // Initialize the database
            let pool = tauri::async_runtime::block_on(init_db(&app_data_dir))
                .expect("Failed to initialize database");

            tracing::info!("Database initialized");

            app.manage(pool);

            // Initialize storage
            let storage = Storage::new(app_data_dir);
            app.manage(storage);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_problems,
            commands::get_problem,
            commands::create_problem,
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
            commands::get_dashboard_stats,
            commands::get_error_type_stats,
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
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

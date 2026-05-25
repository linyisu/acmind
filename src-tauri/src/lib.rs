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
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize tracing: console + file logging
            let app_data_dir = get_app_data_dir(app);
            let log_file = app_data_dir.join("acmind.log");

            let file_appender = tracing_appender::rolling::never(&app_data_dir, "acmind.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            let env_filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,acmind=debug,reqwest=warn"));

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().with_writer(std::io::stderr).pretty())
                .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
                .try_init()
                .ok(); // Ignore if already initialized (double-setup guard)

            tracing::info!("ACMind starting, log file: {:?}", log_file);

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize the database
            let app_data_dir = get_app_data_dir(app);
            let pool = tauri::async_runtime::block_on(init_db(&app_data_dir))
                .expect("Failed to initialize database");

            tracing::info!("Database initialized at {:?}", app_data_dir.join("acmind.sqlite"));

            app.manage(pool);

            // Initialize storage
            let storage = Storage::new(app_data_dir);
            app.manage(storage);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Problems
            commands::list_problems,
            commands::get_problem,
            commands::create_problem,
            commands::update_problem,
            commands::delete_problem,
            // Submissions
            commands::list_submissions_by_problem,
            commands::get_submission,
            commands::create_submission,
            commands::delete_submission,
            // Notes
            commands::list_notes_by_problem,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            // Error Analyses
            commands::list_error_analyses_by_problem,
            commands::create_error_analysis,
            // Knowledge Points
            commands::list_knowledge_points,
            commands::create_knowledge_point,
            // Reports
            commands::list_reports,
            commands::generate_report,
            // AI Analysis
            commands::analyze_problem,
            // Dashboard
            commands::get_dashboard_stats,
            commands::get_error_type_stats,
            // Settings
            commands::get_setting,
            commands::set_setting,
            commands::get_all_settings,
            // Diagnostics
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

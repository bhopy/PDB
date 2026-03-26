mod commands;
mod db;

use db::Database;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn get_app_dir() -> PathBuf {
    // Use the standard app data directory: %APPDATA%/pdb/ on Windows,
    // ~/Library/Application Support/pdb/ on macOS, ~/.local/share/pdb/ on Linux
    let base = dirs_next::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("pdb");
    fs::create_dir_all(&dir).ok();
    dir
}

fn get_db_path() -> PathBuf {
    get_app_dir().join("pdb.db")
}

fn get_attachments_dir() -> PathBuf {
    let dir = get_app_dir().join("attachments");
    // Ensure attachments directory exists
    fs::create_dir_all(&dir).ok();
    dir
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = get_db_path();
    println!("Database path: {:?}", db_path);

    let database = Database::new(&db_path).expect("Failed to initialize database");
    let attachments_dir = get_attachments_dir();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(database)
        .manage(attachments_dir)
        .invoke_handler(tauri::generate_handler![
            commands::get_tree,
            commands::get_article,
            commands::get_article_by_slug,
            commands::search,
            commands::create_article,
            commands::update_article,
            commands::delete_article,
            commands::move_article,
            commands::create_category,
            commands::update_category,
            commands::move_category,
            commands::delete_category,
            commands::get_all_tags,
            commands::get_articles_by_tag,
            commands::add_attachment,
            commands::get_attachments,
            commands::delete_attachment,
            commands::reorder_article,
            commands::reorder_category,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

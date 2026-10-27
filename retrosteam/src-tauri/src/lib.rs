mod commands;
mod db;
mod error;
mod paths;
mod settings;
mod state;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let st = state::AppState::init(app.handle())?;
            app.manage(st);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_settings,
            commands::save_settings,
            commands::set_data_root
        ])
        .run(tauri::generate_context!())
        .expect("error while running RetrosTeam");
}

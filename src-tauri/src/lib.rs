mod catalog;
mod commands;
mod db;
mod error;
mod installer;
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
            commands::set_data_root,
            commands::list_components,
            commands::install_component,
            commands::cancel_install,
            commands::uninstall_component
        ])
        .run(tauri::generate_context!())
        .expect("error while running RetrosTeam");
}

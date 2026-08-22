mod ai;
mod commands;
mod core;
mod domain;
mod vault;

use core::MnemeCore;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let vault = app.path().app_data_dir()?.join("vault");
            let config = app.path().app_config_dir()?;
            app.manage(MnemeCore::new(vault, config).map_err(std::io::Error::other)?);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_config,
            commands::list_sources,
            commands::get_source,
            commands::add_source,
            commands::wikify_source,
            commands::list_entities,
            commands::get_entity,
            commands::search_entities,
            commands::research_query,
            commands::get_graph,
            commands::lint_vault,
            commands::sync_vault,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mneme");
}

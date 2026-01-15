//! TDT Desktop - Tauri application library

pub mod commands;
pub mod error;
pub mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
                // Open devtools automatically in debug builds
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::open_project,
            commands::init_project,
            commands::close_project,
            commands::get_project_info,
            commands::refresh_project,
            // Generic entity commands
            commands::list_entities,
            commands::get_entity,
            commands::delete_entity,
            commands::save_entity,
            commands::get_entity_count,
            commands::get_all_entity_counts,
            // Requirement commands (stats only - CRUD via generic entities)
            commands::get_requirement_stats,
            // Risk commands
            commands::list_risks,
            commands::get_risk,
            commands::create_risk,
            commands::update_risk,
            commands::delete_risk,
            commands::add_risk_mitigation,
            commands::get_risk_stats,
            commands::get_risk_matrix,
            // Component commands
            commands::list_components,
            commands::get_component,
            commands::get_component_by_part_number,
            commands::create_component,
            commands::update_component,
            commands::delete_component,
            commands::get_component_stats,
            commands::get_bom_cost_summary,
            // Traceability commands
            commands::get_links_from,
            commands::get_links_to,
            commands::trace_from,
            commands::trace_to,
            commands::get_coverage_report,
            commands::get_dsm,
            commands::find_orphans,
            commands::find_cycles,
            commands::add_link,
            commands::remove_link,
            commands::get_link_types,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

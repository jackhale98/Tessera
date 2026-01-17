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
            #[cfg(debug_assertions)]
            {
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
            // Deviation commands
            commands::list_deviations,
            commands::get_deviation,
            commands::create_deviation,
            commands::update_deviation,
            commands::delete_deviation,
            commands::approve_deviation,
            commands::reject_deviation,
            commands::activate_deviation,
            commands::close_deviation,
            commands::expire_deviation,
            commands::add_deviation_mitigation,
            commands::get_deviation_stats,
            commands::set_deviation_risk,
            commands::add_deviation_process_link,
            commands::add_deviation_lot_link,
            commands::add_deviation_component_link,
            // NCR commands
            commands::list_ncrs,
            commands::get_ncr,
            commands::create_ncr,
            commands::update_ncr,
            commands::delete_ncr,
            commands::close_ncr,
            commands::advance_ncr_status,
            commands::add_ncr_containment,
            commands::complete_ncr_containment,
            commands::set_ncr_detection,
            commands::set_ncr_affected_items,
            commands::set_ncr_defect,
            commands::set_ncr_cost,
            commands::set_ncr_component_link,
            commands::set_ncr_capa_link,
            commands::get_ncr_stats,
            // CAPA commands
            commands::list_capas,
            commands::get_capa,
            commands::create_capa,
            commands::update_capa,
            commands::delete_capa,
            commands::advance_capa_status,
            commands::set_capa_root_cause,
            commands::add_capa_action,
            commands::update_capa_action_status,
            commands::verify_capa_effectiveness,
            commands::close_capa,
            commands::add_capa_ncr_link,
            commands::add_capa_risk_link,
            commands::get_capa_stats,
            // Lot commands
            commands::list_lots,
            commands::get_lot,
            commands::create_lot,
            commands::update_lot,
            commands::delete_lot,
            commands::set_lot_product,
            commands::add_lot_material,
            commands::remove_lot_material,
            commands::add_lot_step,
            commands::update_lot_step,
            commands::remove_lot_step,
            commands::put_lot_on_hold,
            commands::resume_lot,
            commands::complete_lot,
            commands::force_complete_lot,
            commands::scrap_lot,
            commands::add_lot_ncr,
            commands::remove_lot_ncr,
            commands::add_lot_result,
            commands::set_lot_git_branch,
            commands::mark_lot_branch_merged,
            commands::get_lot_next_step,
            commands::get_lot_stats,
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

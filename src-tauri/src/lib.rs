// ====== Import standard ======
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod machines;
mod config;
mod settings;
mod users;
mod sync;
mod work_types;

// ====== TAURI COMMANDS ======

// ====== TAURI APP ENTRYPOINT ======
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database pool on app startup
            tauri::async_runtime::spawn(async {
                // Initialize both machines and users database pools (they share the same pool)
                if let Err(e) = machines::get_or_init_db_pool().await {
                    eprintln!("Errore inizializzazione pool database: {}", e);
                } else {
                    println!("Pool database inizializzato correttamente");
                }

                // Also initialize SQLite for caching
                if let Err(e) = sync::init_sqlite_db().await {
                    eprintln!("Errore inizializzazione database SQLite: {}", e);
                } else {
                    println!("Database SQLite inizializzato correttamente");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            machines::add_new_machine,
            machines::get_machines,
            machines::get_machine,
            machines::update_machine,
            machines::delete_machine,
            machines::ping_machine,
            machines::select_machine_image,
            machines::get_available_databases,
            machines::check_database_connection,
            machines::test_db_connection,
            settings::save_setting,
            settings::get_setting,
                config::create_database,
                config::check_database_exists,
                users::register_user,
                users::login_user,
                users::get_user_profile,
                users::update_user_profile,
                users::get_all_users,
                users::delete_user,
                users::update_user,
                users::select_user_image,
                sync::get_cached_users,
                sync::sync_users,
                sync::check_sync_status,
                sync::get_cached_machines,
                sync::sync_machines,
                sync::get_cached_work_types,
                sync::sync_work_types,
                sync::start_sync_watcher,
                work_types::get_work_types,
                work_types::get_work_type,
                work_types::add_work_type,
                work_types::update_work_type,
                work_types::delete_work_type,
                work_types::select_work_type_image,
                work_types::load_image_from_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}

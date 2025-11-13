use rusqlite::{Connection, Result as SqliteResult};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

type DbConnection = Arc<Mutex<Connection>>;

static CONFIG_DB: Lazy<DbConnection> = Lazy::new(|| {
    let conn = Connection::open("config.db").expect("Errore apertura DB config");
    init_config_table(&conn).expect("Errore inizializzazione tabella config");
    Arc::new(Mutex::new(conn))
});

fn init_config_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn get_setting_internal(key: &str) -> Result<String, String> {
    let db = CONFIG_DB.lock().map_err(|e| format!("Errore lock DB: {}", e))?;
    let mut stmt = db.prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| format!("Errore preparazione query: {}", e))?;

    let value: Option<String> = stmt.query_row([key], |row| row.get(0)).ok();

    if let Some(val) = value {
        Ok(val)
    } else {
        // Valori di default
        match key {
            "language" => Ok("en".to_string()),
            "theme" => Ok("light".to_string()),
            "notifications" => Ok("true".to_string()),
            "db_host" => Ok("localhost".to_string()),
            "db_port" => Ok("5432".to_string()),
            "db_name" => Ok("mes_service_db".to_string()),
            "db_user" => Ok("postgres".to_string()),
            "db_password" => Ok("postgres".to_string()),
            _ => Err(format!("Impostazione '{}' non trovata", key)),
        }
    }
}

#[tauri::command]
pub fn save_setting(key: String, value: String) -> Result<String, String> {
    let db = CONFIG_DB.lock().map_err(|e| format!("Errore lock DB: {}", e))?;
    db.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [&key, &value],
    )
    .map_err(|e| format!("Errore salvataggio impostazione: {}", e))?;

    Ok(format!("Impostazione '{}' salvata con valore '{}'", key, value))
}

#[tauri::command]
pub fn get_setting(key: String) -> Result<String, String> {
    let db = CONFIG_DB.lock().map_err(|e| format!("Errore lock DB: {}", e))?;
    let mut stmt = db.prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| format!("Errore preparazione query: {}", e))?;

    let value: Option<String> = stmt.query_row([&key], |row| row.get(0)).ok();

    if let Some(val) = value {
        Ok(val)
    } else {
        // Valori di default
        match key.as_str() {
            "language" => Ok("en".to_string()),
            "theme" => Ok("light".to_string()),
            "notifications" => Ok("true".to_string()),
            "db_host" => Ok("localhost".to_string()),
            "db_port" => Ok("5432".to_string()),
            "db_name" => Ok("mes_service_db".to_string()),
            "db_user" => Ok("postgres".to_string()),
            "db_password" => Ok("postgres".to_string()),
            _ => Err(format!("Impostazione '{}' non trovata", key)),
        }
    }
}
use rusqlite::Connection;
use chrono::{DateTime, Utc};
use crate::users::User;
use crate::machines::Machine;
use crate::work_types::WorkType;
use serde::Serialize;
use serde_json::Value as JsonValue;
use tauri::{Manager, Emitter};

pub async fn init_sqlite_db() -> Result<(), String> {
    let db_path = get_db_path();
    let conn = match Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(e) => return Err(format!("Errore apertura database SQLite: {}", e)),
    };

    // Crea tabella users se non esiste
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT,
            image BLOB,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            sync_status TEXT DEFAULT 'pending',
            last_sync TEXT,
            deleted INTEGER DEFAULT 0
        )",
        [],
    ) {
        return Err(format!("Errore creazione tabella users: {}", e));
    }

    // Crea tabella machines se non esiste
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS machines (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            image BLOB,
            user_name TEXT,
            assistant_name TEXT,
            ip_address TEXT,
            description TEXT,
            database_name TEXT,
            works_types TEXT DEFAULT '[]',
            sync_status TEXT DEFAULT 'pending',
            last_sync TEXT,
            deleted INTEGER DEFAULT 0
        )",
        [],
    ) {
        return Err(format!("Errore creazione tabella machines: {}", e));
    }

    // Crea tabella work_types se non esiste
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS work_types (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            image BLOB,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            sync_status TEXT DEFAULT 'pending',
            last_sync TEXT,
            deleted INTEGER DEFAULT 0
        )",
        [],
    ) {
        return Err(format!("Errore creazione tabella work_types: {}", e));
    }

    // Crea indice per sync_status
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_users_sync_status ON users(sync_status)",
        [],
    );

    // Crea indice per last_sync
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_users_last_sync ON users(last_sync)",
        [],
    );

    // Crea indice per machines sync_status
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_machines_sync_status ON machines(sync_status)",
        [],
    );

    // Crea indice per machines last_sync
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_machines_last_sync ON machines(last_sync)",
        [],
    );

    // Crea indice per work_types sync_status
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_work_types_sync_status ON work_types(sync_status)",
        [],
    );

    // Crea indice per work_types last_sync
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_work_types_last_sync ON work_types(last_sync)",
        [],
    );

    Ok(())
}

fn get_db_path() -> String {
    "data.db".to_string()
}

pub fn get_sqlite_connection() -> Result<Connection, String> {
    let db_path = get_db_path();
    Connection::open(&db_path).map_err(|e| format!("Errore apertura connessione SQLite: {}", e))
}

pub async fn sync_users_from_postgres() -> Result<i32, String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    // Ottieni utenti da PostgreSQL
    let pg_users = crate::users::get_users_from_postgres().await?;

    let conn = get_sqlite_connection()?;
    let mut synced_count = 0;

    for pg_user in pg_users {
        // Converte il formato da sqlx a rusqlite
        let created_at_str = pg_user.created_at.to_rfc3339();
        let updated_at_str = pg_user.updated_at.to_rfc3339();

        // Inserisci o aggiorna in SQLite
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO users (id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'synced', ?, ?)",
            rusqlite::params![
                pg_user.id,
                pg_user.username,
                pg_user.email,
                pg_user.image,
                pg_user.password_hash,
                created_at_str,
                updated_at_str,
                Utc::now().to_rfc3339(),
                pg_user.deleted
            ],
        ) {
            eprintln!("Errore sync utente {}: {}", pg_user.id, e);
            continue;
        }
        synced_count += 1;
    }

    Ok(synced_count)
}

pub async fn sync_machines_from_postgres() -> Result<i32, String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    // Ottieni macchine da PostgreSQL
    let pg_machines = crate::machines::get_machines().await?;

    let conn = get_sqlite_connection()?;
    let mut synced_count = 0;

    for pg_machine in pg_machines {
        // Inserisci o aggiorna in SQLite
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO machines (id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'synced', ?, ?)",
            rusqlite::params![
                pg_machine.id,
                pg_machine.name,
                pg_machine.image,
                pg_machine.user_name,
                pg_machine.assistant_name,
                pg_machine.ip_address,
                pg_machine.description,
                pg_machine.database_name,
                serde_json::to_string(&pg_machine.works_types).unwrap_or_else(|_| "[]".to_string()),
                Utc::now().to_rfc3339(),
                pg_machine.deleted
            ],
        ) {
            eprintln!("Errore sync macchina {}: {}", pg_machine.id, e);
            continue;
        }
        synced_count += 1;
    }

    Ok(synced_count)
}

pub async fn sync_work_types_from_postgres() -> Result<i32, String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    // Ottieni work types da PostgreSQL
    let pg_work_types = crate::work_types::get_work_types().await?;

    let conn = get_sqlite_connection()?;
    let mut synced_count = 0;

    for pg_work_type in pg_work_types {
        // Converte il formato da sqlx a rusqlite
        let created_at_str = pg_work_type.created_at.to_rfc3339();
        let updated_at_str = pg_work_type.updated_at.to_rfc3339();

        // Inserisci o aggiorna in SQLite
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO work_types (id, name, image, created_at, updated_at, sync_status, last_sync, deleted)
             VALUES (?, ?, ?, ?, ?, 'synced', ?, ?)",
            rusqlite::params![
                pg_work_type.id,
                pg_work_type.name,
                pg_work_type.image,
                created_at_str,
                updated_at_str,
                Utc::now().to_rfc3339(),
                false // deleted
            ],
        ) {
            eprintln!("Errore sync work type {}: {}", pg_work_type.id, e);
            continue;
        }
        synced_count += 1;
    }

    Ok(synced_count)
}

pub async fn get_users_from_sqlite() -> Result<Vec<User>, String> {
    // Inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE deleted = 0 ORDER BY created_at DESC"
    ).map_err(|e| format!("Errore preparazione query: {}", e))?;

    let user_iter = stmt.query_map([], |row| {
        let created_at_str: String = row.get(5)?;
        let updated_at_str: String = row.get(6)?;
        let last_sync_str: Option<String> = row.get(8)?;
        let deleted_int: i32 = row.get(9)?;

        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            email: row.get(2)?,
            image: row.get(3)?,
            password_hash: row.get(4)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
            sync_status: row.get(7)?,
            last_sync: last_sync_str.map(|s| DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())),
            deleted: deleted_int != 0,
        })
    }).map_err(|e| format!("Errore esecuzione query: {}", e))?;

    let mut users = Vec::new();
    for user in user_iter {
        match user {
            Ok(u) => users.push(u),
            Err(e) => eprintln!("Errore parsing utente: {}", e),
        }
    }

    Ok(users)
}

pub async fn get_machines_from_sqlite() -> Result<Vec<Machine>, String> {
    // Inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted
         FROM machines WHERE deleted = 0 ORDER BY name ASC"
    ).map_err(|e| format!("Errore preparazione query: {}", e))?;

    let machine_iter = stmt.query_map([], |row| {
        let last_sync_str: Option<String> = row.get(10)?;
        let deleted_int: i32 = row.get(11)?;
        let works_types_str: String = row.get(8)?;

        // Deserializza works_types da JSON string a Vec<i32>
        let works_types_vec: Vec<i32> = serde_json::from_str(&works_types_str)
            .unwrap_or_else(|_| Vec::new());

        // Converti in JsonValue per popolare la struct Machine (works_types è JsonValue nel modello)
        let works_types: JsonValue = serde_json::to_value(&works_types_vec).unwrap_or(JsonValue::Array(vec![]));

        Ok(Machine {
            id: row.get(0)?,
            name: row.get(1)?,
            image: row.get(2)?,
            user_name: row.get(3)?,
            assistant_name: row.get(4)?,
            ip_address: row.get(5)?,
            description: row.get(6)?,
            database_name: row.get(7)?,
            works_types,
            sync_status: row.get(9)?,
            last_sync: last_sync_str.map(|s| DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())),
            deleted: deleted_int != 0,
        })
    }).map_err(|e| format!("Errore esecuzione query: {}", e))?;

    let mut machines = Vec::new();
    for machine in machine_iter {
        match machine {
            Ok(m) => machines.push(m),
            Err(e) => eprintln!("Errore parsing macchina: {}", e),
        }
    }

    Ok(machines)
}

pub async fn get_work_types_from_sqlite() -> Result<Vec<WorkType>, String> {
    // Inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, image, created_at, updated_at
         FROM work_types WHERE deleted = 0 ORDER BY name ASC"
    ).map_err(|e| format!("Errore preparazione query: {}", e))?;

    let work_type_iter = stmt.query_map([], |row| {
        let created_at_str: String = row.get(3)?;
        let updated_at_str: String = row.get(4)?;

        Ok(WorkType {
            id: row.get(0)?,
            name: row.get(1)?,
            image: row.get(2)?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc),
        })
    }).map_err(|e| format!("Errore esecuzione query: {}", e))?;

    let mut work_types = Vec::new();
    for work_type in work_type_iter {
        match work_type {
            Ok(wt) => work_types.push(wt),
            Err(e) => eprintln!("Errore parsing work type: {}", e),
        }
    }

    Ok(work_types)
}

pub async fn check_sync_needed() -> Result<bool, String> {
    // Prima prova a ottenere gli utenti da PostgreSQL
    let pg_users_result = crate::users::get_users_from_postgres().await;

    match pg_users_result {
        Ok(pg_users) => {
            // Se riusciamo a raggiungere PostgreSQL, confronta normalmente
            let sqlite_users = get_users_from_sqlite().await?;

            // Controlla se il numero di utenti è diverso
            if sqlite_users.len() != pg_users.len() {
                return Ok(true);
            }

            // Controlla se ci sono utenti con sync_status diverso da 'synced'
            for user in &sqlite_users {
                if user.sync_status != "synced" {
                    return Ok(true);
                }
            }

            // Controlla se ci sono discrepanze negli updated_at
            for pg_user in &pg_users {
                if let Some(sqlite_user) = sqlite_users.iter().find(|u| u.id == pg_user.id) {
                    if sqlite_user.updated_at < pg_user.updated_at {
                        return Ok(true);
                    }
                } else {
                    // Utente presente in PG ma non in SQLite
                    return Ok(true);
                }
            }

            Ok(false)
        },
        Err(_) => {
            // Se non riusciamo a raggiungere PostgreSQL, assumiamo che la cache sia valida
            // (anche se potrebbe essere stale, è meglio che niente)
            println!("Server PostgreSQL non raggiungibile per controllo sync utenti, assumendo cache valida");
            Ok(false)
        }
    }
}

pub async fn save_user_to_sqlite(user: &crate::users::User) -> Result<(), String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;
    let created_at_str = user.created_at.to_rfc3339();
    let updated_at_str = user.updated_at.to_rfc3339();
    let last_sync_str = user.last_sync.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339());

    // Inserisci o aggiorna in SQLite
    conn.execute(
        "INSERT OR REPLACE INTO users (id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'synced', ?, ?)",
        rusqlite::params![
            user.id,
            user.username,
            user.email,
            user.image,
            user.password_hash,
            created_at_str,
            updated_at_str,
            last_sync_str,
            user.deleted
        ],
    ).map_err(|e| format!("Errore salvataggio utente in SQLite: {}", e))?;

    Ok(())
}

pub async fn save_machine_to_sqlite(machine: &crate::machines::Machine) -> Result<(), String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;
    let last_sync_str = machine.last_sync.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339());
    let works_types_str = serde_json::to_string(&machine.works_types)
        .map_err(|e| format!("Errore serializzazione works_types: {}", e))?;

    // Inserisci o aggiorna in SQLite
    conn.execute(
        "INSERT OR REPLACE INTO machines (id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'synced', ?, ?)",
        rusqlite::params![
            machine.id,
            machine.name,
            machine.image,
            machine.user_name,
            machine.assistant_name,
            machine.ip_address,
            machine.description,
            machine.database_name,
            works_types_str,
            last_sync_str,
            machine.deleted
        ],
    ).map_err(|e| format!("Errore salvataggio macchina in SQLite: {}", e))?;

    Ok(())
}

pub async fn save_work_type_to_sqlite(work_type: &WorkType) -> Result<(), String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;
    let created_at_str = work_type.created_at.to_rfc3339();
    let updated_at_str = work_type.updated_at.to_rfc3339();

    // Inserisci o aggiorna in SQLite
    conn.execute(
        "INSERT OR REPLACE INTO work_types (id, name, image, created_at, updated_at, sync_status, last_sync, deleted)
         VALUES (?, ?, ?, ?, ?, 'synced', ?, ?)",
        rusqlite::params![
            work_type.id,
            work_type.name,
            work_type.image,
            created_at_str,
            updated_at_str,
            Utc::now().to_rfc3339(),
            false // deleted
        ],
    ).map_err(|e| format!("Errore salvataggio work type in SQLite: {}", e))?;

    Ok(())
}

pub async fn delete_machine_from_sqlite(id: i32) -> Result<(), String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;

    // Soft delete della macchina da SQLite
    conn.execute(
        "UPDATE machines SET deleted = 1 WHERE id = ?",
        rusqlite::params![id],
    ).map_err(|e| format!("Errore soft delete macchina da SQLite: {}", e))?;

    Ok(())
}

pub async fn delete_work_type_from_sqlite(id: i32) -> Result<(), String> {
    // Prima inizializza SQLite se necessario
    init_sqlite_db().await?;

    let conn = get_sqlite_connection()?;

    // Soft delete del work type da SQLite
    conn.execute(
        "UPDATE work_types SET deleted = 1 WHERE id = ?",
        rusqlite::params![id],
    ).map_err(|e| format!("Errore soft delete work type da SQLite: {}", e))?;

    Ok(())
}

pub async fn check_machines_sync_needed() -> Result<bool, String> {
    // Prima prova a ottenere le macchine da PostgreSQL
    let pg_machines_result = crate::machines::get_machines().await;

    match pg_machines_result {
        Ok(pg_machines) => {
            // Se riusciamo a raggiungere PostgreSQL, confronta normalmente
            let sqlite_machines = get_machines_from_sqlite().await?;

            // Controlla se il numero di macchine è diverso
            if sqlite_machines.len() != pg_machines.len() {
                return Ok(true);
            }

            // Controlla se ci sono macchine con sync_status diverso da 'synced'
            for machine in &sqlite_machines {
                if machine.sync_status != "synced" {
                    return Ok(true);
                }
            }

            Ok(false)
        },
        Err(_) => {
            // Se non riusciamo a raggiungere PostgreSQL, assumiamo che la cache sia valida
            println!("Server PostgreSQL non raggiungibile per controllo sync macchine, assumendo cache valida");
            Ok(false)
        }
    }
}

pub async fn check_work_types_sync_needed() -> Result<bool, String> {
    // Prima prova a ottenere i work types da PostgreSQL
    let pg_work_types_result = crate::work_types::get_work_types().await;

    match pg_work_types_result {
        Ok(pg_work_types) => {
            // Se riusciamo a raggiungere PostgreSQL, confronta normalmente
            let sqlite_work_types = get_work_types_from_sqlite().await?;

            // Controlla se il numero di work types è diverso
            if sqlite_work_types.len() != pg_work_types.len() {
                return Ok(true);
            }

            // Controlla se ci sono work types con sync_status diverso da 'synced'
            for _work_type in &sqlite_work_types {
                // Per ora assumiamo che se è in SQLite è synced
                // In futuro potremmo aggiungere sync_status ai work types
            }

            Ok(false)
        },
        Err(_) => {
            // Se non riusciamo a raggiungere PostgreSQL, assumiamo che la cache sia valida
            println!("Server PostgreSQL non raggiungibile per controllo sync work types, assumendo cache valida");
            Ok(false)
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn sync_users() -> Result<String, String> {
    match sync_users_from_postgres().await {
        Ok(count) => Ok(format!("Sincronizzati {} utenti", count)),
        Err(e) => Err(format!("Errore sincronizzazione utenti: {}", e)),
    }
}

#[tauri::command]
pub async fn sync_machines() -> Result<String, String> {
    match sync_machines_from_postgres().await {
        Ok(count) => Ok(format!("Sincronizzate {} macchine", count)),
        Err(e) => Err(format!("Errore sincronizzazione macchine: {}", e)),
    }
}

#[tauri::command]
pub async fn sync_work_types() -> Result<String, String> {
    match sync_work_types_from_postgres().await {
        Ok(count) => Ok(format!("Sincronizzati {} tipi di lavorazione", count)),
        Err(e) => Err(format!("Errore sincronizzazione tipi di lavorazione: {}", e)),
    }
}

#[tauri::command]
pub async fn get_cached_users() -> Result<Vec<User>, String> {
    get_users_from_sqlite().await
}

#[tauri::command]
pub async fn get_cached_machines() -> Result<Vec<Machine>, String> {
    get_machines_from_sqlite().await
}

#[tauri::command]
pub async fn get_cached_work_types() -> Result<Vec<WorkType>, String> {
    get_work_types_from_sqlite().await
}

#[tauri::command]
pub async fn check_sync_status() -> Result<String, String> {
    let users_needed = check_sync_needed().await?;
    let machines_needed = check_machines_sync_needed().await?;
    let work_types_needed = check_work_types_sync_needed().await?;

    if users_needed || machines_needed || work_types_needed {
        Ok("sync_needed".to_string())
    } else {
        Ok("synced".to_string())
    }
}

#[derive(Serialize)]
pub struct UserDiff {
    pub id: i32,
    pub username: String,
    pub reason: String,
}

#[derive(Serialize)]
pub struct MachineDiff {
    pub id: i32,
    pub name: String,
    pub reason: String,
}

#[derive(Serialize)]
pub struct WorkTypeDiff {
    pub id: i32,
    pub name: String,
    pub reason: String,
}

#[derive(Serialize)]
pub struct Diffs {
    pub users: Vec<UserDiff>,
    pub machines: Vec<MachineDiff>,
    pub work_types: Vec<WorkTypeDiff>,
}

async fn compute_diffs() -> Result<Diffs, String> {
    // Get lists from postgres and sqlite
    let pg_users_result = crate::users::get_users_from_postgres().await;
    let pg_machines_result = crate::machines::get_machines().await;
    let pg_work_types_result = crate::work_types::get_work_types().await;

    // If we can't reach PostgreSQL, no diffs to report
    let pg_users = match pg_users_result {
        Ok(users) => users,
        Err(e) => {
            println!("Impossibile raggiungere PostgreSQL per utenti: {}", e);
            Vec::new()
        }
    };

    let _pg_machines = match pg_machines_result {
        Ok(machines) => machines,
        Err(e) => {
            println!("Impossibile raggiungere PostgreSQL per macchine: {}", e);
            Vec::new()
        }
    };

    let pg_work_types = match pg_work_types_result {
        Ok(work_types) => work_types,
        Err(e) => {
            println!("Impossibile raggiungere PostgreSQL per work types: {}", e);
            Vec::new()
        }
    };

    let sqlite_users = get_users_from_sqlite().await?;
    let sqlite_machines = get_machines_from_sqlite().await?;
    let sqlite_work_types = get_work_types_from_sqlite().await?;

    let mut user_diffs: Vec<UserDiff> = Vec::new();
    let mut machine_diffs: Vec<MachineDiff> = Vec::new();
    let mut work_type_diffs: Vec<WorkTypeDiff> = Vec::new();

    // Build map of sqlite users by id for quick lookup
    let sqlite_users_map: std::collections::HashMap<i32, &User> = sqlite_users.iter().map(|u| (u.id, u)).collect();

    for pg in pg_users.iter() {
        match sqlite_users_map.get(&pg.id) {
            Some(su) => {
                if su.updated_at < pg.updated_at {
                    user_diffs.push(UserDiff { id: pg.id, username: pg.username.clone(), reason: "postgres newer".to_string() });
                }
            }
            None => {
                user_diffs.push(UserDiff { id: pg.id, username: pg.username.clone(), reason: "missing in sqlite".to_string() });
            }
        }
    }

    // Check for users in SQLite that are not in PostgreSQL (shouldn't happen but good to check)
    let pg_users_map: std::collections::HashMap<i32, &User> = pg_users.iter().map(|u| (u.id, u)).collect();
    for sqlite_user in &sqlite_users {
        if !pg_users_map.contains_key(&sqlite_user.id) {
            user_diffs.push(UserDiff { id: sqlite_user.id, username: sqlite_user.username.clone(), reason: "extra in sqlite".to_string() });
        }
    }

    // Machines
    let sqlite_machines_map: std::collections::HashMap<i32, &Machine> = sqlite_machines.iter().map(|m| (m.id, m)).collect();
    for pg in _pg_machines.iter() {
        match sqlite_machines_map.get(&pg.id) {
            Some(sm) => {
                if sm.last_sync.map(|d| d < pg.last_sync.unwrap_or(Utc::now())).unwrap_or(true) {
                    machine_diffs.push(MachineDiff { id: pg.id, name: pg.name.clone(), reason: "postgres newer".to_string() });
                }
            }
            None => {
                machine_diffs.push(MachineDiff { id: pg.id, name: pg.name.clone(), reason: "missing in sqlite".to_string() });
            }
        }
    }

    // Check for machines in SQLite that are not in PostgreSQL
    let pg_machines_map: std::collections::HashMap<i32, &Machine> = _pg_machines.iter().map(|m| (m.id, m)).collect();
    for sqlite_machine in &sqlite_machines {
        if !pg_machines_map.contains_key(&sqlite_machine.id) {
            machine_diffs.push(MachineDiff { id: sqlite_machine.id, name: sqlite_machine.name.clone(), reason: "extra in sqlite".to_string() });
        }
    }

    // Work Types
    let sqlite_work_types_map: std::collections::HashMap<i32, &WorkType> = sqlite_work_types.iter().map(|wt| (wt.id, wt)).collect();
    for pg_wt in pg_work_types.iter() {
        match sqlite_work_types_map.get(&pg_wt.id) {
            Some(swt) => {
                if swt.updated_at < pg_wt.updated_at {
                    work_type_diffs.push(WorkTypeDiff { id: pg_wt.id, name: pg_wt.name.clone(), reason: "postgres newer".to_string() });
                }
            }
            None => {
                work_type_diffs.push(WorkTypeDiff { id: pg_wt.id, name: pg_wt.name.clone(), reason: "missing in sqlite".to_string() });
            }
        }
    }

    // Check for work types in SQLite that are not in PostgreSQL
    let pg_work_types_map: std::collections::HashMap<i32, &WorkType> = pg_work_types.iter().map(|wt| (wt.id, wt)).collect();
    for sqlite_work_type in &sqlite_work_types {
        if !pg_work_types_map.contains_key(&sqlite_work_type.id) {
            work_type_diffs.push(WorkTypeDiff { id: sqlite_work_type.id, name: sqlite_work_type.name.clone(), reason: "extra in sqlite".to_string() });
        }
    }

    Ok(Diffs { users: user_diffs, machines: machine_diffs, work_types: work_type_diffs })
}

/// Start a background watcher that periodically computes diffs between Postgres and the local SQLite cache.
/// When differences are found, emits a Tauri event `sync:needed` with payload { users: [...], machines: [...] }.
#[tauri::command]
pub fn start_sync_watcher(app_handle: tauri::AppHandle) -> Result<String, String> {
    println!("Avvio sync watcher...");

    // Prima controllo immediato delle differenze
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        match compute_diffs().await {
            Ok(diffs) => {
                println!("Controllo iniziale differenze - Utenti: {}, Macchine: {}, Work Types: {}", diffs.users.len(), diffs.machines.len(), diffs.work_types.len());
                if !diffs.users.is_empty() || !diffs.machines.is_empty() || !diffs.work_types.is_empty() {
                    println!("Differenze trovate all'avvio, emissione evento sync:needed");
                    // Emit event to all windows
                    let windows = app_handle_clone.webview_windows();
                    for (_, window) in windows {
                        if let Err(e) = window.emit("sync:needed", &diffs) {
                            eprintln!("Errore emissione evento sync:needed alla finestra: {}", e);
                        }
                    }
                } else {
                    println!("Nessuna differenza trovata all'avvio");
                }
            }
            Err(e) => {
                eprintln!("Errore controllo iniziale differenze: {}", e);
            }
        }
    });

    // Spawn a background task per il controllo periodico
    tauri::async_runtime::spawn(async move {
        use tokio::time::{sleep, Duration};
        println!("Avvio controllo periodico ogni 30 secondi");
        loop {
            sleep(Duration::from_secs(30)).await;
            println!("Controllo periodico differenze...");
            match compute_diffs().await {
                Ok(diffs) => {
                    println!("Controllo periodico - Utenti: {}, Macchine: {}, Work Types: {}", diffs.users.len(), diffs.machines.len(), diffs.work_types.len());
                    if !diffs.users.is_empty() || !diffs.machines.is_empty() || !diffs.work_types.is_empty() {
                        println!("Differenze trovate, emissione evento sync:needed");
                        // Emit event to all windows
                        let windows = app_handle.webview_windows();
                        for (_, window) in windows {
                            if let Err(e) = window.emit("sync:needed", &diffs) {
                                eprintln!("Errore emissione evento sync:needed alla finestra: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Errore controllo periodico differenze: {}", e);
                }
            }
        }
    });

    Ok("sync_watcher_started".to_string())
}
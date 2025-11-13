use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, postgres::PgPoolOptions};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Machine {
    pub id: i32,
    pub name: String,
    pub image: Option<Vec<u8>>,
    pub user_name: Option<String>,
    pub assistant_name: Option<String>,
    pub ip_address: Option<String>,
    pub description: Option<String>,
    pub database_name: Option<String>,
    pub works_types: JsonValue,
    pub sync_status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub deleted: bool,
}

type DbPool = Arc<PgPool>;

static DB_POOL: Lazy<Mutex<Option<DbPool>>> = Lazy::new(|| Mutex::new(None));

pub async fn get_or_init_db_pool() -> Result<(), String> {
    {
        let pool_guard = DB_POOL.lock().map_err(|e| format!("Errore lock pool: {}", e))?;
        if pool_guard.is_some() {
            return Ok(());
        }
    } // lock rilasciato qui

    // Ora connetti senza lock
    let db_url = match crate::config::get_db_config() {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Configurazione database non valida: {}", e);
            return Err(format!("Configurazione database non valida: {}", e));
        }
    };

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Errore connessione database: {}", e);
            return Err(format!("Errore connessione database: {}", e));
        }
    };

    if let Err(e) = ensure_machines_table(&pool).await {
        eprintln!("Errore creazione tabella machines: {}", e);
        return Err(format!("Errore creazione tabella machines: {}", e));
    }

    // Ora setta con lock
    let mut pool_guard = DB_POOL.lock().map_err(|e| format!("Errore lock pool: {}", e))?;
    if pool_guard.is_none() {
        *pool_guard = Some(Arc::new(pool));
    }
    Ok(())
}

pub async fn ensure_machines_table(pool: &PgPool) -> Result<(), String> {
    // Prima proviamo ad alterare la colonna se esiste già con tipo sbagliato
    let _ = sqlx::query(
        "ALTER TABLE machines ALTER COLUMN ip_address TYPE TEXT"
    )
    .execute(pool)
    .await;

    // Converti colonna image da TEXT a BYTEA se necessario
    let _ = sqlx::query(
        "ALTER TABLE machines ALTER COLUMN image TYPE BYTEA USING image::bytea"
    )
    .execute(pool)
    .await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS machines (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            image BYTEA,
            user_name TEXT,
            assistant_name TEXT,
            ip_address TEXT,
            description TEXT,
            database_name TEXT,
            sync_status TEXT NOT NULL DEFAULT 'synced',
            last_sync TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            deleted BOOLEAN NOT NULL DEFAULT FALSE
        )"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Errore creazione tabella: {}", e))?;

    // Aggiungi colonna database_name se non esiste (per aggiornamenti)
    let _ = sqlx::query(
        "ALTER TABLE machines ADD COLUMN IF NOT EXISTS database_name TEXT"
    )
    .execute(pool)
    .await;

    // Aggiungi colonne per il sync se non esistono
    let _ = sqlx::query(
        "ALTER TABLE machines ADD COLUMN IF NOT EXISTS sync_status TEXT NOT NULL DEFAULT 'synced'"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE machines ADD COLUMN IF NOT EXISTS last_sync TIMESTAMP WITH TIME ZONE DEFAULT NOW()"
    )
    .execute(pool)
    .await;

    // Aggiungi colonna deleted se non esiste
    let _ = sqlx::query(
        "ALTER TABLE machines ADD COLUMN IF NOT EXISTS deleted BOOLEAN NOT NULL DEFAULT FALSE"
    )
    .execute(pool)
    .await;

    // Aggiungi colonna works_types se non esiste (array JSON di ID work types)
    let _ = sqlx::query(
        "ALTER TABLE machines ADD COLUMN IF NOT EXISTS works_types JSONB DEFAULT '[]'::jsonb"
    )
    .execute(pool)
    .await;

    Ok(())
}

pub async fn get_db_pool() -> Result<DbPool, String> {
    let pool_guard = DB_POOL.lock().map_err(|e| format!("Errore lock pool: {}", e))?;
    pool_guard.as_ref().cloned().ok_or_else(|| "Pool DB non inizializzato".to_string())
}

pub async fn save_machine_to_postgres(
    name: String,
    image: Option<Vec<u8>>,
    ip_address: Option<String>,
    database_name: Option<String>,
    description: Option<String>,
    works_types: Vec<i32>,
) -> Result<Machine, String> {
    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    let rec = sqlx::query_as::<_, Machine>(
        "INSERT INTO machines (name, image, ip_address, database_name, description, works_types, sync_status, last_sync, deleted)
         VALUES ($1, $2, $3, $4, $5, $6, 'synced', NOW(), false)
         RETURNING id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted"
    )
    .bind(name)
    .bind(image)
    .bind(ip_address)
    .bind(database_name)
    .bind(description)
    .bind(serde_json::to_value(works_types).map_err(|e| format!("Errore serializzazione works_types: {}", e))?)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("Errore inserimento macchina in PostgreSQL: {}", e))?;

    Ok(rec)
}

#[tauri::command]
pub async fn test_db_connection() -> Result<String, String> {
    let db_url = crate::config::get_db_config()
        .map_err(|e| format!("Errore recupero config DB: {}", e))?;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .map_err(|e| format!("Errore connessione DB: {}", e))?;

    // Test semplice
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| format!("Errore query test: {}", e))?;

    Ok("Connessione DB riuscita".to_string())
}

#[tauri::command]
pub async fn add_new_machine(
    name: String,
    image: Option<Vec<u8>>,
    ip_address: Option<String>,
    database_name: Option<String>,
    description: Option<String>,
    worksTypes: Vec<i32>,
) -> Result<Machine, String> {
    // Prima salva su PostgreSQL
    let pg_machine = save_machine_to_postgres(
        name.clone(),
        image.clone(),
        ip_address,
        database_name,
        description,
        worksTypes,
    ).await?;

    // Poi salva su SQLite
    crate::sync::save_machine_to_sqlite(&pg_machine).await?;

    Ok(pg_machine)
}

#[tauri::command]
pub async fn get_machines_from_postgres() -> Result<Vec<Machine>, String> {
    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    let machines = sqlx::query_as::<_, Machine>(
        "SELECT id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted FROM machines WHERE deleted = false ORDER BY id DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("Errore recupero macchine: {}", e))?;

    Ok(machines)
}

#[tauri::command]
pub async fn get_machines() -> Result<Vec<Machine>, String> {
    crate::sync::get_machines_from_sqlite().await
}

#[tauri::command]
pub async fn update_machine(
    id: i32,
    name: String,
    image: Option<Vec<u8>>,
    ip_address: Option<String>,
    database_name: Option<String>,
    description: Option<String>,
    worksTypes: Vec<i32>,
) -> Result<Machine, String> {
    // Converti stringhe vuote in Option<String>
    let ip_address_opt = ip_address.as_ref().map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten();
    let database_name_opt = database_name.as_ref().map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten();
    let description_opt = description.as_ref().map(|s| if s.trim().is_empty() { None } else { Some(s.trim().to_string()) }).flatten();

    // Debug logging
    println!("update_machine chiamata con id: {}, name: {}, ip_address: {:?}, database_name: {:?}, description: {:?}", id, name, ip_address, database_name, description);

    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    // Prima verifichiamo se la macchina esiste e non è stata soft-deleted
    let exists_check = sqlx::query("SELECT id FROM machines WHERE id = $1 AND deleted = false")
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| format!("Errore verifica esistenza macchina: {}", e))?;

    if exists_check.is_none() {
        return Err(format!("Macchina con id {} non trovata o è stata eliminata", id));
    }

    // Debug logging dei parametri bindati
    println!("Parametri bindati - name: {}, image: {}, ip_address: {:?}, database_name: {:?}, description: {:?}, worksTypes: {:?}, id: {}",
             name, image.is_some(), ip_address_opt, database_name_opt, description_opt, worksTypes, id);

    let rec = sqlx::query_as::<_, Machine>(
        "UPDATE machines SET name = $1, image = $2, ip_address = $3, database_name = $4, description = $5, works_types = $6, sync_status = 'synced', last_sync = NOW() WHERE id = $7 AND deleted = false
         RETURNING id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted"
    )
    .bind(&name)
    .bind(&image)
    .bind(&ip_address_opt)
    .bind(&database_name_opt)
    .bind(&description_opt)
    .bind(serde_json::to_value(&worksTypes).map_err(|e| format!("Errore serializzazione worksTypes: {}", e))?)
    .bind(id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("Errore aggiornamento macchina: {}", e))?;

    // Verifica che l'aggiornamento sia andato a buon fine con una query diretta
    let verify_rec = sqlx::query_as::<_, Machine>(
        "SELECT id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted FROM machines WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("Errore verifica aggiornamento: {}", e))?;

    println!("Verifica dopo UPDATE - ip_address: {:?}, database_name: {:?}, worksTypes: {:?}", verify_rec.ip_address, verify_rec.database_name, verify_rec.works_types);

    // Debug logging del risultato
    println!("Macchina aggiornata: id={}, name={}, ip_address={:?}, database_name={:?}, worksTypes: {:?}", rec.id, rec.name, rec.ip_address, rec.database_name, rec.works_types);

    // Aggiorna anche su SQLite per mantenere la cache locale aggiornata
    crate::sync::save_machine_to_sqlite(&rec).await
        .map_err(|e| format!("Errore salvataggio in SQLite: {}", e))?;

    Ok(rec)
}

#[tauri::command]
pub async fn delete_machine(id: i32) -> Result<(), String> {
    // Prima soft delete da PostgreSQL
    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    // Verifica che la macchina esista e non sia già stata soft-deleted
    let exists_check = sqlx::query("SELECT id FROM machines WHERE id = $1 AND deleted = false")
        .bind(id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| format!("Errore verifica esistenza macchina: {}", e))?;

    if exists_check.is_none() {
        return Err(format!("Macchina con id {} non trovata o già eliminata", id));
    }

    // Soft delete della macchina in PostgreSQL
    sqlx::query("UPDATE machines SET deleted = true WHERE id = $1")
    .bind(id)
    .execute(&*pool)
    .await
    .map_err(|e| format!("Errore soft delete macchina da PostgreSQL: {}", e))?;

    // Poi soft delete da SQLite
    crate::sync::delete_machine_from_sqlite(id).await
        .map_err(|e| format!("Errore soft delete macchina da SQLite: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn ping_machine(ip_address: String) -> Result<bool, String> {
    use tokio::process::Command;
    use std::time::Duration;
    use tokio::time::timeout;

    // Usa ping con timeout di 5 secondi
    let ping_result = timeout(
        Duration::from_secs(5),
        Command::new("ping")
            .args(&["-c", "1", "-W", "2", &ip_address])
            .output()
    ).await;

    match ping_result {
        Ok(Ok(output)) => {
            // Su Windows il comando ping è diverso
            #[cfg(target_os = "windows")]
            {
                Ok(output.status.success())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok(output.status.success())
            }
        }
        Ok(Err(_)) | Err(_) => Ok(false), // Timeout o errore comando
    }
}

#[tauri::command]
pub async fn select_machine_image() -> Result<Option<Vec<u8>>, String> {
    use rfd::FileDialog;
    use std::fs;

    let file_path = FileDialog::new()
        .add_filter("Immagini", &["png", "jpg", "jpeg", "gif", "bmp", "webp"])
        .pick_file();

    match file_path {
        Some(path) => {
            match fs::read(&path) {
                Ok(bytes) => Ok(Some(bytes)),
                Err(e) => Err(format!("Errore lettura file immagine: {}", e)),
            }
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn check_database_connection(database_name: String) -> Result<bool, String> {
    let host = crate::settings::get_setting_internal("db_host")?;
    let port = crate::settings::get_setting_internal("db_port")?;
    let user = crate::settings::get_setting_internal("db_user")?;
    let password = crate::settings::get_setting_internal("db_password")?;

    let db_url = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, database_name);

    match PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            // Test semplice
            match sqlx::query("SELECT 1").execute(&pool).await {
                Ok(_) => {
                    pool.close().await;
                    Ok(true)
                }
                Err(_) => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn get_available_databases() -> Result<Vec<String>, String> {
    let host = crate::settings::get_setting_internal("db_host")?;
    let port = crate::settings::get_setting_internal("db_port")?;
    let user = crate::settings::get_setting_internal("db_user")?;
    let password = crate::settings::get_setting_internal("db_password")?;

    // Connetti al database postgres (database di sistema)
    let db_url = format!("postgres://{}:{}@{}:{}/postgres", user, password, host, port);

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .map_err(|e| format!("Errore connessione al database postgres: {}", e))?;

    // Query per ottenere tutti i database non template
    let databases: Vec<String> = sqlx::query_scalar(
        "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Errore recupero database: {}", e))?;

    pool.close().await;
    Ok(databases)
}

#[tauri::command]
pub async fn get_machine(id: i32) -> Result<Machine, String> {
    // Prima cerca in SQLite (cache locale)
    match crate::sync::get_machines_from_sqlite().await {
        Ok(machines) => {
            if let Some(machine) = machines.into_iter().find(|m| m.id == id) {
                return Ok(machine);
            }
        }
        Err(_) => {
            // Se errore lettura SQLite, continua con PostgreSQL
        }
    }

    // Se non trovata in SQLite, cerca in PostgreSQL
    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    let machine = sqlx::query_as::<_, Machine>(
        "SELECT id, name, image, user_name, assistant_name, ip_address, description, database_name, works_types, sync_status, last_sync, deleted FROM machines WHERE id = $1 AND deleted = false"
    )
    .bind(id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("Errore recupero macchina: {}", e))?;

    Ok(machine)
}
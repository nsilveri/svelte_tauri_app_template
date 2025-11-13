use serde::{Serialize, Deserialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use bcrypt::{hash, verify, DEFAULT_COST};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use crate::machines::get_db_pool;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: Option<String>,
    pub image: Option<Vec<u8>>,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sync_status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub email: Option<String>,
    pub image: Option<Vec<u8>>,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub user: User,
    pub token: String, // Per ora semplice, in produzione usare JWT
}

type DbPool = Arc<PgPool>;

static DB_POOL: Lazy<Mutex<Option<DbPool>>> = Lazy::new(|| Mutex::new(None));

pub async fn get_or_init_db_pool() -> Result<(), String> {
    {
        let pool_guard = DB_POOL.lock().map_err(|e| format!("Errore lock pool: {}", e))?;
        if pool_guard.is_some() {
            return Ok(());
        }
    }

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

    if let Err(e) = ensure_users_table(&pool).await {
        eprintln!("Errore creazione tabella users: {}", e);
        return Err(format!("Errore creazione tabella users: {}", e));
    }

    let mut pool_guard = DB_POOL.lock().map_err(|e| format!("Errore lock pool: {}", e))?;
    if pool_guard.is_none() {
        *pool_guard = Some(Arc::new(pool));
    }
    Ok(())
}

pub async fn ensure_users_table(pool: &PgPool) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            email TEXT,
            image BYTEA,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            sync_status TEXT NOT NULL DEFAULT 'synced',
            last_sync TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            deleted BOOLEAN NOT NULL DEFAULT FALSE
        )"
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Errore creazione tabella users: {}", e))?;

    // Migrazione delle colonne esistenti se necessario
    let _ = sqlx::query(
        "ALTER TABLE users ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC'"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE users ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC'"
    )
    .execute(pool)
    .await;

    // Aggiungi colonne per il sync se non esistono
    let _ = sqlx::query(
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS sync_status TEXT NOT NULL DEFAULT 'synced'"
    )
    .execute(pool)
    .await;

    let _ = sqlx::query(
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS last_sync TIMESTAMP WITH TIME ZONE DEFAULT NOW()"
    )
    .execute(pool)
    .await;

    // Aggiungi colonna deleted se non esiste
    let _ = sqlx::query(
        "ALTER TABLE users ADD COLUMN IF NOT EXISTS deleted BOOLEAN NOT NULL DEFAULT FALSE"
    )
    .execute(pool)
    .await;

    // Aggiungi indice unico su username se non esiste
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username)"
    )
    .execute(pool)
    .await;

    // Aggiungi indice unico su email solo per valori non nulli
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email) WHERE email IS NOT NULL"
    )
    .execute(pool)
    .await;

    Ok(())
}

pub async fn save_user_to_postgres(request: &RegisterRequest) -> Result<User, String> {
    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    // Hash della password
    let password_hash = hash(&request.password, DEFAULT_COST)
        .map_err(|e| format!("Errore hashing password: {}", e))?;

    let rec = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, image, password_hash, created_at, updated_at, sync_status, last_sync)
         VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 'synced', CURRENT_TIMESTAMP)
         RETURNING id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync"
    )
    .bind(&request.username)
    .bind(&request.email)
    .bind(&request.image)
    .bind(password_hash)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("Errore inserimento utente in PostgreSQL: {}", e))?;

    Ok(rec)
}

pub async fn check_user_exists_in_postgres(username: &str) -> Result<bool, String> {
    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE username = $1"
    )
    .bind(username)
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("Errore verifica esistenza utente in PostgreSQL: {}", e))?;

    Ok(result > 0)
}

#[tauri::command]
pub async fn register_user(request: RegisterRequest) -> Result<AuthResponse, String> {
    // Validazione input
    if request.username.trim().is_empty() {
        return Err("Username obbligatorio".to_string());
    }
    if request.password.len() < 6 {
        return Err("Password deve essere di almeno 6 caratteri".to_string());
    }

    // Prima controlla se l'utente esiste già in PostgreSQL
    let existing_user = check_user_exists_in_postgres(&request.username).await?;
    if existing_user {
        // L'utente esiste già in PostgreSQL, sincronizza e restituisci errore
        println!("Utente {} esiste già in PostgreSQL, sincronizzazione automatica...", request.username);
        match crate::sync::sync_users_from_postgres().await {
            Ok(_) => println!("Sincronizzazione completata per utente esistente"),
            Err(sync_err) => println!("Errore sincronizzazione automatica: {}", sync_err),
        }
        return Err("Username già in uso".to_string());
    }

    // Se non esiste, procedi con la registrazione normale
    let pg_user = save_user_to_postgres(&request).await?;

    // Poi salva su SQLite
    println!("Tentativo salvataggio utente {} su SQLite", pg_user.id);
    match crate::sync::save_user_to_sqlite(&pg_user).await {
        Ok(_) => println!("Utente {} salvato correttamente su SQLite", pg_user.id),
        Err(e) => {
            println!("Errore salvataggio utente {} su SQLite: {}", pg_user.id, e);
            // Non facciamo fallire la registrazione per errori SQLite, ma loggiamo
        }
    }

    // Genera token semplice (in produzione usare JWT)
    let token = format!("user_{}_{}", pg_user.id, chrono::Utc::now().timestamp());

    Ok(AuthResponse { user: pg_user, token })
}

#[tauri::command]
pub async fn login_user(request: LoginRequest) -> Result<AuthResponse, String> {
    // Inizializza SQLite se necessario
    crate::sync::init_sqlite_db().await?;

    if request.username.trim().is_empty() || request.password.trim().is_empty() {
        return Err("Username e password obbligatori".to_string());
    }

    let conn = crate::sync::get_sqlite_connection()?;

    // Trova utente per username
    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE username = ?1"
    ).map_err(|e| format!("Errore preparazione query: {}", e))?;

    let user_iter = stmt.query_map([request.username], |row| {
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

    let mut found_user = None;
    for user_result in user_iter {
        match user_result {
            Ok(user) => {
                found_user = Some(user);
                break;
            }
            Err(e) => {
                eprintln!("Errore parsing utente: {}", e);
                continue;
            }
        }
    }

    match found_user {
        Some(user) => {
            // Verifica password
            let is_valid = verify(&request.password, &user.password_hash)
                .map_err(|e| format!("Errore verifica password: {}", e))?;

            if is_valid {
                // Genera token semplice
                let token = format!("user_{}_{}", user.id, chrono::Utc::now().timestamp());
                Ok(AuthResponse { user, token })
            } else {
                Err("Password non corretta".to_string())
            }
        }
        None => Err("Utente non trovato".to_string()),
    }
}

#[tauri::command]
pub async fn get_current_user(user_id: i32) -> Result<User, String> {
    // Inizializza SQLite se necessario
    crate::sync::init_sqlite_db().await?;

    let conn = crate::sync::get_sqlite_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE id = ?1"
    ).map_err(|e| format!("Errore preparazione query: {}", e))?;

    let user = stmt.query_row([user_id], |row| {
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
    }).map_err(|e| format!("Errore recupero utente: {}", e))?;

    Ok(user)
}

#[tauri::command]
pub async fn update_user_profile(user_id: i32, email: Option<String>, current_password: String, new_password: Option<String>) -> Result<User, String> {
    // Inizializza SQLite se necessario
    crate::sync::init_sqlite_db().await?;

    let conn = crate::sync::get_sqlite_connection()?;

    // Recupera l'utente corrente
    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE id = ?1"
    ).map_err(|e| format!("Errore preparazione query recupero utente: {}", e))?;

    let mut current_user = stmt.query_row([user_id], |row| {
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
    }).map_err(|e| format!("Errore recupero utente: {}", e))?;

    // Verifica password corrente
    let is_valid = verify(&current_password, &current_user.password_hash)
        .map_err(|e| format!("Errore verifica password: {}", e))?;

    if !is_valid {
        return Err("Password corrente non corretta".to_string());
    }

    // Aggiorna email se cambiata
    let empty_string = String::new();
    let current_email = current_user.email.as_ref().unwrap_or(&empty_string);
    let new_email = email.as_ref().unwrap_or(&empty_string);
    if new_email != current_email {
        // Verifica che la nuova email non sia già in uso
        if !new_email.is_empty() {
            let mut stmt = conn.prepare("SELECT id FROM users WHERE email = ?1 AND id != ?2")
                .map_err(|e| format!("Errore preparazione query verifica email: {}", e))?;

            let email_exists: Option<i32> = stmt.query_row(rusqlite::params![new_email, user_id], |row| row.get(0)).ok();

            if email_exists.is_some() {
                return Err("Email già in uso".to_string());
            }
        }
        current_user.email = email.clone();
    }

    // Aggiorna password se fornita
    let password_hash = if let Some(new_pass) = new_password {
        if new_pass.len() < 6 {
            return Err("Nuova password deve essere di almeno 6 caratteri".to_string());
        }
        hash(&new_pass, DEFAULT_COST)
            .map_err(|e| format!("Errore hashing password: {}", e))?
    } else {
        current_user.password_hash.clone()
    };

    let now = Utc::now();
    let updated_at_str = now.to_rfc3339();

    // Aggiorna utente
    conn.execute(
        "UPDATE users SET email = ?1, password_hash = ?2, updated_at = ?3, sync_status = 'synced', last_sync = ?4
         WHERE id = ?5",
        rusqlite::params![
            current_user.email,
            password_hash,
            updated_at_str,
            now.to_rfc3339(),
            user_id
        ],
    ).map_err(|e| format!("Errore aggiornamento utente: {}", e))?;

    // Recupera l'utente aggiornato
    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE id = ?1"
    ).map_err(|e| format!("Errore preparazione query recupero utente aggiornato: {}", e))?;

    let updated_user = stmt.query_row([user_id], |row| {
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
    }).map_err(|e| format!("Errore recupero utente aggiornato: {}", e))?;

    Ok(updated_user)
}

#[tauri::command]
pub async fn get_user_profile(user_id: i32) -> Result<User, String> {
    // Inizializza SQLite se necessario
    crate::sync::init_sqlite_db().await?;

    let conn = crate::sync::get_sqlite_connection()?;

    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE id = ?1"
    ).map_err(|e| format!("Errore preparazione query recupero profilo utente: {}", e))?;

    let user = stmt.query_row([user_id], |row| {
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
    }).map_err(|e| format!("Errore recupero profilo utente: {}", e))?;

    Ok(user)
}

#[tauri::command]
pub async fn get_users_from_postgres() -> Result<Vec<User>, String> {
    get_or_init_db_pool().await?;
    let pool = get_db_pool().await?;

    let users = sqlx::query_as::<_, User>(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users ORDER BY created_at DESC"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("Errore recupero utenti da PostgreSQL: {}", e))?;

    Ok(users)
}

#[tauri::command]
pub async fn get_all_users() -> Result<Vec<User>, String> {
    // Usa sempre SQLite per gli utenti (come per le macchine)
    crate::sync::get_users_from_sqlite().await
}

#[tauri::command]
pub async fn delete_user(user_id: i32) -> Result<(), String> {
    // Inizializza SQLite se necessario
    crate::sync::init_sqlite_db().await?;

    let conn = crate::sync::get_sqlite_connection()?;

    // Verifica che l'utente esista
    let mut stmt = conn.prepare("SELECT id FROM users WHERE id = ?1")
        .map_err(|e| format!("Errore preparazione query verifica utente: {}", e))?;

    let user_exists: Option<i32> = stmt.query_row([user_id], |row| row.get(0)).ok();

    if user_exists.is_none() {
        return Err("Utente non trovato".to_string());
    }

    // Soft delete dell'utente
    conn.execute("UPDATE users SET deleted = 1 WHERE id = ?1", rusqlite::params![user_id])
        .map_err(|e| format!("Errore soft delete utente: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn update_user(user_id: i32, username: String, email: Option<String>, image: Option<Vec<u8>>) -> Result<User, String> {
    // Inizializza SQLite se necessario
    crate::sync::init_sqlite_db().await?;

    let conn = crate::sync::get_sqlite_connection()?;

    // Validazione input
    if username.trim().is_empty() {
        return Err("Username obbligatorio".to_string());
    }

    // Verifica che l'utente esista
    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE id = ?1"
    ).map_err(|e| format!("Errore preparazione query recupero utente: {}", e))?;

    let current_user_result = stmt.query_row([user_id], |row| {
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
    });

    if current_user_result.is_err() {
        return Err("Utente non trovato".to_string());
    }

    // Verifica che username non sia già in uso da un altro utente
    let mut stmt = conn.prepare("SELECT id FROM users WHERE username = ?1 AND id != ?2")
        .map_err(|e| format!("Errore preparazione query verifica username: {}", e))?;

    let username_exists: Option<i32> = stmt.query_row(rusqlite::params![&username, user_id], |row| row.get(0)).ok();

    if username_exists.is_some() {
        return Err("Username già in uso".to_string());
    }

    // Se email è fornita, verifica che non sia già in uso da un altro utente
    if let Some(ref email_val) = email {
        if !email_val.trim().is_empty() {
            let mut stmt = conn.prepare("SELECT id FROM users WHERE email = ?1 AND id != ?2")
                .map_err(|e| format!("Errore preparazione query verifica email: {}", e))?;

            let email_exists: Option<i32> = stmt.query_row(rusqlite::params![email_val, user_id], |row| row.get(0)).ok();

            if email_exists.is_some() {
                return Err("Email già in uso".to_string());
            }
        }
    }

    let now = Utc::now();
    let updated_at_str = now.to_rfc3339();

    // Aggiorna l'utente
    conn.execute(
        "UPDATE users SET username = ?1, email = ?2, image = ?3, updated_at = ?4, sync_status = 'synced', last_sync = ?5
         WHERE id = ?6",
        rusqlite::params![
            username,
            email,
            image,
            updated_at_str,
            now.to_rfc3339(),
            user_id
        ],
    ).map_err(|e| format!("Errore aggiornamento utente: {}", e))?;

    // Recupera l'utente aggiornato
    let mut stmt = conn.prepare(
        "SELECT id, username, email, image, password_hash, created_at, updated_at, sync_status, last_sync, deleted
         FROM users WHERE id = ?1"
    ).map_err(|e| format!("Errore preparazione query recupero utente aggiornato: {}", e))?;

    let updated_user = stmt.query_row([user_id], |row| {
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
    }).map_err(|e| format!("Errore recupero utente aggiornato: {}", e))?;

    Ok(updated_user)
}

#[tauri::command]
pub async fn select_user_image() -> Result<Option<Vec<u8>>, String> {
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
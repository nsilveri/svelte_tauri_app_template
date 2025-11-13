use sqlx::{postgres::PgPoolOptions};

pub fn get_db_config() -> Result<String, String> {
    let host = crate::settings::get_setting_internal("db_host")?;
    let port = crate::settings::get_setting_internal("db_port")?;
    let name = crate::settings::get_setting_internal("db_name")?;
    let user = crate::settings::get_setting_internal("db_user")?;
    let password = crate::settings::get_setting_internal("db_password")?;

    Ok(format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, name))
}

pub async fn create_database_and_tables() -> Result<String, String> {
    let host = crate::settings::get_setting_internal("db_host")?;
    let port = crate::settings::get_setting_internal("db_port")?;
    let user = crate::settings::get_setting_internal("db_user")?;
    let password = crate::settings::get_setting_internal("db_password")?;
    let db_name = crate::settings::get_setting_internal("db_name")?;

    // Prima connessione senza specificare il database per poterlo creare
    let admin_url = format!("postgres://{}:{}@{}:{}", user, password, host, port);

    // Connetti al database postgres (default)
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&admin_url)
        .await
        .map_err(|e| format!("Errore connessione al database postgres: {}", e))?;

    // Crea il database se non esiste
    sqlx::query(&format!("CREATE DATABASE {} OWNER {}", db_name, user))
        .execute(&pool)
        .await
        .map_err(|e| {
            // Se il database già esiste, non è un errore
            if e.to_string().contains("already exists") {
                format!("Database '{}' già esistente", db_name)
            } else {
                format!("Errore creazione database: {}", e)
            }
        })?;

    // Chiudi la connessione al database admin
    pool.close().await;

    // Ora connetti al database appena creato
    let db_url = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, db_name);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| format!("Errore connessione al database {}: {}", db_name, e))?;

    // Crea la tabella machines se non esiste
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS machines (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            image BYTEA,
            user_name VARCHAR(255),
            assistant_name VARCHAR(255),
            ip_address INET,
            database_name VARCHAR(255),
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Errore creazione tabella machines: {}", e))?;

    // Crea la tabella users se non esiste
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username VARCHAR(255) UNIQUE NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Errore creazione tabella users: {}", e))?;

    // Crea la tabella work_types se non esiste
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS work_types (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) UNIQUE NOT NULL,
            image BYTEA,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| format!("Errore creazione tabella work_types: {}", e))?;

    // Chiudi la connessione
    pool.close().await;

    Ok(format!("Database '{}' e tabelle 'machines', 'users' e 'work_types' creati con successo", db_name))
}

#[tauri::command]
pub async fn create_database() -> Result<String, String> {
    create_database_and_tables().await
}

#[tauri::command]
pub async fn check_database_exists() -> Result<bool, String> {
    let host = crate::settings::get_setting_internal("db_host")?;
    let port = crate::settings::get_setting_internal("db_port")?;
    let name = crate::settings::get_setting_internal("db_name")?;
    let user = crate::settings::get_setting_internal("db_user")?;
    let password = crate::settings::get_setting_internal("db_password")?;

    let db_url = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, name);

    // Try to connect to the target database. If connect succeeds => exists.
    match PgPoolOptions::new().max_connections(1).connect(&db_url).await {
        Ok(pool) => {
            pool.close().await;
            Ok(true)
        }
        Err(e) => {
            let s = e.to_string().to_lowercase();
            // If the error indicates the database does not exist, return false.
            if s.contains("does not exist") || s.contains("database \"") && s.contains("does not exist") {
                Ok(false)
            } else if s.contains("invalid password") || s.contains("password authentication failed") || s.contains("authentication") {
                Err(format!("Authentication error: {}", e))
            } else {
                // For other errors, try to interpret as non-existence conservatively
                Ok(false)
            }
        }
    }
}
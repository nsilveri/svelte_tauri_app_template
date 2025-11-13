use serde::{Deserialize, Serialize};
use std::fs;
use rfd;

use crate::machines::get_db_pool;

// Funzione per assicurarsi che la tabella work_types esista
pub async fn ensure_work_types_table() -> Result<(), String> {
    let pool = get_db_pool().await?;

    // Crea la tabella se non esiste
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
    .execute(&*pool)
    .await
    .map_err(|e| format!("Errore creazione tabella work_types: {}", e))?;

    // Verifica se la colonna image esiste, e se no, la aggiunge
    // Prima controlliamo se la colonna esiste già
    let check_column_query = r#"
        SELECT column_name
        FROM information_schema.columns
        WHERE table_name = 'work_types' AND column_name = 'image'
    "#;

    let column_exists: Option<String> = sqlx::query_scalar(check_column_query)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| format!("Errore verifica colonna image: {}", e))?;

    // Se la colonna non esiste, la aggiungiamo
    if column_exists.is_none() {
        sqlx::query(
            r#"
            ALTER TABLE work_types ADD COLUMN image BYTEA
            "#
        )
        .execute(&*pool)
        .await
        .map_err(|e| format!("Errore aggiunta colonna image: {}", e))?;
    }

    Ok(())
}

// Struct per rappresentare un work type
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkType {
    pub id: i32,
    pub name: String,
    pub image: Option<Vec<u8>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Struct per creare un nuovo work type
#[derive(Debug, Deserialize)]
pub struct NewWorkType {
    pub name: String,
    pub image: Option<Vec<u8>>,
}

// Struct per aggiornare un work type
#[derive(Debug, Deserialize)]
pub struct UpdateWorkType {
    pub name: String,
    pub image: Option<Vec<u8>>,
}

// Ottieni tutti i work types
#[tauri::command]
pub async fn get_work_types() -> Result<Vec<WorkType>, String> {
    // Prima prova a ottenere i work types dalla cache SQLite
    match crate::sync::get_work_types_from_sqlite().await {
        Ok(work_types) if !work_types.is_empty() => {
            return Ok(work_types);
        }
        Ok(_) | Err(_) => {
            // Se la cache è vuota o errore, continua con PostgreSQL
        }
    }

    // Se non disponibili in cache, ottieni da PostgreSQL
    // Assicurati che la tabella esista
    ensure_work_types_table().await?;

    let pool = get_db_pool().await?;

    let query = r#"
        SELECT id, name, image, created_at, updated_at
        FROM work_types
        ORDER BY name ASC
    "#;

    match sqlx::query_as::<_, WorkType>(query)
        .fetch_all(&*pool)
        .await
    {
        Ok(work_types) => Ok(work_types),
        Err(e) => Err(format!("Errore nel recupero dei tipi di lavorazione: {}", e)),
    }
}

// Ottieni un singolo work type per ID
#[tauri::command]
pub async fn get_work_type(id: i32) -> Result<WorkType, String> {
    // Prima cerca in SQLite (cache locale)
    match crate::sync::get_work_types_from_sqlite().await {
        Ok(work_types) => {
            if let Some(work_type) = work_types.into_iter().find(|wt| wt.id == id) {
                return Ok(work_type);
            }
        }
        Err(_) => {
            // Se errore lettura SQLite, continua con PostgreSQL
        }
    }

    // Se non trovato in SQLite, cerca in PostgreSQL
    // Assicurati che la tabella esista
    ensure_work_types_table().await?;

    let pool = get_db_pool().await?;

    let query = r#"
        SELECT id, name, image, created_at, updated_at
        FROM work_types
        WHERE id = $1
    "#;

    match sqlx::query_as::<_, WorkType>(query)
        .bind(id)
        .fetch_one(&*pool)
        .await
    {
        Ok(work_type) => Ok(work_type),
        Err(sqlx::Error::RowNotFound) => Err("Tipo di lavorazione non trovato".to_string()),
        Err(e) => Err(format!("Errore nel recupero del tipo di lavorazione: {}", e)),
    }
}

// Aggiungi un nuovo work type
#[tauri::command]
pub async fn add_work_type(work_type: NewWorkType) -> Result<WorkType, String> {
    // Assicurati che la tabella esista
    ensure_work_types_table().await?;

    let pool = get_db_pool().await?;

    // Verifica se il nome già esiste
    let check_query = "SELECT id FROM work_types WHERE name = $1";
    match sqlx::query(check_query)
        .bind(&work_type.name)
        .fetch_optional(&*pool)
        .await
    {
        Ok(Some(_)) => return Err("Un tipo di lavorazione con questo nome esiste già".to_string()),
        Ok(None) => {},
        Err(e) => return Err(format!("Errore nella verifica del nome: {}", e)),
    }

    let query = r#"
        INSERT INTO work_types (name, image, created_at, updated_at)
        VALUES ($1, $2, NOW(), NOW())
        RETURNING id, name, image, created_at, updated_at
    "#;

    match sqlx::query_as::<_, WorkType>(query)
        .bind(&work_type.name)
        .bind(&work_type.image)
        .fetch_one(&*pool)
        .await
    {
        Ok(work_type_created) => {
            // Salva anche in SQLite per mantenere la cache locale aggiornata
            crate::sync::save_work_type_to_sqlite(&work_type_created).await
                .map_err(|e| format!("Errore salvataggio in SQLite: {}", e))?;

            Ok(work_type_created)
        }
        Err(e) => Err(format!("Errore nell'aggiunta del tipo di lavorazione: {}", e)),
    }
}

// Aggiorna un work type esistente
#[tauri::command]
pub async fn update_work_type(id: i32, work_type: UpdateWorkType) -> Result<WorkType, String> {
    // Assicurati che la tabella esista
    ensure_work_types_table().await?;

    let pool = get_db_pool().await?;

    // Verifica se il nome già esiste per un altro record
    let check_query = "SELECT id FROM work_types WHERE name = $1 AND id != $2";
    match sqlx::query(check_query)
        .bind(&work_type.name)
        .bind(id)
        .fetch_optional(&*pool)
        .await
    {
        Ok(Some(_)) => return Err("Un tipo di lavorazione con questo nome esiste già".to_string()),
        Ok(None) => {},
        Err(e) => return Err(format!("Errore nella verifica del nome: {}", e)),
    }

    let query = r#"
        UPDATE work_types
        SET name = $1, image = $2, updated_at = NOW()
        WHERE id = $3
        RETURNING id, name, image, created_at, updated_at
    "#;

    match sqlx::query_as::<_, WorkType>(query)
        .bind(&work_type.name)
        .bind(&work_type.image)
        .bind(id)
        .fetch_one(&*pool)
        .await
    {
        Ok(work_type_updated) => {
            // Aggiorna anche su SQLite per mantenere la cache locale aggiornata
            crate::sync::save_work_type_to_sqlite(&work_type_updated).await
                .map_err(|e| format!("Errore salvataggio in SQLite: {}", e))?;

            Ok(work_type_updated)
        }
        Err(sqlx::Error::RowNotFound) => Err("Tipo di lavorazione non trovato".to_string()),
        Err(e) => Err(format!("Errore nell'aggiornamento del tipo di lavorazione: {}", e)),
    }
}

// Elimina un work type
#[tauri::command]
pub async fn delete_work_type(id: i32) -> Result<(), String> {
    // Assicurati che la tabella esista
    ensure_work_types_table().await?;

    let pool = get_db_pool().await?;

    let query = "DELETE FROM work_types WHERE id = $1";

    match sqlx::query(query)
        .bind(id)
        .execute(&*pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() == 0 {
                Err("Tipo di lavorazione non trovato".to_string())
            } else {
                // Elimina anche da SQLite per mantenere la cache locale aggiornata
                crate::sync::delete_work_type_from_sqlite(id).await
                    .map_err(|e| format!("Errore eliminazione da SQLite: {}", e))?;

                Ok(())
            }
        }
        Err(e) => Err(format!("Errore nell'eliminazione del tipo di lavorazione: {}", e)),
    }
}

// Seleziona un'immagine per un work type
#[tauri::command]
pub async fn select_work_type_image() -> Result<Option<String>, String> {
    let file_dialog = rfd::FileDialog::new()
        .add_filter("Immagini", &["png", "jpg", "jpeg", "gif", "bmp", "webp"])
        .set_title("Seleziona immagine per il tipo di lavorazione");

    match file_dialog.pick_file() {
        Some(path) => Ok(Some(path.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

// Carica un'immagine da file path
#[tauri::command]
pub async fn load_image_from_path(path: String) -> Result<Vec<u8>, String> {
    match fs::read(&path) {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Errore nel caricamento dell'immagine: {}", e)),
    }
}
// Template per Svelte + Tauri
// Aggiungi qui le tue funzioni Tauri personalizzate

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Simuliamo un semplice storage delle impostazioni
static SETTINGS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Ciao, {}! Questo è un template Svelte + Tauri.", name)
}

#[tauri::command]
fn get_app_info() -> HashMap<String, String> {
    let mut info = HashMap::new();
    info.insert("name".to_string(), "Svelte Tauri Template".to_string());
    info.insert("version".to_string(), "1.0.0".to_string());
    info.insert("description".to_string(), "Un template moderno per applicazioni desktop".to_string());
    info
}

#[tauri::command]
fn save_setting(key: String, value: String) -> Result<String, String> {
    match SETTINGS.lock() {
        Ok(mut settings) => {
            settings.insert(key.clone(), value.clone());
            Ok(format!("Impostazione '{}' salvata con valore '{}'", key, value))
        }
        Err(_) => Err("Errore nel salvataggio dell'impostazione".to_string())
    }
}

#[tauri::command]
fn get_setting(key: String) -> Result<String, String> {
    match SETTINGS.lock() {
        Ok(settings) => {
            match settings.get(&key) {
                Some(value) => Ok(value.clone()),
                None => {
                    // Valori di default
                    match key.as_str() {
                        "language" => Ok("en".to_string()),
                        "theme" => Ok("light".to_string()),
                        "notifications" => Ok("true".to_string()),
                        _ => Err(format!("Impostazione '{}' non trovata", key))
                    }
                }
            }
        }
        Err(_) => Err("Errore nel recupero dell'impostazione".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_app_info, save_setting, get_setting])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
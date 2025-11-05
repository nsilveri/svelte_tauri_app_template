# 🚀 Guida Rapida al Template

Benvenuto nel template Svelte + Tauri! Ecco come iniziare rapidamente:

## ⚡ Avvio Rapido

```bash
# 1. Installa le dipendenze
npm install

# 2. Avvia in modalità sviluppo
npm run dev

# 3. Build per produzione
npm run build
```

## 📄 Struttura delle Pagine

Il template include 3 pagine pronte all'uso:

- **Home** (`/home`) - Pagina principale con card di esempio
- **Settings** (`/settings`) - Configurazioni base (tema, lingua)  
- **About** (`/about`) - Informazioni sul template

## 🎨 Personalizzazione Veloce

### Cambiare il nome dell'app
1. Modifica `src-tauri/tauri.conf.json` → `productName`
2. Aggiorna `package.json` → `name`

### Aggiungere una nuova pagina
1. Crea `src/routes/nuova-pagina/+page.svelte`
2. Aggiungi il link in `src/routes/+layout.svelte`

### Modificare i colori
- I gradienti sono in ogni `+page.svelte`
- TailwindCSS config in `tailwind.config.js`

### Aggiungere funzioni Tauri
1. Aggiungi command in `src-tauri/src/lib.rs`
2. Registra in `invoke_handler![]`
3. Chiama da frontend con `invoke('nome_comando')`

## 🌍 Internazionalizzazione

- File traduzioni: `src/lib/i18n/`
- Usa: `{$_('chiave.traduzione')}`
- Aggiungi lingue modificando `src/lib/i18n/index.js`

## 🔧 Configurazioni Utili

### Icona app
Sostituisci i file in `src-tauri/icons/`

### Dimensioni finestra
Modifica `src-tauri/tauri.conf.json` → `app.windows[0]`

### Permessi 
Aggiungi in `src-tauri/tauri.conf.json` → `app.security`

## 📝 Prossimi Passi

1. Personalizza il contenuto delle pagine
2. Aggiungi le tue funzionalità Tauri
3. Modifica colori e stile
4. Testa su diverse piattaforme
5. Build finale e distribuzione

**Buon sviluppo! 🎉**
# Svelte + Tauri Template

Un template moderno per creare applicazioni desktop con Svelte e Tauri.

## 🚀 Caratteristiche

- **Svelte 5** - Framework reattivo e performante
- **Tauri** - Backend nativo e impacchettamento desktop
- **TailwindCSS** - Styling moderno e responsive  
- **Internazionalizzazione** - Supporto multilingua con svelte-i18n
- **Layout Responsive** - Si adatta a diverse dimensioni schermo
- **Navigazione** - 3 pagine di esempio (Home, Settings, About)
- **Design Moderno** - Interfaccia pulita con effetti glassmorphism
- **TypeScript Support** - Configurazione TypeScript pronta all'uso

## 📦 Struttura del Progetto

```
src/
├── lib/
│   └── i18n/           # File di traduzione
│       ├── index.js
│       ├── en.json
│       └── it.json
└── routes/
    ├── +layout.svelte  # Layout principale con navigazione
    ├── +page.svelte    # Redirect alla home
    ├── home/
    │   └── +page.svelte
    ├── settings/
    │   └── +page.svelte
    └── about/
        └── +page.svelte
```

## 🛠️ Installazione

1. **Clona il template**
   ```bash
   git clone <repository-url>
   cd svelte-tauri-template
   ```

2. **Installa le dipendenze**
   ```bash
   npm install
   ```

3. **Avvia in modalità sviluppo**
   ```bash
   npm run dev
   ```

4. **Build per produzione**
   ```bash
   npm run build
   ```

## 🎨 Personalizzazione

### Aggiungere nuove pagine

1. Crea una nuova cartella in `src/routes/`
2. Aggiungi un file `+page.svelte`
3. Aggiorna la navigazione in `+layout.svelte`

### Modificare i colori e lo stile

Il template usa TailwindCSS. Puoi:
- Modificare `tailwind.config.js` per personalizzare i colori
- Aggiornare le classi CSS nei componenti
- Cambiare i gradienti di sfondo nelle pagine

### Aggiungere nuove lingue

1. Aggiungi un nuovo file JSON in `src/lib/i18n/`
2. Importalo in `src/lib/i18n/index.js`
3. Aggiorna le opzioni lingua nelle impostazioni

### Configurare Tauri

- Modifica `src-tauri/tauri.conf.json` per:
  - Cambiare nome e icona dell'app
  - Configurare finestre e permessi
  - Aggiungere plugin Tauri

## 📝 Scripts Disponibili

- `npm run dev` - Avvia sviluppo con hot reload
- `npm run build` - Build di produzione
- `npm run preview` - Anteprima build locale
- `npm run check` - Controllo TypeScript/Svelte
- `npm run lint` - Linting del codice
- `npm run format` - Formattazione automatica

## 🔧 Configurazione

### Tauri
Configurazione principale in `src-tauri/tauri.conf.json`:
- Cambia `productName` per il nome dell'app
- Modifica `identifier` per un ID univoco
- Personalizza icone in `src-tauri/icons/`

### SvelteKit
Configurazione in `svelte.config.js`:
- Adapter statico per Tauri configurato
- Prerendering abilitato

### TailwindCSS
Configurazione in `tailwind.config.js`:
- Content paths configurati per Svelte
- Plugin base inclusi

## 🚀 Deploy

Per creare un eseguibile:

```bash
npm run build
```

I file generati saranno in `src-tauri/target/release/`.

## 📱 Piattaforme Supportate

- Windows
- macOS  
- Linux

## 🤝 Contribuzioni

Le contribuzioni sono benvenute! Sentiti libero di:
- Aprire issue per bug o suggerimenti
- Creare pull request per miglioramenti
- Condividere il template

## 📄 Licenza

MIT License - vedi [LICENSE](LICENSE) per i dettagli.

## 🙏 Ringraziamenti

- [Svelte](https://svelte.dev/) - Il framework web
- [Tauri](https://tauri.app/) - Il toolkit per app native
- [TailwindCSS](https://tailwindcss.com/) - Il framework CSS
- [svelte-i18n](https://github.com/kaisermann/svelte-i18n) - Libreria i18n

---

**Buon sviluppo! 🎉**

### Key changes

- Single `Add` button with dropdown: contains `Add one` (single form) and `Magic add` (bulk modal).
- "Bulk add" modal with smart parsing (`src/routes/home/table/+page.svelte`): supports several pasted formats (labelled pairs `desc:`/`code:`, blank-line separated blocks, single-line `desc code`, or alternating lines). It's designed to simplify fast imports of many cheats.
- Migration to stable identifiers: lists now use `id` as the Svelte key to avoid duplicate/duplicate DOM elements.
- Robust drag & drop: integration with SortableJS (handle, fallbackOnBody, ghost/chosen classes) and order saving via `update_record_order` that sends an array of `id`s to the backend.
- Row selection + header Up/Down controls: select a single row and move it with header arrows; a toast appears if nothing is selected.
- Per-row Up/Down buttons and actions: each row has actions (View, Edit, Move up/down, Delete).
- Edit page uses `id`: the edit link and page now look up the record by `id` (no longer by `desc`).
- Home refresh after import: after importing a `.cht` the table list is reloaded automatically.
- Logo functionality: frontend `fetchAndSetLogo()` calls the backend which tries to fetch images from RAWG/TheGamesDB and save them for the table.
- Backend localization → i18n keys: the backend now returns error keys (e.g. `settings.api_key_missing`, `home.no_game_found_rawg`, `table.duplicate_desc`) instead of hard-coded Italian strings. These keys are translated in `src/lib/i18n/it.json` and `src/lib/i18n/en.json`.

## Contributing

If you'd like to contribute: open an issue or a PR with a clear description of the feature or bug and, if possible, include reproduction steps. For changes to the bulk parser, include real input examples you want supported.

---

For questions or support, contact the maintainer.
<script>
  import { _ } from 'svelte-i18n';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { writable } from 'svelte/store';

  /** @type {Array<{id: number, name: string, image?: Uint8Array, ip_address?: string, database_name?: string, description?: string}>} */
  let machines = [];
  let loading = true;
  let error = '';
  let dbStatuses = writable(new Map()); // Store per tracciare lo stato di connessione dei database
  
  // Cache and sync state
  let cacheLoaded = false;
  let cacheSynced = false;
  let serverReachable = true;

  // PostgreSQL connection state
  let postgresConnected = false;
  let postgresCheckError = '';

  // Authentication state
  /** @type {{username: string, id?: number} | null} */
  let user = null;
  let showAuthModal = false;
  let isLoginMode = true; // true = login, false = register

  // Current time
  let currentTime = '';

	function updateTime() {
		currentTime = new Date().toLocaleString();
	}  // Form fields
  let username = '';
  let email = '';
  let password = '';
  let confirmPassword = '';
  /** @type {Uint8Array | null} */
  let userImage = null; // Immagine dell'utente
  let authError = '';
  let authLoading = false;

  onMount(async () => {
    // Controlla se l'utente è già loggato
    checkAuth();
    fetchMachines();
    checkPostgresConnection();
    updateTime();
    const interval = setInterval(updateTime, 1000);
    return () => clearInterval(interval);
  });

  async function testServerConnection() {
    try {
      // Test semplice: prova a ottenere le macchine dal server
      await invoke('get_machines');
      return true;
    } catch (e) {
      console.log('Server PostgreSQL non raggiungibile:', e);
      return false;
    }
  }

  function checkAuth() {
    if (typeof window !== 'undefined') {
      const userData = localStorage.getItem('user');
      if (userData) {
        try {
          user = JSON.parse(userData);
        } catch (e) {
          console.error('Errore parsing user data:', e);
          user = null;
        }
      }
    }
  }

  async function fetchMachines() {
    try {
      loading = true;
      error = '';

      // Carica sempre dalla cache SQLite
      const cachedMachines = await invoke('get_machines');
      machines = cachedMachines.map(machine => ({
        ...machine,
        image: machine.image ? new Uint8Array(machine.image) : null
      }));
      cacheLoaded = true;
      console.log('Macchine caricate dalla cache SQLite:', machines.length);

      // Controlla lo stato di connessione per ogni database in background
      checkDatabaseConnections();
      console.error('Errore nel caricamento dalla cache SQLite:', e);
      error = 'Errore nel caricamento delle macchine: ' + String(e);
    } finally {
      loading = false;
    }
  }

  async function checkSyncStatus() {
    // Non più necessario - sempre SQLite
    cacheSynced = true;
  }

  // Controlla lo stato della connessione PostgreSQL
  async function checkPostgresConnection() {
    try {
      const connected = await invoke('check_database_exists');
      postgresConnected = connected;
      postgresCheckError = '';
    } catch (error) {
      console.error('Errore durante il controllo della connessione PostgreSQL:', error);
      postgresConnected = false;
      postgresCheckError = String(error);
    }
  }

  // Controlla lo stato di connessione per ogni database in background
  async function checkDatabaseConnections() {
    if (machines.length === 0) return;

    const statusMap = new Map();
    for (const machine of machines) {
      if (machine.database_name) {
        try {
          const isConnected = await invoke('check_database_connection', { databaseName: machine.database_name });
          statusMap.set(machine.id, isConnected);
        } catch (e) {
          console.error(`Errore controllo connessione DB per ${machine.name}:`, e);
          statusMap.set(machine.id, false);
        }
      }
    }
    dbStatuses.set(statusMap);
  }

	/** @param {{id: number, name: string}} machine */
	function useMachine(machine) {
    // TODO: Implementare la logica per utilizzare la macchina
    alert(`Utilizzo della macchina: ${machine.name}`);
    console.log('Utilizzo macchina:', machine);
  }

	/** @param {Uint8Array | null | undefined} bytes */
	function getImageDataUrl(bytes) {
		if (!bytes) return '';
		// Converti Uint8Array in base64 in modo sicuro per evitare stack overflow
		try {
			// Usa un approccio più sicuro per array grandi
			let binary = '';
			const chunkSize = 1024; // Processa in chunk di 1KB
			for (let i = 0; i < bytes.length; i += chunkSize) {
				const chunk = bytes.slice(i, i + chunkSize);
				binary += String.fromCharCode(...chunk);
			}
			const base64 = btoa(binary);
			return `data:image/jpeg;base64,${base64}`;
		} catch (error) {
			console.error('Errore nella conversione dell\'immagine:', error);
			return ''; // Ritorna stringa vuota in caso di errore
		}
	}

  async function selectUserImage() {
    try {
      const imageBytes = await invoke("select_user_image");
      if (imageBytes) {
        userImage = new Uint8Array(imageBytes);
      }
    } catch (error) {
      console.error('Errore nella selezione dell\'immagine:', error);
    }
  }

  // Authentication functions
  async function handleAuth() {
    if (isLoginMode) {
      await handleLogin();
    } else {
      await handleRegister();
    }
  }

  async function handleLogin() {
    if (!username || !password) {
      authError = 'Inserisci username e password';
      return;
    }

    authLoading = true;
    authError = '';

    try {
      const result = await invoke('login_user', {
        request: {
          username: username.trim(),
          password: password.trim()
        }
      });

      // Salva l'utente nel localStorage
      localStorage.setItem('user', JSON.stringify(result.user));
      user = result.user;

      // Chiudi il modale
      showAuthModal = false;
      resetForm();
    } catch (err) {
      authError = String(err) || 'Errore durante il login';
    } finally {
      authLoading = false;
    }
  }

  async function handleRegister() {
    if (!username || !password || !confirmPassword) {
      authError = 'Username, password e conferma password sono obbligatori';
      return;
    }

    if (password !== confirmPassword) {
      authError = 'Le password non coincidono';
      return;
    }

    if (password.length < 6) {
      authError = 'La password deve essere di almeno 6 caratteri';
      return;
    }

    authLoading = true;
    authError = '';

    try {
      const result = await invoke('register_user', {
        request: {
          username: username.trim(),
          email: email.trim().toLowerCase() || null,
          password: password.trim(),
          image: userImage ? Array.from(userImage) : null
        }
      });

      // Dopo la registrazione, effettua automaticamente il login
      await handleLogin();
    } catch (err) {
      authError = String(err) || 'Errore durante la registrazione';
    } finally {
      authLoading = false;
    }
  }

  function logout() {
    if (typeof window !== 'undefined') {
      localStorage.removeItem('user');
    }
    user = null;
  }

  function openAuthModal(loginMode = true) {
    isLoginMode = loginMode;
    showAuthModal = true;
    resetForm();
  }

  function closeAuthModal() {
    showAuthModal = false;
    resetForm();
  }

  function resetForm() {
    username = '';
    email = '';
    password = '';
    confirmPassword = '';
    userImage = null;
    authError = '';
  }

	/** @param {KeyboardEvent} event */
	function handleKeyPress(event) {
    if (event.key === 'Enter') {
      handleAuth();
    }
  }
</script>

<style>
	:root {
		--page-header-height: 72px; /* adjust as needed */
		--page-footer-height: 0px; /* no footer, so 0 */
		--page-gap: 16px; /* fixed gap between header/footer and main */
		--main-top-gap: 35px; /* distanza dal header - modifica questo valore */
		--main-bottom-gap: 75px; /* distanza dal footer - modifica questo valore */
		--main-left-gap: 20px; /* distanza laterale sinistra - modifica questo valore */
		--main-right-gap: 20px; /* distanza laterale destra - modifica questo valore */

		--container-padding-top: 1rem; /* padding superiore del contenitore - modifica questo valore */
		--container-padding-bottom: 1rem; /* padding inferiore del contenitore - modifica questo valore */
		--container-padding-left: 1rem; /* padding sinistro del contenitore - modifica questo valore */
		--container-padding-right: 1rem; /* padding destro del contenitore - modifica questo valore */
	}

	/* make the header have a fixed height so main can position relative to it */
	header {
		height: var(--page-header-height);
	}

	/* main sits between header and footer with fixed gaps */
	main.main-content {
		position: absolute;
		top: calc(var(--page-header-height) + var(--main-top-gap));
		bottom: calc(var(--page-footer-height) + var(--main-bottom-gap));
		left: var(--main-left-gap);
		right: var(--main-right-gap);
		display: flex;
		justify-content: center;
		align-items: flex-start;
	}

	/* contenitore principale con padding personalizzabile */
	.main-container {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		padding-top: var(--container-padding-top);
		padding-bottom: var(--container-padding-bottom);
		padding-left: var(--container-padding-left);
		padding-right: var(--container-padding-right);
	}

	/* wrapper del contenuto che diventa scrollabile */
	.content-wrapper {
		flex: 1;
		overflow-y: auto;
	}
</style>

<div class="relative min-h-screen flex flex-col" style="background: linear-gradient(135deg, #c9ffe7 0%, #e9e9ff 70%, #dcecff 100%);">
  
  <!-- HEADER CONTENT -->
  <header class="w-full pt-5 px-5 fixed top-0 left-0 right-0 z-10 bg-transparent">
    <div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 p-4 sm:p-2 shadow-lg flex items-center justify-between" style="min-height: var(--page-header-height);">
      <div class="flex-1"></div> <!-- Spacer sinistro -->
      <div class="text-center">
        <h1 class="text-2xl font-semibold text-gray-900 mb-1">{$_('home.title')}</h1>
        
        <p class="text-gray-700">{currentTime}</p>
        
      </div>
      <!-- Elementi a destra -->
      <div class="flex-1 flex justify-end">
        <div class="flex items-center space-x-3">
          {#if cacheLoaded}
            <span class="inline-flex items-center gap-2 px-2 py-1 rounded-full bg-blue-100 text-blue-800 text-xs font-medium">
              <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z" clip-rule="evenodd"/>
              </svg>
              Database SQLite locale
            </span>
          {:else}
            <span class="inline-flex items-center gap-2 px-2 py-1 rounded-full bg-gray-100 text-gray-800 text-xs font-medium">
              <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z" clip-rule="evenodd"/>
              </svg>
              Database
            </span>
          {/if}
          
          {#if postgresConnected}
            <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-green-100 text-green-800 text-sm font-medium border border-green-200">
              <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.707a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/></svg>
              PostgreSQL OK
            </span>
          {:else}
            <span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-red-100 text-red-800 text-sm font-medium border border-red-200">
              <svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm-1-9V6a1 1 0 112 0v3a1 1 0 11-2 0zm0 4a1 1 0 112 0 1 1 0 01-2 0z" clip-rule="evenodd"/></svg>
              PostgreSQL KO
            </span>
          {/if}
          
          {#if user}
            <span class="text-sm text-gray-600">Utente: {user.username}</span>
            <button
              on:click={logout}
              class="bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded text-sm"
            >
              Logout
            </button>
          {:else}
            <button
              on:click={() => openAuthModal(true)}
              class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded text-sm"
            >
              Login
            </button>
          {/if}
        </div>
      </div>
    </div>
  </header>

  <!-- MAIN CONTENT -->
  <main class="main-content">
    <div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg main-container px-5">
      <div class="content-wrapper">
        {#if loading}
          <div class="text-center py-10 text-gray-500">Caricamento macchine...</div>
        {:else if error}
          <div class="text-center py-10 text-red-500">{error}</div>
        {:else if machines.length === 0}
          <div class="text-center py-10">
            <svg class="w-16 h-16 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"></path>
            </svg>
            <h3 class="text-lg font-medium text-gray-900 mb-2">Nessuna macchina presente</h3>
            <p class="text-gray-600">Vai nelle impostazioni per aggiungere delle macchine.</p>
          </div>
        {:else}
          <div class="w-full overflow-y-auto grid gap-4 pb-20" style="grid-template-columns: repeat(auto-fit, minmax(220px, 260px)); grid-auto-rows: minmax(280px, auto); scrollbar-width: thin;">
            {#each machines as machine}
              <div class="bg-white/90 backdrop-blur-sm rounded-lg border border-black/50 p-4 shadow-lg hover:shadow-xl transition-shadow">
                <!-- Immagine della macchina -->
                <div class="w-full h-32 bg-gray-100 rounded-lg mb-3 flex items-center justify-center overflow-hidden">
                  {#if machine.image}
                    <img src={getImageDataUrl(machine.image)} alt="Immagine macchina {machine.name}" class="w-full h-full object-contain" />
                  {:else}
                    <svg class="w-16 h-16 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
                    </svg>
                  {/if}
                </div>
                
                <!-- Informazioni macchina -->
                <div class="text-center mb-3">
                  <h3 class="text-lg font-semibold text-gray-900 mb-2">{machine.name}</h3>
                  {#if machine.ip_address}
                    <p class="text-sm text-gray-600 mb-1">IP: {machine.ip_address}</p>
                  {:else}
                    <p class="text-sm text-gray-600 mb-1">IP: Non specificato</p>
                  {/if}
                  <div class="flex items-center justify-center gap-2">
                    <span class="text-sm text-gray-600">Database:</span>
                    {#if machine.database_name}
                      <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        $dbStatuses.get(machine.id) 
                          ? 'bg-green-100 text-green-800' 
                          : 'bg-red-100 text-red-800'
                      }`}>
                        {$dbStatuses.get(machine.id) ? 'Connesso' : 'Non connesso'}
                      </span>
                    {:else}
                      <span class="text-sm text-gray-500">Non specificato</span>
                    {/if}
                  </div>
                </div>
                
                <!-- Pulsante utilizza -->
                <div class="text-center">
                  <button 
                    type="button" 
                    on:click={() => useMachine(machine)}
                    class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg transition-colors inline-flex items-center gap-2"
                  >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                    </svg>
                    Utilizza
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </main>
</div>

<!-- Modale di autenticazione -->
{#if showAuthModal}
  <!-- Overlay -->
  <div class="fixed inset-0 bg-gray-600 bg-opacity-50 z-50 flex items-center justify-center" on:click={closeAuthModal} on:keydown={(e) => { if (e.key === 'Escape') closeAuthModal(); }} role="button" tabindex="-1">
    <div
      class="bg-white rounded-lg shadow-xl w-96 max-w-[90vw] max-h-[90vh] overflow-y-auto"
      role="dialog"
      aria-modal="true"
      aria-labelledby="auth-modal-title"
      on:click|stopPropagation
      tabindex="-1"
    >
      <div class="p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 id="auth-modal-title" class="text-lg font-medium text-gray-900">
            {isLoginMode ? 'Accedi' : 'Registrati'}
          </h3>
          <button
            on:click={closeAuthModal}
            class="text-gray-400 hover:text-gray-600"
            aria-label="Chiudi modal"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
            </svg>
          </button>
        </div>

        <!-- Tabs per login/registrazione -->
        <div class="flex mb-4">
          <button
            on:click={() => { isLoginMode = true; resetForm(); }}
            class="flex-1 py-2 px-4 text-center {isLoginMode ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700'} rounded-l-md"
          >
            Login
          </button>
          <button
            on:click={() => { isLoginMode = false; resetForm(); }}
            class="flex-1 py-2 px-4 text-center {!isLoginMode ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700'} rounded-r-md"
          >
            Registrati
          </button>
        </div>

        <form on:submit|preventDefault={handleAuth}>
          <div class="space-y-4">
            <div>
              <label for="modal-username" class="block text-sm font-medium text-gray-700">Username</label>
              <input
                id="modal-username"
                type="text"
                bind:value={username}
                on:keypress={handleKeyPress}
                required
                class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                placeholder="Username"
              />
            </div>

            {#if !isLoginMode}
              <div>
                <label for="modal-email" class="block text-sm font-medium text-gray-700">Email (opzionale)</label>
                <input
                  id="modal-email"
                  type="email"
                  bind:value={email}
                  on:keypress={handleKeyPress}
                  class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Email"
                />
              </div>

              <div>
                <label for="user-image-section" class="block text-sm font-medium text-gray-700 mb-2">Immagine profilo (opzionale)</label>
                <div id="user-image-section" class="flex items-center space-x-4">
                  <button
                    type="button"
                    on:click={selectUserImage}
                    class="bg-gray-500 hover:bg-gray-600 text-white font-medium py-2 px-4 rounded-lg transition-colors text-sm"
                  >
                    Seleziona immagine
                  </button>
                  {#if userImage}
                    <button
                      type="button"
                      on:click={() => userImage = null}
                      class="text-red-500 hover:text-red-700 text-sm"
                    >
                      Rimuovi
                    </button>
                  {/if}
                </div>
                {#if userImage}
                  <div class="mt-3">
                    <img
                      src={getImageDataUrl(userImage)}
                      alt="Anteprima immagine profilo"
                      class="w-20 h-20 object-cover rounded-lg border border-gray-300"
                    />
                  </div>
                {/if}
              </div>
            {/if}

            <div>
              <label for="modal-password" class="block text-sm font-medium text-gray-700">Password</label>
              <input
                id="modal-password"
                type="password"
                bind:value={password}
                on:keypress={handleKeyPress}
                required
                class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                placeholder="Password"
              />
            </div>

            {#if !isLoginMode}
              <div>
                <label for="modal-confirm-password" class="block text-sm font-medium text-gray-700">Conferma Password</label>
                <input
                  id="modal-confirm-password"
                  type="password"
                  bind:value={confirmPassword}
                  on:keypress={handleKeyPress}
                  required
                  class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                  placeholder="Conferma Password"
                />
              </div>
            {/if}
          </div>

          {#if authError}
            <div class="mt-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
              {authError}
            </div>
          {/if}

          <div class="flex justify-end space-x-3 mt-6">
            <button
              type="button"
              on:click={closeAuthModal}
              class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 border border-gray-300 rounded-md hover:bg-gray-200"
            >
              Annulla
            </button>
            <button
              type="submit"
              disabled={authLoading}
              class="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {#if authLoading}
                <span class="flex items-center">
                  <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  {isLoginMode ? 'Accesso...' : 'Registrazione...'}
                </span>
              {:else}
                {isLoginMode ? 'Accedi' : 'Registrati'}
              {/if}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
{/if}

<script>
	import { locale, _ } from 'svelte-i18n';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';

	let theme = "light";
	let notifications = true;
	let language = $locale || "en";
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';

	// DB state
	let dbExists = false;
	let dbCheckError = '';

	// Database settings
	let dbHost = "localhost";
	let dbPort = "5432";
	let dbName = "mes_service";
	let dbUser = "postgres";
	let dbPassword = "postgres";

	// Current time
	let currentTime = '';

	function updateTime() {
		currentTime = new Date().toLocaleString();
	}

	async function loadSettings() {
		try {
			theme = await invoke('get_setting', { key: 'theme' });
			notifications = (await invoke('get_setting', { key: 'notifications' })) === 'true';
			language = await invoke('get_setting', { key: 'language' });

			dbHost = await invoke('get_setting', { key: 'db_host' });
			dbPort = await invoke('get_setting', { key: 'db_port' });
			// dbName is fixed to "mes_service_db"
			dbUser = await invoke('get_setting', { key: 'db_user' });
			dbPassword = await invoke('get_setting', { key: 'db_password' });

			// After loading DB settings, check if the configured database exists
			await checkDbExists();
		} catch (error) {
			console.error('Errore nel caricamento delle impostazioni:', error);
		}
	}

	async function checkDbExists() {
		try {
			const exists = await invoke('check_database_exists');
			dbExists = exists;
			dbCheckError = '';
		} catch (error) {
			console.error('Errore durante il controllo del DB:', error);
			dbExists = false;
			dbCheckError = String(error);
		}
	}

	// Load settings on mount
	import { onMount } from 'svelte';
	onMount(() => {
		loadSettings();
		updateTime();
		const interval = setInterval(updateTime, 1000);
		return () => clearInterval(interval);
	});

	async function saveSettings() {
		try {
			// Salva le impostazioni usando Tauri
			await invoke('save_setting', { key: 'language', value: language });
			await invoke('save_setting', { key: 'theme', value: theme });
			await invoke('save_setting', { key: 'notifications', value: notifications.toString() });

			// Salva impostazioni DB
			await invoke('save_setting', { key: 'db_host', value: dbHost });
			await invoke('save_setting', { key: 'db_port', value: dbPort });
			// dbName is fixed, don't save it
			await invoke('save_setting', { key: 'db_user', value: dbUser });
			await invoke('save_setting', { key: 'db_password', value: dbPassword });
			
			// Aggiorna la lingua immediatamente
			$locale = language;
			
			// Salva anche in localStorage per il caricamento veloce
			if (typeof window !== 'undefined') {
				localStorage.setItem('language', language);
			}
			
			toastMsg = $_('settings.settings_saved');
			toastType = 'success';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		} catch (error) {
			console.error('Errore nel salvare le impostazioni:', error);
			toastMsg = 'Errore nel salvare le impostazioni';
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		}
	}

	async function testDbConnection() {
		try {
			const result = await invoke('test_db_connection');
			toastMsg = result;
			toastType = 'success';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		} catch (error) {
			console.error('Errore nella connessione DB:', error);
			toastMsg = 'Errore nella connessione DB: ' + error;
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		}
	}

	async function createDatabase() {
		try {
			const result = await invoke('create_database');
			toastMsg = result;
			toastType = 'success';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
			// After creating database, update existence state
			await checkDbExists();
		} catch (error) {
			console.error('Errore nella creazione del database:', error);
			toastMsg = 'Errore nella creazione del database: ' + error;
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		}
	}

	// Navigation functions
	function goToNewMachine() {
		goto('/settings/machine_man/new_machine');
	}

	function goToMachineList() {
		goto('/settings/machine_man/machine_list');
	}

	function goToUserList() {
		goto('/settings/user_man/user_list');
	}

	function goToWorkTypes() {
		goto('/settings/machine_man/works_type');
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
		max-width: 1400px;
		height: 100%;
		margin: 0 auto;
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

<div class="relative min-h-screen flex flex-col" style="min-height: 100vh;">
	<div class="absolute inset-0 w-full h-full" style="background: linear-gradient(135deg, #f0f0f0 0%, #c9ffe7 70%, #e0c9ff 100%); opacity: 0.7; backdrop-filter: blur(12px); z-index: 0;"></div>
	<header class="fixed top-0 left-0 right-0 w-full pt-5 px-5 z-10">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 p-4 sm:p-2 shadow-lg mx-auto flex items-center justify-between">
			<div class="flex-1"></div> <!-- Spacer sinistro -->
			<div class="text-center">
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">{$_('settings.title')}</h1>
				
				<p class="text-gray-700">
					{currentTime}
				</p>

			</div>
			<!-- DB status badge sempre a destra -->
			<div class="flex-1 flex justify-end">
				{#if dbExists}
					<span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-green-100 text-green-800 text-sm font-medium border border-green-200">
						<svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.707a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/></svg>
						{$_('settings.header_settings.postgresql_connection_ok')}
					</span>
				{:else}
					<span class="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-red-100 text-red-800 text-sm font-medium border border-red-200">
						<svg class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm-1-9V6a1 1 0 112 0v3a1 1 0 11-2 0zm0 4a1 1 0 112 0 1 1 0 01-2 0z" clip-rule="evenodd"/></svg>
						{$_('settings.header_settings.postgresql_connection_not_ok')}
					</span>
				{/if}
			</div>
		</div>
	</header>
	<main class="main-content">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg mx-auto main-container">
			<div class="content-wrapper">
				<div class="w-full grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">

					<!-- Machine Management Card -->
					<div class="bg-white/90 backdrop-blur-sm rounded-lg border border-black/50 p-6 shadow-lg">
						<h2 class="text-lg font-bold text-gray-900 mb-4 text-center">{$_('settings.machine_management.title')}</h2>
						
						<!-- Main view with buttons -->
						<div class="space-y-4">
							
						<div class="grid grid-cols-1 gap-3">
							<!--	
							<button type="button" on:click={goToNewMachine} class="bg-indigo-500 hover:bg-indigo-600 text-white font-medium py-3 px-4 rounded-lg flex items-center justify-center gap-2 transition-colors text-sm">
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
								</svg>
								{$_('settings.machine_management.new_machine')}
							</button>
							-->
								
							<button type="button" on:click={goToMachineList} class="bg-green-500 hover:bg-green-600 text-white font-medium py-3 px-4 rounded-lg flex items-center justify-center gap-2 transition-colors text-sm">
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"></path>
								</svg>
								{$_('settings.machine_management.existing_machines')}
							</button>

							<button type="button" on:click={goToWorkTypes} class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-3 px-4 rounded-lg flex items-center justify-center gap-2 transition-colors text-sm">
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
								</svg>
								{$_('settings.machine_management.work_types')}
							</button>
						</div>

						<!-- barra separatrice -->
						<hr class="my-6 border-t border-gray-300" />
						
						<h2 class="text-lg font-bold text-gray-900 mb-4 text-center">{$_('settings.user_management.title')}</h2>
						<!-- Main view with buttons -->
						<div class="space-y-4">

						<div class="grid grid-cols-1 gap-3">
							<button type="button" on:click={goToUserList} class="bg-purple-500 hover:bg-purple-600 text-white font-medium py-3 px-4 rounded-lg flex items-center justify-center gap-2 transition-colors text-sm">
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z"></path>
								</svg>
								{$_('settings.user_management.users_button')}
							</button>
						</div>

						</div>
					</div>
				</div>
			<!-- Database Settings Card -->
			<div class="bg-white/90 backdrop-blur-sm rounded-lg border border-black/50 p-6 shadow-lg">
				<h2 class="text-lg font-bold text-gray-900 mb-4 text-center">{$_('settings.header_settings.database_settings')}</h2>
				<form class="space-y-4">
					<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
						<div class="text-left">
							<label for="db-host" class="block font-medium mb-1 text-gray-700 text-sm">{$_('settings.database.host')}</label>
							<input type="text" id="db-host" bind:value={dbHost} placeholder="localhost" class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm" />
						</div>
						
						<div class="text-left">
							<label for="db-port" class="block font-medium mb-1 text-gray-700 text-sm">{$_('settings.database.port')}</label>
							<input type="text" id="db-port" bind:value={dbPort} placeholder="5432" class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm" />
						</div>
						
						<div class="text-left">
							<label for="db-name-display" class="block font-medium mb-1 text-gray-700 text-sm">Database Name</label>
							<div id="db-name-display" class="border border-gray-300 rounded-lg px-3 py-2 w-full bg-gray-50 text-gray-700 text-sm font-mono" aria-label="Database name (fixed)">
								{dbName}
							</div>
							<p class="text-xs text-gray-500 mt-1">{$_('settings.database.database_name_fixed')}</p>
						</div>
						
						<div class="text-left">
							<label for="db-user" class="block font-medium mb-1 text-gray-700 text-sm">Username</label>
							<input type="text" id="db-user" bind:value={dbUser} placeholder="postgres" class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm" />
						</div>
						
						<div class="text-left sm:col-span-2">
							<label for="db-password" class="block font-medium mb-1 text-gray-700 text-sm">Password</label>
							<input type="password" id="db-password" bind:value={dbPassword} placeholder="postgres" class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm" />
						</div>
					</div>
					
					<div class="mt-4 flex gap-3 justify-center flex-wrap">
						<button type="button" on:click={saveSettings} class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg flex items-center gap-2 transition-colors text-sm">
							<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path>
							</svg>
							{$_('settings.database.save_db_settings')}
						</button>
						
						<button type="button" on:click={createDatabase} class="font-medium py-2 px-4 rounded-lg flex items-center gap-2 transition-colors text-sm" class:bg-purple-500={!dbExists} class:hover:bg-purple-600={!dbExists} class:text-white={!dbExists} class:bg-gray-200={dbExists} class:text-gray-500={dbExists} disabled={dbExists}>
							<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
							</svg>
							{$_('settings.database.create_database')}
						</button>
						
						<button type="button" on:click={testDbConnection} class="bg-green-500 hover:bg-green-600 text-white font-medium py-2 px-4 rounded-lg flex items-center gap-2 transition-colors text-sm">
							<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
							</svg>
							{$_('settings.database.test_connection')}
						</button>
					</div>
				</form>
			</div>


			<!-- General Settings Card -->
			<div class="bg-white/90 backdrop-blur-sm rounded-lg border border-black/50 p-6 shadow-lg">
				<h2 class="text-lg font-bold text-gray-900 mb-4 text-center">{$_('settings.title')}</h2>
				<form class="space-y-4">
					<!-- Theme selection hidden -->
					<!-- <div class="text-left">
						<label for="theme-select" class="block font-medium mb-2 text-gray-700 text-sm">{$_('settings.theme')}</label>
						<select id="theme-select" bind:value={theme} class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm">
							<option value="light">{$_('settings.light')}</option>
							<option value="dark">{$_('settings.dark')}</option>
						</select>
					</div> -->
					
					<div class="text-left">
						<label for="language-select" class="block font-medium mb-2 text-gray-700 text-sm">{$_('settings.language')}</label>
						<select id="language-select" bind:value={language} class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm">
							<option value="en">{$_('settings.english')}</option>
							<option value="it">{$_('settings.italian')}</option>
						</select>
					</div>
					
					<!-- Notifications checkbox hidden -->
					<!-- <div class="text-left">
						<label class="flex items-center">
							<input type="checkbox" bind:checked={notifications} class="rounded mr-2 focus:ring-2 focus:ring-blue-500" />
							<span class="font-medium text-gray-700 text-sm">{$_('settings.enable_notifications')}</span>
						</label>
						<p class="text-xs text-gray-500 mt-1">{$_('settings.notifications')}</p>
					</div> -->
					
					
					<div class="mt-4 text-center">
						<button type="button" on:click={saveSettings} class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg flex items-center gap-2 mx-auto transition-colors text-sm">
							<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path>
							</svg>
							{$_('settings.save')}
						</button>
					</div>
				</form>
			</div>
			</div>
		</div>
	</main>
</div>
{#if showToast}
	<div class="fixed bottom-4 right-4 z-50 text-white px-4 py-2 rounded shadow-lg" class:bg-green-500={toastType === 'success'} class:bg-red-500={toastType === 'error'}>
		{toastMsg}
	</div>
{/if}

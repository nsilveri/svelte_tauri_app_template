<script>
	import '../app.css';
	import '../lib/i18n';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import { locale } from 'svelte-i18n';
	import { _ } from 'svelte-i18n';

	let localeReady = false;
	let showSyncPopup = false;
	let syncDiffs = { users: [], machines: [], work_types: [] };
	let syncInProgress = false;

	onMount(async () => {
		try {
			const lang = await invoke('get_setting', { key: 'language' }) || 'en';
			$locale = lang;
			// Also save to localStorage for faster initial load
			if (typeof window !== 'undefined') {
				localStorage.setItem('language', lang);
			}
			localeReady = true;
		} catch (e) {
			console.error('Errore nel caricamento della lingua:', e);
			$locale = 'en';
			localeReady = true;
		}

		// Start the sync watcher
		try {
			await invoke('start_sync_watcher');
			console.log('Sync watcher started');
		} catch (e) {
			console.error('Errore nell\'avvio del sync watcher:', e);
		}

		// Listen for sync needed events
		const unlisten = await listen('sync:needed', (event) => {
			console.log('Sync needed event received:', event.payload);
			syncDiffs = event.payload;
			showSyncPopup = true;
		});

		// Cleanup on unmount
		return () => {
			unlisten();
		};
	});

	async function syncData() {
		syncInProgress = true;
		let syncResults = [];
		try {
			if (syncDiffs.users.length > 0) {
				const result = await invoke('sync_users');
				syncResults.push(result);
				console.log('Risultato sync utenti:', result);
			}
			if (syncDiffs.machines.length > 0) {
				const result = await invoke('sync_machines');
				syncResults.push(result);
				console.log('Risultato sync macchine:', result);
			}
			if (syncDiffs.work_types.length > 0) {
				const result = await invoke('sync_work_types');
				syncResults.push(result);
				console.log('Risultato sync work types:', result);
			}
			
			// Mostra un messaggio di successo
			if (syncResults.length > 0) {
				alert(`Sincronizzazione completata!\n${syncResults.join('\n')}`);
			}
			
			showSyncPopup = false;
			syncDiffs = { users: [], machines: [], work_types: [] };
		} catch (error) {
			console.error('Errore durante la sincronizzazione:', error);
			alert(`Errore durante la sincronizzazione: ${error}`);
		} finally {
			syncInProgress = false;
		}
	}

	function dismissSyncPopup() {
		showSyncPopup = false;
	}
</script>
<!--
<header>
	<nav class="fixed top-0 left-0 right-0 bg-blue-200 shadow">
		<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
			<div class="flex justify-between h-8">
				<div class="flex">
					<a href="/" class="flex-shrink-0 flex items-center text-xl font-bold text-center text-gray-900">
						SilCheat
					</a>
				</div>
			</div>
		</div>

	</nav>
</header>
-->

{#if localeReady}
	<!-- Contenuto principale sempre accessibile -->
	<main class="h-full overflow-y-auto">
		<slot />
	</main>

	<!-- Footer sempre visibile -->
	<footer class="fixed bottom-0 left-0 right-0 z-10 bg-transparent px-5 pb-2">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 p-4 sm:p-2 text-center shadow-lg mx-auto flex justify-center gap-4">
			<a href="/home" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded flex items-center gap-2">
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"></path>
				</svg>
				{$_('footer.home')}
			</a>
			<a href="/settings" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded flex items-center gap-2">
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
				</svg>
				{$_('footer.settings')}
			</a>
			<!--
			<a href="/about" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded flex items-center gap-2">
				<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
				</svg>
				{$_('footer.about')}
			</a>
			-->
		</div>
	</footer>
{:else}
	<div class="h-screen flex items-center justify-center">
		<div class="text-center">
			<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
			<p class="text-gray-600">{$_('home.loading')}</p>
		</div>
	</div>
{/if}

<!-- Sync Popup Modal -->
{#if showSyncPopup}
	<div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
		<div class="bg-white rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
			<h3 class="text-lg font-semibold text-gray-900 mb-4">Sincronizzazione Necessaria</h3>
			
			<p class="text-gray-600 mb-4">
				Sono state rilevate differenze tra il database principale e la cache locale. Vuoi sincronizzare ora?
			</p>
			
			{#if syncDiffs.users.length > 0}
				<div class="mb-3">
					<h4 class="font-medium text-gray-800 mb-2">Utenti ({syncDiffs.users.length})</h4>
					<div class="max-h-32 overflow-y-auto bg-gray-50 rounded p-2">
						{#each syncDiffs.users as user}
							<div class="text-sm text-gray-600 mb-1">
								• {user.username}: {user.reason}
							</div>
						{/each}
					</div>
				</div>
			{/if}
			
			{#if syncDiffs.machines.length > 0}
				<div class="mb-4">
					<h4 class="font-medium text-gray-800 mb-2">Macchine ({syncDiffs.machines.length})</h4>
					<div class="max-h-32 overflow-y-auto bg-gray-50 rounded p-2">
						{#each syncDiffs.machines as machine}
							<div class="text-sm text-gray-600 mb-1">
								• {machine.name}: {machine.reason}
							</div>
						{/each}
					</div>
				</div>
			{/if}
			
			{#if syncDiffs.work_types.length > 0}
				<div class="mb-4">
					<h4 class="font-medium text-gray-800 mb-2">Tipi di Lavorazione ({syncDiffs.work_types.length})</h4>
					<div class="max-h-32 overflow-y-auto bg-gray-50 rounded p-2">
						{#each syncDiffs.work_types as workType}
							<div class="text-sm text-gray-600 mb-1">
								• {workType.name}: {workType.reason}
							</div>
						{/each}
					</div>
				</div>
			{/if}
			
			<div class="flex gap-3 justify-end">
				<button 
					on:click={dismissSyncPopup}
					class="px-4 py-2 text-gray-600 bg-gray-200 rounded-lg hover:bg-gray-300 transition-colors"
					disabled={syncInProgress}
				>
					Ignora
				</button>
				<button 
					on:click={syncData}
					class="px-4 py-2 text-white bg-blue-500 rounded-lg hover:bg-blue-600 transition-colors flex items-center gap-2"
					disabled={syncInProgress}
				>
					{#if syncInProgress}
						<svg class="animate-spin -ml-1 mr-1 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						Sincronizzazione...
					{:else}
						Sincronizza Ora
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}
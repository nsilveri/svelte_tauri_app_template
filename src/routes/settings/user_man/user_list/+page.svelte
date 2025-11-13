<script>
	import { _ } from 'svelte-i18n';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';

	/** @type {Array<{id: number, username: string, email?: string, image?: Uint8Array, created_at: string, sync_status?: string, last_sync?: string}>} */
	let users = [];
	let loading = true;
	let error = '';
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';
	let cacheLoaded = false;

	onMount(async () => {
		await fetchUsers();
	});

	async function fetchUsers() {
		try {
			console.log('Caricamento utenti dalla cache SQLite...');
			loading = true;
			error = '';

			// Carica sempre dalla cache SQLite - senza mappatura per velocità
			users = await invoke('get_cached_users');
			cacheLoaded = true;
			console.log('Utenti caricati dalla cache SQLite:', users.length);
		} catch (e) {
			console.error('Errore nel caricamento dalla cache SQLite:', e);
			error = 'Errore nel caricamento degli utenti: ' + String(e);
		} finally {
			loading = false;
		}
	}

	async function checkSyncStatus() {
		// Non più necessario - sempre SQLite
		cacheSynced = true;
	}

	async function performSync() {
		// Non più necessario - sempre SQLite
	}

	function dismissSyncModal() {
		// Non più necessario - sempre SQLite
	}

	/** @param {{id: number, username: string}} user */
	async function deleteUser(user) {
		if (!confirm(`Sei sicuro di voler eliminare l'utente "${user.username}"?`)) {
			return;
		}

		try {
			await invoke('delete_user', { id: user.id });
			toastMsg = 'Utente eliminato con successo';
			toastType = 'success';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
			await fetchUsers(); // Ricarica la lista
		} catch (error) {
			console.error('Errore nell\'eliminazione dell\'utente:', error);
			toastMsg = 'Errore nell\'eliminazione dell\'utente: ' + error;
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		}
	}

	/** @param {{id: number, username: string}} user */
	function editUser(user) {
		goto(`/settings/user_man/edit/${user.id}`);
	}

	/** @param {Uint8Array | number[] | null | undefined} bytes */
	function getImageDataUrl(bytes) {
		if (!bytes) return '';
		
		// Converti in Uint8Array se necessario
		let uint8Array;
		if (bytes instanceof Uint8Array) {
			uint8Array = bytes;
		} else if (Array.isArray(bytes)) {
			uint8Array = new Uint8Array(bytes);
		} else {
			return '';
		}
		
		// Evita overflow dello stack dividendo in chunk
		let binary = '';
		const chunkSize = 8192; // 8KB chunk per evitare overflow
		for (let i = 0; i < uint8Array.length; i += chunkSize) {
			const chunk = uint8Array.slice(i, i + chunkSize);
			binary += String.fromCharCode.apply(null, Array.from(chunk));
		}
		
		return `data:image/jpeg;base64,${btoa(binary)}`;
	}

	function goBack() {
		goto('/settings');
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
		padding-bottom: 1rem; /* spazio dal footer */
	}
</style>

<div class="relative min-h-screen flex flex-col" style="min-height: 100vh;">
	<div class="absolute inset-0 w-full h-full" style="background: linear-gradient(135deg, #f0f0f0 0%, #c9ffe7 70%, #e0c9ff 100%); opacity: 0.7; backdrop-filter: blur(12px); z-index: 0;"></div>

	<header class="fixed top-0 left-0 right-0 w-full pt-5 px-5 z-10">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 p-4 sm:p-2 shadow-lg mx-auto flex items-center justify-between">
			<div class="flex items-center gap-4">
				<button
					on:click={goBack}
					class="text-gray-600 hover:text-gray-800 p-2 rounded-lg hover:bg-white/50 transition-colors"
					aria-label="Torna alle impostazioni"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
					</svg>
				</button>
			</div>
			<div class="text-center">
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">{$_('settings.user_management.user_list')}</h1>
				<p class="text-gray-700">Gestisci gli utenti del sistema</p>
			</div>
			<div class="flex-1"></div>
		</div>
	</header>

	<main class="main-content">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg mx-auto main-container">
			<div class="content-wrapper">
				<div class="flex items-center justify-between mb-6">
					<h2 class="text-xl font-bold text-gray-900">Utenti Registrati</h2>
					<div class="flex items-center gap-4">
						{#if cacheLoaded}
							<span class="inline-flex items-center gap-2 px-2 py-1 rounded-full bg-blue-100 text-blue-800 text-xs font-medium">
								<svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
									<path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z" clip-rule="evenodd"/>
								</svg>
								DB SQLite locale
							</span>
						{:else}
							<span class="inline-flex items-center gap-2 px-2 py-1 rounded-full bg-gray-100 text-gray-800 text-xs font-medium">
								<svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
									<path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v2a1 1 0 01-1 1H4a1 1 0 01-1-1V4zM3 10a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H4a1 1 0 01-1-1v-6zM14 9a1 1 0 00-1 1v6a1 1 0 001 1h2a1 1 0 001-1v-6a1 1 0 00-1-1h-2z" clip-rule="evenodd"/>
								</svg>
								Database
							</span>
						{/if}
						<div class="text-sm text-gray-600">
							Totale utenti: {users.length}
						</div>
					</div>
				</div>

				{#if loading}
					<div class="text-center py-10 text-gray-500">Caricamento utenti...</div>
				{:else if error}
					<div class="text-center py-10 text-red-500">{error}</div>
				{:else if users.length === 0}
					<div class="text-center py-8">
						<svg class="w-12 h-12 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z"></path>
						</svg>
						<h3 class="text-lg font-medium text-gray-900 mb-2">Nessun utente presente</h3>
						<p class="text-gray-600 mb-4">Gli utenti possono registrarsi dalla pagina home.</p>
					</div>
				{:else}
					<div class="space-y-3 max-h-96 overflow-y-auto">
						{#each users as user}
							<div class="border border-gray-200 rounded-lg p-4 bg-gray-50 hover:bg-gray-100 transition-colors">
								<div class="flex items-center justify-between">
									<div class="flex items-center gap-3 flex-1">
										{#if user.image}
											<div class="w-10 h-10 bg-gray-200 rounded-full flex items-center justify-center overflow-hidden">
												<img src={getImageDataUrl(user.image)} alt="Immagine profilo {user.username}" class="w-full h-full object-cover" />
											</div>
										{:else}
											<div class="w-10 h-10 bg-purple-100 rounded-full flex items-center justify-center">
												<svg class="w-5 h-5 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"></path>
												</svg>
											</div>
										{/if}
										<div class="flex-1">
											<h4 class="font-medium text-gray-900">{user.username}</h4>
											<p class="text-sm text-gray-600">{user.email || 'Nessuna email'}</p>
											<p class="text-xs text-gray-500">Creato: {new Date(user.created_at).toLocaleDateString('it-IT')}</p>
										</div>
									</div>
									<div class="flex items-center gap-2">
										<button
											on:click={() => editUser(user)}
											class="text-blue-600 hover:text-blue-800 p-2 rounded-lg hover:bg-blue-50 transition-colors"
											title="Modifica"
										>
											<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
											</svg>
										</button>
										<button
											on:click={() => deleteUser(user)}
											class="text-red-600 hover:text-red-800 p-2 rounded-lg hover:bg-red-50 transition-colors"
											title="Elimina"
										>
											<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
											</svg>
										</button>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</main>
</div>



{#if showToast}
	<div class="fixed bottom-4 right-4 z-50 text-white px-4 py-2 rounded shadow-lg" class:bg-green-500={toastType === 'success'} class:bg-red-500={toastType === 'error'}>
		{toastMsg}
	</div>
{/if}
<script>
	import { _ } from 'svelte-i18n';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { writable } from 'svelte/store';

	/** @type {Array<{id: number, name: string, image?: Uint8Array, ip_address?: string, database_name?: string, description?: string}>} */
	let machines = [];
	let loading = true;
	let error = '';
	let dbStatuses = writable(new Map()); // Store per tracciare lo stato di connessione dei database
	let pingStatuses = writable(new Map()); // Store per tracciare lo stato del ping delle macchine
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';
	/** @type {Array<{id: number, name: string, image?: Uint8Array}>} */
	let workTypes = [];

	onMount(async () => {
		await fetchWorkTypes(); // Carica prima i work types
		await fetchMachines(); // Poi carica le macchine
	});

	async function fetchMachines() {
		try {
			loading = true;
			machines = await invoke('get_machines');

			// Controlla lo stato di connessione per ogni database e ping in background
			checkConnections();
		} catch (e) {
			error = 'Errore nel caricamento delle macchine.';
			console.error('Errore nel caricamento delle macchine:', e);
		} finally {
			loading = false;
		}
	}

	async function fetchWorkTypes() {
		try {
			workTypes = await invoke('get_work_types');
		} catch (e) {
			console.error('Errore nel caricamento dei tipi di lavorazione:', e);
			workTypes = [];
		}
	}

	/** @param {number|string} workTypeId */
	function getWorkTypeName(workTypeId) {
		// Se workTypeId è già un oggetto con name, restituiscilo direttamente
		if (typeof workTypeId === 'object' && workTypeId.name) {
			return workTypeId.name;
		}

		// Converti workTypeId in numero se è una stringa
		const id = typeof workTypeId === 'string' ? parseInt(workTypeId) : workTypeId;

		const workType = workTypes.find(wt => wt.id === id);
		if (workType) {
			return workType.name;
		} else {
			return `ID: ${workTypeId}`;
		}
	}

	// Controlla lo stato di connessione per ogni database e ping in background
	async function checkConnections() {
		if (machines.length === 0) return;

		// Controlla lo stato di connessione per ogni database
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

		// Controlla lo stato del ping per ogni macchina
		const pingMap = new Map();
		for (const machine of machines) {
			if (machine.ip_address) {
				try {
					const isReachable = await invoke('ping_machine', { ipAddress: machine.ip_address });
					pingMap.set(machine.id, isReachable);
				} catch (e) {
					console.error(`Errore ping per ${machine.name}:`, e);
					pingMap.set(machine.id, false);
				}
			}
		}
		pingStatuses.set(pingMap);
	}

	/** @param {{id: number, name: string}} machine */
	async function deleteMachine(machine) {
		if (!confirm(`Sei sicuro di voler eliminare la macchina "${machine.name}"?`)) {
			return;
		}

		try {
			await invoke('delete_machine', { id: machine.id });
			toastMsg = 'Macchina eliminata con successo';
			toastType = 'success';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
			await fetchMachines(); // Ricarica la lista
		} catch (error) {
			console.error('Errore nell\'eliminazione della macchina:', error);
			toastMsg = 'Errore nell\'eliminazione della macchina: ' + error;
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		}
	}

	/** @param {{id: number, name: string}} machine */
	function editMachine(machine) {
		// Naviga alla pagina di modifica macchina passando l'ID come parametro
		goto(`/settings/machine_man/new_machine?id=${machine.id}`);
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
	header.page-header {
		height: var(--page-header-height);
	}

	/* main sits between header and footer with fixed gaps */
	main.main-content {
		position: absolute;
		top: calc(var(--page-header-height) + var(--main-top-gap));
		bottom: calc(var(--page-footer-height) + var(--main-bottom-gap));
		left: var(--main-left-gap);
		right: var(--main-right-gap);
		overflow: hidden; /* inner container will scroll */
		display: flex;
		justify-content: center;
		align-items: flex-start;
	}

	/* scrollable area for the machines list */
	.list-scroll {
		width: 100%;
		max-width: 1400px;
		height: 100%;
		overflow: auto;
		overflow-x: auto;
		-webkit-overflow-scrolling: touch;
		position: relative;
	}

	/* make the small table header sticky inside the scroll container */
	.table-header-sticky {
		position: sticky;
		top: 0; /* sticks to top of the scroll container */
		z-index: 20;
		backdrop-filter: blur(6px);
	}

	/* keep action column fixed width so layout doesn't jump */
	.actions-col { width: 24%; }
	.info-col { width: 58%; }
	.img-col { width: 18%; }

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
</style>

<div class="relative min-h-screen flex flex-col" style="min-height: 100vh;">
	<div class="absolute inset-0 w-full h-full" style="background: linear-gradient(135deg, #f0f0f0 0%, #c9ffe7 70%, #e0c9ff 100%); opacity: 0.7; backdrop-filter: blur(12px); z-index: 0;"></div>

	<header class="page-header fixed top-0 left-0 right-0 w-full pt-5 px-5 z-10">
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
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">{$_('settings.machine_management.machine_list')}</h1>
				<p class="text-gray-700">Gestisci le tue macchine</p>
			</div>
			<div class="flex-1"></div>
		</div>
	</header>

	<main class="main-content">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg mx-auto main-container">
			<!-- Titolo e pulsante sempre visibili -->
			<div class="flex items-center justify-between mb-6 flex-shrink-0">
				<h2 class="text-xl font-bold text-gray-900">Macchine Disponibili</h2>
				<button
					on:click={() => goto('/settings/machine_man/new_machine')}
					class="bg-indigo-500 hover:bg-indigo-600 text-white font-medium py-2 px-4 rounded-lg flex items-center gap-2 transition-colors"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
					</svg>
					Nuova Macchina
				</button>
			</div>

			<!-- Area scorribile solo per la tabella -->
			<div class="list-scroll flex-1">

				{#if loading}
					<div class="text-center py-10 text-gray-500">Caricamento macchine...</div>
				{:else if error}
					<div class="text-center py-10 text-red-500">{error}</div>
				{:else if machines.length === 0}
					<div class="text-center py-8">
						<svg class="w-12 h-12 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"></path>
						</svg>
						<h3 class="text-lg font-medium text-gray-900 mb-2">Nessuna macchina presente</h3>
						<p class="text-gray-600 mb-4">Crea la tua prima macchina per iniziare.</p>
						<button
							on:click={() => goto('/settings/machine_man/new_machine')}
							class="bg-indigo-500 hover:bg-indigo-600 text-white font-medium py-2 px-4 rounded-lg inline-flex items-center gap-2"
						>
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
							</svg>
							Crea Macchina
						</button>
					</div>
				{:else}
					<!-- Header della tabella -->
					<div class="bg-gray-50 border-b border-gray-200 px-6 py-3 table-header-sticky">
						<div class="grid grid-cols-12 gap-4 items-center">
							<div class="col-span-2">
								<span class="text-sm font-medium text-gray-700">Immagine</span>
							</div>
							<div class="col-span-7">
								<span class="text-sm font-medium text-gray-700">Informazioni Macchina</span>
							</div>
							<div class="col-span-3 text-center">
								<span class="text-sm font-medium text-gray-700">Azioni</span>
							</div>
						</div>
					</div>

					<!-- Lista macchine -->
					<div class="divide-y divide-gray-200" style="overflow-x: auto;">
						{#each machines as machine}
							<div class="px-6 py-4 hover:bg-gray-50 transition-colors">
								<div class="grid grid-cols-12 gap-4 items-center" style="min-width: 1000px;">
									<!-- Colonna Immagine -->
									<div class="col-span-2">
										<div class="w-16 h-16 bg-gray-100 rounded-lg flex items-center justify-center overflow-hidden">
											{#if machine.image}
												<img src={getImageDataUrl(machine.image)} alt="Immagine macchina {machine.name}" class="w-full h-full object-cover" />
											{:else}
												<svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
												</svg>
											{/if}
										</div>
									</div>

									<!-- Colonna Informazioni -->
									<div class="col-span-7">
										<div class="flex flex-col">
											<h3 class="text-lg font-semibold text-gray-900 mb-1">{machine.name}</h3>
											<div class="flex flex-col gap-1">
												{#if machine.ip_address}
													<div class="flex items-center gap-2">
														<span class="text-sm text-gray-600">IP: {machine.ip_address}</span>
														<span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
															$pingStatuses.get(machine.id)
																? 'bg-green-100 text-green-800'
																: 'bg-red-100 text-red-800'
														}`}>
															{$pingStatuses.get(machine.id) ? 'Online' : 'Offline'}
														</span>
													</div>
                                                {:else}
                                                    <p class="text-sm text-gray-600 mb-1">IP: Non specificato</p>
												{/if}
												{#if machine.database_name}
													<div class="flex items-center gap-2">
														<span class="text-sm text-gray-600">Database:</span>
														<span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
															$dbStatuses.get(machine.id)
																? 'bg-green-100 text-green-800'
																: 'bg-red-100 text-red-800'
														}`}>
															{$dbStatuses.get(machine.id) ? 'Connesso' : 'Non connesso'}
														</span>
														<span class="text-sm text-gray-600">{machine.database_name}</span>
													</div>
												{/if}
												{#if machine.works_types && machine.works_types.length > 0}
													<div class="flex items-start gap-2">
														<span class="text-sm text-gray-600">Lavorazioni:</span>
														<div class="flex flex-wrap gap-1">
															{#each machine.works_types as workTypeId}
																<span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
																	{getWorkTypeName(workTypeId)}
																</span>
															{/each}
														</div>
													</div>
												{:else}
													<p class="text-sm text-gray-600">Lavorazioni: Nessuna specificata</p>
												{/if}
												{#if machine.description}
													<p class="text-sm text-gray-600 line-clamp-1">{machine.description}</p>
												{/if}
											</div>
										</div>
									</div>

									<!-- Colonna Azioni -->
									<div class="col-span-3 flex items-center justify-center gap-2">
										<button
											on:click={() => editMachine(machine)}
											class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg flex items-center gap-2 transition-colors"
										>
											<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
											</svg>
											Modifica
										</button>
										<button
											on:click={() => deleteMachine(machine)}
											class="bg-red-500 hover:bg-red-600 text-white font-medium py-2 px-4 rounded-lg flex items-center justify-center transition-colors"
											aria-label="Elimina macchina {machine.name}"
										>
											<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
<script>
	import { _ } from 'svelte-i18n';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	let machineName = '';
	let machineDescription = '';
	let machineIp = '';
	let machineDbName = '';
	
	/** @type {Uint8Array | null} */
	let machineImage = null;
	let loading = false;
	let error = '';
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';
	let isEdit = false;
	/** @type {number | null} */
	let machineId = null;
	/** @type {string[]} */
	let availableDatabases = [];
	let loadingDatabases = false;
	/** @type {Array<{id: number, name: string, image?: Uint8Array}>} */
	let availableWorkTypes = [];
	let loadingWorkTypes = false;
	/** @type {number[]} */
	let selectedWorkTypeIds = [];

	// Current time
	let currentTime = '';

	function updateTime() {
		currentTime = new Date().toLocaleString();
	}

	onMount(async () => {
		// Carica i database disponibili
		await loadAvailableDatabases();
		
		// Carica i work types disponibili
		await loadAvailableWorkTypes();
		
		// Controlla se siamo in modalità modifica
		const urlParams = new URLSearchParams($page.url.search);
		const id = urlParams.get('id');
		
		if (id) {
			isEdit = true;
			machineId = parseInt(id);
			await loadMachine(machineId);
		}

		// Start time update
		updateTime();
		const interval = setInterval(updateTime, 1000);
		return () => clearInterval(interval);
	});

	async function loadAvailableDatabases() {
		try {
			loadingDatabases = true;
			availableDatabases = await invoke('get_available_databases');
		} catch (error) {
			console.error('Errore nel caricamento dei database disponibili:', error);
			// In caso di errore, lascia il campo come input text normale
			availableDatabases = [];
		} finally {
			loadingDatabases = false;
		}
	}

	async function loadAvailableWorkTypes() {
		try {
			loadingWorkTypes = true;
			availableWorkTypes = await invoke('get_work_types');
		} catch (error) {
			console.error('Errore nel caricamento dei tipi di lavorazione disponibili:', error);
			availableWorkTypes = [];
		} finally {
			loadingWorkTypes = false;
		}
	}

	/** @param {number} id */
	async function loadMachine(id) {
		try {
			loading = true;
			const machine = await invoke('get_machine', { id });
            console.log('Macchina caricata per modifica:', machine);
			machineName = machine.name || '';
			machineDescription = machine.description || '';
			machineIp = machine.ip_address || '';
			machineDbName = machine.database_name || '';
			machineImage = machine.image || null;
			selectedWorkTypeIds = machine.works_types || [];
		} catch (error) {
			console.error('Errore nel caricamento della macchina:', error);
			toastMsg = 'Errore nel caricamento della macchina';
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		} finally {
			loading = false;
		}
	}

	async function selectMachineImage() {
		try {
			const imageBytes = await invoke("select_machine_image");
			if (imageBytes) {
				machineImage = new Uint8Array(imageBytes);
			}
		} catch (error) {
			console.error('Errore nella selezione dell\'immagine:', error);
		}
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

	async function saveMachine() {
		if (!machineName.trim()) {
			toastMsg = 'Il nome della macchina è obbligatorio';
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
			return;
		}

		try {
			loading = true;
			const machineData = {
				name: machineName.trim(),
				description: machineDescription.trim() || null,
				ip_address: machineIp.trim() || null,
				database_name: machineDbName && machineDbName.trim() ? machineDbName.trim() : null,
				image: machineImage ? Array.from(machineImage) : null
			};

			if (isEdit) {
				const dbName = machineDbName && machineDbName.trim() ? machineDbName.trim() : null;
                //console.log('Salvataggio db macchina modificata con dati:', machineDbName, 'dbName:', dbName);
				const desc = machineDescription && machineDescription.trim() ? machineDescription.trim() : null;
				const ipAddr = machineIp && machineIp.trim() ? machineIp.trim() : null;
				console.log('Dati da aggiornare: Nome:', machineName.trim(), 'IP:', ipAddr, 'DB:', dbName, 'Desc:', desc);
				console.log('Tipi dei dati:', typeof ipAddr, typeof dbName, typeof desc);
				console.log('Valori raw:', {ipAddr, dbName, desc});
				const updateData = {
					id: machineId,
					name: machineName.trim(),
					image: machineImage ? Array.from(machineImage) : null,
					ipAddress: ipAddr || null,
					databaseName: dbName || null,
					description: desc || null,
					worksTypes: selectedWorkTypeIds
				};
				console.log('Oggetto update completo:', updateData);
				await invoke('update_machine', updateData);
				toastMsg = 'Macchina aggiornata con successo';
			} else {
				const dbName = machineDbName && machineDbName.trim() ? machineDbName.trim() : null;
				const desc = machineDescription && machineDescription.trim() ? machineDescription.trim() : null;
				await invoke('add_new_machine', {
					name: machineName.trim(),
					image: machineImage ? Array.from(machineImage) : null,
					ipAddress: machineIp && machineIp.trim() ? machineIp.trim() : null,
					databaseName: dbName || null,
					description: desc || null,
					worksTypes: selectedWorkTypeIds
				});
				toastMsg = 'Macchina creata con successo';
			}

			toastType = 'success';
			showToast = true;
			setTimeout(() => {
				showToast = false;
				goto('/settings/machine_man/machine_list');
			}, 1500);
		} catch (error) {
			console.error('Errore nel salvataggio della macchina:', error);
			toastMsg = 'Errore nel salvataggio della macchina: ' + error;
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		} finally {
			loading = false;
		}
	}

	function goBack() {
		goto('/settings/machine_man/machine_list');
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

	/* footer del container che rimane sempre visibile in basso */
	.container-footer {
		flex-shrink: 0;
		border-top: 1px solid #e5e7eb;
		padding-top: 1.5rem;
		background: rgba(255, 255, 255, 0.9); /* sfondo semi-trasparente per leggibilità */
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
					aria-label="Torna alla lista macchine"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
					</svg>
				</button>
			</div>
			<div class="text-center">
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">
					{isEdit ? $_('settings.machine_management.edit_machine') : $_('settings.machine_management.new_machine')}
				</h1>
				<p class="text-gray-700">{currentTime}</p>
			</div>
			<div class="flex-1"></div>
		</div>
	</header>

	<main class="main-content">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg mx-auto main-container">
			<div class="content-wrapper">
				<form on:submit|preventDefault={saveMachine}>
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
						<!-- Colonna sinistra: Informazioni macchina -->
						<div class="space-y-6">
							<!-- Nome macchina -->
							<div>
								<label for="machine-name" class="block text-sm font-medium text-gray-700 mb-2">
									Nome Macchina *
								</label>
								<input
									id="machine-name"
									type="text"
									bind:value={machineName}
									placeholder="Inserisci il nome della macchina"
									required
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								/>
							</div>

							<!-- Indirizzo IP -->
							<div>
								<label for="machine-ip" class="block text-sm font-medium text-gray-700 mb-2">
									Indirizzo IP (opzionale)
								</label>
								<input
									id="machine-ip"
									type="text"
									bind:value={machineIp}
									placeholder="192.168.1.100"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								/>
							</div>

							<!-- Nome Database -->
							<div>
								<label for="machine-db" class="block text-sm font-medium text-gray-700 mb-2">
									Nome Database (opzionale)
								</label>
								{#if availableDatabases.length > 0}
									<select
										id="machine-db"
										bind:value={machineDbName}
										class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
									>
										<option value="">Seleziona un database...</option>
										{#each availableDatabases as db}
											<option value={db}>{db}</option>
										{/each}
									</select>
								{:else}
									<input
										id="machine-db"
										type="text"
										bind:value={machineDbName}
										placeholder="nome_database"
										class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
									/>
								{/if}
								{#if loadingDatabases}
									<p class="text-xs text-gray-500 mt-1">Caricamento database disponibili...</p>
								{:else if availableDatabases.length > 0}
									<p class="text-xs text-gray-500 mt-1">Seleziona un database dalla lista o inseriscine uno nuovo</p>
								{:else}
									<p class="text-xs text-gray-500 mt-1">Il database deve essere accessibile dal sistema MES</p>
								{/if}
							</div>

							<!-- Tipi di Lavorazione -->
							<div>
								<label for="machine-work-types" class="block text-sm font-medium text-gray-700 mb-2">
									Tipi di Lavorazione
								</label>
								<div id="machine-work-types" class="space-y-2 max-h-40 overflow-y-auto border border-gray-300 rounded-lg p-3">
									{#if loadingWorkTypes}
										<p class="text-sm text-gray-500">Caricamento tipi di lavorazione...</p>
									{:else if availableWorkTypes.length === 0}
										<p class="text-sm text-gray-500">Nessun tipo di lavorazione disponibile</p>
									{:else}
										{#each availableWorkTypes as workType}
											<label class="flex items-center space-x-2 cursor-pointer hover:bg-gray-50 p-1 rounded">
												<input
													type="checkbox"
													value={workType.id}
													bind:group={selectedWorkTypeIds}
													class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
												/>
												<span class="text-sm text-gray-700">{workType.name}</span>
												{#if workType.image}
													<img
														src={getImageDataUrl(workType.image)}
														alt="Icona {workType.name}"
														class="w-4 h-4 object-cover rounded"
													/>
												{/if}
											</label>
										{/each}
									{/if}
								</div>
								<p class="text-xs text-gray-500 mt-1">
									Seleziona i tipi di lavorazione che questa macchina può eseguire
									{#if selectedWorkTypeIds.length > 0}
										({selectedWorkTypeIds.length} selezionati)
									{/if}
								</p>
							</div>
						</div>

						<!-- Colonna destra: Immagine macchina -->
						<div class="space-y-6">
							<div>
								<label for="machine-image-section" class="block text-sm font-medium text-gray-700 mb-2">
									Immagine Macchina (opzionale)
								</label>
								<div id="machine-image-section" class="flex items-center space-x-4">
									<button
										type="button"
										on:click={selectMachineImage}
										class="bg-gray-500 hover:bg-gray-600 text-white font-medium py-2 px-4 rounded-lg transition-colors"
									>
										Seleziona Immagine
									</button>
									{#if machineImage}
										<button
											type="button"
											on:click={() => machineImage = null}
											class="text-red-500 hover:text-red-700 text-sm font-medium"
										>
											Rimuovi
										</button>
									{/if}
								</div>
								{#if machineImage}
									<div class="mt-3 p-2 bg-gray-50 rounded border">
										<img
											src={getImageDataUrl(machineImage)}
											alt="Anteprima immagine macchina"
											class="max-w-full max-h-48 object-contain rounded"
										/>
									</div>
								{/if}
								<p class="text-xs text-gray-500 mt-1">Seleziona un'immagine dal tuo computer</p>
							</div>
						</div>
					</div>

				</form>
			</div>
			
			<!-- Footer del container -->
			<div class="container-footer">
					<div class="flex justify-end space-x-3">
						<button
							type="button"
							on:click={goBack}
							class="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 border border-gray-300 rounded-lg hover:bg-gray-200 transition-colors"
						>
							Annulla
						</button>
						<button
							on:click={saveMachine}
							disabled={loading}
							class="px-6 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
						>
							{#if loading}
								<svg class="animate-spin -ml-1 mr-1 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								{isEdit ? 'Aggiornamento...' : 'Creazione...'}
							{:else}
								{isEdit ? 'Aggiorna Macchina' : 'Crea Macchina'}
							{/if}
						</button>
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
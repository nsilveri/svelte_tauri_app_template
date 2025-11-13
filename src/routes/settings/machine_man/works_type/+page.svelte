<script>
	import { _ } from 'svelte-i18n';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';

	/** @type {Array<{id: number, name: string, image: Uint8Array | null, created_at: string, updated_at: string}>} */
	let workTypes = [];
	let loading = true;
	let error = '';
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';

	onMount(async () => {
		await fetchWorkTypes();
	});

	async function fetchWorkTypes() {
		try {
			loading = true;
			workTypes = await invoke('get_work_types');
		} catch (e) {
			error = 'Errore nel caricamento dei tipi di lavorazione.';
			console.error('Errore nel caricamento dei tipi di lavorazione:', e);
		} finally {
			loading = false;
		}
	}

	/** @param {{id: number, name: string}} workType */
	async function deleteWorkType(workType) {
		if (!confirm(`Sei sicuro di voler eliminare il tipo di lavorazione "${workType.name}"?`)) {
			return;
		}

		try {
			await invoke('delete_work_type', { id: workType.id });
			toastMsg = 'Tipo di lavorazione eliminato con successo';
			toastType = 'success';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
			await fetchWorkTypes(); // Ricarica la lista
		} catch (error) {
			console.error('Errore nell\'eliminazione del tipo di lavorazione:', error);
			toastMsg = 'Errore nell\'eliminazione del tipo di lavorazione: ' + error;
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		}
	}

	/** @param {{id: number, name: string}} workType */
	function editWorkType(workType) {
		// Naviga alla pagina di modifica tipo di lavorazione passando l'ID come parametro
		goto(`/settings/machine_man/works_type/edit?id=${workType.id}`);
	}

	function goBack() {
		goto('/settings');
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
		overflow: hidden; /* inner container will scroll */
		display: flex;
		justify-content: center;
		align-items: flex-start;
	}

	/* scrollable area for the work types list */
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
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">Tipi di Lavorazione</h1>
				<p class="text-gray-700">Gestisci i tipi di lavorazione disponibili</p>
			</div>
			<div class="flex-1"></div>
		</div>
	</header>

	<main class="main-content">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg mx-auto main-container">
			<!-- Titolo e pulsante sempre visibili -->
			<div class="flex items-center justify-between mb-6 flex-shrink-0">
				<h2 class="text-xl font-bold text-gray-900">Tipi di Lavorazione Disponibili</h2>
				<button
					on:click={() => goto('/settings/machine_man/works_type/new')}
					class="bg-indigo-500 hover:bg-indigo-600 text-white font-medium py-2 px-4 rounded-lg flex items-center gap-2 transition-colors"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
					</svg>
					Nuovo Tipo
				</button>
			</div>

			<!-- Area scorribile solo per la tabella -->
			<div class="list-scroll flex-1">

				{#if loading}
					<div class="text-center py-10 text-gray-500">Caricamento tipi di lavorazione...</div>
				{:else if error}
					<div class="text-center py-10 text-red-500">{error}</div>
				{:else if workTypes.length === 0}
					<div class="text-center py-8">
						<svg class="w-12 h-12 text-gray-400 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
						</svg>
						<h3 class="text-lg font-medium text-gray-900 mb-2">Nessun tipo di lavorazione presente</h3>
						<p class="text-gray-600 mb-4">Crea il tuo primo tipo di lavorazione per iniziare.</p>
						<button
							on:click={() => goto('/settings/machine_man/works_type/new')}
							class="bg-indigo-500 hover:bg-indigo-600 text-white font-medium py-2 px-4 rounded-lg inline-flex items-center gap-2"
						>
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path>
							</svg>
							Crea Tipo di Lavorazione
						</button>
					</div>
				{:else}
					<!-- Header della tabella -->
					<div class="bg-gray-50 border-b border-gray-200 px-6 py-3 table-header-sticky">
						<div class="grid grid-cols-12 gap-4 items-center">
							<div class="col-span-2">
								<span class="text-sm font-medium text-gray-700">Immagine</span>
							</div>
							<div class="col-span-6">
								<span class="text-sm font-medium text-gray-700">Nome Tipo di Lavorazione</span>
							</div>
							<div class="col-span-4 text-center">
								<span class="text-sm font-medium text-gray-700">Azioni</span>
							</div>
						</div>
					</div>

					<!-- Lista work types -->
					<div class="divide-y divide-gray-200">
						{#each workTypes as workType}
							<div class="px-6 py-4 hover:bg-gray-50 transition-colors">
								<div class="grid grid-cols-12 gap-4 items-center">
									<!-- Colonna Immagine -->
									<div class="col-span-2">
										<div class="w-16 h-16 bg-gray-100 rounded-lg flex items-center justify-center overflow-hidden">
											{#if workType.image}
												<img src={getImageDataUrl(workType.image)} alt="Immagine tipo lavorazione {workType.name}" class="w-full h-full object-cover" />
											{:else}
												<svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
													<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
												</svg>
											{/if}
										</div>
									</div>

									<!-- Colonna Nome -->
									<div class="col-span-6">
										<div class="flex flex-col">
											<h3 class="text-lg font-semibold text-gray-900 mb-1">{workType.name}</h3>
											<div class="flex flex-col gap-1">
												<p class="text-sm text-gray-600">
													Creato: {new Date(workType.created_at).toLocaleDateString('it-IT')}
												</p>
												{#if workType.updated_at !== workType.created_at}
													<p class="text-sm text-gray-600">
														Aggiornato: {new Date(workType.updated_at).toLocaleDateString('it-IT')}
													</p>
												{/if}
											</div>
										</div>
									</div>

									<!-- Colonna Azioni -->
									<div class="col-span-4 flex items-center justify-center gap-2">
										<button
											on:click={() => editWorkType(workType)}
											class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-4 rounded-lg flex items-center gap-2 transition-colors"
										>
											<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
											</svg>
											Modifica
										</button>
										<button
											on:click={() => deleteWorkType(workType)}
											class="bg-red-500 hover:bg-red-600 text-white font-medium py-2 px-4 rounded-lg flex items-center justify-center transition-colors"
											aria-label="Elimina tipo di lavorazione {workType.name}"
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

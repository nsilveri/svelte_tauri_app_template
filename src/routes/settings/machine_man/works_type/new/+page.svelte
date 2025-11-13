<script>
	import { _ } from 'svelte-i18n';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	let name = '';
	/** @type {Uint8Array | null} */
	let workTypeImage = null;
	let loading = false;
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';
	let isEdit = false;
	/** @type {number | null} */
	let workTypeId = null;

	onMount(() => {
		// Controlla se siamo in modalità modifica
		const urlParams = new URLSearchParams($page.url.search);
		const id = urlParams.get('id');
		
		if (id) {
			isEdit = true;
			workTypeId = parseInt(id);
			loadWorkType(workTypeId);
		}
	});

	/** @param {number} id */
	async function loadWorkType(id) {
		try {
			loading = true;
			const workType = await invoke('get_work_type', { id });
			console.log('Work type caricato per modifica:', workType);
			name = workType.name || '';
			workTypeImage = workType.image || null;
		} catch (error) {
			console.error('Errore nel caricamento del tipo di lavorazione:', error);
			toastMsg = 'Errore nel caricamento del tipo di lavorazione';
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		} finally {
			loading = false;
		}
	}

	async function selectWorkTypeImage() {
		try {
			const imagePath = await invoke("select_work_type_image");
			if (imagePath) {
				const imageBytes = await invoke("load_image_from_path", { path: imagePath });
				workTypeImage = new Uint8Array(imageBytes);
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

	async function saveWorkType() {
		if (!name.trim()) {
			toastMsg = 'Il nome del tipo di lavorazione è obbligatorio';
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
			return;
		}

		try {
			loading = true;
			const workTypeData = {
				name: name.trim(),
				image: workTypeImage ? Array.from(workTypeImage) : null
			};

			if (isEdit) {
				await invoke('update_work_type', { id: workTypeId, workType: workTypeData });
				toastMsg = 'Tipo di lavorazione aggiornato con successo';
			} else {
				const result = await invoke('add_work_type', { workType: workTypeData });
				console.log('Work type creato:', result);
				toastMsg = 'Tipo di lavorazione creato con successo';
			}

			toastType = 'success';
			showToast = true;
			setTimeout(() => {
				showToast = false;
				goto('/settings/machine_man/works_type');
			}, 1500);
		} catch (error) {
			console.error('Errore nel salvataggio del tipo di lavorazione:', error);
			toastMsg = 'Errore nel salvataggio: ' + error;
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		} finally {
			loading = false;
		}
	}

	function goBack() {
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
					aria-label="Torna alla lista tipi di lavorazione"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
					</svg>
				</button>
			</div>
			<div class="text-center">
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">{isEdit ? 'Modifica Tipo di Lavorazione' : 'Nuovo Tipo di Lavorazione'}</h1>
				<p class="text-gray-700">{isEdit ? 'Modifica i dettagli del tipo di lavorazione' : 'Aggiungi un nuovo tipo di lavorazione'}</p>
			</div>
			<div class="flex-1"></div>
		</div>
	</header>

	<main class="main-content">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg mx-auto main-container">
			<div class="content-wrapper">
				<form on:submit|preventDefault={saveWorkType} class="space-y-6">
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
						<!-- Colonna sinistra: Informazioni work type -->
						<div class="space-y-6">
							<div class="text-left">
								<label for="name" class="block font-medium mb-2 text-gray-700 text-sm">
									Nome Tipo di Lavorazione *
								</label>
								<input
									type="text"
									id="name"
									bind:value={name}
									placeholder="Es: Saldatura, Pittura, Montaggio..."
									class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
									required
									disabled={loading}
								/>
								<p class="text-xs text-gray-500 mt-1">Inserisci il nome del tipo di lavorazione</p>
							</div>
						</div>

						<!-- Colonna destra: Immagine work type -->
						<div class="space-y-6">
							<div>
								<label for="work-type-image-section" class="block text-sm font-medium text-gray-700 mb-2">
									Immagine Tipo di Lavorazione (opzionale)
								</label>
								<div id="work-type-image-section" class="flex items-center space-x-4">
									<button
										type="button"
										on:click={selectWorkTypeImage}
										class="bg-gray-500 hover:bg-gray-600 text-white font-medium py-2 px-4 rounded-lg transition-colors"
										disabled={loading}
									>
										Seleziona Immagine
									</button>
									{#if workTypeImage}
										<button
											type="button"
											on:click={() => workTypeImage = null}
											class="text-red-500 hover:text-red-700 text-sm font-medium"
											disabled={loading}
										>
											Rimuovi
										</button>
									{/if}
								</div>
								{#if workTypeImage}
									<div class="mt-3 p-2 bg-gray-50 rounded border">
										<img
											src={getImageDataUrl(workTypeImage)}
											alt="Anteprima immagine tipo di lavorazione"
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
				<div class="flex gap-3 justify-center">
						<button
							type="button"
							on:click={goBack}
							class="bg-gray-500 hover:bg-gray-600 text-white font-medium py-2 px-6 rounded-lg transition-colors text-sm"
							disabled={loading}
						>
							Annulla
						</button>
						<button
							on:click={saveWorkType}
							class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-6 rounded-lg flex items-center gap-2 transition-colors text-sm"
							disabled={loading || !name.trim()}
						>
							{#if loading}
								<svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
									<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
									<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
								</svg>
								{isEdit ? 'Aggiornando...' : 'Salvando...'}
							{:else}
								<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path>
								</svg>
								{isEdit ? 'Aggiorna Tipo di Lavorazione' : 'Salva Tipo di Lavorazione'}
							{/if}
						</button>
					</div>
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
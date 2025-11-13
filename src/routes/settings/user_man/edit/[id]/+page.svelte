<script>
	import { _ } from 'svelte-i18n';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	let username = '';
	let email = '';
	/** @type {Uint8Array | null} */
	let userImage = null;
	let loading = false;
	let error = '';
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';
	/** @type {number | null} */
	let userId = null;

	onMount(async () => {
		// Ottieni l'ID dall'URL
		const id = $page.params.id;
		if (id) {
			userId = parseInt(id);
			await loadUser(userId);
		} else {
			// Se non c'è ID, torna alla lista
			goto('/settings/user_man/user_list');
		}
	});

	/** @param {number} id */
	async function loadUser(id) {
		try {
			loading = true;
			const user = await invoke('get_user_profile', { userId: id });
			username = user.username || '';
			email = user.email || '';
			userImage = user.image || null;
		} catch (error) {
			// Log full error and surface message to the UI instead of redirecting immediately.
			console.error('Errore nel caricamento dell\'utente (loadUser):', error);
			// Show a toast so the user knows something went wrong, and keep them on the page
			// so they can see the error details in the console or retry.
			toastMsg = 'Errore nel caricamento dell\'utente: ' + String(error);
			toastType = 'error';
			showToast = true;
			// Keep the error visible a bit longer to aid debugging
			setTimeout(() => { showToast = false; }, 5000);
			// Set an error variable so the template can optionally render a message
			error = String(error);
		} finally {
			loading = false;
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

	/** @param {Uint8Array | null | undefined} bytes */
	function getImageDataUrl(bytes) {
		if (!bytes) return '';
		let binary = '';
		for (let i = 0; i < bytes.length; i++) {
			binary += String.fromCharCode(bytes[i]);
		}
		return `data:image/jpeg;base64,${btoa(binary)}`;
	}

	async function saveUser() {
		if (!username.trim()) {
			toastMsg = 'Il nome utente è obbligatorio';
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
			return;
		}

		try {
			loading = true;

			await invoke('update_user', {
				user_id: userId,
				username: username.trim(),
				email: email.trim() || null,
				image: userImage ? Array.from(userImage) : null
			});

			toastMsg = 'Utente aggiornato con successo';
			toastType = 'success';
			showToast = true;
			setTimeout(() => {
				showToast = false;
				goto('/settings/user_man/user_list');
			}, 1500);
		} catch (error) {
			console.error('Errore nell\'aggiornamento dell\'utente:', error);
			toastMsg = 'Errore nell\'aggiornamento dell\'utente: ' + String(error);
			toastType = 'error';
			showToast = true;
			setTimeout(() => { showToast = false; }, 2500);
		} finally {
			loading = false;
		}
	}

	function goBack() {
		goto('/settings/user_man/user_list');
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
					aria-label="Torna alla lista utenti"
				>
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"></path>
					</svg>
				</button>
			</div>
			<div class="text-center">
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">
					{$_('settings.user_management.edit_user')}
				</h1>
				<p class="text-gray-700">Modifica i dettagli dell'utente</p>
			</div>
			<div class="flex-1"></div>
		</div>
	</header>

	<main class="main-content">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 shadow-lg mx-auto main-container">
			<div class="content-wrapper">
				{#if loading}
					<div class="text-center py-10 text-gray-500">Caricamento utente...</div>
				{:else if error}
					<div class="text-center py-8">
						<p class="text-red-600 font-medium mb-3">Si è verificato un errore durante il caricamento dell'utente:</p>
						<p class="text-sm text-red-500 mb-4">{error}</p>
						<div class="flex items-center justify-center gap-3">
							<button type="button" on:click={() => loadUser(userId)} class="px-4 py-2 bg-blue-600 text-white rounded">Riprova</button>
							<button type="button" on:click={goBack} class="px-4 py-2 bg-gray-100 text-gray-700 rounded">Torna alla lista</button>
						</div>
					</div>
				{:else}
					<form on:submit|preventDefault={saveUser}>
						<div class="space-y-6">
							<!-- Nome utente -->
							<div>
								<label for="username" class="block text-sm font-medium text-gray-700 mb-2">
									Nome Utente *
								</label>
								<input
									id="username"
									type="text"
									bind:value={username}
									placeholder="Inserisci il nome utente"
									required
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								/>
							</div>

							<!-- Email -->
							<div>
								<label for="email" class="block text-sm font-medium text-gray-700 mb-2">
									Email (opzionale)
								</label>
								<input
									id="email"
									type="email"
									bind:value={email}
									placeholder="user@example.com"
									class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
								/>
							</div>

							<!-- Immagine profilo -->
							<div>
								<label for="user-image-section" class="block text-sm font-medium text-gray-700 mb-2">
									Immagine Profilo (opzionale)
								</label>
								<div id="user-image-section" class="flex items-center space-x-4">
									<button
										type="button"
										on:click={selectUserImage}
										class="bg-gray-500 hover:bg-gray-600 text-white font-medium py-2 px-4 rounded-lg transition-colors"
									>
										Seleziona Immagine
									</button>
									{#if userImage}
										<button
											type="button"
											on:click={() => userImage = null}
											class="text-red-500 hover:text-red-700 text-sm font-medium"
										>
											Rimuovi
										</button>
									{/if}
								</div>
								{#if userImage}
									<div class="mt-3 p-2 bg-gray-50 rounded border">
										<img
											src={getImageDataUrl(userImage)}
											alt="Anteprima immagine profilo"
											class="max-w-full max-h-48 object-contain rounded"
										/>
									</div>
								{/if}
								<p class="text-xs text-gray-500 mt-1">Seleziona un'immagine dal tuo computer</p>
							</div>
						</div>
					</form>
				{/if}
			</div>

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
						type="submit"
						on:click={saveUser}
						disabled={loading}
						class="px-6 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
					>
						{#if loading}
							<svg class="animate-spin -ml-1 mr-1 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							Aggiornamento...
						{:else}
							Aggiorna Utente
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
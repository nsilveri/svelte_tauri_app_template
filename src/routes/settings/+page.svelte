<script>
	import { locale, _ } from 'svelte-i18n';
	import { invoke } from '@tauri-apps/api/core';

	let theme = "light";
	let notifications = true;
	let language = $locale || "en";
	let showToast = false;
	let toastMsg = '';
	let toastType = 'success';

	async function saveSettings() {
		try {
			// Salva le impostazioni usando Tauri
			await invoke('save_setting', { key: 'language', value: language });
			await invoke('save_setting', { key: 'theme', value: theme });
			await invoke('save_setting', { key: 'notifications', value: notifications.toString() });
			
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
</script>

<div class="relative min-h-screen flex flex-col" style="min-height: 100vh;">
	<div class="absolute inset-0 w-full h-full" style="background: linear-gradient(135deg, #f0f0f0 0%, #c9ffe7 70%, #e0c9ff 100%); opacity: 0.7; backdrop-filter: blur(12px); z-index: 0;"></div>
	<header class="w-full pt-5 px-5">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 p-4 sm:p-2 text-center shadow-lg mx-auto">
			<div class="mb-1">
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">{$_('settings.title')}</h1>
				<p class="text-gray-700">
					{$_('settings.description')}
				</p>
			</div>
		</div>
	</header>
	<main class="flex-grow flex justify-center items-center px-4">
		<div class="max-w-lg w-full bg-white/90 backdrop-blur-sm rounded-lg border border-white/20 p-8 shadow-lg">
			<h2 class="text-xl font-bold text-gray-900 mb-6 text-center">{$_('settings.title')}</h2>
			<form class="space-y-6">
				<div class="text-left">
					<label for="theme-select" class="block font-medium mb-2 text-gray-700">{$_('settings.theme')}</label>
					<select id="theme-select" bind:value={theme} class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent">
						<option value="light">{$_('settings.light')}</option>
						<option value="dark">{$_('settings.dark')}</option>
					</select>
				</div>
				
				<div class="text-left">
					<label for="language-select" class="block font-medium mb-2 text-gray-700">{$_('settings.language')}</label>
					<select id="language-select" bind:value={language} class="border border-gray-300 rounded-lg px-3 py-2 w-full focus:ring-2 focus:ring-blue-500 focus:border-transparent">
						<option value="en">{$_('settings.english')}</option>
						<option value="it">{$_('settings.italian')}</option>
					</select>
				</div>
				
				<div class="text-left">
					<label class="flex items-center">
						<input type="checkbox" bind:checked={notifications} class="rounded mr-2 focus:ring-2 focus:ring-blue-500" />
						<span class="font-medium text-gray-700">{$_('settings.enable_notifications')}</span>
					</label>
					<p class="text-sm text-gray-500 mt-1">{$_('settings.notifications')}</p>
				</div>
				
				<div class="pt-4 border-t border-gray-200">
					<h3 class="font-medium text-gray-700 mb-2">{$_('about.template_info')}</h3>
					<div class="space-y-1 text-sm text-gray-600">
						<p><strong>{$_('about.version')}:</strong> 1.0.0</p>
						<p><strong>Framework:</strong> Svelte + Tauri</p>
						<p><strong>License:</strong> MIT</p>
					</div>
				</div>
				
				<div class="mt-6 text-center">
					<button type="button" on:click={saveSettings} class="bg-blue-500 hover:bg-blue-600 text-white font-medium py-2 px-6 rounded-lg flex items-center gap-2 mx-auto transition-colors">
						<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4"></path>
						</svg>
						{$_('settings.save')}
					</button>
				</div>
			</form>
		</div>
	</main>
</div>
{#if showToast}
	<div class="fixed bottom-4 right-4 z-50 text-white px-4 py-2 rounded shadow-lg" class:bg-green-500={toastType === 'success'} class:bg-red-500={toastType === 'error'}>
		{toastMsg}
	</div>
{/if}

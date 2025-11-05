<script>
  import { _ } from 'svelte-i18n';
</script>

<div class="relative min-h-screen flex flex-col" style="min-height: 100vh;">
	<div class="absolute inset-0 w-full h-full" style="background: linear-gradient(120deg, #ffeec9 0%, #ffd6e0 50%, #ffdbce 100%); opacity: 0.8; backdrop-filter: blur(16px); z-index: 0;"></div>
	<header class="w-full pt-5 px-5 fixed top-0 left-0 right-0 z-10 bg-transparent">
		<div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 p-4 sm:p-2 text-center shadow-lg mx-auto">
			<div class="mb-1">
				<h1 class="text-2xl font-semibold text-gray-900 mb-1">{$_('about.title')}</h1>
				<p class="text-gray-700">
					{$_('about.description')}
				</p>
			</div>
		</div>
	</header>
	<main class="flex-grow px-5 pb-8" style="padding-top: 120px; margin-bottom: 45px;">
		<div class="max-w-4xl mx-auto space-y-8">

			<!-- Welcome Section -->
			<div class="bg-white/90 backdrop-blur-sm rounded-lg border border-white/20 p-8 shadow-lg">
				<div class="flex flex-col md:flex-row items-center gap-6">
					<div class="flex-1">
						<h2 class="text-3xl font-bold text-gray-900 mb-4">{$_('home.welcome_title')}</h2>
						<p class="text-lg text-gray-700 leading-relaxed">
							{$_('about.description_long')}
						</p>
					</div>
					<div class="flex-shrink-0">
						<div class="w-32 h-32 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center shadow-lg">
							<svg class="w-16 h-16 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
							</svg>
						</div>
					</div>
				</div>
			</div>

			<!-- Features -->
			<div class="bg-white/90 backdrop-blur-sm rounded-lg border border-white/20 p-8 shadow-lg">
				<h2 class="text-2xl font-bold text-gray-900 mb-6">{$_('about.features')}</h2>

				<div class="grid md:grid-cols-2 gap-6">
					<div class="space-y-6">
						<div class="flex items-start gap-4">
							<div class="flex-shrink-0 w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
								<svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"></path>
								</svg>
							</div>
							<div>
								<h3 class="text-xl font-semibold text-gray-800 mb-3">{$_('about.feature_1')} + {$_('about.feature_2')}</h3>
								<p class="text-gray-600 leading-relaxed">
									{$_('about.feature_1')} - {$_('about.feature_2')}
								</p>
							</div>
						</div>

						<div class="flex items-start gap-4">
							<div class="flex-shrink-0 w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
								<svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"></path>
								</svg>
							</div>
							<div>
								<h3 class="text-xl font-semibold text-gray-800 mb-3">{$_('about.feature_5')}</h3>
								<p class="text-gray-600 leading-relaxed">
									{$_('about.feature_5')} - {$_('about.feature_6')}
								</p>
							</div>
						</div>

						<div class="flex items-start gap-4">
							<div class="flex-shrink-0 w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
								<svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129"></path>
								</svg>
							</div>
							<div>
								<h3 class="text-xl font-semibold text-gray-800 mb-3">{$_('about.feature_4')}</h3>
								<p class="text-gray-600 leading-relaxed">
									{$_('about.feature_4')} - {$_('about.feature_3')}
								</p>
							</div>
						</div>
					</div>

					<div class="space-y-6">
						<div class="flex items-start gap-4">
							<div class="flex-shrink-0 w-12 h-12 bg-yellow-100 rounded-lg flex items-center justify-center">
								<svg class="w-6 h-6 text-yellow-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
								</svg>
							</div>
							<div>
								<h3 class="text-xl font-semibold text-gray-800 mb-3">Performante</h3>
								<p class="text-gray-600 leading-relaxed">
									Costruito per essere veloce e leggero, con caricamento rapido 
									e animazioni fluide grazie a Svelte e Tauri.
								</p>
							</div>
						</div>

						<div class="flex items-start gap-4">
							<div class="flex-shrink-0 w-12 h-12 bg-red-100 rounded-lg flex items-center justify-center">
								<svg class="w-6 h-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"></path>
								</svg>
							</div>
							<div>
								<h3 class="text-xl font-semibold text-gray-800 mb-3">Facilmente Personalizzabile</h3>
								<p class="text-gray-600 leading-relaxed">
									Codice ben strutturato e documentato che rende facile 
									modificare e estendere le funzionalità secondo le tue esigenze.
								</p>
							</div>
						</div>

						<div class="flex items-start gap-4">
							<div class="flex-shrink-0 w-12 h-12 bg-indigo-100 rounded-lg flex items-center justify-center">
								<svg class="w-6 h-6 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
								</svg>
							</div>
							<div>
								<h3 class="text-xl font-semibold text-gray-800 mb-3">Configurabile</h3>
								<p class="text-gray-600 leading-relaxed">
									Include una pagina delle impostazioni per gestire 
									preferenze utente come tema e lingua dell'applicazione.
								</p>
							</div>
						</div>
					</div>
				</div>
			</div>

			<!-- Getting Started -->
			<div class="bg-white/90 backdrop-blur-sm rounded-lg border border-white/20 p-8 shadow-lg">
				<div class="flex flex-col md:flex-row items-center gap-6">
					<div class="flex-1">
						<h2 class="text-2xl font-bold text-gray-900 mb-4">Come Iniziare</h2>
						<p class="text-gray-700 leading-relaxed">
							Questo template fornisce una base solida per costruire applicazioni desktop moderne. 
							Modifica il contenuto delle pagine, aggiungi nuove funzionalità e personalizza lo stile 
							secondo le tue necessità. Buon sviluppo!
						</p>
					</div>
					<div class="flex-shrink-0">
						<div class="w-24 h-24 bg-gradient-to-br from-green-500 to-blue-600 rounded-full flex items-center justify-center shadow-lg">
							<svg class="w-12 h-12 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"></path>
							</svg>
						</div>
					</div>
				</div>
			</div>

		</div>
	</main>
</div>

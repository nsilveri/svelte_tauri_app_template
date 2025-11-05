<script>
  import { _ } from 'svelte-i18n';
  
  // Example cards data
  $: exampleCards = [
    { id: 1, title: $_('home.example_card_1'), description: $_('home.card_description_1') },
    { id: 2, title: $_('home.example_card_2'), description: $_('home.card_description_2') },
    { id: 3, title: $_('home.example_card_3'), description: $_('home.card_description_3') }
  ];
</script>

<div class="min-h-screen flex flex-col" style="background: linear-gradient(135deg, #c9ffe7 0%, #e9e9ff 70%, #dcecff 100%);">
  
  <!-- HEADER CONTENT -->
  <header class="w-full pt-5 px-5 fixed top-0 left-0 right-0 z-10 bg-transparent">
    <div class="w-full bg-white/50 backdrop-blur-sm rounded-lg border border-black/50 p-4 sm:p-2 shadow-lg flex items-center justify-between">
      <div></div>
      <div class="flex-1 text-center">
        <h1 class="text-2xl font-semibold text-gray-900 mb-1">{$_('home.title')}</h1>
        <p class="text-gray-700">{$_('home.description')}</p>
      </div>
      <div></div>
    </div>
  </header>

  <!-- MAIN CONTENT -->
  <main class="flex-grow pt-5 px-5 mb-8" style="margin-top: 6rem; margin-bottom: 2rem;">
    <div class="w-full h-full overflow-y-auto grid gap-4 pb-20" style="grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); scrollbar-width: thin;">
      {#each exampleCards as card}
        <div class="bg-white/90 backdrop-blur-sm rounded-lg border border-white/20 p-6 text-center shadow-lg">
          <div class="w-full h-32 bg-gray-200 rounded-lg mb-4 flex items-center justify-center">
            <svg class="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
            </svg>
          </div>
          <h2 class="text-xl font-semibold text-gray-900 mb-2">{card.title}</h2>
          <p class="text-gray-600 mb-4">{card.description}</p>
          <button class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded font-medium">
            {$_('home.example_action')}
          </button>
        </div>
      {/each}
    </div>
    
    <!-- Info sezione -->
    <div class="mt-8 bg-white/90 backdrop-blur-sm rounded-lg border border-white/20 p-6 shadow-lg">
      <h2 class="text-2xl font-bold text-gray-900 mb-4">{$_('home.welcome_title')}</h2>
      <p class="text-gray-700 leading-relaxed">
        {$_('home.welcome_description')}
      </p>
      <div class="mt-4 flex gap-4">
        <span class="bg-blue-100 text-blue-800 px-3 py-1 rounded-full text-sm">Template</span>
        <span class="bg-green-100 text-green-800 px-3 py-1 rounded-full text-sm">Svelte</span>
        <span class="bg-purple-100 text-purple-800 px-3 py-1 rounded-full text-sm">Tauri</span>
      </div>
    </div>
  </main>
</div>

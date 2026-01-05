<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Activity, MonitorPlay } from 'lucide-vue-next';

interface StartupApp {
    name: string;
    path: string;
    enabled: boolean;
    processing?: boolean;
}

const apps = ref<StartupApp[]>([]);
const loading = ref(true);

const fetchStartupApps = async () => {
    try {
        apps.value = await invoke('get_startup_apps');
    } catch (e) {
        console.error('Error fetching startup apps:', e);
    } finally {
        loading.value = false;
    }
};

const toggleApp = async (app: StartupApp) => {
    if (app.processing) return;
    
    app.processing = true;
    const newState = !app.enabled;
    
    try {
        await invoke('toggle_startup_app', { 
            name: app.name, 
            path: app.path, 
            enable: newState 
        });
        
        // Update local state on success
        app.enabled = newState;
    } catch (e) {
        console.error('Failed to toggle app:', e);
        // Revert visually if needed, but we haven't changed app.enabled yet so it's fine
        // Ideally show a toast here
    } finally {
        app.processing = false;
    }
};

onMounted(() => {
    fetchStartupApps();
});
</script>

<template>
  <div class="h-full flex flex-col p-4 select-none animate-fade-in glass-container overflow-hidden">
     <!-- Header -->
     <div class="flex justify-between items-center mb-6 pb-2 border-b border-white/5">
      <h2 class="text-xl font-bold text-white tracking-wider flex items-center gap-2">
        STARTUP MANAGER
        <span class="text-xs bg-white/10 text-neon-cpu px-2 py-0.5 rounded font-mono">{{ apps.length }} APPS</span>
      </h2>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-neon-cpu"></div>
    </div>

    <!-- List -->
    <div v-else class="flex-1 overflow-y-auto custom-scrollbar pr-2 pb-2">
      <div v-if="apps.length === 0" class="col-span-full flex flex-col items-center justify-center py-20 text-white/30">
        <Activity class="w-12 h-12 mb-4 opacity-20" />
        <p>No startup applications found.</p>
      </div>

      <div v-else class="grid grid-cols-1 gap-3">
        <div v-for="(app, index) in apps" :key="index" 
             class="bg-white/5 rounded-xl p-4 border border-white/5 hover:bg-white/10 transition-all flex items-center justify-between group">
            
            <div class="flex items-center gap-4 overflow-hidden">
                <div class="p-3 rounded-lg bg-white/5 text-neon-cpu group-hover:text-white transition-colors">
                    <MonitorPlay :size="20" />
                </div>
                <div class="min-w-0">
                    <h3 class="font-bold text-white/90 text-sm truncate">{{ app.name }}</h3>
                    <p class="text-xs text-white/40 truncate font-mono" :title="app.path">{{ app.path }}</p>
                </div>
            </div>

            <!-- Toggle (Visual Only for MVP) -->
            <!-- Toggle Switch -->
             <div class="flex items-center gap-3">
                 <span class="text-[10px] uppercase font-bold tracking-wider" 
                       :class="app.enabled ? 'text-neon-cpu' : 'text-white/30'">
                     {{ app.enabled ? 'ON' : 'OFF' }}
                 </span>
                <button 
                  @click.stop="toggleApp(app)"
                  class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none"
                  :class="app.enabled ? 'bg-neon-cpu/80' : 'bg-white/10'"
                  :disabled="app.processing"
                >
                  <span 
                    class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out"
                    :class="app.enabled ? 'translate-x-5' : 'translate-x-0'"
                  />
                </button>
             </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.glass-container {
  background: rgba(17, 24, 39, 0.85);
  backdrop-filter: blur(20px);
}
.text-neon-cpu {
    color: #00d1ff;
    text-shadow: 0 0 10px rgba(0, 209, 255, 0.3);
}

/* Custom Scrollbar */
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(6, 182, 212, 0.3);
  border-radius: 4px;
}
</style>

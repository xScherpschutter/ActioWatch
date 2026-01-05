<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { X, Cpu, HardDrive, Terminal, FolderOpen, Clock, Activity, AlertCircle, Layers, Box } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  pid: number | null;
  isOpen: boolean;
}>();

const emit = defineEmits(['close']);

interface ProcessDetails {
  pid: number;
  name: string;
  cmd: string[];
  exe: string;
  cwd: string;
  root: string;
  status: string;
  run_time: number;
  memory_usage: number;
  cpu_usage: number;
  environ: string[];
}

const details = ref<ProcessDetails | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const activeTab = ref<'general' | 'env' | 'modules'>('general');

const fetchDetails = async () => {
  if (!props.pid) return;
  loading.value = true;
  error.value = null;
  try {
    details.value = await invoke('get_process_details', { pid: props.pid });
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
};

watch(() => props.isOpen, (isOpen) => {
    if (isOpen && props.pid) {
        fetchDetails();
        activeTab.value = 'general';
    }
});

watch(() => props.pid, (newPid) => {
    if (newPid && props.isOpen) fetchDetails();
});

const formatTime = (seconds: number) => {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${h}h ${m}m ${s}s`;
};

const formatBytes = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(2)} GB`;
};

const envVars = computed(() => {
    if (!details.value?.environ) return [];
    return details.value.environ;
});

interface ModuleInfo {
  name: string;
  path: string;
}

const modules = ref<ModuleInfo[]>([]);
const modulesLoading = ref(false);
const modulesLoaded = ref(false);

const fetchModules = async () => {
    if (!props.pid || modulesLoaded.value) return;
    
    modulesLoading.value = true;
    try {
        modules.value = await invoke('get_process_modules', { pid: props.pid });
        modulesLoaded.value = true;
    } catch (e) {
        console.error("Failed to fetch modules", e);
    } finally {
        modulesLoading.value = false;
    }
};

watch(activeTab, (newTab) => {
    if (newTab === 'modules') {
        fetchModules();
    }
});

watch(() => props.pid, () => {
    // Reset module state on new PID
    modules.value = [];
    modulesLoaded.value = false;
});
</script>

<template>
  <Transition
    enter-active-class="transition duration-300 ease-out"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-200 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-center justify-center p-4">
      
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="emit('close')"></div>

      <!-- Modal Content -->
      <div class="relative w-full max-w-2xl bg-slate-900/95 border border-white/10 rounded-2xl shadow-2xl backdrop-blur-xl overflow-hidden flex flex-col max-h-[90vh]">
        
        <!-- Header -->
        <div class="flex items-center justify-between p-6 border-b border-white/5 bg-white/5">
            <div class="flex items-center gap-4">
                <div class="w-12 h-12 rounded-xl bg-neon-cpu/10 flex items-center justify-center border border-neon-cpu/20">
                    <Activity class="w-6 h-6 text-neon-cpu" />
                </div>
                <div>
                    <h2 class="text-xl font-bold text-white">{{ details?.name || 'Process Details' }}</h2>
                    <span class="text-xs font-mono text-white/50">PID: {{ pid }}</span>
                </div>
            </div>
            <button @click="emit('close')" class="p-2 rounded-lg hover:bg-white/10 text-gray-400 hover:text-white transition-colors">
                <X class="w-5 h-5" />
            </button>
        </div>

        <!-- Body -->
        <div class="flex flex-col flex-grow overflow-hidden">
            
            <!-- Loading State -->
            <div v-if="loading" class="flex flex-col items-center justify-center py-20">
                <div class="w-8 h-8 border-2 border-neon-cpu border-t-transparent rounded-full animate-spin"></div>
                <p class="mt-4 text-sm text-gray-400">Fetching detailed stats...</p>
            </div>

            <!-- Error State -->
            <div v-else-if="error" class="p-6">
                 <div class="flex items-center gap-3 p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400">
                    <AlertCircle class="w-5 h-5 flex-shrink-0" />
                    <p class="text-sm">{{ error }}</p>
                </div>
            </div>

            <!-- Content -->
            <div v-else-if="details" class="flex flex-col h-full overflow-hidden">
                
                <!-- Tabs -->
                <div class="flex border-b border-white/5 px-6 shrink-0">
                    <button 
                        @click="activeTab = 'general'"
                        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2"
                        :class="activeTab === 'general' ? 'border-neon-cpu text-white' : 'border-transparent text-gray-400 hover:text-white'"
                    >
                        <Activity class="w-4 h-4" /> General
                    </button>
                    <button 
                        @click="activeTab = 'env'"
                        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2"
                        :class="activeTab === 'env' ? 'border-neon-cpu text-white' : 'border-transparent text-gray-400 hover:text-white'"
                    >
                         <Layers class="w-4 h-4" /> Environment
                         <span class="text-[10px] bg-white/10 px-1.5 rounded">{{ envVars.length }}</span>
                    </button>
                     <button 
                        @click="activeTab = 'modules'"
                        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2"
                        :class="activeTab === 'modules' ? 'border-neon-cpu text-white' : 'border-transparent text-gray-400 hover:text-white'"
                    >
                        <Box class="w-4 h-4" /> Modules
                        <span v-if="modules.length > 0" class="text-[10px] bg-white/10 px-1.5 rounded">{{ modules.length }}</span>
                    </button>
                </div>

                <!-- Tab Panels -->
                <div class="p-6 overflow-y-auto custom-scrollbar flex-grow min-h-0">
                    
                    <!-- General Tab -->
                    <div v-if="activeTab === 'general'" class="space-y-6 animate-fade-in">
                        <!-- Main Stats Grid -->
                        <div class="grid grid-cols-3 gap-4">
                            <div class="p-4 rounded-lg bg-white/5 border border-white/5 flex flex-col gap-1">
                                <span class="text-xs text-gray-400 flex items-center gap-1"><Cpu class="w-3 h-3" /> CPU Usage</span>
                                <span class="text-xl font-mono text-neon-cpu">{{ details.cpu_usage.toFixed(1) }}%</span>
                            </div>
                            <div class="p-4 rounded-lg bg-white/5 border border-white/5 flex flex-col gap-1">
                                <span class="text-xs text-gray-400 flex items-center gap-1"><HardDrive class="w-3 h-3" /> Memory</span>
                                <span class="text-xl font-mono text-neon-ram">{{ formatBytes(details.memory_usage) }}</span>
                            </div>
                            <div class="p-4 rounded-lg bg-white/5 border border-white/5 flex flex-col gap-1">
                                <span class="text-xs text-gray-400 flex items-center gap-1"><Clock class="w-3 h-3" /> Run Time</span>
                                <span class="text-xl font-mono text-white">{{ formatTime(details.run_time) }}</span>
                            </div>
                        </div>

                         <!-- Command Line Arguments -->
                         <div class="space-y-2">
                            <label class="text-xs uppercase font-bold text-gray-500 tracking-wider flex items-center gap-2">
                                <Terminal class="w-3 h-3" /> Command Line
                            </label>
                            <div class="p-3 rounded-lg bg-black/40 border border-white/10 font-mono text-xs text-gray-300 break-all select-all flex flex-wrap gap-2">
                                <span v-for="(arg, i) in details.cmd" :key="i" class="bg-white/5 px-2 py-1 rounded border border-white/5">
                                    {{ arg }}
                                </span>
                                <span v-if="details.cmd.length === 0" class="text-white/30 italic">No arguments</span>
                            </div>
                        </div>

                        <!-- Executable Path -->
                        <div class="space-y-2">
                            <label class="text-xs uppercase font-bold text-gray-500 tracking-wider flex items-center gap-2">
                                <HardDrive class="w-3 h-3" /> Executable Path
                            </label>
                            <div class="p-3 rounded-lg bg-black/40 border border-white/10 font-mono text-xs text-blue-200 break-all select-all">
                                {{ details.exe || 'N/A' }}
                            </div>
                        </div>

                         <!-- Working Directory -->
                        <div class="space-y-2">
                            <label class="text-xs uppercase font-bold text-gray-500 tracking-wider flex items-center gap-2">
                                <FolderOpen class="w-3 h-3" /> Working Directory
                            </label>
                            <div class="p-3 rounded-lg bg-black/40 border border-white/10 font-mono text-xs text-gray-300 break-all select-all">
                                {{ details.cwd || 'N/A' }}
                            </div>
                        </div>

                        <!-- Status -->
                        <div class="flex items-center justify-between p-3 rounded-lg bg-white/5 border border-white/5">
                            <span class="text-sm text-gray-400">Current Status</span>
                            <span class="px-2 py-1 rounded text-xs font-bold bg-green-500/20 text-green-400 border border-green-500/30 uppercase tracking-wide">
                                {{ details.status }}
                            </span>
                        </div>
                    </div>

                    <!-- Environment Tab -->
                    <div v-else-if="activeTab === 'env'" class="space-y-4 animate-fade-in">
                        <div v-if="envVars.length === 0" class="flex flex-col items-center justify-center py-10 text-white/30">
                            <Layers class="w-12 h-12 mb-4 opacity-20" />
                            <p>No environment variables available.</p>
                        </div>
                        <div v-else class="space-y-2">
                             <div v-for="(env, index) in envVars" :key="index" class="p-3 rounded-lg bg-black/20 border border-white/5 text-xs font-mono break-all hover:bg-black/40 transition-colors">
                                {{ env }}
                             </div>
                        </div>
                    </div>

                     <!-- Modules Tab -->
                     <div v-else-if="activeTab === 'modules'" class="space-y-4 animate-fade-in">
                        <div v-if="modulesLoading" class="flex flex-col items-center justify-center py-12">
                            <div class="w-6 h-6 border-2 border-neon-cpu border-t-transparent rounded-full animate-spin"></div>
                             <p class="mt-2 text-xs text-gray-400">Loading modules...</p>
                        </div>
                        <div v-else-if="modules.length === 0" class="flex flex-col items-center justify-center py-10 text-white/30">
                            <Box class="w-12 h-12 mb-4 opacity-20" />
                            <p>No modules found or accessible.</p>
                        </div>
                        <div v-else class="space-y-2">
                             <div v-for="(mod, index) in modules" :key="index" class="p-3 rounded-lg bg-black/20 border border-white/5 hover:bg-black/40 transition-colors flex flex-col gap-1">
                                <span class="text-sm font-bold text-white/90">{{ mod.name }}</span>
                                <span class="text-[10px] font-mono text-gray-400 break-all">{{ mod.path }}</span>
                             </div>
                        </div>
                    </div>

                </div>
            </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.02);
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

.animate-fade-in {
    animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(5px); }
    to { opacity: 1; transform: translateY(0); }
}
</style>

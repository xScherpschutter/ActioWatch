<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Cpu, Check, X } from 'lucide-vue-next';

const props = defineProps<{
  isOpen: boolean;
  pid: number | null;
  cpuCount: number;
}>();

const emit = defineEmits(['close', 'save']);

const selectedCpus = ref<number[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const fetchAffinity = async () => {
  if (!props.pid) return;
  loading.value = true;
  error.value = null;
  try {
    const affinity = await invoke<number[]>('get_process_affinity', { pid: props.pid });
    selectedCpus.value = affinity;
  } catch (e) {
    console.error('Failed to get affinity:', e);
    error.value = `${e}`;
    // Default to all CPUs on error? No, safer to show empty or error state.
    selectedCpus.value = Array.from({ length: props.cpuCount }, (_, i) => i);
  } finally {
    loading.value = false;
  }
};

watch(() => props.isOpen, (newVal) => {
  if (newVal && props.pid) {
    fetchAffinity();
  }
});

const toggleCpu = (index: number) => {
  if (selectedCpus.value.includes(index)) {
    // Prevent unchecking the last CPU
    if (selectedCpus.value.length > 1) {
      selectedCpus.value = selectedCpus.value.filter(c => c !== index);
    }
  } else {
    selectedCpus.value.push(index);
  }
};

const selectAll = () => {
  selectedCpus.value = Array.from({ length: props.cpuCount }, (_, i) => i);
};

const save = async () => {
  if (!props.pid) return;
  try {
    await invoke('set_process_affinity', { pid: props.pid, cpus: selectedCpus.value });
    emit('save');
    emit('close');
  } catch (e) {
    console.error('Failed to set affinity:', e);
    error.value = `Error saving: ${e}`;
  }
};

</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <!-- Backdrop -->
    <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="emit('close')"></div>

    <!-- Modal Content -->
    <div class="relative w-full max-w-md bg-[#0f172a] border border-white/10 rounded-xl shadow-2xl overflow-hidden flex flex-col max-h-[80vh]">
      
      <!-- Header -->
      <div class="px-6 py-4 border-b border-white/10 flex justify-between items-center bg-white/5">
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-neon-cpu/10 text-neon-cpu">
            <Cpu class="w-5 h-5" />
          </div>
          <h3 class="text-lg font-bold text-white tracking-wide">Processor Affinity</h3>
        </div>
        <button @click="emit('close')" class="text-gray-400 hover:text-white transition-colors">
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 overflow-y-auto custom-scrollbar">
        <p class="text-sm text-gray-400 mb-4">
          Select which processors are allowed to run this process.
        </p>

        <div v-if="error" class="mb-4 text-xs text-red-400 bg-red-500/10 p-2 rounded border border-red-500/20 break-all">
          {{ error }}
        </div>

        <div v-if="loading" class="flex justify-center py-8">
           <div class="w-8 h-8 border-2 border-neon-cpu border-t-transparent rounded-full animate-spin"></div>
        </div>

        <div v-else class="grid grid-cols-2 sm:grid-cols-4 gap-3">
           <button 
             v-for="i in cpuCount" 
             :key="i-1"
             @click="toggleCpu(i-1)"
             class="flex items-center justify-center gap-2 p-2 rounded border transition-all text-xs font-mono"
             :class="selectedCpus.includes(i-1) 
               ? 'bg-neon-cpu/20 border-neon-cpu text-white shadow-[0_0_8px_rgba(0,209,255,0.2)]' 
               : 'bg-white/5 border-white/10 text-gray-500 hover:border-white/30'"
           >
             <span>CPU {{ i-1 }}</span>
             <Check v-if="selectedCpus.includes(i-1)" class="w-3 h-3" />
           </button>
        </div>

      </div>

      <!-- Footer -->
      <div class="px-6 py-4 border-t border-white/10 bg-white/5 flex justify-between items-center">
        <button @click="selectAll" class="text-xs text-neon-cpu hover:text-white transition-colors underline">
          Select All
        </button>
        <div class="flex gap-3">
          <button @click="emit('close')" class="px-4 py-2 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-white/10 transition-colors">
            Cancel
          </button>
          <button @click="save" class="px-4 py-2 rounded-lg text-sm font-bold bg-neon-cpu text-black hover:bg-cyan-300 shadow-[0_0_15px_rgba(0,209,255,0.3)] transition-all">
            Apply Changes
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

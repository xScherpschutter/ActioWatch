<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { RefreshCcw } from 'lucide-vue-next';

interface PortInfo {
  pid: number | null;
  process_name: string;
  port: number;
  protocol: string;
  address: string;
}

const ports = ref<PortInfo[]>([]);
const loading = ref(false);
const searchQuery = ref('');

const filteredPorts = computed(() => {
  if (!searchQuery.value) return ports.value;
  const query = searchQuery.value.toLowerCase();
  return ports.value.filter(p => 
    p.port.toString().includes(query) ||
    p.process_name.toLowerCase().includes(query) ||
    p.address.toLowerCase().includes(query) ||
    (p.pid?.toString().includes(query))
  );
});

const fetchPorts = async () => {
  loading.value = true;
  try {
    ports.value = await invoke('get_open_ports');
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
};

onMounted(() => {
  fetchPorts();
});
</script>

<template>
  <div class="h-full flex flex-col glass-container text-white select-none animate-fade-in">
     <!-- Toolbar / Header -->
     <div class="p-4 flex justify-between items-center border-b border-white/5">
      <div class="flex items-center gap-3">
        <h2 class="text-lg font-bold text-white tracking-wider flex items-center gap-2">
          OPEN PORTS
        </h2>
        <span class="bg-neon-cpu/10 text-neon-cpu px-2 py-0.5 rounded text-xs font-mono font-bold">{{ ports.length }}</span>
      </div>
      
      <div class="flex gap-2">
        <div class="relative group">
            <input 
              v-model="searchQuery" 
              type="text" 
              placeholder="Search ports..." 
              class="glass-input text-xs rounded-lg pl-3 pr-8 py-1.5 focus:outline-none focus:ring-1 focus:ring-neon-cpu/50 w-48 transition-all placeholder-gray-500 text-white"
            />
             <!-- Search Icon could go here -->
        </div>
        <button 
          @click="fetchPorts" 
          class="p-1.5 rounded-lg hover:bg-white/10 text-gray-400 hover:text-white transition-colors"
          :class="{ 'animate-spin': loading }"
          title="Refresh"
        >
          <RefreshCcw :size="16" />
        </button>
      </div>
    </div>

    <!-- Table Header -->
    <div class="grid grid-cols-12 gap-4 px-6 py-3 text-xs font-bold text-white/70 uppercase tracking-wider border-b border-white/10 bg-white/5">
        <div class="col-span-2">Proto</div>
        <div class="col-span-2">Port</div>
        <div class="col-span-3">Address</div>
        <div class="col-span-2">PID</div>
        <div class="col-span-3">Process</div>
    </div>

    <!-- Scrollable Content -->
    <div class="overflow-y-auto flex-grow custom-scrollbar px-2 py-2">
      <div v-for="(port, index) in filteredPorts" :key="index" 
            class="grid grid-cols-12 gap-4 px-4 py-2.5 border-b border-white/5 hover:bg-white/5 transition-colors text-sm items-center rounded-lg group">
          
          <!-- Protocol -->
          <div class="col-span-2 font-mono text-neon-cpu text-xs">{{ port.protocol || 'TCP' }}</div>
          
          <!-- Port -->
          <div class="col-span-2 font-mono text-white/90 font-bold">{{ port.port }}</div>
          
          <!-- Address -->
          <div class="col-span-3 font-mono text-white/60 text-xs truncate" :title="port.address">{{ port.address }}</div>
          
          <!-- PID -->
          <div class="col-span-2 text-white/50 font-mono text-xs">{{ port.pid || '-' }}</div>
          
          <!-- Process Name -->
          <div class="col-span-3 font-medium text-white/90 truncate flex items-center gap-2" :title="port.process_name">
             <div class="w-1.5 h-1.5 rounded-full bg-green-500" v-if="port.pid"></div>
             <div class="w-1.5 h-1.5 rounded-full bg-gray-600" v-else></div>
             {{ port.process_name }}
          </div>
      </div>
       
       <!-- Empty State -->
       <div v-if="filteredPorts.length === 0" class="flex flex-col items-center justify-center py-20 text-white/30">
        <p>No ports found</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.glass-container {
  background: rgba(17, 24, 39, 0.85); /* Darker, less transparent */
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.glass-input {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.text-neon-cpu {
  color: #00d1ff;
  text-shadow: 0 0 10px rgba(0, 209, 255, 0.3);
}

/* Custom Scrollbar (Matched to ProcessManager) */
.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(6, 182, 212, 0.3);
  border-radius: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(6, 182, 212, 0.5);
}
</style>

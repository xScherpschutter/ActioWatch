<script setup lang="ts">
import { ref, computed } from 'vue';
import { Search, X, Box } from 'lucide-vue-next';

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_usage: number;
  thread_count: number;
}

const props = defineProps<{
  processes: ProcessInfo[];
  totalCpu: number;
  memoryUsed: number;
  memoryTotal: number;
}>();

const emit = defineEmits(['kill-process']);

const searchQuery = ref('');

const totalThreads = computed(() => {
  return props.processes.reduce((acc, process) => acc + process.thread_count, 0);
});

const filteredProcesses = computed(() => {
  if (!searchQuery.value) return props.processes;
  const query = searchQuery.value.toLowerCase();
  return props.processes.filter(p => 
    p.name.toLowerCase().includes(query) || 
    p.pid.toString().includes(query)
  );
});

const getUsageColor = (usage: number) => {
  if (usage > 70) return 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]';
  if (usage > 40) return 'bg-orange-500 shadow-[0_0_8px_rgba(249,115,22,0.6)]';
  return 'bg-neon-cpu shadow-[0_0_8px_rgba(0,209,255,0.6)]';
};

const formatBytes = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(1)} GB`;
};
</script>

<template>
  <div class="h-full flex flex-col bg-gray-900/90 text-white select-none">

    <!-- Toolbar -->
    <div class="p-4 flex gap-3">
      <div class="relative flex-grow group">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 group-focus-within:text-neon-cpu transition-colors" />
        <input 
          v-model="searchQuery"
          type="text" 
          placeholder="Filter processes by name or PID..." 
          class="w-full bg-gray-800/50 border border-white/10 rounded-lg pl-10 pr-4 py-2 text-sm focus:outline-none focus:border-neon-cpu/50 focus:ring-1 focus:ring-neon-cpu/50 transition-all placeholder-gray-600"
        />
      </div>
    </div>

    <!-- Table Header -->
    <div class="grid grid-cols-12 gap-4 px-6 py-2 text-xs font-bold text-gray-500 uppercase tracking-wider border-b border-white/5">
      <div class="col-span-4">Process Name</div>
      <div class="col-span-1">PID</div>
      <div class="col-span-1">Threads</div>
      <div class="col-span-2">CPU %</div>
      <div class="col-span-2">Memory</div>
      <div class="col-span-2 text-right">Action</div>
    </div>

    <!-- Process List -->
    <div class="flex-grow overflow-y-auto custom-scrollbar px-2">
      <div v-for="process in filteredProcesses" :key="process.pid" 
           class="grid grid-cols-12 gap-4 px-4 py-3 items-center hover:bg-white/5 rounded-lg transition-colors group border-b border-white/5 last:border-0">
        
        <!-- Name -->
        <div class="col-span-4 flex items-center gap-3">
          <div class="w-8 h-8 rounded bg-gray-700/50 flex items-center justify-center text-gray-400">
            <Box class="w-4 h-4" />
          </div>
          <span class="font-medium text-sm text-gray-200 group-hover:text-white truncate">{{ process.name }}</span>
        </div>

        <!-- PID -->
        <div class="col-span-1 font-mono text-xs text-gray-500">
          {{ process.pid }}
        </div>

        <!-- Threads -->
        <div class="col-span-1 font-mono text-xs text-gray-500">
          {{ process.thread_count }}
        </div>

        <!-- CPU -->
        <div class="col-span-2">
          <div class="flex flex-col gap-1">
            <div class="flex justify-between text-[10px] text-gray-400 font-mono">
              <span>{{ process.cpu_usage.toFixed(1) }}%</span>
            </div>
            <div class="h-1.5 bg-gray-700/50 rounded-full overflow-hidden">
              <div 
                class="h-full rounded-full transition-all duration-500"
                :class="getUsageColor(process.cpu_usage)"
                :style="{ width: `${Math.min(process.cpu_usage, 100)}%` }"
              ></div>
            </div>
          </div>
        </div>

        <!-- Memory -->
        <div class="col-span-2">
          <div class="flex flex-col gap-1">
            <div class="flex justify-between text-[10px] text-gray-400 font-mono">
              <span>{{ formatBytes(process.memory_usage) }}</span>
            </div>
            <div class="h-1.5 bg-gray-700/50 rounded-full overflow-hidden">
              <div 
                class="h-full bg-neon-ram rounded-full transition-all duration-500"
                :style="{ width: `${Math.min((process.memory_usage / memoryTotal) * 100, 100)}%` }"
              ></div>
            </div>
          </div>
        </div>

        <!-- Action -->
        <div class="col-span-2 flex justify-end">
          <button @click="emit('kill-process', process.pid)" 
                  class="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-red-500/20 hover:border-red-500/50 border border-transparent transition-all">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- Footer Stats -->
    <div class="h-16 bg-gray-800/40 border-t border-white/5 backdrop-blur-md px-6 flex items-center justify-between">
      <div class="flex gap-8">
        <div class="flex flex-col">
          <span class="text-[10px] text-gray-500 uppercase font-bold">Total Processes</span>
          <span class="text-lg font-mono text-white">{{ processes.length }}</span>
        </div>
        <div class="flex flex-col">
          <span class="text-[10px] text-gray-500 font-bold uppercase tracking-wider">Threads</span>
          <span class="text-lg font-mono font-bold text-white">{{ totalThreads }}</span>
        </div>
      </div>

      <div class="flex gap-8 items-center">
        <!-- Total CPU -->
        <div class="w-48">
          <div class="flex justify-between text-xs mb-1">
             <span class="text-gray-400">Total CPU Load</span>
             <span class="text-neon-cpu font-bold">{{ totalCpu.toFixed(1) }}%</span>
          </div>
          <div class="h-1.5 bg-gray-700 rounded-full overflow-hidden">
            <div class="h-full bg-neon-cpu shadow-[0_0_10px_rgba(0,209,255,0.5)] transition-all duration-300"
                 :style="{ width: `${Math.min(totalCpu, 100)}%` }"></div>
          </div>
        </div>

        <!-- Memory -->
        <div class="w-48">
          <div class="flex justify-between text-xs mb-1">
             <span class="text-gray-400">Memory</span>
             <span class="text-neon-ram font-bold">{{ formatBytes(memoryUsed) }} / {{ formatBytes(memoryTotal) }}</span>
          </div>
          <div class="h-1.5 bg-gray-700 rounded-full overflow-hidden">
            <div class="h-full bg-neon-ram shadow-[0_0_10px_rgba(66,255,0,0.5)] transition-all duration-300"
                 :style="{ width: `${(memoryUsed / memoryTotal) * 100}%` }"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

```vue
<script setup lang="ts">
import { ArrowDown, ArrowUp } from 'lucide-vue-next';

const props = defineProps<{
  stats: any;
  totalCpu: number;
  memoryUsed: number;
  memoryTotal: number;
}>();

const formatSpeed = (bytes: number) => {
  if (bytes === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

// Helper to format bytes
const formatBytes = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024);
  return gb.toFixed(1);
};
</script>

<template>
  <div class="h-full flex flex-col p-4 space-y-4 select-none">
    <!-- CPU Section -->
    <div class="glass-panel rounded-xl p-3 relative overflow-hidden group">
      <div class="flex justify-between items-center mb-1 relative z-10">
        <span class="text-xs font-bold text-gray-400 tracking-wider">CPU LOAD</span>
        <span class="text-lg font-bold text-neon-cpu neon-text-cpu">{{ props.stats.cpu_usage.toFixed(0) }}%</span>
      </div>
      <!-- Simulated Wave Chart -->
      <div class="h-8 w-full bg-gray-900/50 rounded flex items-end overflow-hidden relative">
        <div class="absolute inset-0 opacity-20 bg-neon-cpu/20"></div>
        <div class="w-full h-full flex items-end gap-0.5 px-1 pb-1">
           <div v-for="i in 20" :key="i" 
                class="flex-1 bg-neon-cpu rounded-t-sm transition-all duration-300"
                :style="{ height: `${Math.random() * 80 + 20}%`, opacity: 0.5 + (i/40) }">
           </div>
        </div>
      </div>
    </div>

    <!-- RAM Section -->
    <div class="glass-panel rounded-xl p-3 relative overflow-hidden group">
      <div class="flex justify-between items-center mb-1 relative z-10">
        <span class="text-xs font-bold text-gray-400 tracking-wider">RAM USAGE</span>
        <span class="text-lg font-bold text-neon-ram neon-text-ram">
          {{ ((props.memoryUsed / props.memoryTotal) * 100).toFixed(0) }}%
        </span>
      </div>
      <div class="flex justify-between text-xs text-gray-500 mb-2">
        <span>{{ formatBytes(props.memoryUsed) }} GB</span>
        <span>{{ formatBytes(props.memoryTotal) }} GB</span>
      </div>
      <!-- Simulated Wave Chart -->
      <div class="h-8 w-full bg-gray-900/50 rounded flex items-end overflow-hidden relative">
        <div class="absolute inset-0 opacity-20 bg-neon-ram/20"></div>
        <div class="w-full h-full flex items-end gap-0.5 px-1 pb-1">
           <div v-for="i in 20" :key="i" 
                class="flex-1 bg-neon-ram rounded-t-sm transition-all duration-500"
                :style="{ height: `${Math.random() * 60 + 30}%`, opacity: 0.5 + (i/40) }">
           </div>
        </div>
      </div>
    </div>

    <!-- GPU Section (Conditional) -->
    <div v-if="props.stats.gpu_usage !== undefined" class="glass-panel rounded-xl p-3 relative overflow-hidden group">
      <div class="flex justify-between items-center mb-1 relative z-10">
        <span class="text-xs font-bold text-gray-400 tracking-wider">GPU LOAD</span>
        <span class="text-lg font-bold text-neon-gpu neon-text-gpu">{{ props.stats.gpu_usage.toFixed(0) }}%</span>
      </div>
      <!-- Simulated Wave Chart -->
      <div class="h-8 w-full bg-gray-900/50 rounded flex items-end overflow-hidden relative">
        <div class="absolute inset-0 opacity-20 bg-neon-gpu/20"></div>
        <div class="w-full h-full flex items-end gap-0.5 px-1 pb-1">
           <div v-for="i in 20" :key="i" 
                class="flex-1 bg-neon-gpu rounded-t-sm transition-all duration-700"
                :style="{ height: `${Math.random() * (props.stats.gpu_usage / 2) + (props.stats.gpu_usage / 2)}%`, opacity: 0.5 + (i/40) }">
           </div>
        </div>
      </div>
    </div>

    <div class="flex-grow"></div>

    <!-- Network Footer -->
    <div class="grid grid-cols-2 gap-2 mt-auto">
      <!-- Network -->
      <div class="glass-panel rounded-lg p-2 col-span-2 space-y-2">
        <div class="flex justify-between items-center text-xs text-gray-400">
          <div class="flex items-center gap-1">
            <ArrowUp class="w-3 h-3 text-neon-alert" />
            <span>Upload</span>
          </div>
          <span class="font-mono text-white">{{ formatSpeed(props.stats.network_up || 0) }}</span>
        </div>
        <div class="flex justify-between items-center text-xs text-gray-400">
          <div class="flex items-center gap-1">
            <ArrowDown class="w-3 h-3 text-neon-ram" />
            <span>Download</span>
          </div>
          <span class="font-mono text-white">{{ formatSpeed(props.stats.network_down || 0) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
```

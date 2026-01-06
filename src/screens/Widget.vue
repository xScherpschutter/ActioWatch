<script setup lang="ts">

const props = withDefaults(defineProps<{
  stats: any;
  totalCpu: number;
  memoryUsed: number;
  memoryTotal: number;
}>(), {
    stats: () => ({
        gpu_usage: undefined,
        network_up: 0,
        network_down: 0,
        disk_read: 0,
        disk_write: 0
    })
});

const formatSpeed = (bytes: number) => {
  if (!bytes) return '0 B/s';
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
  <div class="h-full flex flex-col p-4 space-y-4 select-none glass-container">
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
        <!-- <div class="absolute inset-0 opacity-20 bg-neon-ram/20"></div> -->
        <div class="w-full h-full flex items-end gap-0.5 px-1 pb-1">
           <div v-for="i in 20" :key="i" 
                class="flex-1 bg-neon-ram rounded-t-sm transition-all duration-500"
                :style="{ height: `${Math.random() * 60 + 30}%`, opacity: 0.5 + (i/40) }">
           </div>
        </div>
      </div>
    </div>

    <!-- GPU Section (Conditional) -->
    <div v-if="props.stats?.gpu_usage != null" class="glass-panel rounded-xl p-3 relative overflow-hidden group">
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

    <!-- Disk Section -->
    <div class="glass-panel rounded-xl p-3 relative overflow-hidden group">
      <div class="flex justify-between items-center mb-1 relative z-10">
        <span class="text-xs font-bold text-gray-400 tracking-wider">DISK I/O</span>
        <div class="flex gap-2 text-[10px] font-mono">
            <span class="text-orange-300">R: {{ formatSpeed(props.stats.disk_read || 0) }}</span>
            <span class="text-blue-300">W: {{ formatSpeed(props.stats.disk_write || 0) }}</span>
        </div>
      </div>
      <!-- Simulated Wave Chart for Disk -->
      <div class="h-8 w-full bg-gray-900/50 rounded flex items-end overflow-hidden relative">
        <div class="absolute inset-0 opacity-20 bg-orange-500/10"></div>
        <div class="w-full h-full flex items-end gap-0.5 px-1 pb-1">
           <!-- Visualize Read (Orange) and Write (Blue) mixed? Or just activity? Just activity based on sum? -->
           <div v-for="i in 20" :key="i" 
                class="flex-1 bg-orange-400 rounded-t-sm transition-all duration-300"
                :style="{ height: `${Math.min(((props.stats.disk_read + props.stats.disk_write) / 1024 / 1024) * 10, 100) * Math.random()}%`, opacity: 0.5 + (i/40) }">
           </div>
        </div>
      </div>
    </div>



    <!-- Network Graph Section -->
    <div class="glass-panel rounded-xl p-3 relative overflow-hidden group">
        <div class="flex justify-between items-center mb-1 relative z-10">
            <span class="text-xs font-bold text-gray-400 tracking-wider">NETWORK</span>
             <div class="flex gap-2 text-[10px] font-mono">
                <span class="text-neon-alert">Up: {{ formatSpeed(props.stats.network_up || 0) }}</span>
                <span class="text-neon-ram">Down: {{ formatSpeed(props.stats.network_down || 0) }}</span>
            </div>
        </div>
         <!-- Simulated Wave Chart for Network -->
      <div class="h-8 w-full bg-gray-900/50 rounded flex items-end overflow-hidden relative">
        <div class="absolute inset-0 opacity-20 bg-blue-500/10"></div>
        <div class="w-full h-full flex items-end gap-0.5 px-1 pb-1">
           <div v-for="i in 20" :key="i" 
                class="flex-1 bg-blue-400 rounded-t-sm transition-all duration-300"
                :style="{ height: `${Math.min(((props.stats.network_down + props.stats.network_up) / 1024 / 1024) * 5, 100) * Math.random()}%`, opacity: 0.5 + (i/40) }">
           </div>
        </div>
      </div>
    </div>

    <!-- Temperature Section -->
    <div v-if="props.stats?.components && props.stats.components.length > 0" class="glass-panel rounded-xl p-3 relative overflow-hidden group">
      <div class="flex justify-between items-center mb-1 relative z-10">
        <span class="text-xs font-bold text-gray-400 tracking-wider">TEMPERATURES</span>
      </div>
      <div class="flex flex-col gap-1">
         <div v-for="comp in props.stats.components.slice(0, 3)" :key="comp.label" class="flex justify-between text-xs items-center">
             <span class="text-gray-400 truncate w-24" :title="comp.label">{{ comp.label }}</span>
             <span class="font-mono font-bold" :class="(comp.temperature || 0) > 80 ? 'text-orange-500' : 'text-neon-cpu'">
                 {{ (comp.temperature || 0).toFixed(0) }}°C
             </span>
         </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.glass-container {
  background: rgba(17, 24, 39, 0.7);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  padding: 1rem;
}

.glass-panel {
  background: rgba(31, 41, 55, 0.5);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
}
</style>

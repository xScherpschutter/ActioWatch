<script setup lang="ts">
import { Thermometer, Activity } from 'lucide-vue-next';

interface ComponentInfo {
  label: String;
  temperature: number;
  max_temperature: number;
  critical_temperature: number | null;
}

withDefaults(defineProps<{
  components?: ComponentInfo[];
}>(), {
  components: () => []
});

const getTempColor = (temp: number, critical: number | null) => {
  if (critical && temp >= critical) return 'text-red-500 shadow-[0_0_10px_rgba(239,68,68,0.5)]';
  if (temp > 80) return 'text-orange-500';
  if (temp > 60) return 'text-yellow-400';
  return 'text-neon-cpu';
};

const getProgressBarColor = (temp: number) => {
    if (temp > 80) return 'bg-orange-500 shadow-[0_0_8px_rgba(249,115,22,0.6)]';
    if (temp > 60) return 'bg-yellow-400 shadow-[0_0_8px_rgba(250,204,21,0.6)]';
    return 'bg-neon-cpu shadow-[0_0_8px_rgba(0,209,255,0.6)]';
}
</script>

<template>
  <div class="h-full flex flex-col p-4 select-none animate-fade-in glass-container overflow-hidden">
     <!-- Header -->
     <div class="flex justify-between items-center mb-6 pb-2 border-b border-white/5">
      <h2 class="text-xl font-bold text-white tracking-wider flex items-center gap-2">
        HARDWARE MONITOR
        <span class="text-xs bg-white/10 text-neon-cpu px-2 py-0.5 rounded font-mono">{{ components.length }} SENSORS</span>
      </h2>
    </div>

    <!-- Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 overflow-y-auto custom-scrollbar pr-2 pb-2">
      <div v-for="(comp, index) in components" :key="index" 
           class="bg-white/5 rounded-xl p-4 border border-white/5 hover:bg-white/10 transition-all group flex flex-col gap-3">
        
        <div class="flex justify-between items-start">
            <div class="flex items-center gap-3">
                <div class="p-2 rounded-lg bg-white/5 text-gray-400 group-hover:text-white transition-colors">
                    <Thermometer :size="20" />
                </div>
                <div>
                     <h3 class="font-bold text-white/90 text-sm truncate w-32" :title="comp.label.toString()">{{ comp.label }}</h3>
                     <span class="text-[10px] text-white/40 uppercase tracking-widest font-bold">Temperature</span>
                </div>
            </div>
            <div class="text-2xl font-mono font-bold" :class="getTempColor(comp.temperature || 0, comp.critical_temperature)">
                {{ (comp.temperature || 0).toFixed(0) }}°C
            </div>
        </div>

        <!-- Progress Bar -->
        <div class="mt-2">
            <div class="flex justify-between text-[10px] text-white/40 mb-1 font-mono">
                <span>0°C</span>
                <span v-if="comp.critical_temperature">Crit: {{ comp.critical_temperature }}°C</span>
                <span v-else>Max: {{ comp.max_temperature }}°C</span>
            </div>
            <div class="h-2 bg-gray-700/50 rounded-full overflow-hidden">
                <div class="h-full rounded-full transition-all duration-1000 ease-out"
                     :class="getProgressBarColor(comp.temperature || 0)"
                     :style="{ width: `${Math.min(((comp.temperature || 0) / (comp.critical_temperature || 100)) * 100, 100)}%` }">
                </div>
            </div>
        </div>
      </div>

       <!-- Empty State -->
       <div v-if="components.length === 0" class="col-span-full flex flex-col items-center justify-center py-20 text-white/30">
        <Activity class="w-12 h-12 mb-4 opacity-20" />
        <p>No hardware sensors detected.</p>
        <p class="text-xs mt-2 opacity-50">Note: Sensor support depends on OS and Hardware compatibility.</p>
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

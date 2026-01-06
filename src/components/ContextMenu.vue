<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { Activity, Cpu, ChevronsUp, ChevronUp, Minus, ChevronDown, ChevronsDown, X, ChevronRight } from 'lucide-vue-next';

defineProps<{
  x: number;
  y: number;
  pid: number;
}>();

const emit = defineEmits(['close', 'set-priority', 'set-affinity', 'kill']);

const menuRef = ref<HTMLElement | null>(null);

const handleClickOutside = (event: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    emit('close');
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});

const priorities = [
  { label: 'Realtime', value: 'Realtime', icon: ChevronsUp, color: 'text-red-500' },
  { label: 'High', value: 'High', icon: ChevronUp, color: 'text-orange-400' },
  { label: 'Above Normal', value: 'Above Normal', icon: ChevronUp, color: 'text-yellow-400' },
  { label: 'Normal', value: 'Normal', icon: Minus, color: 'text-blue-400' },
  { label: 'Below Normal', value: 'Below Normal', icon: ChevronDown, color: 'text-cyan-400' },
  { label: 'Low', value: 'Low', icon: ChevronsDown, color: 'text-green-400' },
];

</script>

<template>
  <div 
    ref="menuRef"
    class="fixed z-50 w-48 bg-gray-900/95 backdrop-blur-md border border-white/10 rounded-lg shadow-xl text-sm text-gray-200"
    :style="{ top: `${y}px`, left: `${x}px` }"
  >
    <div class="py-1">
      <div class="px-3 py-1 text-xs font-bold text-gray-500 uppercase tracking-wider border-b border-white/5 mb-1">
        Process {{ pid }}
      </div>

      <!-- Priority Submenu (Simplified as a list for now, or could be nested) -->
      <div class="group relative px-3 py-2 hover:bg-white/10 cursor-pointer flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Activity class="w-4 h-4 text-neon-cpu" />
          <span>Set Priority</span>
        </div>
        <ChevronRight class="w-3 h-3 text-gray-500" />
        
        <!-- Nested Menu -->
        <div class="absolute left-full top-0 ml-1 w-40 bg-gray-900/95 backdrop-blur-md border border-white/10 rounded-lg shadow-xl hidden group-hover:block">
           <div v-for="p in priorities" :key="p.value"
                @click.stop="emit('set-priority', p.value)"
                class="px-3 py-2 hover:bg-white/10 cursor-pointer flex items-center gap-2">
             <component :is="p.icon" class="w-3 h-3" :class="p.color" />
             <span>{{ p.label }}</span>
           </div>
        </div>
      </div>

      <div @click="emit('set-affinity')" class="px-3 py-2 hover:bg-white/10 cursor-pointer flex items-center gap-2">
        <Cpu class="w-4 h-4 text-neon-ram" />
        <span>Set Affinity</span>
      </div>

      <div class="my-1 border-t border-white/10"></div>

      <div @click="emit('kill')" class="px-3 py-2 hover:bg-red-500/20 text-red-400 hover:text-red-300 cursor-pointer flex items-center gap-2">
        <X class="w-4 h-4" />
        <span>End Task</span>
      </div>
    </div>
  </div>
</template>

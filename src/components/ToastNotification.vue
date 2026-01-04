<script setup lang="ts">
import { ref } from 'vue';
import { AlertTriangle, XCircle, Fan, Info, ChevronDown, ChevronUp } from 'lucide-vue-next';

defineProps<{
  type: 'warning' | 'alert';
  title: string;
  message: string;
  details?: string;
  visible: boolean;
}>();

const emit = defineEmits(['close', 'action']);

const expanded = ref(false);

const toggleDetails = () => {
    expanded.value = !expanded.value;
};
</script>

<template>
  <Transition
    enter-active-class="transform ease-out duration-300 transition"
    enter-from-class="translate-y-2 opacity-0 sm:translate-y-0 sm:translate-x-2"
    enter-to-class="translate-y-0 opacity-100 sm:translate-x-0"
    leave-active-class="transition ease-in duration-100"
    leave-from-class="opacity-100"
    leave-to-class="opacity-0"
  >
    <div v-if="visible" 
         class="fixed bottom-4 right-4 w-96 glass-toast rounded-xl overflow-hidden z-50 shadow-2xl transition-all duration-300 border border-white/10"
         :class="type === 'alert' ? 'shadow-[0_0_15px_rgba(239,68,68,0.3)]' : 'shadow-[0_0_15px_rgba(0,209,255,0.3)]'">
      
      <!-- Accent Line -->
      <div class="absolute left-0 top-0 bottom-0 w-1" 
           :class="type === 'alert' ? 'bg-neon-alert' : 'bg-neon-gpu'"></div>

      <div class="p-4 relative">
        <div class="flex items-start gap-3">
          <!-- Icon -->
          <div class="flex-shrink-0 mt-0.5">
            <div class="p-2 rounded-full bg-white/5 border border-white/10">
              <AlertTriangle v-if="type === 'alert'" class="h-5 w-5 text-neon-alert" />
              <Info v-else class="h-5 w-5 text-neon-cpu" /> <!-- Using neon-cpu/gpu color -->
            </div>
          </div>

          <!-- Content -->
          <div class="flex-1 w-0">
            <h3 class="text-sm font-bold text-white leading-5">{{ title }}</h3>
            <p class="mt-1 text-xs text-blue-200/70 leading-relaxed">
              {{ message }}
            </p>
            
            <!-- Expanded Details -->
            <div v-if="expanded && details" class="mt-3 p-3 rounded-lg bg-black/30 border border-white/5 text-[10px] font-mono text-gray-300 break-all animate-fade-in">
                {{ details }}
            </div>
          </div>

          <!-- Close Button -->
          <button @click="emit('close')" class="flex-shrink-0 text-gray-500 hover:text-white transition-colors">
            <XCircle class="h-5 w-5" />
          </button>
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center justify-end gap-2 mt-4 ml-11">
             <button v-if="type === 'alert'"
                      @click="emit('action')"
                      class="px-3 py-1.5 rounded-md text-xs font-semibold bg-neon-alert/10 text-neon-alert border border-neon-alert/20 hover:bg-neon-alert/20 transition-all flex items-center gap-1.5">
                <Fan class="w-3.5 h-3.5" />
                FIX ISSUE
              </button>

              <button v-if="details"
                      @click="toggleDetails"
                      class="px-3 py-1.5 rounded-md text-xs font-medium text-gray-300 hover:text-white hover:bg-white/5 transition-all flex items-center gap-1">
                DETAILS
                <component :is="expanded ? ChevronUp : ChevronDown" class="w-3.5 h-3.5" />
              </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.glass-toast {
  background: rgba(15, 23, 42, 0.85);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
}

.animate-fade-in {
    animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(-5px); }
    to { opacity: 1; transform: translateY(0); }
}
</style>

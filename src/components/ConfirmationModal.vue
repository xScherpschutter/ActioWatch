<script setup lang="ts">
import { AlertTriangle } from 'lucide-vue-next';

defineProps<{
  isOpen: boolean;
  title?: string;
  message?: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'danger' | 'warning' | 'info';
}>();

const emit = defineEmits(['confirm', 'cancel']);
</script>

<template>
  <Transition
    enter-active-class="transition duration-200 ease-out"
    enter-from-class="opacity-0 scale-95"
    enter-to-class="opacity-100 scale-100"
    leave-active-class="transition duration-150 ease-in"
    leave-from-class="opacity-100 scale-100"
    leave-to-class="opacity-0 scale-95"
  >
    <div v-if="isOpen" class="fixed inset-0 z-[150] flex items-center justify-center p-4">
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/70 backdrop-blur-sm" @click="emit('cancel')"></div>

      <!-- Modal -->
      <div class="relative w-full max-w-sm bg-slate-900 border border-white/10 rounded-xl shadow-2xl p-6 flex flex-col gap-4">
        
        <div class="flex items-start gap-4">
            <div class="w-10 h-10 rounded-full bg-red-500/10 flex items-center justify-center shrink-0 border border-red-500/20">
                <AlertTriangle class="w-5 h-5 text-red-500" />
            </div>
            <div>
                <h3 class="text-lg font-bold text-white">{{ title || 'Are you sure?' }}</h3>
                <p class="text-sm text-gray-400 mt-1">{{ message || 'This action cannot be undone.' }}</p>
            </div>
        </div>

        <div class="flex justify-end gap-3 mt-2">
            <button 
                @click="emit('cancel')"
                class="px-4 py-2 rounded-lg bg-white/5 hover:bg-white/10 text-gray-300 hover:text-white text-sm font-medium transition-colors"
            >
                {{ cancelText || 'Cancel' }}
            </button>
            <button 
                @click="emit('confirm')"
                class="px-4 py-2 rounded-lg bg-red-500 hover:bg-red-600 text-white text-sm font-bold shadow-lg shadow-red-500/20 transition-all"
            >
                {{ confirmText || 'Confirm' }}
            </button>
        </div>

      </div>
    </div>
  </Transition>
</template>

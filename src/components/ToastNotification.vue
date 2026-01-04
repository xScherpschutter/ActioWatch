<script setup lang="ts">
import { AlertTriangle, XCircle, Fan, Info } from 'lucide-vue-next';

defineProps<{
  type: 'warning' | 'alert';
  title: string;
  message: string;
  details?: string;
  visible: boolean;
}>();

const emit = defineEmits(['close', 'action']);
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
         class="fixed bottom-4 right-4 w-80 glass rounded-lg overflow-hidden z-50 border-l-4"
         :class="type === 'alert' ? 'border-l-neon-alert' : 'border-l-neon-gpu'">
      
      <div class="p-4">
        <div class="flex items-start">
          <div class="flex-shrink-0">
            <AlertTriangle v-if="type === 'alert'" class="h-6 w-6 text-neon-alert" />
            <Info v-else class="h-6 w-6 text-neon-gpu" />
          </div>
          <div class="ml-3 w-0 flex-1 pt-0.5">
            <p class="text-sm font-medium text-white">{{ title }}</p>
            <p class="mt-1 text-sm text-gray-300">
              {{ message }} 
              <span v-if="details" :class="type === 'alert' ? 'text-neon-alert font-bold' : 'text-neon-gpu font-bold'">
                {{ details }}
              </span>
            </p>
            
            <div class="mt-3 flex space-x-3">
              <button v-if="type === 'alert'"
                      @click="emit('action')"
                      class="bg-neon-alert/10 text-neon-alert hover:bg-neon-alert/20 px-3 py-1.5 rounded text-xs font-medium transition-colors flex items-center gap-1 border border-neon-alert/30">
                <Fan class="w-3 h-3" /> FAN CONTROL
              </button>
              <button v-else
                      @click="emit('action')"
                      class="bg-neon-gpu/10 text-neon-gpu hover:bg-neon-gpu/20 px-3 py-1.5 rounded text-xs font-medium transition-colors border border-neon-gpu/30">
                DETAILS
              </button>
              
              <button @click="emit('close')"
                      class="text-gray-400 hover:text-white text-xs font-medium transition-colors">
                {{ type === 'alert' ? 'DISMISS' : 'CLOSE' }}
              </button>
            </div>
          </div>
          <div class="ml-4 flex-shrink-0 flex">
            <button @click="emit('close')" class="inline-flex text-gray-400 hover:text-gray-200 focus:outline-none">
              <XCircle class="h-5 w-5" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

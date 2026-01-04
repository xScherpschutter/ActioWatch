<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import Widget from "./screens/Widget.vue";
import ProcessManager from "./screens/ProcessManager.vue";
import ToastNotification from "./components/ToastNotification.vue";
import TitleBar from "./components/TitleBar.vue";
import { isWindows } from "./utils/platform";


// State
const currentView = ref<'widget' | 'process'>('process'); // Default to process manager for now
const stats = ref({
  cpu_usage: 0,
  memory_used: 0,
  memory_total: 0,
  top_processes: []
});

// Platform detection
const showTitleBar = ref(false);

// Notifications
const showNotification = ref(false);
const notificationData = ref({
  type: 'alert' as 'alert' | 'warning',
  title: '',
  message: '',
  details: ''
});

// Event Listeners
let unlistenStats: () => void;
let unlistenViewChange: () => void;

onMounted(async () => {
  // Check platform and show title bar only on Windows
  showTitleBar.value = await isWindows();

  // Listen for stats updates
  unlistenStats = await listen('stats-update', (event: any) => {
    stats.value = event.payload;
  });

  // Listen for view changes from tray
  unlistenViewChange = await listen('view-change', (event: any) => {
    currentView.value = event.payload as 'widget' | 'process';
  });
});

onUnmounted(() => {
  if (unlistenStats) unlistenStats();
  if (unlistenViewChange) unlistenViewChange();
});

// Actions
const killProcess = async (pid: number) => {
  try {
    await invoke('kill_process', { pid });
    // Optimistic update or wait for next stat refresh
    notificationData.value = {
      type: 'warning',
      title: 'Process Terminated',
      message: `Process ${pid} has been killed.`,
      details: ''
    };
    showNotification.value = true;
    setTimeout(() => showNotification.value = false, 3000);
  } catch (e) {
    console.error(e);
    notificationData.value = {
      type: 'alert',
      title: 'Error',
      message: `Failed to kill process ${pid}`,
      details: String(e)
    };
    showNotification.value = true;
  }
};

const toggleView = () => {
  currentView.value = currentView.value === 'widget' ? 'process' : 'widget';
};
</script>

<template>
  <div class="h-screen w-screen bg-gray-900 text-white overflow-hidden flex flex-col relative">
    
    <!-- Custom Title Bar (Windows only) -->
    <TitleBar v-if="showTitleBar" />
    
    <!-- View Switcher (Temporary for dev/testing if needed, or hidden) -->
    <div :class="['absolute right-2 z-50 opacity-0 hover:opacity-100 transition-opacity', showTitleBar ? 'top-10' : 'top-2']">
      <button @click="toggleView" class="bg-gray-800 p-2 rounded text-xs border border-white/10">
        Switch View
      </button>
    </div>

    <!-- Main Content -->
    <div :class="['flex-grow overflow-hidden', showTitleBar ? 'mt-8' : '']">
      <Transition mode="out-in" enter-active-class="transition duration-300 ease-out" enter-from-class="opacity-0 scale-95" enter-to-class="opacity-100 scale-100" leave-active-class="transition duration-200 ease-in" leave-from-class="opacity-100 scale-100" leave-to-class="opacity-0 scale-95">
        <component 
          :is="currentView === 'widget' ? Widget : ProcessManager"
          :stats="stats"
          :processes="stats.top_processes"
          :totalCpu="stats.cpu_usage"
          :memoryUsed="stats.memory_used"
          :memoryTotal="stats.memory_total"
          @kill-process="killProcess"
        />
      </Transition>
    </div>

    <!-- Notifications -->
    <ToastNotification 
      :visible="showNotification"
      :type="notificationData.type"
      :title="notificationData.title"
      :message="notificationData.message"
      :details="notificationData.details"
      @close="showNotification = false"
      @action="console.log('Notification action clicked')"
    />
  </div>
</template>

<style>
/* Global transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
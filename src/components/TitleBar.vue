<script setup lang="ts">
import { ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ListTree, LayoutGrid, Settings, Network } from 'lucide-vue-next';

// Props
const props = defineProps<{
  currentView: 'widget' | 'process' | 'settings' | 'ports';
}>();

// Emits
const emit = defineEmits<{
  viewChange: [view: 'widget' | 'process' | 'settings' | 'ports'];
}>();

const appWindow = getCurrentWindow();
const isMaximized = ref(false);

// Window control functions
const minimizeWindow = async () => {
  await appWindow.minimize();
};

const toggleMaximize = async () => {
  if (isMaximized.value) {
    await appWindow.unmaximize();
  } else {
    await appWindow.maximize();
  }
  isMaximized.value = !isMaximized.value;
};

const closeWindow = async () => {
  await appWindow.close();
};

// Listen for window maximize/unmaximize events
appWindow.onResized(async () => {
  isMaximized.value = await appWindow.isMaximized();
});

</script>

<template>
  <div class="title-bar glass" data-tauri-drag-region>
    <div class="title-bar-content" data-tauri-drag-region>
      <div class="title-bar-title" data-tauri-drag-region>
        <svg 
          class="app-icon" 
          width="16" 
          height="16" 
          viewBox="0 0 24 24" 
          fill="none" 
          xmlns="http://www.w3.org/2000/svg"
        >
          <path 
            d="M13 2L3 14h8l-1 8 10-12h-8l1-8z" 
            fill="currentColor" 
            class="text-cyan-400"
          />
        </svg>
        <span>ActioWatch</span>
      </div>
      
      <!-- Navigation Buttons -->
      <div class="title-bar-navigation" data-tauri-drag-region>
        <button 
          class="nav-btn" 
          :class="{ active: props.currentView === 'process' }"
          @click="emit('viewChange', 'process')"
          title="Process Manager"
        >
          <ListTree :size="16" />
        </button>
        <button 
          class="nav-btn" 
          :class="{ active: props.currentView === 'ports' }"
          @click="emit('viewChange', 'ports')"
          title="Open Ports"
        >
          <Network :size="16" />
        </button>
        <button 
          class="nav-btn" 
          :class="{ active: props.currentView === 'widget' }"
          @click="emit('viewChange', 'widget')"
          title="Widget"
        >
          <LayoutGrid :size="16" />
        </button>
        <button 
          class="nav-btn" 
          :class="{ active: props.currentView === 'settings' }"
          @click="emit('viewChange', 'settings')"
          title="Settings"
        >
          <Settings :size="16" />
        </button>
      </div>
      
      <div class="title-bar-controls">
        <button 
          class="title-bar-button minimize-button" 
          @click="minimizeWindow"
          aria-label="Minimize"
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="2" y="5.5" width="8" height="1" fill="currentColor"/>
          </svg>
        </button>
        
        <button 
          class="title-bar-button maximize-button" 
          @click="toggleMaximize"
          aria-label="Maximize/Restore"
        >
          <svg v-if="!isMaximized" width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="2" y="2" width="8" height="8" stroke="currentColor" stroke-width="1" fill="none"/>
          </svg>
          <svg v-else width="12" height="12" viewBox="0 0 12 12" fill="none">
            <rect x="2.5" y="3.5" width="6" height="6" stroke="currentColor" stroke-width="1" fill="none"/>
            <rect x="3.5" y="2.5" width="6" height="6" stroke="currentColor" stroke-width="1" fill="none"/>
          </svg>
        </button>
        
        <button 
          class="title-bar-button close-button" 
          @click="closeWindow"
          aria-label="Close"
        >
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M2 2L10 10M10 2L2 10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.title-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  z-index: 9999;
  user-select: none;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(17, 24, 39, 0.8);
  backdrop-filter: blur(12px);
}

.title-bar-content {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px;
}

.title-bar-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  padding-left: 4px;
  min-width: 150px;
}

.app-icon {
  filter: drop-shadow(0 0 4px rgba(0, 209, 255, 0.5));
}

/* Navigation */
.title-bar-navigation {
  display: flex;
  align-items: center;
  gap: 4px;
  flex: 1;
  justify-content: center;
  padding: 0 16px;
}

.nav-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition: all 0.15s ease;
}

.nav-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: rgba(255, 255, 255, 0.9);
  border-color: rgba(255, 255, 255, 0.1);
}

.nav-btn.active {
  background: rgba(6, 182, 212, 0.2);
  color: #06b6d4;
  border-color: rgba(6, 182, 212, 0.3);
  box-shadow: 0 0 8px rgba(6, 182, 212, 0.2);
}

.title-bar-controls {
  display: flex;
  align-items: center;
  gap: 1px;
  height: 100%;
}

.title-bar-button {
  width: 46px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  transition: all 0.15s ease;
  outline: none;
}

.title-bar-button:hover {
  color: rgba(255, 255, 255, 1);
  background: rgba(255, 255, 255, 0.1);
}

.title-bar-button:active {
  background: rgba(255, 255, 255, 0.15);
}

.close-button:hover {
  background: #e81123;
  color: white;
}

.close-button:active {
  background: #c20d1e;
}

.minimize-button:hover svg,
.maximize-button:hover svg {
  filter: drop-shadow(0 0 2px rgba(255, 255, 255, 0.3));
}

.close-button:hover svg {
  filter: drop-shadow(0 0 3px rgba(232, 17, 35, 0.5));
}

/* Animation for maximize button */
.maximize-button svg {
  transition: transform 0.2s ease;
}

.maximize-button:hover svg {
  transform: scale(1.1);
}
</style>

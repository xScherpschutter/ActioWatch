<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { Settings } from 'lucide-vue-next';

const autostartEnabled = ref(false);
const loading = ref(true);

// Check current autostart status
onMounted(async () => {
  try {
    autostartEnabled.value = await isEnabled();
  } catch (error) {
    console.error('Error checking autostart status:', error);
  } finally {
    loading.value = false;
  }
});

// Toggle autostart
const toggleAutostart = async () => {
  loading.value = true;
  try {
    if (autostartEnabled.value) {
      await disable();
      autostartEnabled.value = false;
    } else {
      await enable();
      autostartEnabled.value = true;
    }
  } catch (error) {
    console.error('Error toggling autostart:', error);
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="settings-container">
    <div class="settings-header">
      <Settings :size="24" class="header-icon" />
      <h2>Settings</h2>
    </div>

    <div class="settings-content">
      <div class="setting-item">
        <div class="setting-info">
          <h3>Start on System Boot</h3>
          <p>Launch ActioWatch automatically when your computer starts</p>
        </div>
        <div class="setting-control">
          <button 
            class="toggle-button"
            :class="{ active: autostartEnabled }"
            :disabled="loading"
            @click="toggleAutostart"
          >
            <div class="toggle-slider" :class="{ active: autostartEnabled }"></div>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 2rem;
  max-width: 800px;
  margin: 0 auto;
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.header-icon {
  color: #06b6d4;
}

.settings-header h2 {
  font-size: 1.5rem;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin: 0;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  transition: all 0.2s ease;
}

.setting-item:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(6, 182, 212, 0.3);
}

.setting-info h3 {
  font-size: 1rem;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
  margin: 0 0 0.5rem 0;
}

.setting-info p {
  font-size: 0.875rem;
  color: rgba(255, 255, 255, 0.6);
  margin: 0;
}

.toggle-button {
  position: relative;
  width: 52px;
  height: 28px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 14px;
  cursor: pointer;
  transition: all 0.3s ease;
  padding: 0;
}

.toggle-button:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
}

.toggle-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toggle-button.active {
  background: linear-gradient(135deg, #06b6d4 0%, #0891b2 100%);
  border-color: #06b6d4;
}

.toggle-slider {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 22px;
  height: 22px;
  background: white;
  border-radius: 50%;
  transition: transform 0.3s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.toggle-slider.active {
  transform: translateX(24px);
}
</style>

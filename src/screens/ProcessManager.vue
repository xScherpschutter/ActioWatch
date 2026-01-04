<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { Search, X, Box, ListTree, List, Info } from 'lucide-vue-next';
import { isWindows } from "../utils/platform";
import ProcessDetailsModal from '../components/ProcessDetailsModal.vue';

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_usage: number;
  thread_count: number;
  children: ProcessInfo[];
}

interface TreeNode extends ProcessInfo {
  level: number;
  isOpen: boolean; 
}

const props = defineProps<{
  processes: ProcessInfo[];
  totalCpu: number;
  memoryUsed: number;
  memoryTotal: number;
}>();

const emit = defineEmits(['kill-process']);

const searchQuery = ref('');
const viewMode = ref<'list' | 'tree'>('list');
const isWindowsPlatform = ref(false);

// Modal State
const showDetailsModal = ref(false);
const selectedProcessPid = ref<number | null>(null);

const openDetails = (pid: number) => {
    selectedProcessPid.value = pid;
    showDetailsModal.value = true;
};

onMounted(async () => {
  isWindowsPlatform.value = await isWindows();
});

const totalThreads = computed(() => {
  if (isWindowsPlatform.value) return 0;
  
  // Need to traverse tree to count total threads if props.processes is just roots
  let count = 0;
  const traverse = (nodes: ProcessInfo[]) => {
    for (const node of nodes) {
      count += node.thread_count;
      traverse(node.children);
    }
  };
  traverse(props.processes);
  return count;
});

// Helper to calculate total count for footer
const totalProcesses = computed(() => {
  let count = 0;
  const traverse = (nodes: ProcessInfo[]) => {
    for (const node of nodes) {
      count++;
      traverse(node.children);
    }
  };
  traverse(props.processes);
  return count;
});


const processedData = computed(() => {
  // Use recursion to flatten for List View or when Searching
  const flatten = (nodes: ProcessInfo[]): TreeNode[] => {
    let result: TreeNode[] = [];
    nodes.forEach(node => {
      // Add self
      result.push({ ...node, level: 0, isOpen: true });
      // Add children (recurse)
      result = result.concat(flatten(node.children));
    });
    return result;
  };

  const traverseTree = (nodes: ProcessInfo[], level: number): TreeNode[] => {
      let result: TreeNode[] = [];
      nodes.forEach(node => {
          result.push({ ...node, level: level, isOpen: true });
          result = result.concat(traverseTree(node.children, level + 1));
      });
      return result;
  }

  // Filter first if searching
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    // Use flat list for search
    const allProcesses = flatten(props.processes);
    return allProcesses.filter(p => 
      p.name.toLowerCase().includes(query) || 
      p.pid.toString().includes(query)
    );
  }

  if (viewMode.value === 'tree') {
    // Backend already sends a tree (roots with children).
    // We need to linearize it for the v-for rendering, maintaining order and adding 'level'
     return traverseTree(props.processes, 0);
  } else {
    // List mode: Flatten and Sort by CPU
    const flat = flatten(props.processes);
    return flat.sort((a, b) => b.cpu_usage - a.cpu_usage);
  }
});

const getUsageColor = (usage: number) => {
  if (usage > 70) return 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]';
  if (usage > 40) return 'bg-orange-500 shadow-[0_0_8px_rgba(249,115,22,0.6)]';
  return 'bg-neon-cpu shadow-[0_0_8px_rgba(0,209,255,0.6)]';
};

const formatBytes = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024);
  return `${gb.toFixed(1)} GB`;
};
</script>

<template>
  <div class="h-full flex flex-col glass-container text-white select-none">

    <!-- Toolbar -->
    <div class="p-4 flex gap-3 items-center">
      <div class="relative flex-grow group">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 group-focus-within:text-neon-cpu transition-colors" />
        <input 
          v-model="searchQuery"
          type="text" 
          placeholder="Filter processes by name or PID..." 
          class="w-full glass-input border border-white/10 rounded-lg pl-10 pr-4 py-2 text-sm focus:outline-none focus:border-neon-cpu/50 focus:ring-1 focus:ring-neon-cpu/50 transition-all placeholder-gray-600"
        />
      </div>

      <!-- View Toggle -->
      <div class="flex bg-white/5 rounded-lg p-1 border border-white/10">
        <button 
          @click="viewMode = 'list'"
          :class="['p-2 rounded-md transition-all', viewMode === 'list' ? 'bg-white/10 text-neon-cpu shadow-sm' : 'text-gray-400 hover:text-white']"
          title="List View"
        >
          <List class="w-4 h-4" />
        </button>
        <button 
          @click="viewMode = 'tree'"
          :class="['p-2 rounded-md transition-all', viewMode === 'tree' ? 'bg-white/10 text-neon-cpu shadow-sm' : 'text-gray-400 hover:text-white']"
          title="Tree View"
        >
          <ListTree class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Table Header -->
    <div class="grid grid-cols-12 gap-4 px-6 py-2 text-xs font-bold text-white/80 uppercase tracking-wider border-b border-white/10">
      <!-- Adjust column spans based on platform -->
      <div :class="isWindowsPlatform ? 'col-span-5' : 'col-span-4'">Process Name</div>
      <div class="col-span-1">PID</div>
      <div v-if="!isWindowsPlatform" class="col-span-1">Threads</div>
      <div class="col-span-2">CPU %</div>
      <div class="col-span-2">Memory</div>
      <div class="col-span-2 text-right">Action</div>
    </div>

    <!-- Process List -->
    <div class="flex-grow overflow-y-auto custom-scrollbar px-2">
      <div v-for="process in processedData" :key="process.pid" 
           class="grid grid-cols-12 gap-4 px-4 py-2 items-center hover:bg-white/5 rounded-lg transition-colors group border-b border-white/5 last:border-0">
        
        <!-- Name (with Indentation for Tree) -->
        <div :class="isWindowsPlatform ? 'col-span-5' : 'col-span-4'" class="flex items-center gap-3 overflow-hidden">
           <div :style="{ marginLeft: `${process.level * 20}px` }" class="flex-shrink-0 transition-all duration-300">
              <div v-if="process.level > 0" class="w-3 h-4 border-l border-b border-white/20 inline-block -translate-y-1 mr-2"></div>
           </div>

          <div class="w-8 h-8 rounded shrink-0 bg-white/10 flex items-center justify-center text-cyan-400" :class="{'ml-2': process.level > 0}">
            <Box class="w-4 h-4" />
          </div>
          <div class="flex flex-col truncate min-w-0">
             <span class="font-medium text-sm text-white/90 group-hover:text-white truncate" :title="process.name">{{ process.name }}</span>
             <span v-if="viewMode === 'tree' && process.children?.length" class="text-[10px] text-white/40">{{ process.children.length }} sub-procs</span>
          </div>
        </div>

        <!-- PID -->
        <div class="col-span-1 font-mono text-xs text-white/60">
          {{ process.pid }}
        </div>

        <!-- Threads (Only if not Windows) -->
        <div v-if="!isWindowsPlatform" class="col-span-1 font-mono text-xs text-white/60">
          {{ process.thread_count }}
        </div>

        <!-- CPU -->
        <div class="col-span-2">
          <div class="flex flex-col gap-1">
            <div class="flex justify-between text-[10px] text-white/70 font-mono">
              <span>{{ process.cpu_usage.toFixed(1) }}%</span>
            </div>
            <div class="h-1.5 bg-gray-700/50 rounded-full overflow-hidden">
              <div 
                class="h-full rounded-full transition-all duration-500"
                :class="getUsageColor(process.cpu_usage)"
                :style="{ width: `${Math.min(process.cpu_usage, 100)}%` }"
              ></div>
            </div>
          </div>
        </div>

        <!-- Memory -->
        <div class="col-span-2">
          <div class="flex flex-col gap-1">
            <div class="flex justify-between text-[10px] text-white/70 font-mono">
              <span>{{ formatBytes(process.memory_usage) }}</span>
            </div>
            <div class="h-1.5 bg-gray-700/50 rounded-full overflow-hidden">
              <div 
                class="h-full bg-neon-ram rounded-full transition-all duration-500"
                :style="{ width: `${Math.min((process.memory_usage / memoryTotal) * 100, 100)}%` }"
              ></div>
            </div>
          </div>
        </div>

        <!-- Action -->
        <div class="col-span-2 flex justify-end gap-2">
           <button @click="openDetails(process.pid)" 
                  class="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-white/10 hover:border-white/20 border border-transparent transition-all"
                  title="View Details">
            <Info class="w-4 h-4" />
          </button>
          <button @click="emit('kill-process', process.pid)" 
                  class="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-red-500/20 hover:border-red-500/50 border border-transparent transition-all"
                  title="Kill Process">
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>
      
      <!-- Empty State -->
      <div v-if="processedData.length === 0" class="flex flex-col items-center justify-center py-20 text-white/40">
        <Search class="w-12 h-12 mb-4 opacity-20" />
        <p>No processes found matching "{{ searchQuery }}"</p>
      </div>

    </div>

    <!-- Footer Stats -->
    <div class="h-16 glass-footer border-t border-white/5 px-6 flex items-center justify-between">
      <div class="flex gap-8">
        <div class="flex flex-col">
          <span class="text-[10px] text-white/60 uppercase font-bold">Total Processes</span>
          <span class="text-lg font-mono text-white">{{ totalProcesses }}</span>
        </div>
        <div v-if="!isWindowsPlatform" class="flex flex-col">
          <span class="text-[10px] text-white/60 font-bold uppercase tracking-wider">Threads</span>
          <span class="text-lg font-mono font-bold text-white">{{ totalThreads }}</span>
        </div>
      </div>

      <div class="flex gap-8 items-center">
        <!-- Total CPU -->
        <div class="w-48">
          <div class="flex justify-between text-xs mb-1">
             <span class="text-white/70">Total CPU Load</span>
             <span class="text-neon-cpu font-bold">{{ totalCpu.toFixed(1) }}%</span>
          </div>
          <div class="h-1.5 bg-gray-700 rounded-full overflow-hidden">
            <div class="h-full bg-neon-cpu shadow-[0_0_10px_rgba(0,209,255,0.5)] transition-all duration-300"
                 :style="{ width: `${Math.min(totalCpu, 100)}%` }"></div>
          </div>
        </div>

        <!-- Memory -->
        <div class="w-48">
          <div class="flex justify-between text-xs mb-1">
             <span class="text-white/70">Memory</span>
             <span class="text-neon-ram font-bold">{{ formatBytes(memoryUsed) }} / {{ formatBytes(memoryTotal) }}</span>
          </div>
          <div class="h-1.5 bg-gray-700 rounded-full overflow-hidden">
            <div class="h-full bg-neon-ram shadow-[0_0_10px_rgba(66,255,0,0.5)] transition-all duration-300"
                 :style="{ width: `${(memoryUsed / memoryTotal) * 100}%` }"></div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- Details Modal -->
    <ProcessDetailsModal 
        :is-open="showDetailsModal" 
        :pid="selectedProcessPid" 
        @close="showDetailsModal = false" 
    />

  </div>
</template>

<style scoped>
.glass-container {
  background: rgba(17, 24, 39, 0.7);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.glass-input {
  background: rgba(31, 41, 55, 0.5);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.glass-footer {
  background: rgba(31, 41, 55, 0.6);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
}

.custom-scrollbar::-webkit-scrollbar {
  width: 8px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(6, 182, 212, 0.3);
  border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(6, 182, 212, 0.5);
}
</style>

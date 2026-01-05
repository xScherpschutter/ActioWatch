<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { Search, X, Box, ListTree, List, Info, ChevronRight, ChevronDown, ChevronsDown, ChevronsRight } from 'lucide-vue-next';
import { isWindows } from "../utils/platform";
import ProcessDetailsModal from '../components/ProcessDetailsModal.vue';
import ConfirmationModal from '../components/ConfirmationModal.vue';

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  total_cpu_usage: number;
  memory_usage: number;
  total_memory_usage: number;
  disk_read: number;
  disk_write: number;
  total_disk_read: number;
  total_disk_write: number;
  thread_count: number;
  children: ProcessInfo[];
}

interface TreeNode extends ProcessInfo {
  level: number;
  isOpen: boolean; 
  hasChildren: boolean;
}

const props = withDefaults(defineProps<{
  processes: ProcessInfo[];
  totalCpu: number;
  memoryUsed: number;
  memoryTotal: number;
  networkUp?: number;
  networkDown?: number;
}>(), {
  networkUp: 0,
  networkDown: 0,
});

const emit = defineEmits(['kill-process']);

const searchQuery = ref('');
const viewMode = ref<'list' | 'tree'>('list');
const isWindowsPlatform = ref(false);

const collapsedPids = ref(new Set<number>());

const toggleCollapse = (pid: number) => {
  if (collapsedPids.value.has(pid)) {
    collapsedPids.value.delete(pid);
  } else {
    collapsedPids.value.add(pid);
  }
};

const expandAll = () => {
    collapsedPids.value.clear();
};

const collapseAll = () => {
    const allPidsWithChildren = new Set<number>();
    const findParents = (nodes: ProcessInfo[]) => {
        for (const node of nodes) {
            if (node.children && node.children.length > 0) {
                allPidsWithChildren.add(node.pid);
                findParents(node.children);
            }
        }
    };
    findParents(props.processes);
    collapsedPids.value = allPidsWithChildren;
};

// Modal State
const showDetailsModal = ref(false);
const selectedProcessPid = ref<number | null>(null);

const openDetails = (pid: number) => {
    selectedProcessPid.value = pid;
    showDetailsModal.value = true;
};

// Confirmation State
const showConfirmation = ref(false);
const processToKill = ref<ProcessInfo | null>(null);

const confirmKill = (process: ProcessInfo) => {
    processToKill.value = process;
    showConfirmation.value = true;
};

const executeKill = () => {
    if (processToKill.value) {
        emit('kill-process', processToKill.value.pid);
        showConfirmation.value = false;
        processToKill.value = null;
    }
};

onMounted(async () => {
  isWindowsPlatform.value = await isWindows();
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
      result.push({ ...node, level: 0, isOpen: true, hasChildren: node.children && node.children.length > 0 });
      // Add children (recurse)
      result = result.concat(flatten(node.children));
    });
    return result;
  };

  const traverseTree = (nodes: ProcessInfo[], level: number): TreeNode[] => {
      let result: TreeNode[] = [];
      nodes.forEach(node => {
          const isCollapsed = collapsedPids.value.has(node.pid);
          // Standard Tree View: Expanded by default unless in collapsedPids set
          const isOpen = !isCollapsed;
          const hasChildren = node.children && node.children.length > 0;
          
          result.push({ ...node, level: level, isOpen, hasChildren });
          
          if (isOpen && hasChildren) {
              result = result.concat(traverseTree(node.children, level + 1));
          }
      });
      return result;
  }

  // Filter first if searching
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    
    if (viewMode.value === 'tree') {
        const matches: ProcessInfo[] = [];
        const findMatches = (nodes: ProcessInfo[]) => {
            nodes.forEach(node => {
                if (node.name.toLowerCase().includes(query) || node.pid.toString().includes(query)) {
                    matches.push(node);
                }
                if (node.children) {
                    findMatches(node.children);
                }
            });
        };
        findMatches(props.processes);
        // Return matches as roots, enabling expansion of their children via traverseTree
        return traverseTree(matches, 0);
    } else {
        // List mode: Flat filter
        const allProcesses = flatten(props.processes);
        return allProcesses.filter(p => 
          p.name.toLowerCase().includes(query) || 
          p.pid.toString().includes(query)
        ).sort((a, b) => (b.total_cpu_usage || 0) - (a.total_cpu_usage || 0));
    }
  }

  if (viewMode.value === 'tree') {
    // Backend already sends a tree (roots with children).
    // We need to linearize it for the v-for rendering, maintaining order and adding 'level'
     return traverseTree(props.processes, 0);
  } else {
    // List mode: Flatten and Sort by CPU
    const flat = flatten(props.processes);
    // Sort logic safeguarded
    return flat.sort((a, b) => (b.total_cpu_usage || 0) - (a.total_cpu_usage || 0));
  }
});

const getUsageColor = (usage: number) => {
  if (usage > 70) return 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]';
  if (usage > 40) return 'bg-orange-500 shadow-[0_0_8px_rgba(249,115,22,0.6)]';
  return 'bg-neon-cpu shadow-[0_0_8px_rgba(0,209,255,0.6)]';
};

const formatBytes = (bytes: number) => {
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(1)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(1)} MB`;
};

const formatDiskBytes = (bytes: number) => {
    if (bytes === 0) return '0 B/s';
    const kbs = bytes / 1024;
    if (kbs < 1024) return `${kbs.toFixed(1)} KB/s`;
    const mbs = kbs / 1024;
    return `${mbs.toFixed(1)} MB/s`;
}

const formatNetworkBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const kbs = bytes / 1024;
    if (kbs < 1024) return `${kbs.toFixed(1)} KB`;
    const mbs = kbs / 1024;
    return `${mbs.toFixed(1)} MB`;
}

</script>

<template>
  <div class="h-full flex flex-col glass-container text-white select-none pt-5">
    <!-- Header -->
      <div class="flex justify-between items-center border-b border-white/5 px-6">
        <h2 class="text-xl font-bold text-white tracking-wider flex items-center gap-2">
          PROCESS MANAGER
        </h2>
      </div>

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

       <!-- Expand/Collapse Controls (Only in Tree Mode) -->
      <div v-if="viewMode === 'tree'" class="flex bg-white/5 rounded-lg p-1 border border-white/10">
          <button 
           @click="expandAll"
           class="p-2 rounded-md transition-all text-gray-400 hover:text-white hover:bg-white/10"
           title="Expand All"
          >
            <ChevronsDown class="w-4 h-4" />
          </button>
           <button 
           @click="collapseAll"
           class="p-2 rounded-md transition-all text-gray-400 hover:text-white hover:bg-white/10"
           title="Collapse All"
          >
            <ChevronsRight class="w-4 h-4" />
          </button>
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
      <!-- Name (3 cols) -->
      <div class="col-span-3">Process Name</div>
      <!-- PID (1 col) -->
      <div class="col-span-1">PID</div>
      <!-- CPU (2 cols) -> Reduced to 1.5? No, keep 2 -->
      <div class="col-span-2">CPU %</div>
      <!-- Memory (2 cols) -->
      <div class="col-span-2">Memory</div>
      <!-- Disk (2 cols) split -->
      <div class="col-span-2 flex justify-between">
          <span>Disk R</span>
          <span>Disk W</span>
      </div>
      <!-- Action (2 cols) -->
      <div class="col-span-2 text-right">Action</div>
    </div>

    <!-- Process List -->
    <div class="flex-grow overflow-y-auto custom-scrollbar px-2">
      <div v-for="process in processedData" :key="process.pid" 
           class="grid grid-cols-12 gap-4 px-4 py-2 items-center hover:bg-white/5 rounded-lg transition-colors group border-b border-white/5 last:border-0">
        
        <!-- Name (with Indentation for Tree) -->
        <div class="col-span-3 flex items-center gap-2 overflow-hidden">
           <!-- Indentation Spacer -->
           <div :style="{ width: `${process.level * 20}px` }" class="flex-shrink-0 transition-all duration-300"></div>

           <!-- Expand/Collapse Button (Tree View Only) -->
           <button 
             v-if="viewMode === 'tree' && process.hasChildren"
             @click.stop="toggleCollapse(process.pid)"
             class="p-0.5 rounded hover:bg-white/10 text-gray-400 hover:text-white transition-colors flex-shrink-0"
           >
             <component :is="process.isOpen ? ChevronDown : ChevronRight" class="w-3 h-3" />
           </button>
           <!-- Spacer if no children but indented -->
           <div v-else-if="viewMode === 'tree'" class="w-4 flex-shrink-0"></div>

          <div class="w-8 h-8 rounded shrink-0 bg-white/10 flex items-center justify-center text-cyan-400">
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

        <!-- CPU -->
        <div class="col-span-2">
          <div class="flex flex-col gap-1">
            <div class="flex justify-between text-[10px] text-white/70 font-mono">
              <!-- Safety check for total_cpu_usage -->
              <span>{{ (process.total_cpu_usage || 0).toFixed(1) }}%</span>
            </div>
            <div class="h-1.5 bg-gray-700/50 rounded-full overflow-hidden">
              <div 
                class="h-full rounded-full transition-all duration-500"
                :class="getUsageColor(process.total_cpu_usage || 0)"
                :style="{ width: `${Math.min(process.total_cpu_usage || 0, 100)}%` }"
              ></div>
            </div>
          </div>
        </div>

        <!-- Memory -->
        <div class="col-span-2">
          <div class="flex flex-col gap-1">
            <div class="flex justify-between text-[10px] text-white/70 font-mono">
              <!-- Safety check for total_memory_usage -->
              <span>{{ formatBytes(process.total_memory_usage || 0) }}</span>
            </div>
            <div class="h-1.5 bg-gray-700/50 rounded-full overflow-hidden">
              <div 
                class="h-full bg-neon-ram rounded-full transition-all duration-500"
                :style="{ width: `${Math.min((process.total_memory_usage / memoryTotal) * 100, 100)}%` }"
              ></div>
            </div>
          </div>
        </div>

        <!-- Disk I/O -->
        <div class="col-span-2 flex justify-between font-mono text-[10px] text-white/60">
            <span class="text-orange-300">{{ formatDiskBytes(process.total_disk_read || 0) }}</span>
            <span class="text-blue-300">{{ formatDiskBytes(process.total_disk_write || 0) }}</span>
        </div>

        <!-- Action -->
        <div class="col-span-2 flex justify-end gap-2">
           <button @click="openDetails(process.pid)" 
                  class="p-1.5 rounded-md text-gray-400 hover:text-white hover:bg-white/10 hover:border-white/20 border border-transparent transition-all"
                  title="View Details">
            <Info class="w-4 h-4" />
          </button>
          <button @click="confirmKill(process)" 
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
      </div>

      <div class="flex gap-8 items-center">
         <!-- Network -->
         <div class="flex gap-4 border-r border-white/10 pr-4">
             <div class="flex flex-col items-end">
                 <span class="text-[9px] text-gray-400 uppercase tracking-widest">Down</span>
                 <span class="text-xs font-mono text-neon-cpu">{{ formatNetworkBytes(networkDown) }}/s</span>
             </div>
             <div class="flex flex-col items-end">
                 <span class="text-[9px] text-gray-400 uppercase tracking-widest">Up</span>
                 <span class="text-xs font-mono text-blue-400">{{ formatNetworkBytes(networkUp) }}/s</span>
             </div>
         </div>

        <!-- Total CPU -->
        <div class="w-32">
          <div class="flex justify-between text-xs mb-1">
             <span class="text-white/70">CPU</span>
             <span class="text-neon-cpu font-bold">{{ totalCpu.toFixed(1) }}%</span>
          </div>
          <div class="h-1.5 bg-gray-700 rounded-full overflow-hidden">
            <div class="h-full bg-neon-cpu shadow-[0_0_10px_rgba(0,209,255,0.5)] transition-all duration-300"
                 :style="{ width: `${Math.min(totalCpu, 100)}%` }"></div>
          </div>
        </div>

        <!-- Memory -->
        <div class="w-32">
          <div class="flex justify-between text-xs mb-1">
             <span class="text-white/70">Mem</span>
             <span class="text-neon-ram font-bold">{{ formatBytes(memoryUsed) }}</span>
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

    <ConfirmationModal
        :is-open="showConfirmation"
        title="Kill Process?"
        :message="`Are you sure you want to terminate ${processToKill?.name} (PID: ${processToKill?.pid})? Unsaved data may be lost.`"
        confirm-text="Kill Process"
        @confirm="executeKill"
        @cancel="showConfirmation = false"
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

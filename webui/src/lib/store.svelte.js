import { uiStore } from './stores/ui.svelte.js';
import { configStore } from './stores/config.svelte.js';
import { modulesStore } from './stores/modules.svelte.js';
import { logsStore } from './stores/logs.svelte.js';
import { deviceStore } from './stores/device.svelte.js';

// Aggregator store to maintain backward compatibility with existing components
export const store = {
  // UI & Localization
  get theme() { return uiStore.theme; },
  get isSystemDark() { return uiStore.isSystemDark; },
  get lang() { return uiStore.lang; },
  get seed() { return uiStore.seed; },
  get availableLanguages() { return uiStore.availableLanguages; },
  get L() { return uiStore.L; },
  get toast() { return uiStore.toast; },
  // Compatibility array for toasts if needed, though we use single object toast now
  get toasts() { return uiStore.toast.visible ? [uiStore.toast] : []; },
  
  showToast: uiStore.showToast,
  setTheme: uiStore.setTheme,
  setLang: uiStore.setLang,
  
  // Initialization
  init: async () => {
    await uiStore.init();
    await configStore.load(); // Load config after UI init
  },

  // Config
  get config() { return configStore.config; },
  set config(v) { configStore.config = v; },
  loadConfig: configStore.load,
  saveConfig: configStore.save,

  // Modules
  get modules() { return modulesStore.modules; },
  set modules(v) { modulesStore.modules = v; },
  get modeStats() { return modulesStore.modeStats; },
  loadModules: modulesStore.load,
  saveModules: modulesStore.save,

  // Logs
  get logs() { return logsStore.logs; },
  loadLogs: logsStore.load,

  // Device / Status
  get device() { return deviceStore.device; },
  get version() { return deviceStore.version; },
  get storage() { return deviceStore.storage; },
  get systemInfo() { return deviceStore.systemInfo; },
  get activePartitions() { return deviceStore.activePartitions; },
  loadStatus: deviceStore.load,

  // Combined Loading State
  get loading() {
    return {
      config: configStore.loading,
      modules: modulesStore.loading,
      logs: logsStore.loading,
      status: deviceStore.loading
    };
  },

  // Combined Saving State
  get saving() {
    return {
      config: configStore.saving,
      modules: modulesStore.saving
    };
  }
};
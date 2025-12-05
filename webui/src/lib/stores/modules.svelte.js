import { API } from '../api';
import { uiStore } from './ui.svelte.js';
import { configStore } from './config.svelte.js';

export const createModulesStore = () => {
  let modules = $state([]);
  let loading = $state(false);
  let saving = $state(false);

  const modeStats = $derived.by(() => {
    let auto = 0;
    let magic = 0;
    modules.forEach(m => {
      if (m.mode === 'magic') magic++;
      else auto++;
    });
    return { auto, magic };
  });

  async function load() {
    loading = true;
    try {
      modules = await API.scanModules(configStore.config.moduledir);
    } catch (e) {
      uiStore.showToast(uiStore.L.modules.scanError, 'error');
    }
    loading = false;
  }

  async function save() {
    saving = true;
    try {
      await API.saveModules(modules);
      uiStore.showToast(uiStore.L.modules.saveSuccess);
    } catch (e) {
      uiStore.showToast(uiStore.L.modules.saveFailed, 'error');
    }
    saving = false;
  }

  return {
    get modules() { return modules; },
    set modules(v) { modules = v; },
    get loading() { return loading; },
    get saving() { return saving; },
    get modeStats() { return modeStats; },
    load,
    save
  };
};

export const modulesStore = createModulesStore();
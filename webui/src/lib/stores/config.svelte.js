import { API } from '../api';
import { DEFAULT_CONFIG } from '../constants';
import { uiStore } from './ui.svelte.js';

export const createConfigStore = () => {
  let config = $state({ ...DEFAULT_CONFIG });
  let loading = $state(false);
  let saving = $state(false);

  async function load() {
    loading = true;
    try {
      config = await API.loadConfig();
      if (uiStore.L?.config) uiStore.showToast(uiStore.L.config.loadSuccess);
    } catch (e) {
      if (uiStore.L?.config) uiStore.showToast(uiStore.L.config.loadError, 'error');
    }
    loading = false;
  }

  async function save() {
    saving = true;
    try {
      await API.saveConfig(config);
      uiStore.showToast(uiStore.L.config.saveSuccess);
    } catch (e) {
      uiStore.showToast(uiStore.L.config.saveFailed, 'error');
    }
    saving = false;
  }

  return {
    get config() { return config; },
    set config(v) { config = v; },
    get loading() { return loading; },
    get saving() { return saving; },
    load,
    save
  };
};

export const configStore = createConfigStore();
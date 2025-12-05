import { API } from '../api';
import { modulesStore } from './modules.svelte.js';

export const createDeviceStore = () => {
  let device = $state({ model: 'Loading...', android: '-', kernel: '-', selinux: '-' });
  let version = $state('...');
  let storage = $state({ used: '-', size: '-', percent: '0%' });
  let systemInfo = $state({ kernel: '-', selinux: '-', mountBase: '-' });
  let activePartitions = $state([]);
  let loading = $state(false);

  async function load() {
    loading = true;
    try {
      device = await API.getDeviceStatus();
      version = await API.getVersion();
      
      if (modulesStore.modules.length === 0) {
        await modulesStore.load();
      }
    } catch (e) {}
    loading = false;
  }

  return {
    get device() { return device; },
    get version() { return version; },
    get storage() { return storage; },
    get systemInfo() { return systemInfo; },
    get activePartitions() { return activePartitions; },
    get loading() { return loading; },
    load
  };
};

export const deviceStore = createDeviceStore();
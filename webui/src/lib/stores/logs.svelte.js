import { API } from '../api';
import { uiStore } from './ui.svelte.js';

export const createLogsStore = () => {
  let logs = $state([]);
  let loading = $state(false);

  async function load(silent = false) {
    if (!silent) loading = true;
    try {
      const raw = await API.readLogs();
      
      if (!raw) {
        logs = [];
      } else {
        logs = raw.split('\n').map(line => {
          let type = 'debug';
          if (line.includes('ERROR') || line.includes('[E]')) type = 'error';
          else if (line.includes('WARN') || line.includes('[W]')) type = 'warn';
          else if (line.includes('INFO') || line.includes('[I]')) type = 'info';
          
          return { text: line, type };
        });
      }
    } catch (e) {
      logs = [{ text: `Error loading logs: ${e.message}`, type: 'error' }];
      
      if (!silent && uiStore.L?.logs) {
        uiStore.showToast(uiStore.L.logs.readFailed, 'error');
      }
    }
    loading = false;
  }

  return {
    get logs() { return logs; },
    get loading() { return loading; },
    load
  };
};

export const logsStore = createLogsStore();
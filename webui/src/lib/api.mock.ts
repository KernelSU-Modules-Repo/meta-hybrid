import { DEFAULT_CONFIG } from './constants';
import type { AppConfig, Module, StorageStatus, SystemInfo, DeviceInfo } from './types';

const DELAY_MS = 600;

function delay<T>(data: T): Promise<T> {
  return new Promise(resolve => setTimeout(() => resolve(data), DELAY_MS));
}

export const MockAPI = {
  loadConfig: async (): Promise<AppConfig> => {
    return delay(DEFAULT_CONFIG);
  },

  saveConfig: async (config: AppConfig): Promise<void> => {
    console.log('[Mock] Saving config:', config);
    return delay(undefined);
  },

  scanModules: async (): Promise<Module[]> => {
    // Generate dummy modules with the NEW granular structure
    const modules: Module[] = [
      {
        id: 'magisk_module_1',
        name: 'Awesome Mod',
        version: '1.0.0',
        author: 'Developer A',
        description: 'A simple module that does magic stuff',
        config: {
          default_mode: 'auto',
          partitions: {}
        },
        detected_partitions: ['system']
      },
      {
        id: 'fix_vendor_bug',
        name: 'Vendor Fixer',
        version: '2.1',
        author: 'Fixer B',
        description: 'Fixes specific vendor partition issues.',
        config: {
          default_mode: 'magic', // Simulating a saved setting
          partitions: {
            'vendor': 'magic'
          }
        },
        detected_partitions: ['vendor', 'odm']
      },
      {
        id: 'complex_overlay',
        name: 'UI Themer',
        version: 'v12',
        author: 'Themer C',
        description: 'Replaces system UI assets via OverlayFS.',
        config: {
          default_mode: 'overlay',
          partitions: {}
        },
        detected_partitions: ['product', 'system_ext']
      },
      {
        id: 'hymo_test_module',
        name: 'HymoFS Test',
        version: '0.9-beta',
        author: 'Hymo Dev',
        description: 'Testing HymoFS injection on specific paths.',
        config: {
          default_mode: 'hymo',
          partitions: {
            'system': 'overlay' // Mixed mode test
          }
        },
        detected_partitions: ['system', 'vendor']
      }
    ];
    return delay(modules);
  },

  saveModules: async (modules: Module[]): Promise<void> => {
    console.log('[Mock] Saving granular module settings:', JSON.stringify(modules, null, 2));
    return delay(undefined);
  },

  readLogs: async (): Promise<string> => {
    return delay(`[INFO] Daemon started\n[INFO] Storage: Tmpfs\n[WARN] Module 'fix_vendor_bug' fell back to Magic Mount\n[INFO] OverlayFS mounted for 3 modules\n[DEBUG] HymoFS injection active`);
  },

  getStorageUsage: async (): Promise<StorageStatus> => {
    return delay({
      size: '8.0 GB',
      used: '1.2 GB',
      percent: '15%',
      type: 'tmpfs',
      hymofs_available: true
    });
  },

  getSystemInfo: async (): Promise<SystemInfo> => {
    return delay({
      kernel: '5.10.101-android12-9-ge6234 (Mock)',
      selinux: 'Enforcing',
      mountBase: '/dev/loop10',
      activeMounts: ['system', 'vendor', 'product']
    });
  },

  getDeviceStatus: async (): Promise<DeviceInfo> => {
    return delay({
      model: 'Pixel 6 Pro (Mock)',
      android: '13 (API 33)',
      kernel: '5.10.101',
      selinux: 'Enforcing'
    });
  },

  getVersion: async (): Promise<string> => {
    return delay("1.2.0-mock");
  },

  openLink: async (url: string): Promise<void> => {
    console.log('[Mock] Opening link:', url);
    window.open(url, '_blank');
  },

  fetchSystemColor: async (): Promise<string | null> => {
    return delay('#6750A4'); 
  }
};
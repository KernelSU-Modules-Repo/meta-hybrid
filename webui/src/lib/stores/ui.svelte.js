import { API } from '../api';
import { DEFAULT_SEED } from '../constants';
import { Monet } from '../theme';

const localeModules = import.meta.glob('../../locales/*.json', { eager: true });

export const createUIStore = () => {
  let theme = $state('auto');
  let isSystemDark = $state(false);
  let lang = $state('en');
  let seed = $state(DEFAULT_SEED);
  let loadedLocale = $state(null);
  let toast = $state({ id: 'init', text: '', type: 'info', visible: false });

  const availableLanguages = Object.entries(localeModules).map(([path, mod]) => {
    const match = path.match(/\/([^/]+)\.json$/);
    const code = match ? match[1] : 'en';
    const name = mod.default?.lang?.display || code.toUpperCase();
    return { code, name };
  }).sort((a, b) => {
    if (a.code === 'en') return -1;
    if (b.code === 'en') return 1;
    return a.code.localeCompare(b.code);
  });

  function getFallbackLocale() {
    return {
        common: { appName: "Magic Mount", saving: "...", theme: "Theme", language: "Language", themeAuto: "Auto", themeLight: "Light", themeDark: "Dark" },
        lang: { display: "English" },
        tabs: { status: "Status", config: "Config", modules: "Modules", logs: "Logs" },
        status: { deviceTitle: "Device Info", moduleTitle: "Modules", moduleActive: "Active Modules", modelLabel: "Model", androidLabel: "Android Ver", kernelLabel: "Kernel", selinuxLabel: "SELinux", reboot: "Reboot Device", copy: "Copy Info" },
        config: { title: "Config", verboseLabel: "Verbose", verboseOff: "Off", verboseOn: "On", moduleDir: "Module Dir", tempDir: "Temp Dir", mountSource: "Mount Source", logFile: "Log File", partitions: "Partitions", autoPlaceholder: "Auto", reload: "Reload", save: "Save", reset: "Reset", invalidPath: "Invalid path", loadSuccess: "Config Loaded", loadError: "Load Error", loadDefault: "Using Default", saveSuccess: "Saved", saveFailed: "Save Failed", umountLabel: "Umount", umountOff: "Unmount", umountOn: "No Unmount" },
        modules: { title: "Modules", desc: "Modules strictly managed by Magic Mount.", scanning: "Scanning...", reload: "Refresh", save: "Save", empty: "No magic-mounted modules", scanError: "Scan Failed", saveSuccess: "Saved", saveFailed: "Failed", searchPlaceholder: "Search", filterLabel: "Filter", filterAll: "All", toggleError: "Toggle Failed" },
        logs: { title: "Logs", loading: "Loading...", refresh: "Refresh", empty: "Empty", copy: "Copy", copySuccess: "Copied", copyFail: "Failed", searchPlaceholder: "Search", filterLabel: "Level", levels: { all: "All", info: "Info", warn: "Warn", error: "Error" }, current: "Current", old: "Old", readFailed: "Read Failed", readException: "Exception" },
        info: { title: "About", projectLink: "Repository", donate: "Donate", contributors: "Contributors", loading: "Loading...", loadFail: "Failed to load", noBio: "No bio available" }
    };
  }

  const L = $derived(loadedLocale || getFallbackLocale());

  function showToast(msg, type = 'info') {
    // 修复：添加唯一的 ID
    const id = Date.now().toString();
    toast = { id, text: msg, type, visible: true };
    setTimeout(() => { 
        // 仅当 ID 匹配时才隐藏，防止覆盖新 Toast
        if (toast.id === id) toast.visible = false; 
    }, 3000);
  }

  function applyTheme() {
    const isDark = theme === 'auto' ? isSystemDark : theme === 'dark';
    const attr = isDark ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', attr);
    Monet.apply(seed, isDark);
  }

  function setTheme(newTheme) {
    theme = newTheme;
    localStorage.setItem('mm-theme', newTheme);
    applyTheme();
  }

  async function setLang(code) {
    const path = `../../locales/${code}.json`;
    if (localeModules[path]) {
      try {
        const mod = localeModules[path];
        loadedLocale = mod.default; 
        lang = code;
        localStorage.setItem('mm-lang', code);
      } catch (e) {
        console.error(`Failed to load locale: ${code}`, e);
        if (code !== 'en') await setLang('en');
      }
    }
  }

  async function init() {
    const savedLang = localStorage.getItem('mm-lang') || 'en';
    await setLang(savedLang);
    
    theme = localStorage.getItem('mm-theme') || 'auto';
    
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    isSystemDark = mediaQuery.matches;
    
    mediaQuery.addEventListener('change', (e) => {
      isSystemDark = e.matches;
      if (theme === 'auto') {
        applyTheme();
      }
    });
    
    const sysColor = await API.fetchSystemColor();
    if (sysColor) {
      seed = sysColor;
    }
    applyTheme();
  }

  return {
    get theme() { return theme; },
    get isSystemDark() { return isSystemDark; },
    get lang() { return lang; },
    get seed() { return seed; },
    get toast() { return toast; },
    get availableLanguages() { return availableLanguages; },
    get L() { return L; },
    showToast,
    setTheme,
    setLang,
    init,
    get toasts() { return toast.visible ? [toast] : []; } 
  };
};

export const uiStore = createUIStore();
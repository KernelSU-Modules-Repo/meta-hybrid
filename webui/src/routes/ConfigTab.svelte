<script lang="ts">
  import { store } from '../lib/store.svelte';
  import { ICONS } from '../lib/constants';
  import ChipInput from '../components/ChipInput.svelte';
  import BottomActions from '../components/BottomActions.svelte';
  import { slide } from 'svelte/transition';
  import './ConfigTab.css';

  let initialConfigStr = $state('');
  const isValidPath = (p: string) => !p || (p.startsWith('/') && p.length > 1);
  let invalidModuleDir = $derived(store.config ? !isValidPath(store.config.moduledir) : false);
  let invalidTempDir = $derived(store.config ? (store.config.tempdir && !isValidPath(store.config.tempdir)) : false);

  let isDirty = $derived.by(() => {
    if (!initialConfigStr || !store.config) return false;
    return JSON.stringify(store.config) !== initialConfigStr;
  });

  $effect(() => {
    if (!store.loading.config && store.config) {
      if (!initialConfigStr || initialConfigStr === JSON.stringify(store.config)) {
        initialConfigStr = JSON.stringify(store.config);
      }
    }
  });

  function save() {
    if (!store.config) return;
    if (invalidModuleDir || invalidTempDir) {
      store.showToast(store.L.config.invalidPath, "error");
      return;
    }
    store.saveConfig().then(() => {
        if (store.config) initialConfigStr = JSON.stringify(store.config);
    });
  }
  
  function reload() {
    store.loadConfig().then(() => {
        if (store.config) initialConfigStr = JSON.stringify(store.config);
    });
  }

  function resetTempDir() {
    if (store.config) store.config.tempdir = "";
  }

  function toggle(key: keyof typeof store.config) {
    if (store.config && typeof store.config[key] === 'boolean') {
      (store.config as any)[key] = !store.config[key];
    }
  }
</script>

<div class="config-container">
  {#if store.config}
  <section class="config-group">
    <div class="input-card">
      <div class="text-field-row" class:error={invalidModuleDir}>
        <div class="icon-slot">
          <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.modules} fill="currentColor"/></svg>
        </div>
        <div class="field-content">
          <label for="c-moduledir">{store.L.config.moduleDir}</label>
          <input type="text" id="c-moduledir" bind:value={store.config.moduledir} placeholder="/data/adb/modules" />
        </div>
      </div>
      
      <div class="divider"></div>

      <div class="text-field-row" class:error={invalidTempDir}>
        <div class="icon-slot">
          <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.timer} fill="currentColor"/></svg>
        </div>
        <div class="field-content">
          <label for="c-tempdir">{store.L.config.tempDir}</label>
          <input type="text" id="c-tempdir" bind:value={store.config.tempdir} placeholder={store.L.config.autoPlaceholder} />
        </div>
        {#if store.config.tempdir}
          <button class="mini-btn" onclick={resetTempDir} title={store.L.config.reset}>
             ✕
          </button>
        {/if}
      </div>
    </div>
  </section>

  <section class="config-group">
    <div class="partition-card">
      <div class="partition-header">
        <div class="p-icon">
          <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.storage} fill="currentColor"/></svg>
        </div>
        <div class="p-text">
          <span class="p-title">{store.L.config.partitions}</span>
          <span class="p-desc">Add partitions to mount</span>
        </div>
      </div>
      <div class="p-input">
        <ChipInput bind:values={store.config.partitions} placeholder="e.g. product, system_ext..." />
      </div>
    </div>
  </section>

  <section class="config-group">
    <div class="options-grid">
      <div class="option-tile static-input">
        <div class="tile-top">
          <div class="tile-icon neutral">
            <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.ksu} fill="currentColor"/></svg>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.mountSource}</span>
          <input class="tile-input-overlay" type="text" bind:value={store.config.mountsource} />
        </div>
      </div>

      <button 
        class="option-tile clickable secondary" 
        class:active={store.config.force_ext4} 
        onclick={() => toggle('force_ext4')}
      >
        <div class="tile-top">
          <div class="tile-icon">
            <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.save} fill="currentColor"/></svg>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.forceExt4}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable error" 
        class:active={store.config.enable_nuke} 
        onclick={() => toggle('enable_nuke')}
      >
        <div class="tile-top">
          <div class="tile-icon">
            <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.cat_paw} fill="currentColor"/></svg>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.enableNuke}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable tertiary" 
        class:active={store.config.disable_umount} 
        onclick={() => toggle('disable_umount')}
      >
        <div class="tile-top">
          <div class="tile-icon">
            <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.anchor} fill="currentColor"/></svg>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.disableUmount}</span>
        </div>
      </button>

      <button 
        class="option-tile clickable primary" 
        class:active={store.config.verbose} 
        onclick={() => toggle('verbose')}
      >
        <div class="tile-top">
          <div class="tile-icon">
            <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.description} fill="currentColor"/></svg>
          </div>
        </div>
        <div class="tile-bottom">
          <span class="tile-label">{store.L.config.verboseLabel}</span>
        </div>
      </button>

      {#if store.config.verbose}
        <button 
          class="option-tile clickable secondary" 
          class:active={store.config.dry_run} 
          onclick={() => toggle('dry_run')}
          transition:slide
        >
          <div class="tile-top">
            <div class="tile-icon">
              <svg width="24" height="24" viewBox="0 0 24 24"><path d={ICONS.ghost} fill="currentColor"/></svg>
            </div>
          </div>
          <div class="tile-bottom">
            <span class="tile-label">{store.L.config.dryRun}</span>
          </div>
        </button>
      {/if}
    </div>
  </section>
  {:else}
    <div class="loading-placeholder">
       <span class="loader"></span>
    </div>
  {/if}
</div>

<BottomActions>
  <button 
    class="btn-tonal" 
    onclick={reload}
    disabled={store.loading.config}
    title={store.L.config.reload}
  >
    <svg width="20" height="20" viewBox="0 0 24 24"><path d={ICONS.refresh} fill="currentColor"/></svg>
  </button>
  <div class="spacer"></div>
  <button class="btn-filled" onclick={save} disabled={store.saving.config || !isDirty}>
    <svg width="18" height="18" viewBox="0 0 24 24"><path d={ICONS.save} fill="currentColor"/></svg>
    {store.saving.config ? store.L.common.saving : store.L.config.save}
  </button>
</BottomActions>
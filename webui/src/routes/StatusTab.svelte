<script>
  import { onMount } from 'svelte';
  import { store } from '../lib/store.svelte';
  import { ICONS } from '../lib/constants';
  import { BUILTIN_PARTITIONS } from '../lib/constants_gen';
  import Skeleton from '../components/Skeleton.svelte';
  import './StatusTab.css';

  onMount(() => {
    store.loadStatus();
  });

  let displayPartitions = $derived([...new Set([...BUILTIN_PARTITIONS, ...store.config.partitions])]);
  let storageLabel = $derived(store.storage.type === 'tmpfs' ? store.systemInfo.mountBase : store.L.status.storageDesc);
</script>

<div class="dashboard-grid">
  <div class="storage-card">
    {#if store.loading.status}
      <div class="storage-header">
        <Skeleton width="100px" height="20px" />
        <Skeleton width="60px" height="32px" />
      </div>
      <div class="progress-track progress-track-skeleton">
        <Skeleton width="100%" height="8px" borderRadius="4px" />
      </div>
      <div class="storage-details">
        <Skeleton width="150px" height="12px" />
        <Skeleton width="80px" height="12px" />
      </div>
    {:else}
      <div class="storage-header">
        <div class="storage-title-group">
          <span class="storage-title">{store.L.status.storageTitle}</span>
          {#if store.storage.type && store.storage.type !== 'unknown'}
            <span class="storage-type-badge {store.storage.type === 'tmpfs' ? 'type-tmpfs' : 'type-ext4'}">
              {store.storage.type.toUpperCase()}
            </span>
          {/if}
        </div>
        <div class="storage-value">
          {store.storage.percent}
        </div>
      </div>
      <div class="progress-track">
        <div class="progress-fill" style="width: {store.storage.percent}"></div>
      </div>
      <div class="storage-details">
        <span>{storageLabel}</span>
        <span>{store.storage.used} / {store.storage.size}</span>
      </div>
    {/if}
  </div>

  <div class="stats-row">
    <div class="stat-card">
      {#if store.loading.status}
        <Skeleton width="40px" height="32px" />
        <Skeleton width="60px" height="12px" style="margin-top: 8px" />
      {:else}
        <div class="stat-value">{store.modules.length}</div>
        <div class="stat-label">{store.L.status.moduleActive}</div>
      {/if}
    </div>
    <div class="stat-card">
      {#if store.loading.status}
        <Skeleton width="40px" height="32px" />
        <Skeleton width="60px" height="12px" style="margin-top: 8px" />
      {:else}
        <div class="stat-value">{store.config.mountsource}</div>
        <div class="stat-label">{store.L.config.mountSource}</div>
      {/if}
    </div>
  </div>

  <div class="mode-card">
    <div class="mode-title">{store.L.status.activePartitions}</div>
    <div class="partition-grid">
      {#if store.loading.status}
        {#each Array(4) as _}
          <Skeleton width="60px" height="24px" borderRadius="8px" />
        {/each}
      {:else}
        {#each displayPartitions as part}
          <div class="part-chip {store.activePartitions.includes(part) ? 'active' : 'inactive'}">
            {part}
          </div>
        {/each}
      {/if}
    </div>
  </div>

  <div class="mode-card">
    <div class="mode-title">{store.L.status.sysInfoTitle}</div>
    <div class="info-grid">
      <div class="info-item">
        <span class="info-label">{store.L.status.kernel}</span>
        {#if store.loading.status}
          <Skeleton width="80%" height="16px" style="margin-top: 4px" />
        {:else}
          <span class="info-val">{store.systemInfo.kernel}</span>
        {/if}
      </div>
      <div class="info-item">
        <span class="info-label">{store.L.status.selinux}</span>
        {#if store.loading.status}
          <Skeleton width="40%" height="16px" style="margin-top: 4px" />
        {:else}
          <span class="info-val">{store.systemInfo.selinux}</span>
        {/if}
      </div>
      <div class="info-item full-width">
        <span class="info-label">{store.L.status.mountBase}</span>
        {#if store.loading.status}
          <Skeleton width="90%" height="16px" style="margin-top: 4px" />
        {:else}
          <span class="info-val mono">{store.systemInfo.mountBase}</span>
        {/if}
      </div>
    </div>
  </div>

  <div class="mode-card">
    <div class="mode-title" style="margin-bottom: 8px;">{store.L.status.modeStats}</div>
    {#if store.loading.status}
      <div class="skeleton-group">
        <div class="skeleton-row">
          <Skeleton width="80px" height="20px" />
          <Skeleton width="30px" height="20px" />
        </div>
        <div class="skeleton-row">
          <Skeleton width="80px" height="20px" />
          <Skeleton width="30px" height="20px" />
        </div>
      </div>
    {:else}
      <div class="mode-row">
        <div class="mode-name">
          <div class="dot" style="background-color: var(--md-sys-color-primary)"></div>
          {store.L.status.modeAuto}
        </div>
        <span class="mode-count">{store.modeStats.auto}</span>
      </div>
      <div class="mode-divider"></div>
      <div class="mode-row">
        <div class="mode-name">
          <div class="dot" style="background-color: var(--md-sys-color-tertiary)"></div>
          {store.L.status.modeMagic}
        </div>
        <span class="mode-count">{store.modeStats.magic}</span>
      </div>
    {/if}
  </div>
</div>

<div class="bottom-actions">
  <div class="spacer"></div>
  <button 
    class="btn-tonal" 
    onclick={() => store.loadStatus()} 
    disabled={store.loading.status}
    title={store.L.logs.refresh}
  >
    <svg viewBox="0 0 24 24" width="20" height="20"><path d={ICONS.refresh} fill="currentColor"/></svg>
  </button>
</div>
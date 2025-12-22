<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/store.svelte';
  import TopBar from './components/TopBar.svelte';
  import NavBar from './components/NavBar.svelte';
  import Toast from './components/Toast.svelte';
  import StatusTab from './routes/StatusTab.svelte';
  import ConfigTab from './routes/ConfigTab.svelte';
  import ModulesTab from './routes/ModulesTab.svelte';
  import LogsTab from './routes/LogsTab.svelte';
  import InfoTab from './routes/InfoTab.svelte';
  import GranaryTab from './routes/GranaryTab.svelte';
  import WinnowingTab from './routes/WinnowingTab.svelte';
  import './app.css';
  import './layout.css';

  let activeTab = $state('status');
  let dragOffset = $state(0);
  let isDragging = $state(false);
  let containerWidth = $state(0);
  let touchStartX = 0;
  let touchStartY = 0;
  let isReady = $state(false);

  let visibleTabs = $derived.by(() => {
    const tabs = ['status', 'config', 'modules', 'logs', 'granary'];
    if (store.conflicts.length > 0) {
      tabs.push('winnowing');
    }
    tabs.push('info');
    return tabs;
  });

  function switchTab(id: string) {
    activeTab = id;
  }
  
  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.changedTouches[0].screenX;
    touchStartY = e.changedTouches[0].screenY;
    isDragging = true;
    dragOffset = 0;
  }

  function handleTouchMove(e: TouchEvent) {
    if (!isDragging) return;
    const currentX = e.changedTouches[0].screenX;
    const currentY = e.changedTouches[0].screenY;
    let diffX = currentX - touchStartX;
    const diffY = currentY - touchStartY;
    if (Math.abs(diffY) > Math.abs(diffX)) return;
    
    if (e.cancelable) e.preventDefault();

    const currentIndex = visibleTabs.indexOf(activeTab);
    if ((currentIndex === 0 && diffX > 0) || (currentIndex === visibleTabs.length - 1 && diffX < 0)) {
      diffX = diffX / 3;
    }
    dragOffset = diffX;
  }

  function handleTouchEnd() {
    if (!isDragging) return;
    isDragging = false;
    const threshold = containerWidth * 0.33 || 80;
    const currentIndex = visibleTabs.indexOf(activeTab);
    let nextIndex = currentIndex;
    if (dragOffset < -threshold && currentIndex < visibleTabs.length - 1) {
      nextIndex = currentIndex + 1;
    } else if (dragOffset > threshold && currentIndex > 0) {
      nextIndex = currentIndex - 1;
    }

    if (nextIndex !== currentIndex) {
      switchTab(visibleTabs[nextIndex]);
    }
    dragOffset = 0;
  }

  $effect(() => {
    if (activeTab === 'winnowing' && !visibleTabs.includes('winnowing')) {
      activeTab = 'granary';
    }
  });

  onMount(async () => {
    try {
      await store.init();
    } finally {
      isReady = true;
    }
  });

  let baseTranslateX = $derived(visibleTabs.indexOf(activeTab) * -(100 / visibleTabs.length));
</script>

<div class="app-root">
  {#if !isReady}
    <div class="loading-container">
       <div class="spinner"></div>
       <span class="loading-text">Loading...</span>
    </div>
  {:else}
    <TopBar />
    <main class="main-content" 
          bind:clientWidth={containerWidth}
          ontouchstart={handleTouchStart} 
          ontouchmove={handleTouchMove}
          ontouchend={handleTouchEnd}
          ontouchcancel={handleTouchEnd}>
      <div class="swipe-track"
           style:transform={`translateX(calc(${baseTranslateX}% + ${dragOffset}px))`}
           style:width={`${visibleTabs.length * 100}%`}
           style:transition={isDragging ? 'none' : 'transform 0.3s cubic-bezier(0.25, 0.8, 0.5, 1)'}>
        
        <div class="swipe-page" style:width={`${100 / visibleTabs.length}%`}><div class="page-scroller"><StatusTab /></div></div>
        <div class="swipe-page" style:width={`${100 / visibleTabs.length}%`}><div class="page-scroller"><ConfigTab /></div></div>
        <div class="swipe-page" style:width={`${100 / visibleTabs.length}%`}><div class="page-scroller"><ModulesTab /></div></div>
        <div class="swipe-page" style:width={`${100 / visibleTabs.length}%`}><div class="page-scroller"><LogsTab /></div></div>
        <div class="swipe-page" style:width={`${100 / visibleTabs.length}%`}><div class="page-scroller"><GranaryTab /></div></div>
        {#if store.conflicts.length > 0}
            <div class="swipe-page" style:width={`${100 / visibleTabs.length}%`}><div class="page-scroller"><WinnowingTab /></div></div>
        {/if}
        <div class="swipe-page" style:width={`${100 / visibleTabs.length}%`}><div class="page-scroller"><InfoTab /></div></div>
        
      </div>
    </main>
    <NavBar {activeTab} onTabChange={switchTab} />
  {/if}
  <Toast />
</div>
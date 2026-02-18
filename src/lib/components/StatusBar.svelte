<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getAllPods } from "../stores/pods.svelte";
  import * as api from "../api/tauri";

  let dockerHealthy = $state(true);
  let appVersion = $state("");
  let healthInterval: ReturnType<typeof setInterval>;

  let pods = $derived(getAllPods());
  let runningCount = $derived(pods.filter(p => p.status === "running").length);
  let stoppedCount = $derived(pods.filter(p => p.status === "stopped").length);
  let totalCpu = $derived(
    Math.round(pods.reduce((sum, p) => sum + p.cpuPercent, 0) * 10) / 10
  );
  let totalRamMB = $derived(
    Math.round(pods.reduce((sum, p) => sum + p.memoryUsed, 0) / 1024 / 1024)
  );

  onMount(async () => {
    try {
      const { getVersion } = await import("@tauri-apps/api/app");
      appVersion = await getVersion();
    } catch { /* not in Tauri */ }

    healthInterval = setInterval(async () => {
      try {
        dockerHealthy = await api.checkDockerHealth();
      } catch {
        dockerHealthy = false;
      }
    }, 30000);
  });

  onDestroy(() => {
    clearInterval(healthInterval);
  });
</script>

<div class="status-bar" data-testid="status-bar">
  <div class="status-bar__left">
    <span class="status-bar__docker" data-testid="docker-health">
      <span class="health-dot" class:health-dot--ok={dockerHealthy} class:health-dot--error={!dockerHealthy}></span>
      Docker
    </span>
    <span class="status-bar__divider">|</span>
    <span>{runningCount} running, {stoppedCount} stopped</span>
    {#if runningCount > 0}
      <span class="status-bar__divider">|</span>
      <span>CPU: {totalCpu}%</span>
      <span class="status-bar__divider">|</span>
      <span>RAM: {totalRamMB} MB</span>
    {/if}
  </div>
  <div class="status-bar__right">
    {#if appVersion}
      <span class="text-secondary" data-testid="app-version">v{appVersion}</span>
    {/if}
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    padding: 0 var(--space-4);
    background-color: var(--bg-surface);
    border-top: 1px solid var(--bg-border);
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    flex-shrink: 0;
  }
  .status-bar__left {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .status-bar__right {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .status-bar__divider {
    color: var(--bg-border);
  }
  .status-bar__docker {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }
  .health-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .health-dot--ok {
    background-color: var(--status-running);
  }
  .health-dot--error {
    background-color: var(--status-error);
  }
</style>

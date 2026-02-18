<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Plus, Settings as SettingsIcon } from "lucide-svelte";
  import NookLogo from "./lib/components/NookLogo.svelte";
  import {
    loadPods,
    initPodListeners,
    destroyPodListeners,
    updatePod,
    removePodFromStore,
    getPod,
  } from "./lib/stores/pods.svelte";
  import { initLogListeners, destroyLogListeners, removeLogsForPod } from "./lib/stores/logs.svelte";
  import { loadSettings } from "./lib/stores/settings.svelte";
  import * as api from "./lib/api/tauri";
  import PodGrid from "./lib/components/PodGrid.svelte";
  import AddPodDialog from "./lib/components/AddPodDialog.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";
  import OnboardingChecklist from "./lib/components/OnboardingChecklist.svelte";
  import ErrorPopup from "./lib/components/ErrorPopup.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import DeleteConfirmDialog from "./lib/components/DeleteConfirmDialog.svelte";
  import NotificationToast from "./lib/components/NotificationToast.svelte";
  import PodSettingsDialog from "./lib/components/PodSettingsDialog.svelte";
  import type { Pod, PodStatusChanged } from "./lib/types";
  import { listen } from "@tauri-apps/api/event";

  type View = "main" | "settings" | "onboarding";

  let view = $state<View>("main");
  let showAddDialog = $state(false);
  let globalError = $state<{ title: string; message: string } | null>(null);
  let deleteTarget = $state<{ id: string; name: string } | null>(null);
  let crashNotification = $state<string | null>(null);
  let settingsTarget = $state<{ id: string; name: string; projectPath: string } | null>(null);
  let crashUnlisten: (() => void) | null = null;

  onMount(async () => {
    // Check if running inside Tauri WebView
    const isTauri = !!(window as any).__TAURI_INTERNALS__;
    if (!isTauri) {
      console.warn("Not running inside Tauri — IPC unavailable, running in preview mode");
      return;
    }

    try {
      await loadSettings();
      await loadPods();
      await initPodListeners();
      await initLogListeners();

      // Listen for crashes (pod goes to error from running)
      crashUnlisten = await listen<PodStatusChanged>("pod-status-changed", (event) => {
        const { podId, status } = event.payload;
        if (status === "error") {
          const pod = getPod(podId);
          if (pod && (pod.status === "running" || pod.status === "starting")) {
            crashNotification = `Pod "${pod.alias || pod.name}" crashed unexpectedly`;
          }
        }
      });
    } catch (e) {
      console.error("Initialization failed:", e);
      globalError = {
        title: "Initialization Failed",
        message: String(e),
      };
    }
  });

  onDestroy(() => {
    destroyPodListeners();
    destroyLogListeners();
    if (crashUnlisten) crashUnlisten();
  });

  async function handleAddPod(path: string) {
    showAddDialog = false;
    try {
      const pod = await api.addPod(path);
      updatePod(pod);
    } catch (e) {
      console.error("Failed to add pod:", e);
    }
  }

  async function handleStartPod(id: string) {
    try {
      await api.startPod(id);
    } catch (e) {
      console.error("Failed to start pod:", e);
    }
  }

  async function handleStopPod(id: string) {
    try {
      await api.stopPod(id);
    } catch (e) {
      console.error("Failed to stop pod:", e);
    }
  }

  function handleDeletePod(id: string) {
    const pod = getPod(id);
    deleteTarget = { id, name: pod?.alias || pod?.name || id };
  }

  async function confirmDeletePod(removeVolumes: boolean) {
    if (!deleteTarget) return;
    const { id } = deleteTarget;
    deleteTarget = null;
    try {
      await api.removePod(id, removeVolumes);
      removePodFromStore(id);
      removeLogsForPod(id);
    } catch (e) {
      console.error("Failed to remove pod:", e);
    }
  }

  async function handleTerminal(id: string) {
    try {
      await api.openTerminal(id);
    } catch (e) {
      console.error("Failed to open terminal:", e);
    }
  }

  async function handleExposePort(podId: string, containerPort: number) {
    try {
      await api.exposePort(podId, containerPort, containerPort);
      // Reload pod data
      const pods = await api.listPods();
      const pod = pods.find((p: Pod) => p.id === podId);
      if (pod) updatePod(pod);
    } catch (e) {
      console.error("Failed to expose port:", e);
    }
  }

  async function handleUnexposePort(podId: string, containerPort: number) {
    try {
      await api.unexposePort(podId, containerPort);
      const pods = await api.listPods();
      const pod = pods.find((p: Pod) => p.id === podId);
      if (pod) updatePod(pod);
    } catch (e) {
      console.error("Failed to unexpose port:", e);
    }
  }

  async function handleIgnorePort(podId: string, containerPort: number) {
    try {
      await api.ignorePort(podId, containerPort);
      const pods = await api.listPods();
      const pod = pods.find((p: Pod) => p.id === podId);
      if (pod) updatePod(pod);
    } catch (e) {
      console.error("Failed to ignore port:", e);
    }
  }

  function handleRetry(id: string) {
    handleStartPod(id);
  }

  function handleDismiss(_id: string) {
    // Transition to stopped - handled by store on next status change
  }

  async function handleRestart(id: string) {
    try {
      await api.restartPod(id);
    } catch (e) {
      console.error("Failed to restart pod:", e);
    }
  }

  async function handleForceStop(id: string) {
    try {
      await api.forceStopPod(id);
    } catch (e) {
      console.error("Failed to force stop pod:", e);
    }
  }

  async function handleRebuild(id: string) {
    try {
      await api.rebuildPod(id);
    } catch (e) {
      console.error("Failed to rebuild pod:", e);
    }
  }

  function handlePodSettings(id: string) {
    const pod = getPod(id);
    if (pod) {
      settingsTarget = { id, name: pod.name, projectPath: pod.projectPath };
    }
  }

  async function handleOnboardingReady() {
    try {
      const settings = await api.getSettings();
      settings.onboardingComplete = true;
      await api.saveSettings(settings);
    } catch (e) {
      console.error("Failed to save onboarding state:", e);
    }
    view = "main";
  }
</script>

<div class="app">
  <!-- Header -->
  <header class="app-header">
    <div class="app-header__brand">
      <NookLogo size={22} />
      <h1 class="app-header__title">Nook</h1>
    </div>
    <div class="app-header__actions">
      <button
        class="btn-primary"
        data-testid="add-pod-button"
        aria-label="Add Pod"
        onclick={() => (showAddDialog = true)}
      >
        <Plus size={14} />
        Add Pod
      </button>
      <button
        class="btn-secondary"
        data-testid="settings-button"
        aria-label="Settings"
        onclick={() => (view = view === "settings" ? "main" : "settings")}
      >
        <SettingsIcon size={14} />
      </button>
    </div>
  </header>

  <!-- Main content -->
  <main class="app-main">
    {#if view === "onboarding"}
      <OnboardingChecklist onReady={handleOnboardingReady} />
    {:else}
      <PodGrid
        onStart={handleStartPod}
        onStop={handleStopPod}
        onDelete={handleDeletePod}
        onTerminal={handleTerminal}
        onExposePort={handleExposePort}
        onUnexposePort={handleUnexposePort}
        onIgnorePort={handleIgnorePort}
        onRetry={handleRetry}
        onDismiss={handleDismiss}
        onRestart={handleRestart}
        onForceStop={handleForceStop}
        onRebuild={handleRebuild}
        onSettings={handlePodSettings}
        onAddPod={() => (showAddDialog = true)}
      />
    {/if}
  </main>

  <!-- Status Bar -->
  <StatusBar />

  <!-- Overlays -->
  {#if showAddDialog}
    <AddPodDialog onAdd={handleAddPod} onClose={() => (showAddDialog = false)} />
  {/if}

  {#if view === "settings"}
    <SettingsPanel onClose={() => (view = "main")} />
  {/if}

  {#if deleteTarget}
    <DeleteConfirmDialog
      podName={deleteTarget.name}
      onConfirm={confirmDeletePod}
      onCancel={() => (deleteTarget = null)}
    />
  {/if}

  {#if globalError}
    <ErrorPopup
      title={globalError.title}
      message={globalError.message}
      onRetry={() => {
        globalError = null;
        location.reload();
      }}
      onDismiss={() => (globalError = null)}
    />
  {/if}
  {#if settingsTarget}
    <PodSettingsDialog
      podId={settingsTarget.id}
      podName={settingsTarget.name}
      projectPath={settingsTarget.projectPath}
      onClose={() => (settingsTarget = null)}
    />
  {/if}

  {#if crashNotification}
    <NotificationToast
      message={crashNotification}
      onDismiss={() => (crashNotification = null)}
    />
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .app-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    background-color: var(--bg-surface);
    border-bottom: 1px solid var(--bg-border);
    flex-shrink: 0;
    -webkit-app-region: drag;
  }

  .app-header__brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    -webkit-app-region: drag;
  }

  .app-header__title {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }

  .app-header__actions {
    display: flex;
    gap: var(--space-2);
    -webkit-app-region: no-drag;
  }

  .app-header__actions button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .app-main {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>

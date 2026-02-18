<script lang="ts">
  import { getRunningPods, getStoppedPods, getAllPods } from "../stores/pods.svelte";
  import PodTile from "./PodTile.svelte";
  import PodTileInactive from "./PodTileInactive.svelte";
  import { PackageOpen } from "lucide-svelte";

  interface Props {
    onStart: (id: string) => void;
    onStop: (id: string) => void;
    onDelete: (id: string) => void;
    onTerminal: (id: string) => void;
    onExposePort: (podId: string, containerPort: number) => void;
    onUnexposePort: (podId: string, containerPort: number) => void;
    onIgnorePort: (podId: string, containerPort: number) => void;
    onRetry: (id: string) => void;
    onDismiss: (id: string) => void;
    onRestart: (id: string) => void;
    onForceStop: (id: string) => void;
    onRebuild: (id: string) => void;
    onSettings: (id: string) => void;
    onAddPod: () => void;
  }

  let {
    onStart,
    onStop,
    onDelete,
    onTerminal,
    onExposePort,
    onUnexposePort,
    onIgnorePort,
    onRetry,
    onDismiss,
    onRestart,
    onForceStop,
    onRebuild,
    onSettings,
    onAddPod,
  }: Props = $props();

  let running = $derived(getRunningPods());
  let stopped = $derived(getStoppedPods());
  let isEmpty = $derived(getAllPods().length === 0);
</script>

<div class="pod-grid">
  {#if isEmpty}
    <div class="empty-state" data-testid="empty-state">
      <PackageOpen size={48} />
      <h2>No pods yet</h2>
      <p class="text-secondary">Add a dev container project to get started.</p>
      <button
        class="btn-primary"
        onclick={onAddPod}
        aria-label="Add your first pod"
      >
        Add Pod
      </button>
    </div>
  {:else}
    <!-- Running pods first -->
    {#each running as pod (pod.id)}
      <PodTile
        {pod}
        {onStop}
        {onDelete}
        {onTerminal}
        {onExposePort}
        {onUnexposePort}
        {onIgnorePort}
        {onRetry}
        {onDismiss}
        {onRestart}
        {onForceStop}
        {onRebuild}
        {onSettings}
      />
    {/each}

    <!-- Stopped pods -->
    {#each stopped as pod (pod.id)}
      <PodTileInactive {pod} {onStart} {onDelete} />
    {/each}
  {/if}
</div>

<style>
  .pod-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    overflow-y: auto;
    flex: 1;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
    padding: var(--space-6);
    color: var(--text-secondary);
    text-align: center;
    flex: 1;
  }

  .empty-state h2 {
    font-size: var(--font-size-lg);
    color: var(--text-primary);
    font-weight: 600;
  }

  .empty-state p {
    font-size: var(--font-size-sm);
  }
</style>

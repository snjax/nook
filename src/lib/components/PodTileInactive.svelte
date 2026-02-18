<script lang="ts">
  import { Play, Trash2 } from "lucide-svelte";
  import type { Pod } from "../types";
  import { formatTimeAgo } from "../utils/format";

  interface Props {
    pod: Pod;
    onStart: (id: string) => void;
    onDelete: (id: string) => void;
  }

  let { pod, onStart, onDelete }: Props = $props();
</script>

<div
  class="pod-tile-inactive"
  data-testid="pod-tile-{pod.id}"
  aria-label="Pod {pod.name}"
>
  <div class="pod-tile-inactive__left">
    <span class="status-dot status-dot--stopped"></span>
    <span class="pod-tile-inactive__name text-ellipsis">{pod.alias || pod.name}</span>
    <span class="pod-tile-inactive__meta text-secondary">
      {pod.projectPath}
      {#if pod.image}
        &middot; image: {pod.image}
      {/if}
      &middot; stopped {formatTimeAgo(pod.uptimeSecs)}
    </span>
  </div>
  <div class="pod-tile-inactive__actions">
    <button
      class="btn-primary"
      data-testid="pod-start-{pod.id}"
      aria-label="Start pod {pod.name}"
      onclick={() => onStart(pod.id)}
    >
      <Play size={14} />
      Start
    </button>
    <button
      class="btn-danger"
      data-testid="pod-delete-{pod.id}"
      aria-label="Delete pod {pod.name}"
      onclick={() => onDelete(pod.id)}
    >
      <Trash2 size={14} />
      Del
    </button>
  </div>
</div>

<style>
  .pod-tile-inactive {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 var(--space-4);
    background-color: var(--bg-surface);
    border: 1px solid var(--bg-border);
    border-left: 3px solid var(--status-stopped);
    border-radius: var(--radius-md);
  }

  .pod-tile-inactive__left {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    flex: 1;
  }

  .pod-tile-inactive__name {
    font-size: var(--font-size-md);
    font-weight: 600;
    max-width: 200px;
  }

  .pod-tile-inactive__meta {
    font-size: var(--font-size-sm);
  }

  .pod-tile-inactive__actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .pod-tile-inactive__actions button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }
</style>

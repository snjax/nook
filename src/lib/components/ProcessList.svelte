<script lang="ts">
  import type { Process } from "../types";
  import { formatBytes } from "../utils/format";

  interface Props {
    processes: Process[];
    podId: string;
  }

  let { processes, podId }: Props = $props();
</script>

<div
  class="process-list"
  data-testid="pod-processes-{podId}"
  aria-label="Process list for pod {podId}"
>
  <div class="process-list__header text-secondary">Processes:</div>
  <div class="process-list__items">
    {#each processes as proc (proc.pid)}
      <div class="process-list__row mono">
        <span class="process-list__name text-ellipsis">{proc.name}</span>
        <span class="process-list__cpu">CPU {proc.cpuPercent.toFixed(1)}%</span>
        <span class="process-list__ram">RAM {formatBytes(proc.memoryBytes)}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .process-list {
    font-size: var(--font-size-sm);
  }

  .process-list__header {
    font-family: var(--font-ui);
    margin-bottom: var(--space-1);
  }

  .process-list__items {
    max-height: 150px;
    overflow-y: auto;
  }

  .process-list__row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-1) 0;
  }

  .process-list__name {
    flex: 1;
    min-width: 0;
  }

  .process-list__cpu,
  .process-list__ram {
    flex-shrink: 0;
    color: var(--text-secondary);
    min-width: 80px;
    text-align: right;
  }
</style>

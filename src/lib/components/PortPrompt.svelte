<script lang="ts">
  import { Zap } from "lucide-svelte";
  import type { DetectedPort } from "../types";

  interface Props {
    port: DetectedPort;
    podId: string;
    onExpose: (containerPort: number) => void;
    onIgnore: (containerPort: number) => void;
  }

  let { port, podId, onExpose, onIgnore }: Props = $props();
</script>

<div
  class="port-prompt"
  data-testid="port-prompt-{podId}-{port.containerPort}"
  aria-label="New port detected: {port.containerPort}"
>
  <div class="port-prompt__info">
    <Zap size={14} />
    <span class="mono">:{port.containerPort}</span>
    {#if port.protocol}
      <span class="port-prompt__protocol">{port.protocol}</span>
    {/if}
    {#if port.processName}
      <span class="port-prompt__process text-secondary">{port.processName}</span>
    {/if}
  </div>
  <div class="port-prompt__actions">
    <button
      class="btn-primary"
      data-testid="port-expose-{podId}-{port.containerPort}"
      aria-label="Expose port {port.containerPort}"
      onclick={() => onExpose(port.containerPort)}
    >
      Expose
    </button>
    <button
      class="btn-secondary"
      data-testid="port-ignore-{podId}-{port.containerPort}"
      aria-label="Ignore port {port.containerPort}"
      onclick={() => onIgnore(port.containerPort)}
    >
      Ignore
    </button>
  </div>
</div>

<style>
  .port-prompt {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    background-color: var(--port-prompt-bg);
    border-radius: var(--radius-sm);
  }

  .port-prompt__info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--accent);
    font-size: var(--font-size-sm);
  }

  .port-prompt__protocol {
    color: var(--text-primary);
  }

  .port-prompt__process {
    font-size: var(--font-size-xs);
  }

  .port-prompt__actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }
</style>

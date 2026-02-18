<script lang="ts">
  import { Terminal, Square, Trash2, RotateCcw, XCircle, RotateCw, RefreshCw, Zap, ChevronDown, ChevronRight, Settings, Clipboard, Check } from "lucide-svelte";
  import type { Pod } from "../types";
  import { formatUptime, formatCpuPercent, formatMemory } from "../utils/format";
  import { getCpuHistory, getRamHistory } from "../stores/pods.svelte";
  import ResourceChart from "./ResourceChart.svelte";
  import PortBadge from "./PortBadge.svelte";
  import PortPrompt from "./PortPrompt.svelte";
  import ProcessList from "./ProcessList.svelte";

  interface Props {
    pod: Pod;
    onStop: (id: string) => void;
    onDelete: (id: string) => void;
    onTerminal: (id: string) => void;
    onExposePort: (podId: string, containerPort: number) => void;
    onUnexposePort: (podId: string, containerPort: number) => void;
    onIgnorePort: (podId: string, containerPort: number) => void;
    onRetry?: (id: string) => void;
    onDismiss?: (id: string) => void;
    onRestart?: (id: string) => void;
    onForceStop?: (id: string) => void;
    onRebuild?: (id: string) => void;
    onSettings?: (id: string) => void;
  }

  let {
    pod,
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
  }: Props = $props();

  let portsCollapsed = $state(false);
  let processesCollapsed = $state(false);
  let cmdCopied = $state(false);

  function getAttachCmd(): string {
    const target = pod.containerName || pod.containerId || pod.name;
    const shell = pod.defaultShell || "/bin/sh";
    const parts = ["docker exec -it"];
    if (pod.remoteUser) parts.push(`-u ${pod.remoteUser}`);
    if (pod.remoteWorkspaceFolder) parts.push(`-w ${pod.remoteWorkspaceFolder}`);
    parts.push(target, shell);
    return parts.join(" ");
  }

  async function copyAttachCmd() {
    await navigator.clipboard.writeText(getAttachCmd());
    cmdCopied = true;
    setTimeout(() => (cmdCopied = false), 1500);
  }

  function getBorderColor(): string {
    switch (pod.status) {
      case "running":
        return "var(--status-running)";
      case "error":
        return "var(--status-error)";
      case "starting":
      case "stopping":
        return "var(--status-pending)";
      default:
        return "var(--status-stopped)";
    }
  }

  function getStatusDotClass(): string {
    return `status-dot status-dot--${pod.status}`;
  }
</script>

<div
  class="pod-tile"
  data-testid="pod-tile-{pod.id}"
  aria-label="Pod {pod.name}"
  style="border-left-color: {getBorderColor()}"
>
  <!-- Header -->
  <div class="pod-tile__header">
    <div class="pod-tile__title">
      {#if pod.status === "starting" || pod.status === "stopping"}
        <span class="spinner"></span>
      {:else}
        <span class={getStatusDotClass()} data-testid="pod-status-{pod.id}"></span>
      {/if}
      <span class="pod-tile__name text-ellipsis">{pod.alias || pod.name}</span>
    </div>
    <div class="pod-tile__actions">
      {#if pod.status === "running"}
        <button
          class="btn-secondary"
          data-testid="pod-terminal-{pod.id}"
          aria-label="Open terminal for {pod.name}"
          onclick={() => onTerminal(pod.id)}
        >
          <Terminal size={14} />
          Terminal
        </button>
        <button
          class="btn-secondary"
          data-testid="pod-restart-{pod.id}"
          aria-label="Restart pod {pod.name}"
          onclick={() => onRestart?.(pod.id)}
        >
          <RotateCw size={14} />
          Restart
        </button>
        <button
          class="btn-secondary"
          data-testid="pod-rebuild-{pod.id}"
          aria-label="Rebuild pod {pod.name}"
          onclick={() => onRebuild?.(pod.id)}
        >
          <RefreshCw size={14} />
          Rebuild
        </button>
      {/if}
      {#if pod.status === "error"}
        {#if onRetry}
          <button
            class="btn-primary"
            aria-label="Retry pod {pod.name}"
            onclick={() => onRetry?.(pod.id)}
          >
            <RotateCcw size={14} />
            Retry
          </button>
        {/if}
        {#if onDismiss}
          <button
            class="btn-secondary"
            aria-label="Dismiss error for {pod.name}"
            onclick={() => onDismiss?.(pod.id)}
          >
            <XCircle size={14} />
            Dismiss
          </button>
        {/if}
      {/if}
      {#if pod.status === "stopping"}
        <button
          class="btn-danger"
          data-testid="pod-force-stop-{pod.id}"
          aria-label="Force stop pod {pod.name}"
          onclick={() => onForceStop?.(pod.id)}
        >
          <Zap size={14} />
          Force Kill
        </button>
      {:else}
        <button
          class="btn-secondary"
          data-testid="pod-stop-{pod.id}"
          aria-label="Stop pod {pod.name}"
          onclick={() => onStop(pod.id)}
          disabled={pod.status === "starting"}
        >
          <Square size={14} />
          Stop
        </button>
      {/if}
      <button
        class="btn-icon"
        data-testid="pod-settings-{pod.id}"
        aria-label="Settings for {pod.name}"
        onclick={() => onSettings?.(pod.id)}
      >
        <Settings size={14} />
      </button>
      <button
        class="btn-danger"
        data-testid="pod-delete-{pod.id}"
        aria-label="Delete pod {pod.name}"
        onclick={() => onDelete(pod.id)}
        disabled={pod.status === "stopping"}
      >
        <Trash2 size={14} />
        Del
      </button>
    </div>
  </div>

  <!-- Metadata -->
  <div class="pod-tile__meta text-secondary">
    {pod.projectPath}
    {#if pod.containerName && pod.containerName !== pod.name}
      &middot; container: {pod.containerName}
    {/if}
    {#if pod.image}
      &middot; image: {pod.image}
    {/if}
    {#if pod.status === "running"}
      &middot; up {formatUptime(pod.uptimeSecs)}
    {:else if pod.status === "starting"}
      &middot; Starting...
    {:else if pod.status === "stopping"}
      &middot; Stopping...
    {/if}
  </div>

  <!-- Error message -->
  {#if pod.status === "error" && pod.errorMessage}
    <div class="pod-tile__error">
      {pod.errorMessage}
    </div>
  {/if}

  <!-- Attach command -->
  {#if pod.status === "running"}
    <div class="pod-tile__attach-cmd">
      <code class="pod-tile__attach-cmd-text text-ellipsis">{getAttachCmd()}</code>
      <button
        class="btn-icon"
        onclick={copyAttachCmd}
        aria-label="Copy attach command"
        data-testid="pod-copy-cmd-{pod.id}"
      >
        {#if cmdCopied}
          <Check size={13} />
        {:else}
          <Clipboard size={13} />
        {/if}
      </button>
    </div>
  {/if}

  <!-- Resource charts -->
  {#if pod.status === "running"}
    <div class="pod-tile__resources">
      <div class="pod-tile__resource">
        <span class="text-secondary">CPU</span>
        <ResourceChart
          data={getCpuHistory(pod.id)}
          color="var(--accent)"
          podId={pod.id}
          metric="cpu"
        />
        <span class="mono">{formatCpuPercent(pod.cpuPercent)}</span>
      </div>
      <div class="pod-tile__resource">
        <span class="text-secondary">RAM</span>
        <ResourceChart
          data={getRamHistory(pod.id)}
          color="var(--status-running)"
          podId={pod.id}
          metric="ram"
        />
        <span class="mono">{formatMemory(pod.memoryUsed, pod.memoryLimit)}</span>
      </div>
    </div>

    <!-- Ports -->
    {#if pod.exposedPorts.length > 0 || pod.detectedPorts.length > 0}
      <button class="section-toggle" data-testid="pod-ports-toggle-{pod.id}" onclick={() => (portsCollapsed = !portsCollapsed)}>
        {#if portsCollapsed}<ChevronRight size={14} />{:else}<ChevronDown size={14} />{/if}
        <span class="text-secondary">Ports</span>
      </button>
      {#if !portsCollapsed}
        <div class="pod-tile__ports" data-testid="pod-ports-{pod.id}">
          {#each pod.exposedPorts as port (port.containerPort)}
            <PortBadge
              {port}
              podId={pod.id}
              onRemove={(cp) => onUnexposePort(pod.id, cp)}
            />
          {/each}
          {#each pod.detectedPorts as port (port.containerPort)}
            <PortPrompt
              {port}
              podId={pod.id}
              onExpose={(cp) => onExposePort(pod.id, cp)}
              onIgnore={(cp) => onIgnorePort(pod.id, cp)}
            />
          {/each}
        </div>
      {/if}
    {/if}

    <!-- Processes -->
    {#if pod.processes.length > 0}
      <button class="section-toggle" data-testid="pod-processes-toggle-{pod.id}" onclick={() => (processesCollapsed = !processesCollapsed)}>
        {#if processesCollapsed}<ChevronRight size={14} />{:else}<ChevronDown size={14} />{/if}
        <span class="text-secondary">Processes</span>
      </button>
      {#if !processesCollapsed}
        <ProcessList processes={pod.processes} podId={pod.id} />
      {/if}
    {/if}
  {/if}
</div>

<style>
  .pod-tile {
    background-color: var(--bg-surface);
    border: 1px solid var(--bg-border);
    border-left: 3px solid var(--status-running);
    border-radius: var(--radius-md);
    padding: var(--space-4);
    min-height: 200px;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .pod-tile__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .pod-tile__title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }

  .pod-tile__name {
    font-size: var(--font-size-md);
    font-weight: 600;
  }

  .pod-tile__actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .pod-tile__actions button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .pod-tile__meta {
    font-size: var(--font-size-sm);
  }

  .pod-tile__attach-cmd {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background-color: var(--bg-primary);
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-sm);
    padding: 2px 6px;
    max-width: 100%;
  }

  .pod-tile__attach-cmd-text {
    flex: 1;
    font-size: 11px;
    line-height: 1.2;
    color: var(--text-secondary);
    min-width: 0;
    user-select: all;
  }

  .pod-tile__attach-cmd :global(.btn-icon) {
    flex-shrink: 0;
    padding: 2px;
  }

  .pod-tile__error {
    color: var(--status-error);
    font-size: var(--font-size-sm);
    padding: var(--space-2) var(--space-3);
    background-color: rgba(248, 81, 73, 0.1);
    border-radius: var(--radius-sm);
  }

  .pod-tile__resources {
    display: flex;
    gap: var(--space-6);
  }

  .pod-tile__resource {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
  }

  .pod-tile__ports {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-height: 300px;
    overflow-y: auto;
  }

  .section-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    color: var(--text-secondary);
    font-size: var(--font-size-sm);
  }
  .section-toggle:hover {
    color: var(--text-primary);
  }
</style>

<script lang="ts">
  import { X, Clipboard, ExternalLink } from "lucide-svelte";
  import { open } from "@tauri-apps/plugin-shell";
  import type { ExposedPort } from "../types";

  interface Props {
    port: ExposedPort;
    podId: string;
    onRemove: (containerPort: number) => void;
  }

  let { port, podId, onRemove }: Props = $props();

  function getStatusClass(): string {
    if (port.status === "active") return "badge--active";
    if (port.status === "hostPortBusy") return "badge--busy";
    return "badge--error";
  }

  function getStatusText(): string {
    if (port.status === "active") return "";
    if (port.status === "hostPortBusy") return "port busy";
    if (typeof port.status === "object" && "error" in port.status) {
      return port.status.error;
    }
    return "";
  }
</script>

<div
  class="port-badge {getStatusClass()}"
  data-testid="port-badge-{podId}-{port.containerPort}"
  aria-label="Exposed port {port.containerPort}"
>
  <span class="port-badge__port mono">:{port.containerPort}</span>
  <span class="port-badge__arrow">&rarr;</span>
  <span
    class="port-badge__host mono port-badge__link"
    role="link"
    tabindex="0"
    onclick={() => open(`http://localhost:${port.hostPort}`)}
    onkeydown={(e) => { if (e.key === 'Enter') open(`http://localhost:${port.hostPort}`); }}
  >localhost:{port.hostPort}</span>
  {#if port.protocol}
    <span class="port-badge__protocol">{port.protocol}</span>
  {/if}
  <button
    class="btn-icon"
    data-testid="port-copy-{podId}-{port.containerPort}"
    aria-label="Copy URL for port {port.containerPort}"
    onclick={() => navigator.clipboard.writeText(`http://localhost:${port.hostPort}`)}
  >
    <Clipboard size={14} />
  </button>
  <button
    class="btn-icon"
    data-testid="port-open-{podId}-{port.containerPort}"
    aria-label="Open port {port.containerPort} in browser"
    onclick={() => open(`http://localhost:${port.hostPort}`)}
  >
    <ExternalLink size={14} />
  </button>
  {#if getStatusText()}
    <span class="port-badge__status">{getStatusText()}</span>
  {/if}
  <button
    class="btn-icon port-badge__remove"
    data-testid="port-remove-{podId}-{port.containerPort}"
    aria-label="Remove port {port.containerPort}"
    onclick={() => onRemove(port.containerPort)}
  >
    <X size={14} />
  </button>
</div>

<style>
  .port-badge {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2);
    font-size: var(--font-size-sm);
    border-radius: var(--radius-sm);
  }

  .port-badge--active {
    color: var(--port-exposed);
  }

  .port-badge--busy .port-badge__status {
    color: var(--status-pending);
  }

  .port-badge--error .port-badge__status {
    color: var(--status-error);
  }

  .port-badge__link {
    color: var(--accent);
    cursor: pointer;
    text-decoration: none;
  }

  .port-badge__link:hover {
    text-decoration: underline;
  }

  .port-badge__arrow {
    color: var(--text-secondary);
  }

  .port-badge__protocol {
    color: var(--text-secondary);
  }

  .port-badge__status {
    font-size: var(--font-size-xs);
  }

  .port-badge__remove {
    margin-left: auto;
  }
</style>

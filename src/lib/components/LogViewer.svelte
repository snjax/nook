<script lang="ts">
  import { tick } from "svelte";
  import { Clipboard, Trash2, ArrowDown } from "lucide-svelte";
  import { getLogsForPod } from "../stores/logs.svelte";
  import * as api from "../api/tauri";
  import type { LogEntry } from "../types";

  interface Props {
    podId: string;
    expanded?: boolean;
  }

  let { podId, expanded = false }: Props = $props();

  let collapsed = $state(!expanded);
  let filterText = $state("");
  let logContainer: HTMLDivElement;
  let autoScroll = $state(true);
  let filterTimeout: ReturnType<typeof setTimeout>;

  let allLogs = $derived(getLogsForPod(podId));
  let filteredLogs = $derived.by(() => {
    if (!filterText) return allLogs;
    const lower = filterText.toLowerCase();
    return allLogs.filter(e => e.message.toLowerCase().includes(lower));
  });

  function isErrorLine(entry: LogEntry): boolean {
    return entry.level === "stderr" || /error|Error|fail|panic/i.test(entry.message);
  }

  function handleScroll() {
    if (!logContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = logContainer;
    autoScroll = scrollHeight - scrollTop - clientHeight < 40;
  }

  async function scrollToBottom() {
    autoScroll = true;
    await tick();
    if (logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }

  async function copyLogs() {
    const text = filteredLogs.map(e => e.message).join("\n");
    await navigator.clipboard.writeText(text);
  }

  async function clearLogs() {
    await api.clearPodLogs(podId);
  }

  function handleFilterInput(e: Event) {
    clearTimeout(filterTimeout);
    filterTimeout = setTimeout(() => {
      filterText = (e.target as HTMLInputElement).value;
    }, 300);
  }

  $effect(() => {
    // Auto-scroll when new logs arrive
    if (autoScroll && logContainer && filteredLogs.length > 0) {
      tick().then(() => {
        if (logContainer) {
          logContainer.scrollTop = logContainer.scrollHeight;
        }
      });
    }
  });
</script>

{#if !collapsed}
  <div class="log-viewer" data-testid="log-viewer-{podId}">
    <div class="log-viewer__toolbar">
      <input
        type="text"
        class="log-viewer__filter"
        placeholder="Filter logs..."
        data-testid="log-filter-{podId}"
        aria-label="Filter logs"
        oninput={handleFilterInput}
      />
      <button class="btn-icon" data-testid="log-copy-{podId}" aria-label="Copy logs" onclick={copyLogs}>
        <Clipboard size={14} />
      </button>
      <button class="btn-icon" data-testid="log-clear-{podId}" aria-label="Clear logs" onclick={clearLogs}>
        <Trash2 size={14} />
      </button>
    </div>
    <div class="log-viewer__content" bind:this={logContainer} onscroll={handleScroll}>
      {#each filteredLogs as entry, i (i)}
        <div class="log-line" class:log-line--error={isErrorLine(entry)}>
          {entry.message}
        </div>
      {/each}
      {#if filteredLogs.length === 0}
        <div class="log-viewer__empty text-secondary">No logs</div>
      {/if}
    </div>
    {#if !autoScroll}
      <button class="log-viewer__jump" onclick={scrollToBottom}>
        <ArrowDown size={12} />
        Jump to bottom
      </button>
    {/if}
  </div>
{/if}

<style>
  .log-viewer {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-sm);
    max-height: 300px;
    overflow: hidden;
  }
  .log-viewer__toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--bg-border);
    background-color: var(--bg-surface);
  }
  .log-viewer__filter {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-sm);
    padding: 2px var(--space-2);
    font-size: var(--font-size-xs);
    color: var(--text-primary);
  }
  .log-viewer__content {
    overflow-y: auto;
    padding: var(--space-2);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    line-height: 1.4;
    background-color: var(--bg-primary);
    flex: 1;
  }
  .log-line {
    white-space: pre-wrap;
    word-break: break-all;
  }
  .log-line--error {
    color: var(--status-error);
  }
  .log-viewer__empty {
    padding: var(--space-4);
    text-align: center;
    font-size: var(--font-size-xs);
  }
  .log-viewer__jump {
    position: absolute;
    bottom: var(--space-2);
    right: var(--space-2);
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    background-color: var(--bg-surface);
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    cursor: pointer;
    color: var(--text-secondary);
  }
  .log-viewer__jump:hover {
    color: var(--text-primary);
  }
</style>

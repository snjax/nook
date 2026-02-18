<script lang="ts">
  import { CheckCircle, XCircle, RefreshCw } from "lucide-svelte";
  import type { DependencyCheck } from "../types";
  import * as api from "../api/tauri";

  interface Props {
    onReady: () => void;
  }

  let { onReady }: Props = $props();

  let checks = $state<DependencyCheck[]>([]);
  let checking = $state(true);

  let allSatisfied = $derived(checks.length > 0 && checks.every((c) => c.satisfied));

  async function runChecks() {
    checking = true;
    try {
      checks = await api.checkDependencies();
    } catch (e) {
      console.error("Failed to check dependencies:", e);
    } finally {
      checking = false;
    }
  }

  // Run checks on mount
  runChecks();
</script>

<div class="onboarding" data-testid="onboarding" aria-label="Onboarding checklist">
  <h2>Welcome to Nook</h2>
  <p class="text-secondary">Checking prerequisites...</p>

  <div class="onboarding__checks">
    {#each checks as check (check.name)}
      <div class="onboarding__check">
        <div class="onboarding__check-status">
          {#if check.satisfied}
            <CheckCircle size={20} color="var(--status-running)" />
          {:else}
            <XCircle size={20} color="var(--status-error)" />
          {/if}
        </div>
        <div class="onboarding__check-info">
          <div class="onboarding__check-name">
            {check.name}
            <span class="onboarding__check-detail text-secondary">
              {check.details}
            </span>
          </div>
          {#if !check.satisfied && check.fixHint}
            <div class="onboarding__check-hint mono">{check.fixHint}</div>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <div class="onboarding__actions">
    <button
      class="btn-secondary"
      onclick={runChecks}
      disabled={checking}
      aria-label="Re-check dependencies"
    >
      <RefreshCw size={14} />
      Re-check
    </button>
    {#if allSatisfied}
      <button
        class="btn-primary"
        onclick={onReady}
        aria-label="Ready, let's go"
      >
        Ready, let's go
      </button>
    {/if}
  </div>
</div>

<style>
  .onboarding {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-5);
    padding: var(--space-6);
    max-width: 500px;
    margin: auto;
  }

  h2 {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }

  .onboarding__checks {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .onboarding__check {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-3);
    background-color: var(--bg-surface);
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-md);
  }

  .onboarding__check-status {
    flex-shrink: 0;
    padding-top: 2px;
  }

  .onboarding__check-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .onboarding__check-name {
    font-weight: 500;
  }

  .onboarding__check-detail {
    font-weight: 400;
    margin-left: var(--space-2);
  }

  .onboarding__check-hint {
    font-size: var(--font-size-xs);
    color: var(--accent);
  }

  .onboarding__actions {
    display: flex;
    gap: var(--space-3);
  }

  .onboarding__actions button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }
</style>

<script lang="ts">
  import { AlertTriangle } from "lucide-svelte";

  interface Props {
    title: string;
    message: string;
    onRetry?: () => void;
    onDismiss?: () => void;
    onQuit?: () => void;
  }

  let { title, message, onRetry, onDismiss, onQuit }: Props = $props();
</script>

<div class="modal-overlay" data-testid="error-popup" aria-label="Error popup">
  <div class="modal-content error-popup">
    <div class="error-popup__icon">
      <AlertTriangle size={32} color="var(--status-error)" />
    </div>
    <h2>{title}</h2>
    <p class="text-secondary">{message}</p>
    <div class="error-popup__actions">
      {#if onRetry}
        <button class="btn-primary" onclick={onRetry} aria-label="Retry">
          Retry
        </button>
      {/if}
      {#if onDismiss}
        <button class="btn-secondary" onclick={onDismiss} aria-label="Dismiss">
          Dismiss
        </button>
      {/if}
      {#if onQuit}
        <button class="btn-danger" onclick={onQuit} aria-label="Quit">
          Quit
        </button>
      {/if}
    </div>
  </div>
</div>

<style>
  .error-popup {
    text-align: center;
  }

  .error-popup__icon {
    margin-bottom: var(--space-4);
  }

  h2 {
    font-size: var(--font-size-lg);
    font-weight: 600;
    margin-bottom: var(--space-2);
  }

  p {
    font-size: var(--font-size-sm);
    margin-bottom: var(--space-5);
  }

  .error-popup__actions {
    display: flex;
    gap: var(--space-3);
    justify-content: center;
  }
</style>

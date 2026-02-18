<script lang="ts">
  import { onMount } from "svelte";
  import { X } from "lucide-svelte";

  interface Props {
    message: string;
    onDismiss: () => void;
  }

  let { message, onDismiss }: Props = $props();

  onMount(() => {
    const timer = setTimeout(onDismiss, 10000);
    return () => clearTimeout(timer);
  });
</script>

<div class="toast" data-testid="notification-toast" role="alert">
  <span class="toast__message">{message}</span>
  <button class="btn-icon" aria-label="Dismiss notification" onclick={onDismiss}>
    <X size={14} />
  </button>
</div>

<style>
  .toast {
    position: fixed;
    bottom: 36px;
    right: var(--space-4);
    background-color: var(--bg-surface);
    border: 1px solid var(--status-error);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    z-index: 200;
    max-width: 400px;
    font-size: var(--font-size-sm);
    color: var(--status-error);
  }
  .toast__message {
    flex: 1;
  }
</style>

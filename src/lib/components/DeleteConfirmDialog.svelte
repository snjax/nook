<script lang="ts">
  interface Props {
    podName: string;
    onConfirm: (removeVolumes: boolean) => void;
    onCancel: () => void;
  }

  let { podName, onConfirm, onCancel }: Props = $props();
  let removeVolumes = $state(false);
</script>

<div class="overlay" data-testid="delete-confirm-dialog" role="dialog" aria-label="Confirm delete {podName}">
  <div class="dialog">
    <h3>Delete {podName}?</h3>
    <p class="text-secondary">This will remove the pod from Nook. The container and its data may be deleted.</p>
    <label class="checkbox-label">
      <input
        type="checkbox"
        bind:checked={removeVolumes}
        data-testid="delete-volumes-checkbox"
      />
      Remove volumes
    </label>
    <div class="dialog__actions">
      <button class="btn-secondary" onclick={onCancel}>Cancel</button>
      <button
        class="btn-danger"
        data-testid="delete-confirm-button"
        onclick={() => onConfirm(removeVolumes)}
      >
        Delete
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background-color: var(--bg-surface);
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-md);
    padding: var(--space-5);
    min-width: 360px;
    max-width: 480px;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
  .dialog h3 {
    font-size: var(--font-size-md);
    font-weight: 600;
  }
  .dialog p {
    font-size: var(--font-size-sm);
  }
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-sm);
    cursor: pointer;
  }
  .dialog__actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }
</style>

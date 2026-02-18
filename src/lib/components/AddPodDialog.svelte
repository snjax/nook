<script lang="ts">
  import { FolderOpen, Search } from "lucide-svelte";
  import { pickDirectory } from "../api/tauri";

  interface Props {
    onAdd: (path: string) => void;
    onClose: () => void;
  }

  let { onAdd, onClose }: Props = $props();

  let manualPath = $state("");

  async function handleBrowse() {
    const dir = await pickDirectory();
    if (dir) {
      manualPath = dir;
    }
  }

  function handleManualAdd() {
    if (manualPath.trim()) {
      onAdd(manualPath.trim());
    }
  }
</script>

<div class="modal-overlay" onclick={onClose} role="dialog" aria-label="Add Pod" data-testid="add-pod-dialog">
  <div class="modal-content" onclick={(e) => e.stopPropagation()}>
    <h2>Add Pod</h2>

    <div class="add-dialog__section">
      <label class="add-dialog__label">Project path</label>
      <div class="add-dialog__input-row">
        <input
          type="text"
          bind:value={manualPath}
          placeholder="/path/to/project"
          data-testid="add-pod-path-input"
          aria-label="Project path"
        />
        <button
          class="btn-secondary"
          onclick={handleBrowse}
          data-testid="add-pod-browse"
          aria-label="Browse for project directory"
        >
          <Search size={14} />
          Browse...
        </button>
        <button
          class="btn-primary"
          onclick={handleManualAdd}
          disabled={!manualPath.trim()}
          data-testid="add-pod-submit"
          aria-label="Add pod"
        >
          <FolderOpen size={14} />
          Add
        </button>
      </div>
    </div>

    <div class="add-dialog__footer">
      <button class="btn-secondary" onclick={onClose}>Cancel</button>
    </div>
  </div>
</div>

<style>
  h2 {
    font-size: var(--font-size-lg);
    font-weight: 600;
    margin-bottom: var(--space-4);
  }

  .add-dialog__section {
    margin-bottom: var(--space-4);
  }

  .add-dialog__label {
    display: block;
    font-size: var(--font-size-sm);
    color: var(--text-secondary);
    margin-bottom: var(--space-2);
  }

  .add-dialog__input-row {
    display: flex;
    gap: var(--space-2);
  }

  .add-dialog__input-row button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    flex-shrink: 0;
  }

  .add-dialog__footer {
    display: flex;
    justify-content: flex-end;
    padding-top: var(--space-4);
    border-top: 1px solid var(--bg-border);
  }
</style>

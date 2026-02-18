<script lang="ts">
  import { X, FolderOpen, Clipboard } from "lucide-svelte";
  import * as api from "../api/tauri";
  import type { PodStateConfig } from "../api/tauri";

  interface Props {
    podId: string;
    podName: string;
    projectPath: string;
    onClose: () => void;
  }

  let { podId, podName, projectPath, onClose }: Props = $props();

  let config = $state<PodStateConfig | null>(null);
  let loading = $state(true);
  let saving = $state(false);

  // Load pod settings on mount
  $effect(() => {
    loadConfig();
  });

  async function loadConfig() {
    loading = true;
    try {
      config = await api.getPodSettings(podId);
    } catch (e) {
      console.error("Failed to load pod settings:", e);
      // Create default config if not found
      config = {
        name: podName,
        projectPath,
        shell: "",
        autoExposePorts: [],
        portMappings: {},
        alias: "",
        notExposeFilters: [],
        manualExposePorts: [],
        terminalOverride: "",
        workingDir: "",
        bindAddress: "",
        remoteUser: "",
      };
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    if (!config) return;
    saving = true;
    try {
      await api.savePodSettings(podId, config);
      onClose();
    } catch (e) {
      console.error("Failed to save pod settings:", e);
    } finally {
      saving = false;
    }
  }

  async function openFolder() {
    try {
      await api.openInFileManager(projectPath);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }

  async function copyPath() {
    await navigator.clipboard.writeText(projectPath);
  }

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="pod-settings-overlay" onclick={handleOverlayClick}>
  <div
    class="pod-settings-panel"
    data-testid="pod-settings-{podId}"
    role="dialog"
    aria-label="Pod settings for {podName}"
  >
    <!-- Header -->
    <div class="pod-settings-panel__header">
      <h2>Pod Settings</h2>
      <button
        class="btn-icon"
        onclick={onClose}
        aria-label="Close pod settings"
        data-testid="pod-settings-close-{podId}"
      >
        <X size={18} />
      </button>
    </div>

    <!-- Body -->
    <div class="pod-settings-panel__body">
      {#if loading}
        <div class="pod-settings-panel__loading">
          <span class="spinner"></span>
          <span class="text-secondary">Loading settings...</span>
        </div>
      {:else if config}
        <!-- General Section -->
        <div class="settings-section">
          <h3 class="settings-section__title">General</h3>

          <div class="settings-field">
            <label for="pod-alias-{podId}">Alias</label>
            <span class="settings-field__help">Display name override for this pod.</span>
            <input
              id="pod-alias-{podId}"
              type="text"
              bind:value={config.alias}
              placeholder={podName}
              aria-label="Pod alias"
              data-testid="pod-settings-alias-{podId}"
            />
          </div>

          <div class="settings-field">
            <label>Project Path</label>
            <span class="settings-field__help">Location of the devcontainer project.</span>
            <div class="pod-settings-path">
              <span class="pod-settings-path__text mono text-ellipsis" title={projectPath}>
                {projectPath}
              </span>
              <button
                class="btn-icon"
                onclick={copyPath}
                aria-label="Copy project path"
                data-testid="pod-settings-copy-path-{podId}"
              >
                <Clipboard size={14} />
              </button>
              <button
                class="btn-icon"
                onclick={openFolder}
                aria-label="Open in file manager"
                data-testid="pod-settings-open-folder-{podId}"
              >
                <FolderOpen size={14} />
              </button>
            </div>
          </div>
        </div>

        <!-- Terminal Section -->
        <div class="settings-section">
          <h3 class="settings-section__title">Terminal</h3>

          <div class="settings-field">
            <label for="pod-remote-user-{podId}">Remote User</label>
            <span class="settings-field__help">User to run as inside the container (e.g. vscode). Leave empty for auto-detect from devcontainer config.</span>
            <input
              id="pod-remote-user-{podId}"
              type="text"
              bind:value={config.remoteUser}
              placeholder="vscode"
              aria-label="Remote user"
              data-testid="pod-settings-remote-user-{podId}"
            />
          </div>

          <div class="settings-field">
            <label for="pod-shell-{podId}">Shell Override</label>
            <span class="settings-field__help">Shell to use inside the container (e.g. /bin/zsh). Leave empty for auto-detect.</span>
            <input
              id="pod-shell-{podId}"
              type="text"
              bind:value={config.shell}
              placeholder="/bin/zsh"
              aria-label="Shell override"
              data-testid="pod-settings-shell-{podId}"
            />
          </div>

          <div class="settings-field">
            <label for="pod-terminal-{podId}">Terminal Override</label>
            <span class="settings-field__help">Terminal emulator to use for this pod. Leave empty to use global setting.</span>
            <input
              id="pod-terminal-{podId}"
              type="text"
              bind:value={config.terminalOverride}
              placeholder="alacritty, kitty, etc."
              aria-label="Terminal override"
              data-testid="pod-settings-terminal-{podId}"
            />
          </div>

          <div class="settings-field">
            <label for="pod-workdir-{podId}">Working Directory</label>
            <span class="settings-field__help">Default working directory inside the container. Leave empty for auto-detect (/workspaces/project-name).</span>
            <input
              id="pod-workdir-{podId}"
              type="text"
              bind:value={config.workingDir}
              placeholder="/workspaces/{podName}"
              aria-label="Working directory"
              data-testid="pod-settings-workdir-{podId}"
            />
          </div>
        </div>
      {/if}
    </div>

    <!-- Footer -->
    <div class="pod-settings-panel__footer">
      <button
        class="btn-secondary"
        onclick={onClose}
        aria-label="Cancel pod settings"
        data-testid="pod-settings-cancel-{podId}"
      >
        Cancel
      </button>
      <button
        class="btn-primary"
        onclick={handleSave}
        disabled={saving || loading || !config}
        aria-label="Save pod settings"
        data-testid="pod-settings-save-{podId}"
      >
        {saving ? "Saving..." : "Save"}
      </button>
    </div>
  </div>
</div>

<style>
  .pod-settings-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 100;
  }

  .pod-settings-panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 380px;
    max-width: 100%;
    background-color: var(--bg-primary);
    border-left: 1px solid var(--bg-border);
    display: flex;
    flex-direction: column;
    z-index: 101;
  }

  .pod-settings-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4);
    border-bottom: 1px solid var(--bg-border);
  }

  .pod-settings-panel__header h2 {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }

  .pod-settings-panel__body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .pod-settings-panel__loading {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-6) 0;
    justify-content: center;
  }

  /* Sections */
  .settings-section {
    border-bottom: 1px solid var(--bg-border);
    padding-bottom: var(--space-4);
    margin-bottom: var(--space-2);
  }

  .settings-section:last-of-type {
    border-bottom: none;
    margin-bottom: 0;
  }

  .settings-section__title {
    font-size: var(--font-size-sm);
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: var(--space-3);
  }

  /* Fields */
  .settings-field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-bottom: var(--space-3);
  }

  .settings-field:last-child {
    margin-bottom: 0;
  }

  .settings-field label {
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    font-weight: 500;
  }

  .settings-field__help {
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
  }

  /* Path display */
  .pod-settings-path {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    background-color: var(--bg-surface);
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-sm);
    padding: var(--space-2) var(--space-3);
  }

  .pod-settings-path__text {
    flex: 1;
    font-size: var(--font-size-xs);
    color: var(--text-secondary);
    min-width: 0;
  }

  .pod-settings-path :global(.btn-icon) {
    flex-shrink: 0;
  }

  /* Footer */
  .pod-settings-panel__footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-4);
    border-top: 1px solid var(--bg-border);
  }
</style>

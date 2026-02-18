<script lang="ts">
  import { X, ChevronDown, ChevronRight, Plus, Trash2 } from "lucide-svelte";
  import {
    getSettings,
    saveSettings as doSaveSettings,
  } from "../stores/settings.svelte";
  import * as api from "../api/tauri";
  import type { Settings, NotExposeFilter } from "../types";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let settings = $state<Settings>({ ...getSettings() });
  let saving = $state(false);
  let error = $state("");
  let advancedOpen = $state(false);
  let detectedTerminal = $state("");

  // Field validation errors
  let errors = $state<Record<string, string>>({});

  // Editable string representations
  let exposeProtocolsText = $state(settings.exposeProtocols.join(", "));
  let terminalText = $state(settings.terminal);

  // Editable arrays for filters and port mappings
  let notExposeFilters = $state<NotExposeFilter[]>(
    settings.notExposeFilters.map((f) => ({ ...f })),
  );
  let portProtocolEntries = $state<{ port: string; protocol: string }[]>(
    Object.entries(settings.portProtocols).map(([port, protocol]) => ({
      port: String(port),
      protocol: String(protocol),
    })),
  );

  // Detect terminal on mount
  api.getDetectedTerminal().then((t) => {
    detectedTerminal = t;
  }).catch(() => {
    detectedTerminal = "";
  });

  function addFilter() {
    notExposeFilters = [...notExposeFilters, { protocol: "", port: undefined }];
  }

  function removeFilter(index: number) {
    notExposeFilters = notExposeFilters.filter((_, i) => i !== index);
  }

  function addPortMapping() {
    portProtocolEntries = [...portProtocolEntries, { port: "", protocol: "" }];
  }

  function removePortMapping(index: number) {
    portProtocolEntries = portProtocolEntries.filter((_, i) => i !== index);
  }

  function validate(): boolean {
    const newErrors: Record<string, string> = {};

    if (settings.statsInterval <= 0) {
      newErrors.statsInterval = "Must be greater than 0";
    }
    if (settings.portsScanInterval <= 0) {
      newErrors.portsScanInterval = "Must be greater than 0";
    }
    if (settings.processScanInterval <= 0) {
      newErrors.processScanInterval = "Must be greater than 0";
    }

    // Validate port numbers in filters
    for (let i = 0; i < notExposeFilters.length; i++) {
      const f = notExposeFilters[i];
      if (f.port !== undefined && f.port !== null) {
        const p = Number(f.port);
        if (isNaN(p) || p < 1 || p > 65535) {
          newErrors[`filter_port_${i}`] = "Port must be 1-65535";
        }
      }
    }

    // Validate port numbers in port mappings
    for (let i = 0; i < portProtocolEntries.length; i++) {
      const entry = portProtocolEntries[i];
      if (entry.port) {
        const p = Number(entry.port);
        if (isNaN(p) || p < 1 || p > 65535) {
          newErrors[`mapping_port_${i}`] = "Port must be 1-65535";
        }
      }
    }

    const validLogLevels = ["trace", "debug", "info", "warn", "error"];
    if (!validLogLevels.includes(settings.logLevel)) {
      newErrors.logLevel = "Invalid log level";
    }

    errors = newErrors;
    return Object.keys(newErrors).length === 0;
  }

  async function handleSave() {
    if (!validate()) return;

    saving = true;
    error = "";
    try {
      settings.exposeProtocols = exposeProtocolsText
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      settings.terminal = terminalText;

      // Build notExposeFilters
      settings.notExposeFilters = notExposeFilters
        .filter((f) => f.protocol || f.port)
        .map((f) => ({
          protocol: f.protocol || undefined,
          port: f.port ? Number(f.port) : undefined,
        }));

      // Build portProtocols
      const pp: Record<number, string> = {};
      for (const entry of portProtocolEntries) {
        if (entry.port && entry.protocol) {
          pp[Number(entry.port)] = entry.protocol;
        }
      }
      settings.portProtocols = pp;

      await doSaveSettings(settings);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleReset() {
    try {
      const defaults = await api.getDefaultSettings();
      settings = { ...defaults };
      exposeProtocolsText = defaults.exposeProtocols.join(", ");
      terminalText = defaults.terminal;
      notExposeFilters = defaults.notExposeFilters.map((f) => ({ ...f }));
      portProtocolEntries = Object.entries(defaults.portProtocols).map(
        ([port, protocol]) => ({ port: String(port), protocol: String(protocol) }),
      );
      errors = {};
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="settings-panel" data-testid="settings-panel">
  <div class="settings-panel__header">
    <h2>Settings</h2>
    <button class="btn-icon" onclick={onClose} aria-label="Close settings">
      <X size={18} />
    </button>
  </div>

  <div class="settings-panel__body">
    <!-- Ports & Network Section -->
    <div class="settings-section">
      <h3 class="settings-section__title">Ports &amp; Network</h3>

      <div class="settings-field">
        <label for="port-action">Default Port Action</label>
        <span class="settings-field__help">What to do when a new port is detected in a container.</span>
        <select
          id="port-action"
          bind:value={settings.portAction}
          aria-label="Default port action"
          data-testid="settings-port-action"
        >
          <option value="prompt">Prompt</option>
          <option value="autoExpose">Auto-expose</option>
          <option value="ignore">Ignore</option>
        </select>
      </div>

      <div class="settings-field">
        <label for="expose-protocols">Expose Protocols</label>
        <span class="settings-field__help">Comma-separated list of protocols to auto-detect.</span>
        <input
          id="expose-protocols"
          type="text"
          bind:value={exposeProtocolsText}
          aria-label="Expose protocols"
          data-testid="settings-expose-protocols"
        />
      </div>

      <div class="settings-field">
        <label>Auto-Ignore Filters</label>
        <span class="settings-field__help">Ports matching these filters will be automatically ignored.</span>
        <div class="settings-list">
          {#each notExposeFilters as filter, i}
            <div class="settings-list__row">
              <input
                type="text"
                bind:value={filter.protocol}
                placeholder="Protocol (e.g. dns)"
                aria-label="Filter protocol {i + 1}"
                class="settings-list__input"
              />
              <input
                type="number"
                bind:value={filter.port}
                placeholder="Port"
                min="1"
                max="65535"
                aria-label="Filter port {i + 1}"
                class="settings-list__input settings-list__input--port"
              />
              <button
                class="btn-icon"
                onclick={() => removeFilter(i)}
                aria-label="Remove filter {i + 1}"
              >
                <Trash2 size={14} />
              </button>
            </div>
            {#if errors[`filter_port_${i}`]}
              <span class="settings-field__error">{errors[`filter_port_${i}`]}</span>
            {/if}
          {/each}
          <button
            class="btn-secondary settings-list__add"
            onclick={addFilter}
            aria-label="Add ignore filter"
            data-testid="settings-add-filter"
          >
            <Plus size={14} />
            Add Filter
          </button>
        </div>
      </div>

      <div class="settings-field">
        <label>Custom Port-to-Protocol Mapping</label>
        <span class="settings-field__help">Map specific ports to protocol names for identification.</span>
        <div class="settings-list">
          {#each portProtocolEntries as entry, i}
            <div class="settings-list__row">
              <input
                type="number"
                bind:value={entry.port}
                placeholder="Port"
                min="1"
                max="65535"
                aria-label="Mapping port {i + 1}"
                class="settings-list__input settings-list__input--port"
              />
              <input
                type="text"
                bind:value={entry.protocol}
                placeholder="Protocol (e.g. http)"
                aria-label="Mapping protocol {i + 1}"
                class="settings-list__input"
              />
              <button
                class="btn-icon"
                onclick={() => removePortMapping(i)}
                aria-label="Remove mapping {i + 1}"
              >
                <Trash2 size={14} />
              </button>
            </div>
            {#if errors[`mapping_port_${i}`]}
              <span class="settings-field__error">{errors[`mapping_port_${i}`]}</span>
            {/if}
          {/each}
          <button
            class="btn-secondary settings-list__add"
            onclick={addPortMapping}
            aria-label="Add port mapping"
            data-testid="settings-add-mapping"
          >
            <Plus size={14} />
            Add Mapping
          </button>
        </div>
      </div>
    </div>

    <!-- Terminal Section -->
    <div class="settings-section">
      <h3 class="settings-section__title">Terminal</h3>

      <div class="settings-field">
        <label for="terminal">Terminal Emulator</label>
        {#if !terminalText && detectedTerminal}
          <span class="settings-field__help">Auto-detected: {detectedTerminal}</span>
        {:else}
          <span class="settings-field__help">Leave empty to auto-detect the system terminal.</span>
        {/if}
        <input
          id="terminal"
          type="text"
          bind:value={terminalText}
          placeholder="alacritty, kitty, etc."
          aria-label="Terminal"
          data-testid="settings-terminal"
        />
      </div>
    </div>

    <!-- Monitoring Section -->
    <div class="settings-section">
      <h3 class="settings-section__title">Monitoring</h3>

      <div class="settings-field">
        <label for="stats-interval">Stats Interval (ms)</label>
        <span class="settings-field__help">How often to poll container CPU/memory. Default: 2000</span>
        <input
          id="stats-interval"
          type="number"
          bind:value={settings.statsInterval}
          aria-label="Stats interval"
          data-testid="settings-stats-interval"
        />
        {#if errors.statsInterval}
          <span class="settings-field__error">{errors.statsInterval}</span>
        {/if}
      </div>

      <div class="settings-field">
        <label for="ports-scan-interval">Port Scan Interval (ms)</label>
        <span class="settings-field__help">How often to scan for new ports. Default: 3000</span>
        <input
          id="ports-scan-interval"
          type="number"
          bind:value={settings.portsScanInterval}
          aria-label="Port scan interval"
          data-testid="settings-ports-scan-interval"
        />
        {#if errors.portsScanInterval}
          <span class="settings-field__error">{errors.portsScanInterval}</span>
        {/if}
      </div>

      <div class="settings-field">
        <label for="process-scan-interval">Process Scan Interval (ms)</label>
        <span class="settings-field__help">How often to refresh process lists. Default: 5000</span>
        <input
          id="process-scan-interval"
          type="number"
          bind:value={settings.processScanInterval}
          aria-label="Process scan interval"
          data-testid="settings-process-scan-interval"
        />
        {#if errors.processScanInterval}
          <span class="settings-field__error">{errors.processScanInterval}</span>
        {/if}
      </div>
    </div>

    <!-- Advanced Section -->
    <div class="settings-section">
      <button
        class="settings-section__toggle"
        onclick={() => (advancedOpen = !advancedOpen)}
        aria-expanded={advancedOpen}
        aria-label="Toggle advanced settings"
        data-testid="settings-advanced-toggle"
      >
        {#if advancedOpen}
          <ChevronDown size={16} />
        {:else}
          <ChevronRight size={16} />
        {/if}
        <h3 class="settings-section__title settings-section__title--inline">Advanced</h3>
      </button>

      {#if advancedOpen}
        <div class="settings-section__collapsible">
          <div class="settings-field">
            <label for="docker-socket">Docker Socket Path</label>
            <span class="settings-field__help">Leave empty to use the system default.</span>
            <input
              id="docker-socket"
              type="text"
              bind:value={settings.dockerSocketPath}
              placeholder="/var/run/docker.sock"
              aria-label="Docker socket path"
              data-testid="settings-docker-socket"
            />
          </div>

          <div class="settings-field">
            <label for="log-level">Log Level</label>
            <span class="settings-field__help">Requires app restart to take effect.</span>
            <select
              id="log-level"
              bind:value={settings.logLevel}
              aria-label="Log level"
              data-testid="settings-log-level"
            >
              <option value="trace">Trace</option>
              <option value="debug">Debug</option>
              <option value="info">Info</option>
              <option value="warn">Warn</option>
              <option value="error">Error</option>
            </select>
            {#if errors.logLevel}
              <span class="settings-field__error">{errors.logLevel}</span>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    {#if error}
      <div class="text-error">{error}</div>
    {/if}
  </div>

  <div class="settings-panel__footer">
    <button
      class="btn-secondary"
      onclick={handleReset}
      aria-label="Reset to defaults"
      data-testid="settings-reset"
    >
      Reset Defaults
    </button>
    <div class="settings-panel__footer-right">
      <button class="btn-secondary" onclick={onClose}>Cancel</button>
      <button class="btn-primary" onclick={handleSave} disabled={saving}>
        {saving ? "Saving..." : "Save"}
      </button>
    </div>
  </div>
</div>

<style>
  .settings-panel {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 420px;
    max-width: 100%;
    background-color: var(--bg-surface);
    border-left: 1px solid var(--bg-border);
    display: flex;
    flex-direction: column;
    z-index: 50;
  }

  .settings-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4);
    border-bottom: 1px solid var(--bg-border);
  }

  h2 {
    font-size: var(--font-size-lg);
    font-weight: 600;
  }

  .settings-panel__body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
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

  .settings-section__title--inline {
    margin-bottom: 0;
    display: inline;
  }

  .settings-section__toggle {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    background: transparent;
    color: var(--text-secondary);
    padding: 0;
    cursor: pointer;
    width: 100%;
    text-align: left;
  }

  .settings-section__toggle:hover {
    color: var(--text-primary);
  }

  .settings-section__collapsible {
    margin-top: var(--space-3);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
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

  .settings-field__error {
    font-size: var(--font-size-xs);
    color: var(--status-error);
  }

  /* Select */
  select {
    font-family: var(--font-ui);
    font-size: var(--font-size-sm);
    color: var(--text-primary);
    background-color: var(--bg-primary);
    border: 1px solid var(--bg-border);
    border-radius: var(--radius-sm);
    padding: var(--space-2) var(--space-3);
    outline: none;
    -webkit-appearance: none;
    appearance: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23969696' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
    padding-right: 28px;
  }

  select:focus {
    border-color: var(--accent);
  }

  /* Add/remove list rows */
  .settings-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .settings-list__row {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }

  .settings-list__input {
    flex: 1;
  }

  .settings-list__input--port {
    flex: 0 0 90px;
    max-width: 90px;
  }

  .settings-list__add {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    align-self: flex-start;
    font-size: var(--font-size-xs);
    padding: var(--space-1) var(--space-2);
  }

  /* Footer */
  .settings-panel__footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4);
    border-top: 1px solid var(--bg-border);
  }

  .settings-panel__footer-right {
    display: flex;
    gap: var(--space-2);
  }
</style>

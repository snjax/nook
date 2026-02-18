import { listen } from "@tauri-apps/api/event";
import type { LogEntry, LogBatchEvent } from "../types";

let logs = $state<Map<string, LogEntry[]>>(new Map());
let unlistener: (() => void) | null = null;

export function getLogsForPod(podId: string): LogEntry[] {
  return logs.get(podId) ?? [];
}

export function appendLogs(podId: string, entries: LogEntry[]): void {
  const newMap = new Map(logs);
  const existing = newMap.get(podId) ?? [];
  const updated = [...existing, ...entries];
  // Keep last 10000 entries
  if (updated.length > 10000) {
    updated.splice(0, updated.length - 10000);
  }
  newMap.set(podId, updated);
  logs = newMap;
}

export function clearLogsForPod(podId: string): void {
  const newMap = new Map(logs);
  newMap.delete(podId);
  logs = newMap;
}

export function removeLogsForPod(podId: string): void {
  clearLogsForPod(podId);
}

export async function initLogListeners(): Promise<void> {
  if (unlistener) {
    unlistener();
  }
  unlistener = await listen<LogBatchEvent>("pod-log-update", (event) => {
    const { podId, entries } = event.payload;
    appendLogs(podId, entries);
  });
}

export function destroyLogListeners(): void {
  if (unlistener) {
    unlistener();
    unlistener = null;
  }
}

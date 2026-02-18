import { listen } from "@tauri-apps/api/event";
import * as api from "../api/tauri";
import type {
  Pod,
  PodStatsUpdate,
  PodStatusChanged,
  ProcessListUpdate,
  PortDetectedEvent,
} from "../types";

const MAX_HISTORY_POINTS = 60;

// Reactive state — use Map + reassignment for guaranteed Svelte reactivity.
let pods = $state<Map<string, Pod>>(new Map());
let cpuHistory = $state<Map<string, number[]>>(new Map());
let ramHistory = $state<Map<string, number[]>>(new Map());
let initialized = $state(false);

export function getAllPods(): Pod[] {
  return Array.from(pods.values());
}

export function getRunningPods(): Pod[] {
  return Array.from(pods.values()).filter(
    (p) => p.status === "running" || p.status === "starting" || p.status === "stopping" || p.status === "error",
  );
}

export function getStoppedPods(): Pod[] {
  return Array.from(pods.values()).filter((p) => p.status === "stopped");
}

export function getPod(id: string): Pod | undefined {
  return pods.get(id);
}

export function getCpuHistory(podId: string): number[] {
  return cpuHistory.get(podId) ?? [];
}

export function getRamHistory(podId: string): number[] {
  return ramHistory.get(podId) ?? [];
}

export function isInitialized(): boolean {
  return initialized;
}

function pushHistory(map: Map<string, number[]>, key: string, value: number) {
  const arr = map.get(key) ?? [];
  arr.push(value);
  if (arr.length > MAX_HISTORY_POINTS) {
    arr.shift();
  }
  map.set(key, arr);
}

export async function loadPods(): Promise<void> {
  try {
    const podList = await api.listPods();
    const newMap = new Map<string, Pod>();
    for (const pod of podList) {
      newMap.set(pod.id, pod);
    }
    pods = newMap;
    initialized = true;
  } catch (e) {
    console.error("Failed to load pods:", e);
  }
}

export function setPods(newPods: Pod[]): void {
  const newMap = new Map<string, Pod>();
  for (const pod of newPods) {
    newMap.set(pod.id, pod);
  }
  pods = newMap;
  initialized = true;
}

export function updatePod(pod: Pod): void {
  const newMap = new Map(pods);
  newMap.set(pod.id, pod);
  pods = newMap;
}

export function removePodFromStore(id: string): void {
  const newMap = new Map(pods);
  newMap.delete(id);
  pods = newMap;
  cpuHistory.delete(id);
  ramHistory.delete(id);
}

let unlisteners: Array<() => void> = [];

// Throttle stats updates: batch into a single Map swap per animation frame
let pendingStats = new Map<string, PodStatsUpdate>();
let statsRafId: number | null = null;

function flushStats() {
  statsRafId = null;
  if (pendingStats.size === 0) return;

  const newPods = new Map(pods);
  const newCpu = new Map(cpuHistory);
  const newRam = new Map(ramHistory);

  for (const [podId, { cpuPercent, memoryUsed, memoryLimit }] of pendingStats) {
    const pod = newPods.get(podId);
    if (pod) {
      newPods.set(podId, { ...pod, cpuPercent, memoryUsed, memoryLimit });
      pushHistory(newCpu, podId, cpuPercent);
      pushHistory(newRam, podId, memoryUsed);
    }
  }

  pendingStats.clear();
  pods = newPods;
  cpuHistory = newCpu;
  ramHistory = newRam;
}

export async function initPodListeners(): Promise<void> {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  unlisteners = [];

  unlisteners.push(
    await listen<PodStatsUpdate>("pod-stats-update", (event) => {
      const { podId } = event.payload;
      pendingStats.set(podId, event.payload);
      if (statsRafId === null) {
        statsRafId = requestAnimationFrame(flushStats);
      }
    }),
  );

  unlisteners.push(
    await listen<PodStatusChanged>("pod-status-changed", (event) => {
      const { podId, status, errorMessage } = event.payload;
      const pod = pods.get(podId);
      if (pod) {
        const newMap = new Map(pods);
        newMap.set(podId, { ...pod, status, errorMessage });
        pods = newMap;
      }
    }),
  );

  unlisteners.push(
    await listen<ProcessListUpdate>("process-list-update", (event) => {
      const { podId, processes } = event.payload;
      const pod = pods.get(podId);
      if (pod) {
        const newMap = new Map(pods);
        newMap.set(podId, { ...pod, processes });
        pods = newMap;
      }
    }),
  );

  unlisteners.push(
    await listen<PortDetectedEvent>("port-detected", (event) => {
      const { podId, port } = event.payload;
      const pod = pods.get(podId);
      if (pod) {
        const newMap = new Map(pods);
        newMap.set(podId, {
          ...pod,
          detectedPorts: [...pod.detectedPorts, port],
        });
        pods = newMap;
      }
    }),
  );
}

export function destroyPodListeners(): void {
  for (const unlisten of unlisteners) {
    unlisten();
  }
  unlisteners = [];
  if (statsRafId !== null) {
    cancelAnimationFrame(statsRafId);
    statsRafId = null;
  }
  pendingStats.clear();
}

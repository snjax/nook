import { invoke } from "@tauri-apps/api/core";

import type { LogEntry } from "../types";

export interface MockPod {
  id: string;
  name: string;
  projectPath: string;
  image: string;
  status: string;
  uptimeSecs?: number | null;
  cpuPercent?: number;
  memoryUsed?: number;
  memoryLimit?: number;
  defaultShell?: string;
  exposedPorts?: unknown[];
  detectedPorts?: unknown[];
  processes?: unknown[];
  errorMessage?: string | null;
  startedAt?: number | null;
  alias?: string | null;
  remoteUser?: string | null;
  remoteWorkspaceFolder?: string | null;
  containerName?: string | null;
}

export async function testInjectPods(pods: MockPod[]): Promise<void> {
  return invoke("test_inject_pods", { pods });
}

export async function testInjectDetectedPort(
  podId: string,
  port: number,
  protocol: string,
  processName?: string,
): Promise<void> {
  return invoke("test_inject_detected_port", {
    podId,
    port,
    protocol,
    processName,
  });
}

export async function testSetStats(
  podId: string,
  cpuPercent: number,
  memoryUsed: number,
  memoryLimit: number,
): Promise<void> {
  return invoke("test_set_stats", { podId, cpuPercent, memoryUsed, memoryLimit });
}

export async function testReset(): Promise<void> {
  return invoke("test_reset");
}

export async function testInjectLogs(
  podId: string,
  entries: LogEntry[],
): Promise<void> {
  return invoke("test_inject_logs", { podId, entries });
}

export async function isTestApiAvailable(): Promise<boolean> {
  try {
    await invoke("test_reset");
    return true;
  } catch {
    return false;
  }
}

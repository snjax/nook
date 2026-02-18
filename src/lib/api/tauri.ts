import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  DependencyCheck,
  ExposedPort,
  LogEntry,
  Pod,
  Settings,
} from "../types";

export async function listPods(): Promise<Pod[]> {
  return invoke<Pod[]>("list_pods");
}

export async function addPod(path: string): Promise<Pod> {
  return invoke<Pod>("add_pod", { path });
}

export async function startPod(id: string): Promise<void> {
  return invoke("start_pod", { id });
}

export async function stopPod(id: string): Promise<void> {
  return invoke("stop_pod", { id });
}

export async function removePod(
  id: string,
  removeVolumes: boolean = false,
): Promise<void> {
  return invoke("remove_pod", { id, removeVolumes });
}

export async function openTerminal(id: string): Promise<void> {
  return invoke("open_terminal", { id });
}

export async function exposePort(
  podId: string,
  containerPort: number,
  hostPort: number,
): Promise<ExposedPort> {
  return invoke<ExposedPort>("expose_port", { podId, containerPort, hostPort });
}

export async function unexposePort(
  podId: string,
  containerPort: number,
): Promise<void> {
  return invoke("unexpose_port", { podId, containerPort });
}

export async function ignorePort(
  podId: string,
  containerPort: number,
): Promise<void> {
  return invoke("ignore_port", { podId, containerPort });
}

export async function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export async function saveSettings(settings: Settings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function checkDependencies(): Promise<DependencyCheck[]> {
  return invoke<DependencyCheck[]>("check_dependencies");
}

export async function getDetectedTerminal(): Promise<string> {
  return invoke<string>("get_detected_terminal");
}

export async function getDefaultSettings(): Promise<Settings> {
  return invoke<Settings>("get_default_settings");
}

export async function restartPod(id: string): Promise<void> {
  return invoke("restart_pod", { id });
}

export async function forceStopPod(id: string): Promise<void> {
  return invoke("force_stop_pod", { id });
}

export async function rebuildPod(id: string): Promise<void> {
  return invoke("rebuild_pod", { id });
}

export async function checkDockerHealth(): Promise<boolean> {
  return invoke<boolean>("check_docker_health");
}

export async function getPodLogs(
  id: string,
  tail?: number,
  filter?: string,
): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("get_pod_logs", { id, tail, filter });
}

export async function clearPodLogs(id: string): Promise<void> {
  return invoke("clear_pod_logs", { id });
}

export async function cancelBuild(id: string): Promise<void> {
  return invoke("cancel_build", { id });
}

export interface PodStateConfig {
  name: string;
  projectPath: string;
  shell: string;
  autoExposePorts: { containerPort: number; hostPort: number; protocol: string }[];
  portMappings: Record<string, number>;
  alias: string;
  notExposeFilters: { protocol?: string; port?: number }[];
  manualExposePorts: { containerPort: number; hostPort: number; protocol: string }[];
  terminalOverride: string;
  workingDir: string;
  bindAddress: string;
  remoteUser: string;
}

export async function getPodSettings(id: string): Promise<PodStateConfig> {
  return invoke<PodStateConfig>("get_pod_settings", { id });
}

export async function savePodSettings(
  id: string,
  config: PodStateConfig,
): Promise<void> {
  return invoke("save_pod_settings", { id, config });
}

export async function openInFileManager(path: string): Promise<void> {
  return invoke("open_in_file_manager", { path });
}

export async function pickDirectory(): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false, title: "Select project directory" });
  if (Array.isArray(selected)) return selected[0] ?? null;
  return selected;
}

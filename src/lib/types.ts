export type PodStatus =
  | "running"
  | "stopped"
  | "starting"
  | "stopping"
  | "error";

export interface Pod {
  id: string;
  name: string;
  projectPath: string;
  image: string;
  status: PodStatus;
  uptimeSecs: number | null;
  cpuPercent: number;
  memoryUsed: number;
  memoryLimit: number;
  defaultShell: string;
  exposedPorts: ExposedPort[];
  detectedPorts: DetectedPort[];
  processes: Process[];
  errorMessage: string | null;
  containerId: string | null;
  startedAt: number | null;
  alias: string | null;
  remoteUser: string | null;
  remoteWorkspaceFolder: string | null;
  containerName: string | null;
}

export type PortStatus =
  | "active"
  | "hostPortBusy"
  | { error: string };

export interface ExposedPort {
  containerPort: number;
  hostPort: number;
  protocol: string;
  status: PortStatus;
  autoExpose: boolean;
}

export type DetectionMethod =
  | "wellKnown"
  | "processName"
  | "nmap"
  | "bannerGrab"
  | "unknown";

export type Confidence = "high" | "low";

export interface DetectedPort {
  containerPort: number;
  protocol: string;
  processName: string;
  detectionMethod: DetectionMethod;
  confidence: Confidence;
}

export interface Process {
  pid: number;
  name: string;
  cpuPercent: number;
  memoryBytes: number;
}

// Event payloads

export interface PodStatsUpdate {
  podId: string;
  cpuPercent: number;
  memoryUsed: number;
  memoryLimit: number;
}

export interface PortDetectedEvent {
  podId: string;
  port: DetectedPort;
}

export interface ProcessListUpdate {
  podId: string;
  processes: Process[];
}

export interface PodStatusChanged {
  podId: string;
  status: PodStatus;
  errorMessage: string | null;
}

export interface NotExposeFilter {
  protocol?: string;
  port?: number;
}

export type PortAction = "prompt" | "autoExpose" | "ignore";

export interface Settings {
  exposeProtocols: string[];
  notExposeFilters: NotExposeFilter[];
  portProtocols: Record<number, string>;
  terminal: string;
  statsInterval: number;
  portsScanInterval: number;
  processScanInterval: number;
  dockerSocketPath: string;
  onboardingComplete: boolean;
  logLevel: string;
  portAction: PortAction;
}

export interface DependencyCheck {
  name: string;
  satisfied: boolean;
  details: string;
  fixHint: string | null;
}

// Log types

export type LogSource = "build" | "container";
export type LogLevel = "stdout" | "stderr";

export interface LogEntry {
  timestamp: number;
  message: string;
  source: LogSource;
  level: LogLevel;
}

export interface LogBatchEvent {
  podId: string;
  entries: LogEntry[];
}

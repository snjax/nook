import type { MockPod } from "../../../src/lib/api/test-api";

export function emptyState(): MockPod[] {
  return [];
}

export function singleRunningPod(): MockPod[] {
  return [
    {
      id: "pod-running-1",
      name: "my-project",
      projectPath: "/home/user/projects/my-project",
      image: "node:20-slim",
      status: "running",
      uptimeSecs: 8040,
      cpuPercent: 3.2,
      memoryUsed: 268435456,
      memoryLimit: 536870912,
      defaultShell: "/bin/zsh",
      exposedPorts: [
        {
          containerPort: 3000,
          hostPort: 3000,
          protocol: "http",
          status: "active",
          autoExpose: true,
        },
        {
          containerPort: 5432,
          hostPort: 5432,
          protocol: "postgres",
          status: "active",
          autoExpose: true,
        },
      ],
      detectedPorts: [],
      processes: [
        { pid: 1, name: "node", cpuPercent: 2.1, memoryBytes: 188743680 },
        { pid: 42, name: "postgres", cpuPercent: 0.8, memoryBytes: 67108864 },
        { pid: 100, name: "sh", cpuPercent: 0.0, memoryBytes: 2097152 },
      ],
    },
  ];
}

export function singleStoppedPod(): MockPod[] {
  return [
    {
      id: "pod-stopped-1",
      name: "backend-api",
      projectPath: "/home/user/projects/backend-api",
      image: "python:3.12",
      status: "stopped",
      uptimeSecs: 259200,
    },
  ];
}

export function singleErrorPod(): MockPod[] {
  return [
    {
      id: "pod-error-1",
      name: "broken-project",
      projectPath: "/home/user/projects/broken",
      image: "node:20",
      status: "error",
      errorMessage:
        "Container failed to start: exit code 1.\nError: Cannot find module 'express'",
    },
  ];
}

export function startingPod(): MockPod[] {
  return [
    {
      id: "pod-starting-1",
      name: "new-project",
      projectPath: "/home/user/projects/new-project",
      image: "node:20",
      status: "starting",
    },
  ];
}

export function mixedPods(): MockPod[] {
  return [
    ...singleRunningPod(),
    ...singleStoppedPod(),
    ...singleErrorPod(),
  ];
}

export function manyPods(count: number = 50): MockPod[] {
  const pods: MockPod[] = [];
  for (let i = 0; i < count; i++) {
    pods.push({
      id: `pod-many-${i}`,
      name: `project-${i}`,
      projectPath: `/home/user/projects/project-${i}`,
      image: i % 2 === 0 ? "node:20" : "python:3.12",
      status: i % 3 === 0 ? "running" : "stopped",
      cpuPercent: i % 3 === 0 ? Math.random() * 100 : 0,
      memoryUsed: i % 3 === 0 ? Math.floor(Math.random() * 536870912) : 0,
      memoryLimit: 536870912,
    });
  }
  return pods;
}

export function longNamePod(): MockPod[] {
  return [
    {
      id: "pod-long-name",
      name: "very-long-project-name-that-might-overflow-the-container-width",
      projectPath: "/home/user/projects/very-long-project-name-that-might-overflow",
      image: "mcr.microsoft.com/devcontainers/javascript-node:20-bullseye",
      status: "running",
      cpuPercent: 45.7,
      memoryUsed: 500000000,
      memoryLimit: 536870912,
      processes: [
        { pid: 1, name: "node", cpuPercent: 40.0, memoryBytes: 400000000 },
      ],
    },
  ];
}

export function manyPortsPod(): MockPod[] {
  const exposedPorts = Array.from({ length: 8 }, (_, i) => ({
    containerPort: 3000 + i,
    hostPort: 3000 + i,
    protocol: "http",
    status: "active" as const,
    autoExpose: true,
  }));

  const detectedPorts = Array.from({ length: 7 }, (_, i) => ({
    containerPort: 4000 + i,
    protocol: "unknown",
    processName: `service-${i}`,
    detectionMethod: "wellKnown" as const,
    confidence: "high" as const,
  }));

  return [
    {
      id: "pod-many-ports",
      name: "microservices",
      projectPath: "/home/user/projects/microservices",
      image: "node:20",
      status: "running",
      cpuPercent: 25.0,
      memoryUsed: 268435456,
      memoryLimit: 1073741824,
      exposedPorts,
      detectedPorts,
      processes: [
        { pid: 1, name: "node", cpuPercent: 15.0, memoryBytes: 200000000 },
      ],
    },
  ];
}

export function manyProcessesPod(): MockPod[] {
  const processes = Array.from({ length: 30 }, (_, i) => ({
    pid: i + 1,
    name: `worker-${i}`,
    cpuPercent: Math.random() * 10,
    memoryBytes: Math.floor(Math.random() * 100000000),
  }));

  return [
    {
      id: "pod-many-procs",
      name: "worker-pool",
      projectPath: "/home/user/projects/worker-pool",
      image: "python:3.12",
      status: "running",
      cpuPercent: 78.3,
      memoryUsed: 800000000,
      memoryLimit: 1073741824,
      processes,
    },
  ];
}

export function podWithBusyPort(): MockPod[] {
  return [
    {
      id: "pod-busy-port",
      name: "conflict-project",
      projectPath: "/home/user/projects/conflict",
      image: "node:20",
      status: "running",
      cpuPercent: 5.0,
      memoryUsed: 100000000,
      memoryLimit: 536870912,
      exposedPorts: [
        {
          containerPort: 8080,
          hostPort: 8080,
          protocol: "http",
          status: "hostPortBusy",
          autoExpose: true,
        },
      ],
    },
  ];
}

export function stoppingPod(): MockPod[] {
  return [
    {
      id: "pod-stopping-1",
      name: "shutting-down-project",
      projectPath: "/home/user/projects/shutting-down",
      image: "node:20",
      status: "stopping",
      cpuPercent: 1.0,
      memoryUsed: 100000000,
      memoryLimit: 536870912,
    },
  ];
}

export function podWithRestart(): MockPod[] {
  return [
    {
      id: "pod-restart-1",
      name: "restart-project",
      projectPath: "/home/user/projects/restart",
      image: "node:20-slim",
      status: "running",
      uptimeSecs: 3600,
      cpuPercent: 5.0,
      memoryUsed: 200000000,
      memoryLimit: 536870912,
      defaultShell: "/bin/bash",
      exposedPorts: [
        {
          containerPort: 3000,
          hostPort: 3000,
          protocol: "http",
          status: "active",
          autoExpose: true,
        },
      ],
      processes: [
        { pid: 1, name: "node", cpuPercent: 3.0, memoryBytes: 150000000 },
      ],
    },
  ];
}

export function podWithAlias(): MockPod[] {
  return [
    {
      id: "pod-alias-1",
      name: "my-long-project-name",
      projectPath: "/home/user/projects/my-long-project",
      image: "node:20",
      status: "running",
      uptimeSecs: 1200,
      cpuPercent: 2.5,
      memoryUsed: 150000000,
      memoryLimit: 536870912,
      alias: "My App",
      processes: [
        { pid: 1, name: "node", cpuPercent: 2.0, memoryBytes: 100000000 },
      ],
    },
  ];
}

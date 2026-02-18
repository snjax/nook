export function formatUptime(secs: number | null): string {
  if (secs === null || secs === undefined) return "";
  if (secs < 60) return `${secs}s`;
  if (secs < 3600) return `${Math.floor(secs / 60)}m`;
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return `${h}h ${m}m`;
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i > 1 ? 0 : 1)} ${units[i]}`;
}

export function formatTimeAgo(secs: number | null): string {
  if (secs === null || secs === undefined) return "";
  if (secs < 60) return "just now";
  if (secs < 3600) return `${Math.floor(secs / 60)}m ago`;
  if (secs < 86400) return `${Math.floor(secs / 3600)}h ago`;
  return `${Math.floor(secs / 86400)}d ago`;
}

export function formatCpuPercent(cpu: number): string {
  return `${cpu.toFixed(1)}%`;
}

export function formatMemory(used: number, limit: number): string {
  return `${formatBytes(used)}/${formatBytes(limit)}`;
}
